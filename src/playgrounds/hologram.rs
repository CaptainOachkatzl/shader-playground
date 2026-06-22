use bevy::{
    asset::Asset,
    mesh::CircleMeshBuilder,
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
};

use crate::playgrounds::PlaygroundScene;

pub struct HologramPlugin;

impl Plugin for HologramPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(PlaygroundScene::Hologram),
            (setup, bevy::asset::handle_internal_asset_events).chain(),
        );
    }
}

fn setup(mut commands: Commands) {
    let scene = bsn! {
        Visibility::Visible
        Transform
        DespawnOnExit<PlaygroundScene>(PlaygroundScene::Hologram)
        Children [
            // circular base
            Mesh3d(asset_value(CircleMeshBuilder::new(4.0, 256)))
            MeshMaterial3d::<StandardMaterial>(asset_value(Color::WHITE))
            Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),

            // cube
            Mesh3d(asset_value(Cuboid::new(1.0, 1.0, 1.0)))
            MeshMaterial3d::<HologramMaterial>(asset_value(HologramMaterial{
                base: StandardMaterial::default(),
                extension: HologramExtension {  },

            }))
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

const SHADER_PATH: &str = "shaders/hologram.wgsl";

pub type HologramMaterial = ExtendedMaterial<StandardMaterial, HologramExtension>;

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct HologramExtension {}

impl MaterialExtension for HologramExtension {
    // fn vertex_shader() -> ShaderRef {
    //     SHADER_PATH.into()
    // }

    fn fragment_shader() -> ShaderRef {
        SHADER_PATH.into()
    }

    fn alpha_mode() -> Option<AlphaMode> {
        Some(AlphaMode::Blend)
    }
}
