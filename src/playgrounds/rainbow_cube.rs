use bevy::{
    asset::Asset,
    mesh::CircleMeshBuilder,
    mesh::{Mesh, MeshVertexAttribute, MeshVertexBufferLayoutRef, VertexFormat},
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
            .add_systems(OnExit(PlaygroundScene::RainbowCube), cleanup);
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
        let mut face_ids = Vec::new();

        // bevy cuboid is created with 4 vertices for each face
        for face in 0..6u32 {
            for _ in 0..4 {
                face_ids.push(face);
            }
        }

        let mesh = Mesh::from(Cuboid::new(1.0, 1.0, 1.0))
            .with_inserted_attribute(ATTRIBUTE_FACE_ID, face_ids);

        parent.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(custom_materials.add(RainbowCubeMaterial {})),
            Transform::from_xyz(0.0, 0.5, 0.0),
        ));
        // light
        parent.spawn((
            PointLight {
                shadows_enabled: true,
                ..default()
            },
            Transform::from_xyz(4.0, 8.0, 4.0),
        ));
    });
}

fn cleanup(mut commands: Commands, root: Query<Entity, With<SceneRoot>>) {
    commands.entity(root.single().unwrap()).despawn();
}

const SHADER_PATH: &str = "shaders/rainbow_cube.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct RainbowCubeMaterial {}

const ATTRIBUTE_FACE_ID: MeshVertexAttribute =
    MeshVertexAttribute::new("FaceId", 987654322, VertexFormat::Uint32);

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
            ATTRIBUTE_FACE_ID.at_shader_location(3),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}
