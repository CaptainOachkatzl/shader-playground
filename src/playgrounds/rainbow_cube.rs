use bevy::{
    asset::Asset,
    mesh::CircleMeshBuilder,
    mesh::{Mesh, MeshVertexBufferLayoutRef},
    pbr::{Material, MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypePath,
    render::render_resource::{
        AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError,
    },
    shader::ShaderRef,
};

use crate::playgrounds::PlaygroundScene;

pub struct RainbowCubePlugin;

impl Plugin for RainbowCubePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(PlaygroundScene::RainbowCube), setup)
            .add_systems(OnExit(PlaygroundScene::RainbowCube), cleanup)
            .add_systems(
                Update,
                update.run_if(in_state(PlaygroundScene::RainbowCube)),
            );
    }
}

#[derive(Component)]
struct SceneRoot;

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut custom_materials: ResMut<Assets<RainbowCubeMaterial>>,
) {
    let root = commands
        .spawn((SceneRoot, Transform::default(), Visibility::Visible))
        .id();

    commands.entity(root).with_children(|parent| {
        // circular base
        parent.spawn((
            Mesh3d(meshes.add(CircleMeshBuilder::new(4.0, 256))),
            MeshMaterial3d(standard_materials.add(Color::WHITE)),
            Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ));

        // cube
        let mesh = Mesh::from(Cuboid::new(1.0, 1.0, 1.0));
        parent.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(custom_materials.add(RainbowCubeMaterial {
                animation_progress: 0.0,
            })),
            Transform::from_xyz(0.0, 0.5, 0.0),
        ));
        // light
        parent.spawn((
            PointLight {
                shadow_maps_enabled: true,
                ..default()
            },
            Transform::from_xyz(4.0, 8.0, 4.0),
        ));
    });
}

fn cleanup(mut commands: Commands, root: Query<Entity, With<SceneRoot>>) {
    commands.entity(root.single().unwrap()).despawn();
}

fn update(time: Res<Time>, mut custom_materials: ResMut<Assets<RainbowCubeMaterial>>) {
    for (_, material) in custom_materials.iter_mut() {
        material.animation_progress = time.elapsed_secs() % 1.0;
    }
}

const SHADER_PATH: &str = "shaders/rainbow_cube.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct RainbowCubeMaterial {
    #[uniform(0)]
    pub animation_progress: f32,
}

impl Material for RainbowCubeMaterial {
    fn vertex_shader() -> ShaderRef {
        SHADER_PATH.into()
    }
    fn fragment_shader() -> ShaderRef {
        SHADER_PATH.into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}
