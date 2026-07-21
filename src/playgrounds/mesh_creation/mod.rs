use bevy::{
    asset::RenderAssetUsages,
    core_pipeline::schedule::camera_driver,
    ecs::component::Mutable,
    prelude::*,
    render::{
        Extract, Render, RenderApp, RenderStartup, RenderSystems,
        extract_component::ExtractComponent,
        extract_resource::ExtractResource,
        mesh::allocator::MeshAllocator,
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
            OnEnter((init_mesh, setup, bevy::asset::handle_internal_asset_events,).chain()),
            RunInState(
                Update,
                send_mesh_deform_message.run_if(trigger_deformation_pressed)
            ),
        );
    }
}

fn trigger_deformation_pressed(keyboard: Res<ButtonInput<KeyCode>>) -> bool {
    keyboard.just_pressed(KeyCode::Space)
}

fn send_mesh_deform_message(mut msg_queue: MessageWriter<RenderPluginMessage>) {
    msg_queue.write(RenderPluginMessage(RenderPluginActions::StartDeformation));
}

fn init_mesh(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mut mesh = Cuboid::default().mesh().build();
    mesh.asset_usage = RenderAssetUsages::RENDER_WORLD;
    let mesh_handle = meshes.add(mesh);

    commands.insert_resource(MeshData { mesh_handle });
}

#[derive(Component, Default, Clone, Copy, ExtractComponent)]
struct Deformable;

fn setup(
    mut commands: Commands,
    mesh_data: Res<MeshData>,
    mut msg_queue: MessageWriter<RenderPluginMessage>,
) {
    commands.init_resource::<MeshCreationUniforms>();
    let mesh_handle = mesh_data.mesh_handle.clone();

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
            .add_systems(
                ExtractSchedule,
                (
                    handle_messages,
                    extract_state::<PlaygroundScene>,
                    extract_conditionally::<MeshData>,
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

enum RenderPluginActions {
    Reset,
    StartDeformation,
}

#[derive(Message)]
struct RenderPluginMessage(RenderPluginActions);

fn handle_messages(
    mut deformation_messages: Extract<MessageReader<RenderPluginMessage>>,
    mut mesh_creation_state: ResMut<MeshCreationState>,
) {
    for message in deformation_messages.read() {
        match message.0 {
            RenderPluginActions::Reset => *mesh_creation_state = MeshCreationState::Loading,
            RenderPluginActions::StartDeformation => {
                if *mesh_creation_state == MeshCreationState::Waiting {
                    *mesh_creation_state = MeshCreationState::Execute;
                }
            }
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

#[derive(Resource, Clone, ExtractResource)]
struct MeshData {
    mesh_handle: Handle<Mesh>,
}

#[derive(Resource, Default, Debug, Clone, ExtractResource, ShaderType)]
struct MeshCreationUniforms {
    num_vertices: u32,
    vertex_start: u32,
    vertex_end: u32,
    index_start: u32,
    index_end: u32,
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
                storage_buffer::<Vec<Vertex>>(false),
                storage_buffer::<Vec<Vertex>>(false),
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
    Waiting,
    Execute,
    Finished,
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
                    *state = MeshCreationState::Waiting;
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
    mesh_data: Res<MeshData>,
    uniforms: Res<MeshCreationUniforms>,
    mut state: ResMut<MeshCreationState>,
    #[allow(unused)] debug_buffer: Res<DebugBuffer>,
) {
    if *state != MeshCreationState::Execute {
        return;
    }
    let render_device = render_context.render_device();

    let input_vertex_slice = mesh_allocator
        .mesh_vertex_slice(&mesh_data.mesh_handle.id())
        .unwrap();

    let input_stride = size_of::<Vertex>() as u64;
    let input_size =
        (input_vertex_slice.range.end - input_vertex_slice.range.start) as u64 * input_stride;
    let input_offset = input_vertex_slice.range.start as u64 * input_stride;

    let mut uniform_buffer = UniformBuffer::from(uniforms.into_inner());
    uniform_buffer.write_buffer(&render_device, &queue);

    let output_vertex_buffer = render_device.create_buffer(&BufferDescriptor {
        label: Some("output vertex buffer"),
        size: 1024,
        usage: BufferUsages::STORAGE,
        mapped_at_creation: false,
    });

    let output_index_buffer = render_device.create_buffer(&BufferDescriptor {
        label: Some("output index buffer"),
        size: 1024,
        usage: BufferUsages::STORAGE,
        mapped_at_creation: false,
    });

    let bind_group = render_device.create_bind_group(
        Some("mesh creation bind group"),
        &pipeline_cache.get_bind_group_layout(&pipeline.mesh_bind_group_layout),
        &BindGroupEntries::sequential((
            BindingResource::Buffer(BufferBinding {
                buffer: input_vertex_slice.buffer,
                offset: input_offset,
                size: NonZero::new(input_size),
            }),
            output_vertex_buffer.as_entire_binding(),
            output_index_buffer.as_entire_binding(),
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
    *state = MeshCreationState::Finished;

    //print_shader_debug_value(&mut render_context, &debug_buffer);
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
