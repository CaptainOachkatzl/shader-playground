pub mod custom_rendering_material;

use bevy::{mesh::RectangleMeshBuilder, prelude::*, sprite_render::Material2dPlugin};
use xs_bevy_state_scoped_systems::add_state_scoped_systems;

use crate::{
    camera::setup_3d_camera,
    playgrounds::{
        PlaygroundScene, custom_image_rendering::custom_rendering_material::CustomRenderMaterial,
    },
};

pub struct CustomImageRenderingPlugin;

impl Plugin for CustomImageRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<CustomRenderMaterial>::default());
        add_state_scoped_systems!(
            app,
            PlaygroundScene::CustomImageRendering,
            OnEnter((setup, bevy::asset::handle_internal_asset_events).chain()),
            OnExit(setup_3d_camera),
        );
    }
}

fn setup(mut commands: Commands, camera_q: Query<Entity, With<Camera3d>>) {
    if let Ok(cam_entity) = camera_q.single() {
        commands.entity(cam_entity).despawn();
    }

    commands.spawn((
        Camera2d,
        DespawnOnExit(PlaygroundScene::CustomImageRendering),
    ));

    let scene = bsn! {
        Visibility::Visible
        Transform
        DespawnOnExit<PlaygroundScene>(PlaygroundScene::CustomImageRendering)
        Children [
            Mesh2d(asset_value(RectangleMeshBuilder::new(200., 200.)))
            MeshMaterial2d::<CustomRenderMaterial>(asset_value(CustomRenderMaterial{})),
            Transform,
        ]
    };

    commands.spawn_scene(scene);
}
