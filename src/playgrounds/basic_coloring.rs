use bevy::{
    mesh::{CircleMeshBuilder, MeshVertexAttribute, MeshVertexBufferLayoutRef, VertexFormat},
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::render_resource::{
        AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError,
    },
    shader::ShaderRef,
};
use xs_bevy_state_scoped_systems::add_state_scoped_systems;

use crate::playgrounds::PlaygroundScene;

pub struct BasicColoringPlugin;

impl Plugin for BasicColoringPlugin {
    fn build(&self, app: &mut App) {
        add_state_scoped_systems!(
            app,
            PlaygroundScene::BasicColoring,
            OnEnter(setup),
            OnExit(cleanup),
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
    mut custom_materials: ResMut<Assets<BasicColoringMaterial>>,
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

        // The cube mesh has 24 vertices (6 faces, 4 vertices per face), so we insert one BlendColor for each
        let colors: Vec<_> = (0..24)
            .map(|index| {
                // use HSV color space to circle between red, green and blue.
                // advancing by 120 degrees shifts to the next base color, starting with red at 0 degrees.
                Into::<LinearRgba>::into(Hsva::new(index as f32 * 120.0 % 360.0, 1.0, 1.0, 1.0))
                    .to_f32_array()
            })
            .collect();

        // cube
        let mesh = Mesh::from(Cuboid::new(1.0, 1.0, 1.0))
            .with_inserted_attribute(ATTRIBUTE_BLEND_COLOR, colors);

        parent.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(custom_materials.add(BasicColoringMaterial {
                color: LinearRgba::WHITE,
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

const SHADER_PATH: &str = "shaders/custom_shader.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct BasicColoringMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
}

// A "high" random id should be used for custom attributes to ensure consistent sorting and avoid collisions with other attributes.
// See the MeshVertexAttribute docs for more info.
pub const ATTRIBUTE_BLEND_COLOR: MeshVertexAttribute =
    MeshVertexAttribute::new("BlendColor", 988540917, VertexFormat::Float32x4);

impl Material for BasicColoringMaterial {
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
            ATTRIBUTE_BLEND_COLOR.at_shader_location(1),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}
