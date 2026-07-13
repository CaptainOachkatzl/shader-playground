use bevy::{mesh::CircleMeshBuilder, prelude::*};

use crate::playgrounds::PlaygroundScene;

pub struct SpecializedPipelinePlugin;

impl Plugin for SpecializedPipelinePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(PlaygroundScene::SpecializedPipeline),
            (setup, bevy::asset::handle_internal_asset_events).chain(),
        );
    }
}

fn setup(mut commands: Commands) {
    let scene = bsn! {
        Visibility::Visible
        Transform
        DespawnOnExit<PlaygroundScene>(PlaygroundScene::SpecializedPipeline)
        Children [
            // circular base
            Mesh3d(asset_value(CircleMeshBuilder::new(4.0, 256)))
            MeshMaterial3d::<StandardMaterial>(asset_value(Color::WHITE))
            Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),

            // cube
            Mesh3d(asset_value(Cuboid::new(1.0, 1.0, 1.0)))
            Transform::from_xyz(0.0, 0.5, 0.0),

            // light
            PointLight {
                shadow_maps_enabled: true,
            }
            Transform::from_xyz(4.0, 8.0, 4.0),
        ]

    };

    commands.spawn_scene(scene);
}

const SHADER_PATH: &str = "shaders/specialized_pipeline.wgsl";
