use bevy::{
    asset::RenderAssetUsages,
    core_pipeline::schedule::camera_driver,
    ecs::component::Mutable,
    prelude::*,
    render::{
        Extract, Render, RenderApp, RenderStartup, RenderSystems,
        extract_resource::ExtractResource,
        mesh::allocator::{MeshAllocator, MeshAllocatorSettings},
        render_resource::{
            binding_types::{storage_buffer, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice, RenderGraph, RenderQueue},
    },
    shader::ShaderCacheError,
};
use std::{borrow::Cow, num::NonZero};
use xs_bevy_state_scoped_systems::add_state_scoped_systems;

use crate::playgrounds::PlaygroundScene;

const SHADER_ASSET_PATH: &str = "shaders/mesh_manipulation.wgsl";

const WORKGROUP_SIZE: u32 = 1;

pub struct MeshManipulationPlugin;

impl Plugin for MeshManipulationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MeshManipulationUniforms {
            pane_x_count: 64,
            pane_y_count: 64,
        })
        .add_message::<RenderStateReset>()
        .add_plugins(MeshManipulationComputePlugin);

        add_state_scoped_systems!(
            app,
            PlaygroundScene::MeshManipulation,
            OnEnter((init_mesh, setup, bevy::asset::handle_internal_asset_events,).chain()),
        );
    }
}

fn init_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut reset: MessageWriter<RenderStateReset>,
    // mut layouts: ResMut<MeshVertexBufferLayouts>,
) {
    let mut mesh = Plane3d::default().mesh().size(5.0, 5.0).build();
    println!("get_vertex_buffer_size: {}", mesh.get_vertex_buffer_size());
    // println!(
    //     "get_mesh_vertex_buffer_layout: {:?}",
    //     mesh.get_mesh_vertex_buffer_layout(&mut layouts)
    // );
    mesh.asset_usage = RenderAssetUsages::default();
    let mesh_handle = meshes.add(mesh);

    commands.insert_resource(MeshData { mesh_handle });

    reset.write(RenderStateReset);
}

fn setup(
    mut commands: Commands,
    mesh_data: Res<MeshData>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        DespawnOnExit(PlaygroundScene::MeshManipulation),
        Mesh3d(mesh_data.mesh_handle.clone()),
        MeshMaterial3d(materials.add(StandardMaterial::default())),
        Transform::default(),
    ));
}

struct MeshManipulationComputePlugin;

impl Plugin for MeshManipulationComputePlugin {
    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<RenderState>()
            // ATTN: MeshAllocatorSettings MUST BE INSERTED IN RENDER WORLD
            .insert_resource(MeshAllocatorSettings {
                extra_buffer_usages: BufferUsages::STORAGE,
                ..Default::default()
            })
            .add_systems(
                ExtractSchedule,
                (
                    extract_state::<PlaygroundScene>,
                    reset_render_state,
                    extract_conditionally::<MeshData>,
                    extract_conditionally::<MeshManipulationUniforms>,
                ),
            )
            .add_systems(RenderStartup, init_pipeline)
            .add_systems(
                Render,
                (
                    prepare_bind_group.in_set(RenderSystems::PrepareBindGroups),
                    update.in_set(RenderSystems::Prepare),
                )
                    .run_if(in_state(PlaygroundScene::MeshManipulation)),
            )
            .add_systems(
                RenderGraph,
                mesh_manipulation
                    .before(camera_driver)
                    .run_if(in_state(PlaygroundScene::MeshManipulation)),
            );
    }
}

