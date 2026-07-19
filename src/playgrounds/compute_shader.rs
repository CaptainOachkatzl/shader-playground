use bevy::{
    asset::RenderAssetUsages,
    core_pipeline::schedule::camera_driver,
    ecs::component::Mutable,
    prelude::*,
    render::{
        Extract, Render, RenderApp, RenderStartup, RenderSystems,
        extract_resource::ExtractResource,
        render_asset::RenderAssets,
        render_resource::{
            binding_types::{texture_storage_2d, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice, RenderGraph, RenderQueue},
        texture::GpuImage,
    },
    shader::ShaderCacheError,
};
use std::borrow::Cow;
use xs_bevy_state_scoped_systems::add_state_scoped_systems;

use crate::playgrounds::PlaygroundScene;

const SHADER_ASSET_PATH: &str = "shaders/game_of_life.wgsl";

const DISPLAY_FACTOR: u32 = 4;
const SIZE: UVec2 = UVec2::new(1280 / DISPLAY_FACTOR, 720 / DISPLAY_FACTOR);
const WORKGROUP_SIZE: u32 = 8;

pub struct ComputeShaderPlugin;

impl Plugin for ComputeShaderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameOfLifeUniforms {
            alive_color: LinearRgba::RED,
        })
        .add_message::<RenderStateReset>()
        .add_plugins(GameOfLifeComputePlugin);

        add_state_scoped_systems!(
            app,
            PlaygroundScene::ComputeShader,
            OnEnter(
                (
                    init_images,
                    setup,
                    bevy::asset::handle_internal_asset_events,
                )
                    .chain()
            ),
            OnExit(setup_3d_camera),
            RunInState(Update, switch_textures),
        );
    }
}

fn init_images(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut reset: MessageWriter<RenderStateReset>,
) {
    let mut image = Image::new_target_texture(SIZE.x, SIZE.y, TextureFormat::Rgba32Float, None);
    image.asset_usage = RenderAssetUsages::RENDER_WORLD;
    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
    let image0 = images.add(image.clone());
    let image1 = images.add(image);

    commands.insert_resource(GameOfLifeImages {
        texture_a: image0,
        texture_b: image1,
    });

    reset.write(RenderStateReset);
}

fn setup(
    mut commands: Commands,
    images: Res<GameOfLifeImages>,
    camera_q: Query<Entity, With<Camera3d>>,
) {
    if let Ok(cam_entity) = camera_q.single() {
        commands.entity(cam_entity).despawn();
    }

    commands.spawn((
        Camera2d::default(),
        DespawnOnExit(PlaygroundScene::ComputeShader),
    ));

    commands.spawn((
        DespawnOnExit(PlaygroundScene::ComputeShader),
        Sprite {
            image: images.texture_a.clone(),
            custom_size: Some(SIZE.as_vec2()),
            ..default()
        },
        Transform::from_scale(Vec3::splat(DISPLAY_FACTOR as f32)),
    ));
}

fn setup_3d_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-6.0, 4.5, 0.).looking_at(Vec3::ZERO, Vec3::Y),
        Msaa::Off,
    ));
}

// Switch texture to display every frame to show the one that was written to most recently.
fn switch_textures(images: Res<GameOfLifeImages>, mut sprite: Single<&mut Sprite>) {
    if sprite.image == images.texture_a {
        sprite.image = images.texture_b.clone();
    } else {
        sprite.image = images.texture_a.clone();
    }
}

struct GameOfLifeComputePlugin;

impl Plugin for GameOfLifeComputePlugin {
    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<GameOfLifeState>()
            .add_systems(
                ExtractSchedule,
                (
                    extract_state::<PlaygroundScene>,
                    reset_render_state,
                    extract_conditionally::<GameOfLifeImages>,
                    extract_conditionally::<GameOfLifeUniforms>,
                ),
            )
            .add_systems(RenderStartup, init_game_of_life_pipeline)
            .add_systems(
                Render,
                (
                    prepare_bind_group.in_set(RenderSystems::PrepareBindGroups),
                    update.in_set(RenderSystems::Prepare),
                )
                    .run_if(in_state(PlaygroundScene::ComputeShader)),
            )
            .add_systems(
                RenderGraph,
                game_of_life
                    .before(camera_driver)
                    .run_if(in_state(PlaygroundScene::ComputeShader)),
            );
    }
}

fn reset_render_state(
    mut reset: Extract<MessageReader<RenderStateReset>>,
    mut render_state: ResMut<GameOfLifeState>,
) {
    if reset.read().count() > 0 {
        *render_state = GameOfLifeState::Loading;
    }
}

fn extract_state<S: States>(mut commands: Commands, state: Extract<Res<State<S>>>) {
    commands.insert_resource(State::new(state.get().clone()));
}

fn extract_conditionally<R: ExtractResource<(), Mutability = Mutable>>(
    mut commands: Commands,
    condition: Extract<Res<State<PlaygroundScene>>>,
    main_resource: Extract<Option<Res<R::Source>>>,
    target_resource: Option<ResMut<R>>,
) {
    if *condition.get() != PlaygroundScene::ComputeShader {
        return;
    }

    if let Some(main_resource) = main_resource.as_ref() {
        if let Some(mut target_resource) = target_resource {
            if main_resource.is_changed() {
                *target_resource = R::extract_resource(main_resource);
            }
        } else {
            commands.insert_resource(R::extract_resource(main_resource));
        }
    }
}

#[derive(Message)]
struct RenderStateReset;

#[derive(Resource, Clone, ExtractResource)]
struct GameOfLifeImages {
    texture_a: Handle<Image>,
    texture_b: Handle<Image>,
}

