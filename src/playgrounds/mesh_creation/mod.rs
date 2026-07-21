use bevy::{
    asset::RenderAssetUsages,
    core_pipeline::schedule::camera_driver,
    ecs::component::Mutable,
    mesh::Indices,
    prelude::*,
    render::{
        Extract, Render, RenderApp, RenderStartup, RenderSystems,
        extract_component::ExtractComponent,
        extract_resource::ExtractResource,
        mesh::allocator::{MeshAllocator, MeshAllocatorSettings, MeshBufferSlice},
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

use crate::{playgrounds::PlaygroundScene, utils::extract_state};

const SHADER_ASSET_PATH: &str = "shaders/mesh_creation.wgsl";

const WORKGROUP_SIZE_X: u32 = 1;
const WORKGROUP_SIZE_Y: u32 = 1;

pub struct MeshCreationPlugin;

impl Plugin for MeshCreationPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<RenderPluginMessage>()
            .add_plugins(MeshCreationComputePlugin);

        add_state_scoped_systems!(
            app,
            PlaygroundScene::MeshCreation,
            OnEnter((setup, bevy::asset::handle_internal_asset_events,).chain()),
            RunInState(
                Update,
                (
                    remove_deform_component,
                    insert_deform_component.run_if(trigger_deformation_pressed)
                )
                    .chain()
            ),
        );
    }
}

fn trigger_deformation_pressed(keyboard: Res<ButtonInput<KeyCode>>) -> bool {
    keyboard.just_pressed(KeyCode::Space)
}

fn insert_deform_component(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    deformables: Query<(Entity, &mut Mesh3d), With<Deformable>>,
) {
    for (entity, mut mesh_3d) in deformables {
        let empty_mesh = {
            let mut mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::RENDER_WORLD,
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.; 3]; 50])
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.; 3]; 50])
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.; 2]; 50])
            .with_inserted_indices(Indices::U32(vec![0; 50]));

            mesh.asset_usage = RenderAssetUsages::RENDER_WORLD;
            mesh
        };

        let handle = meshes.add(empty_mesh);
        let old_handle = mesh_3d.0.clone();
        mesh_3d.0 = handle.clone();
        commands.entity(entity).insert(Deform {
            old_mesh_handle: old_handle,
            new_mesh_handle: handle,
        });
    }
}

fn remove_deform_component(mut commands: Commands, deformables: Query<Entity, With<Deform>>) {
    for entity in deformables {
        commands.entity(entity).remove::<Deform>();
    }
}

#[derive(Component, Default, Clone)]
struct Deformable;

#[derive(Component, Default, Clone, ExtractComponent)]
struct Deform {
    old_mesh_handle: Handle<Mesh>,
    new_mesh_handle: Handle<Mesh>,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut msg_queue: MessageWriter<RenderPluginMessage>,
) {
    commands.init_resource::<MeshCreationUniforms>();

    let mut mesh = Cuboid::default().mesh().build();
    mesh.asset_usage = RenderAssetUsages::RENDER_WORLD;

    let mut uniforms = MeshCreationUniforms::default();
    uniforms.num_vertices = mesh.count_vertices() as u32;
    uniforms.num_indices = mesh.indices().map(|v| v.len() as u32).unwrap_or(0);
    commands.insert_resource(uniforms);

    let mesh_handle = meshes.add(mesh);

    let scene = bsn_list! {
        Deformable
        Mesh3d(mesh_handle)
        MeshMaterial3d<StandardMaterial>(asset_value(Color::WHITE))
        Transform
        DespawnOnExit<PlaygroundScene>(PlaygroundScene::MeshCreation),

        PointLight {
            shadow_maps_enabled: true,
            intensity: 2_000_000.0,
        }
        Transform::from_xyz(4.0, 8.0, 4.0)
        DespawnOnExit<PlaygroundScene>(PlaygroundScene::MeshCreation),
    };

    commands.spawn_scene_list(scene);
    msg_queue.write(RenderPluginMessage(RenderPluginActions::Reset));
}

struct MeshCreationComputePlugin;

impl Plugin for MeshCreationComputePlugin {
    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<MeshCreationState>()
            .insert_resource(MeshAllocatorSettings {
                extra_buffer_usages: BufferUsages::STORAGE,
                ..Default::default()
            })
            .add_systems(
                ExtractSchedule,
                (
                    handle_messages,
                    debug_extract,
                    extract_state::<PlaygroundScene>,
                    extract_conditionally::<MeshCreationUniforms>,
                ),
            )
            .add_systems(RenderStartup, (init_pipeline, init_debug_buffer));

        add_state_scoped_systems!(
            render_app,
            PlaygroundScene::MeshCreation,
            RunInState(Render, poll_pipeline_loading.in_set(RenderSystems::Prepare)),
            RunInState(RenderGraph, execute_pipeline.before(camera_driver))
        );
    }
}

fn debug_extract(mut commands: Commands, query: Extract<Query<&Deform>>) {
    for deform in &query {
        commands.spawn(deform.clone());
    }
}

enum RenderPluginActions {
    Reset,
}

#[derive(Message)]
struct RenderPluginMessage(RenderPluginActions);

fn handle_messages(
    mut deformation_messages: Extract<MessageReader<RenderPluginMessage>>,
    mut mesh_creation_state: ResMut<MeshCreationState>,
) {
    for message in deformation_messages.read() {
        match &message.0 {
            RenderPluginActions::Reset => *mesh_creation_state = MeshCreationState::Loading,
        }
    }
}

