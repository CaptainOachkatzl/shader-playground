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
        app.add_systems(
            OnEnter(PlaygroundScene::RainbowCube),
            (
                setup,
                bevy::asset::handle_internal_asset_events,
                test_setup,
            )
                .chain(),
        )
        .add_systems(
            Update,
            update.run_if(in_state(PlaygroundScene::RainbowCube)),
        );
    }
}

#[derive(Component, Default, Clone)]
struct SceneRoot;

#[derive(Component, Default, Clone)]
struct Debug;

fn setup(mut commands: Commands) {
    commands.spawn_scene(create_scene());
}

fn test_setup(
    meshes: Res<Assets<Mesh>>,
    name_q: Query<&Name, With<SceneRoot>>,
    debug_q: Query<&Mesh3d, With<Debug>>,
) {
    for name in name_q {
        println!("{:?}", name);
    }

    for handle in debug_q {
        println!("{:?}", meshes.get(&handle.0));
    }
}

fn create_scene() -> impl Scene {
    bsn! {
        #MyScene
        SceneRoot
        Visibility::Visible
        Transform
        DespawnOnExit<PlaygroundScene>(PlaygroundScene::RainbowCube)
        Children [
            // circular base
            Mesh3d(asset_value(CircleMeshBuilder::new(4.0, 256)))
            MeshMaterial3d::<StandardMaterial>(asset_value(Color::WHITE))
            Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),

            // cube
            Debug
            Mesh3d(asset_value(Cuboid::new(1.0, 1.0, 1.0)))
            MeshMaterial3d::<RainbowCubeMaterial>(asset_value(RainbowCubeMaterial {
                animation_progress: 0.0,
            }))
            Transform::from_xyz(0.0, 0.5, 0.0),

            // light
            PointLight {
                shadow_maps_enabled: true,
            }
            Transform::from_xyz(4.0, 8.0, 4.0),
        ]

    }
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