#[derive(Resource, Clone, ExtractResource, ShaderType)]
struct GameOfLifeUniforms {
    alive_color: LinearRgba,
}

#[derive(Resource)]
struct GameOfLifeImageBindGroups([BindGroup; 2]);

fn prepare_bind_group(
    mut commands: Commands,
    pipeline: Res<GameOfLifePipeline>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    game_of_life_images: Res<GameOfLifeImages>,
    game_of_life_uniforms: Res<GameOfLifeUniforms>,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    queue: Res<RenderQueue>,
) {
    let view_a = gpu_images.get(&game_of_life_images.texture_a).unwrap();
    let view_b = gpu_images.get(&game_of_life_images.texture_b).unwrap();

    // Uniform buffer is used here to demonstrate how to set up a uniform in a compute shader
    // Alternatives such as storage buffers or push constants may be more suitable for your use case
    let mut uniform_buffer = UniformBuffer::from(game_of_life_uniforms.into_inner());
    uniform_buffer.write_buffer(&render_device, &queue);

    let bind_group_0 = render_device.create_bind_group(
        None,
        &pipeline_cache.get_bind_group_layout(&pipeline.texture_bind_group_layout),
        &BindGroupEntries::sequential((
            &view_a.texture_view,
            &view_b.texture_view,
            &uniform_buffer,
        )),
    );
    let bind_group_1 = render_device.create_bind_group(
        None,
        &pipeline_cache.get_bind_group_layout(&pipeline.texture_bind_group_layout),
        &BindGroupEntries::sequential((
            &view_b.texture_view,
            &view_a.texture_view,
            &uniform_buffer,
        )),
    );
    commands.insert_resource(GameOfLifeImageBindGroups([bind_group_0, bind_group_1]));
}

#[derive(Resource)]
struct GameOfLifePipeline {
    texture_bind_group_layout: BindGroupLayoutDescriptor,
    init_pipeline: CachedComputePipelineId,
    update_pipeline: CachedComputePipelineId,
}

fn init_game_of_life_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
) {
    let texture_bind_group_layout = BindGroupLayoutDescriptor::new(
        "GameOfLifeImages",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::ReadOnly),
                texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::WriteOnly),
                uniform_buffer::<GameOfLifeUniforms>(false),
            ),
        ),
    );
    let shader = asset_server.load(SHADER_ASSET_PATH);
    let init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        layout: vec![texture_bind_group_layout.clone()],
        shader: shader.clone(),
        entry_point: Some(Cow::from("init")),
        ..default()
    });
    let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        layout: vec![texture_bind_group_layout.clone()],
        shader,
        entry_point: Some(Cow::from("update")),
        ..default()
    });

    commands.insert_resource(GameOfLifePipeline {
        texture_bind_group_layout,
        init_pipeline,
        update_pipeline,
    });
}

#[derive(Resource, Default)]
enum GameOfLifeState {
    #[default]
    Loading,
    Init,
    Update(usize),
}

fn update(
    pipeline: Res<GameOfLifePipeline>,
    pipeline_cache: Res<PipelineCache>,
    mut state: ResMut<GameOfLifeState>,
) {
    // if the corresponding pipeline has loaded, transition to the next stage
    match *state {
        GameOfLifeState::Loading => {
            match pipeline_cache.get_compute_pipeline_state(pipeline.init_pipeline) {
                CachedPipelineState::Ok(_) => {
                    *state = GameOfLifeState::Init;
                }
                // If the shader hasn't loaded yet, just wait.
                CachedPipelineState::Err(ShaderCacheError::ShaderNotLoaded(_)) => {}
                CachedPipelineState::Err(err) => {
                    panic!("Initializing assets/{SHADER_ASSET_PATH}:\n{err}")
                }
                _ => {}
            }
        }
        GameOfLifeState::Init => {
            if let CachedPipelineState::Ok(_) =
                pipeline_cache.get_compute_pipeline_state(pipeline.update_pipeline)
            {
                *state = GameOfLifeState::Update(1);
            }
        }
        GameOfLifeState::Update(0) => {
            *state = GameOfLifeState::Update(1);
        }
        GameOfLifeState::Update(1) => {
            *state = GameOfLifeState::Update(0);
        }
        GameOfLifeState::Update(_) => unreachable!(),
    }
}

fn game_of_life(
    mut render_context: RenderContext,
    bind_groups: Res<GameOfLifeImageBindGroups>,
    pipeline_cache: Res<PipelineCache>,
    pipeline: Res<GameOfLifePipeline>,
    state: Res<GameOfLifeState>,
) {
    let mut pass = render_context
        .command_encoder()
        .begin_compute_pass(&ComputePassDescriptor::default());

    // select the pipeline based on the current state
    match *state {
        GameOfLifeState::Loading => {}
        GameOfLifeState::Init => {
            let init_pipeline = pipeline_cache
                .get_compute_pipeline(pipeline.init_pipeline)
                .unwrap();
            pass.set_bind_group(0, &bind_groups.0[0], &[]);
            pass.set_pipeline(init_pipeline);
            pass.dispatch_workgroups(SIZE.x / WORKGROUP_SIZE, SIZE.y / WORKGROUP_SIZE, 1);
        }
        GameOfLifeState::Update(index) => {
            let update_pipeline = pipeline_cache
                .get_compute_pipeline(pipeline.update_pipeline)
                .unwrap();
            pass.set_bind_group(0, &bind_groups.0[index], &[]);
            pass.set_pipeline(update_pipeline);
            pass.dispatch_workgroups(SIZE.x / WORKGROUP_SIZE, SIZE.y / WORKGROUP_SIZE, 1);
        }
    }
}