fn extract_conditionally<R: ExtractResource<(), Mutability = Mutable>>(
    mut commands: Commands,
    condition: Extract<Res<State<PlaygroundScene>>>,
    main_resource: Extract<Option<Res<R::Source>>>,
    target_resource: Option<ResMut<R>>,
) {
    if *condition.get() != PlaygroundScene::MeshCreation {
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

#[derive(Resource, Default, Debug, Clone, ExtractResource, ShaderType)]
struct MeshCreationUniforms {
    num_vertices: u32,
    num_indices: u32,
    input_vertex_start: u32,
    input_vertex_end: u32,
    input_index_start: u32,
    input_index_end: u32,
    output_vertex_start: u32,
    output_vertex_end: u32,
    output_index_start: u32,
    output_index_end: u32,
}

#[derive(ShaderType)]
struct Vertex {
    position: Vec3,
    normal: Vec3,
    uv: Vec2,
}

#[derive(Resource, Clone)]
struct DebugBuffer {
    buffer: Buffer,
    readback: Buffer,
}

fn init_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
) {
    let mesh_bind_group_layout = BindGroupLayoutDescriptor::new(
        "MeshData",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                storage_buffer::<Vec<Vertex>>(false),
                storage_buffer::<Vec<u32>>(false),
                uniform_buffer::<MeshCreationUniforms>(false),
                storage_buffer::<u32>(false),
            ),
        ),
    );
    let shader = asset_server.load(SHADER_ASSET_PATH);
    let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        layout: vec![mesh_bind_group_layout.clone()],
        shader: shader.clone(),
        entry_point: Some(Cow::from("main")),
        ..default()
    });

    commands.insert_resource(MeshCreationPipeline {
        mesh_bind_group_layout,
        pipeline,
    });
}

fn init_debug_buffer(mut commands: Commands, render_device: Res<RenderDevice>) {
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

#[derive(Resource)]
struct MeshCreationPipeline {
    mesh_bind_group_layout: BindGroupLayoutDescriptor,
    pipeline: CachedComputePipelineId,
}

#[derive(Resource, Default, PartialEq, Eq)]
enum MeshCreationState {
    #[default]
    Loading,
    Ready,
}

fn poll_pipeline_loading(
    pipeline: Res<MeshCreationPipeline>,
    pipeline_cache: Res<PipelineCache>,
    mut state: ResMut<MeshCreationState>,
) {
    // if the corresponding pipeline has loaded, transition to the next stage
    match *state {
        MeshCreationState::Loading => {
            match pipeline_cache.get_compute_pipeline_state(pipeline.pipeline) {
                CachedPipelineState::Ok(_) => {
                    *state = MeshCreationState::Ready;
                }
                // If the shader hasn't loaded yet, just wait.
                CachedPipelineState::Err(ShaderCacheError::ShaderNotLoaded(_)) => {}
                CachedPipelineState::Err(err) => {
                    panic!("Initializing assets/{SHADER_ASSET_PATH}:\n{err}")
                }
                _ => {}
            }
        }
        _ => {}
    }
}

fn execute_pipeline(
    mut render_context: RenderContext,
    queue: Res<RenderQueue>,
    pipeline_cache: Res<PipelineCache>,
    pipeline: Res<MeshCreationPipeline>,
    mesh_allocator: Res<MeshAllocator>,
    uniforms: Res<MeshCreationUniforms>,
    state: Res<MeshCreationState>,
    deform_q: Query<&Deform>,
    #[allow(unused)] debug_buffer: Res<DebugBuffer>,
) {
    if *state != MeshCreationState::Ready {
        return;
    }

    for Deform {
        old_mesh_handle,
        new_mesh_handle,
    } in deform_q
    {
        let render_device = render_context.render_device();

        let mut uniforms = uniforms.clone();

        let input_vertex_slice = mesh_allocator
            .mesh_vertex_slice(&old_mesh_handle.id())
            .unwrap();
        uniforms.input_vertex_start = input_vertex_slice.range.start;
        uniforms.input_vertex_end = input_vertex_slice.range.end;

        let input_index_slice = mesh_allocator
            .mesh_index_slice(&old_mesh_handle.id())
            .unwrap();
        uniforms.input_index_start = input_index_slice.range.start;
        uniforms.input_index_end = input_index_slice.range.end;

        let output_vertex_slice = mesh_allocator
            .mesh_vertex_slice(&new_mesh_handle.id())
            .unwrap();
        uniforms.output_vertex_start = output_vertex_slice.range.start;
        uniforms.output_vertex_end = output_vertex_slice.range.end;

        let output_index_slice = mesh_allocator
            .mesh_index_slice(&new_mesh_handle.id())
            .unwrap();
        uniforms.output_index_start = output_index_slice.range.start;
        uniforms.output_index_end = output_index_slice.range.end;

        let mut uniform_buffer = UniformBuffer::from(uniforms);
        uniform_buffer.write_buffer(&render_device, &queue);

        let bind_group = render_device.create_bind_group(
            Some("mesh creation bind group"),
            &pipeline_cache.get_bind_group_layout(&pipeline.mesh_bind_group_layout),
            &BindGroupEntries::sequential((
                input_vertex_slice.buffer.as_entire_buffer_binding(),
                input_index_slice.buffer.as_entire_buffer_binding(),
                &uniform_buffer,
                debug_buffer.buffer.as_entire_binding(),
            )),
        );

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        let mesh_creation_pipeline = pipeline_cache
            .get_compute_pipeline(pipeline.pipeline)
            .unwrap();
        pass.set_bind_group(0, &bind_group, &[]);
        pass.set_pipeline(mesh_creation_pipeline);
        pass.dispatch_workgroups(WORKGROUP_SIZE_X, WORKGROUP_SIZE_Y, 1);

        //print_shader_debug_value(&mut render_context, &debug_buffer);
    }
}

#[allow(unused)]
fn print_shader_debug_value(render_context: &mut RenderContext, debug_buffer: &DebugBuffer) {
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