fn reset_render_state(
    mut reset: Extract<MessageReader<RenderStateReset>>,
    mut render_state: ResMut<RenderState>,
) {
    if reset.read().count() > 0 {
        *render_state = RenderState::Loading;
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
    if *condition.get() != PlaygroundScene::MeshManipulation {
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
struct MeshData {
    mesh_handle: Handle<Mesh>,
}

#[derive(Resource, Clone, ExtractResource, ShaderType)]
struct MeshManipulationUniforms {
    pane_x_count: u32,
    pane_y_count: u32,
}

#[derive(Resource)]
struct MeshManipulationBindGroups(BindGroup);

#[derive(Resource, Clone)]
struct DebugBuffer {
    buffer: Buffer,
    readback: Buffer,
}

fn init_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
    render_device: Res<RenderDevice>,
) {
    let mesh_bind_group_layout = BindGroupLayoutDescriptor::new(
        "MeshData",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                storage_buffer::<Vec<Vertex>>(false),
                storage_buffer::<u32>(false),
                uniform_buffer::<MeshManipulationUniforms>(false),
            ),
        ),
    );
    let shader = asset_server.load(SHADER_ASSET_PATH);
    let init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        layout: vec![mesh_bind_group_layout.clone()],
        shader: shader.clone(),
        entry_point: Some(Cow::from("init")),
        ..default()
    });
    let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        layout: vec![mesh_bind_group_layout.clone()],
        shader,
        entry_point: Some(Cow::from("update")),
        ..default()
    });

    commands.insert_resource(MeshManipulationPipeline {
        mesh_bind_group_layout,
        init_pipeline,
        update_pipeline,
    });

    let debug_buffer = render_device.create_buffer(&BufferDescriptor {
        label: Some("debug buffer"),
        size: 4,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let debug_readback = render_device.create_buffer(&BufferDescriptor {
        label: Some("debug readback"),
        size: 4,
        usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    commands.insert_resource(DebugBuffer {
        buffer: debug_buffer,
        readback: debug_readback,
    });
}

fn prepare_bind_group(
    mut commands: Commands,
    pipeline: Res<MeshManipulationPipeline>,
    mesh_allocator: Res<MeshAllocator>,
    mesh_data: Res<MeshData>,
    uniforms: Res<MeshManipulationUniforms>,
    debug_buffer: Res<DebugBuffer>,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    queue: Res<RenderQueue>,
) {
    let vertex_slice = mesh_allocator
        .mesh_vertex_slice(&mesh_data.mesh_handle.id())
        .unwrap();

    let mut uniform_buffer = UniformBuffer::from(uniforms.into_inner());
    uniform_buffer.write_buffer(&render_device, &queue);

    let bind_group = render_device.create_bind_group(
        Some("mesh manipulation bind group"),
        &pipeline_cache.get_bind_group_layout(&pipeline.mesh_bind_group_layout),
        &BindGroupEntries::sequential((
            BindingResource::Buffer(BufferBinding {
                buffer: vertex_slice.buffer,
                offset: vertex_slice.range.start as u64,
                size: NonZero::new((vertex_slice.range.count() * 48) as u64),
            }),
            BindingResource::Buffer(BufferBinding {
                buffer: &debug_buffer.buffer,
                offset: 0,
                size: NonZero::new(4),
            }),
            &uniform_buffer,
        )),
    );
    commands.insert_resource(MeshManipulationBindGroups(bind_group));
}

#[derive(Resource)]
struct MeshManipulationPipeline {
    mesh_bind_group_layout: BindGroupLayoutDescriptor,
    init_pipeline: CachedComputePipelineId,
    update_pipeline: CachedComputePipelineId,
}

#[derive(ShaderType)]
struct Vertex {
    position: Vec3,
    normal: Vec3,
    uv: Vec2,
}

#[derive(Resource, Default)]
enum RenderState {
    #[default]
    Loading,
    Init,
    Update,
}

fn update(
    pipeline: Res<MeshManipulationPipeline>,
    pipeline_cache: Res<PipelineCache>,
    mut state: ResMut<RenderState>,
) {
    // if the corresponding pipeline has loaded, transition to the next stage
    match *state {
        RenderState::Loading => {
            match pipeline_cache.get_compute_pipeline_state(pipeline.init_pipeline) {
                CachedPipelineState::Ok(_) => {
                    *state = RenderState::Init;
                }
                // If the shader hasn't loaded yet, just wait.
                CachedPipelineState::Err(ShaderCacheError::ShaderNotLoaded(_)) => {}
                CachedPipelineState::Err(err) => {
                    panic!("Initializing assets/{SHADER_ASSET_PATH}:\n{err}")
                }
                _ => {}
            }
        }
        RenderState::Init => {
            if let CachedPipelineState::Ok(_) =
                pipeline_cache.get_compute_pipeline_state(pipeline.update_pipeline)
            {
                *state = RenderState::Update;
            }
        }
        RenderState::Update => {}
    }
}

fn mesh_manipulation(
    mut render_context: RenderContext,
    bind_groups: Res<MeshManipulationBindGroups>,
    pipeline_cache: Res<PipelineCache>,
    pipeline: Res<MeshManipulationPipeline>,
    state: Res<RenderState>,
    debug_buffer: Res<DebugBuffer>,
) {
    let mut pass = render_context
        .command_encoder()
        .begin_compute_pass(&ComputePassDescriptor::default());

    // select the pipeline based on the current state
    match *state {
        RenderState::Loading => {}
        RenderState::Init => {
            let init_pipeline = pipeline_cache
                .get_compute_pipeline(pipeline.init_pipeline)
                .unwrap();
            pass.set_bind_group(0, &bind_groups.0, &[]);
            pass.set_pipeline(init_pipeline);
            pass.dispatch_workgroups(WORKGROUP_SIZE, 1, 1);
        }
        RenderState::Update => {
            let update_pipeline = pipeline_cache
                .get_compute_pipeline(pipeline.update_pipeline)
                .unwrap();
            pass.set_bind_group(0, &bind_groups.0, &[]);
            pass.set_pipeline(update_pipeline);
            pass.dispatch_workgroups(WORKGROUP_SIZE, 1, 1);
        }
    }

    drop(pass);

    let encoder = render_context.command_encoder();

    encoder.copy_buffer_to_buffer(&debug_buffer.buffer, 0, &debug_buffer.readback, 0, 4);

    let _ = render_context
        .render_device()
        .wgpu_device()
        .poll(PollType::Wait {
            submission_index: None,
            timeout: None,
        });

    let slice = debug_buffer.readback.slice(..);

    slice.map_async(MapMode::Read, |_| {});

    let _ = render_context
        .render_device()
        .wgpu_device()
        .poll(PollType::Wait {
            submission_index: None,
            timeout: None,
        });

    let data = slice.get_mapped_range();

    let value = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);

    println!("{value}");

    drop(data);

    debug_buffer.readback.unmap();
}
