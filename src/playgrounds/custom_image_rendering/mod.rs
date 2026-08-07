pub mod custom_rendering_material;

use bevy::{
    mesh::RectangleMeshBuilder,
    prelude::*,
    sprite_render::Material2dPlugin,
};
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

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    camera_q: Query<Entity, With<Camera3d>>,
) {
    if let Ok(cam_entity) = camera_q.single() {
        commands.entity(cam_entity).despawn();
    }

    commands.spawn((
        Camera2d,
        DespawnOnExit(PlaygroundScene::CustomImageRendering),
    ));

    let image_width = 200;
    let image_height = 200;
    let material = CustomRenderMaterial::new(&mut images, image_width, image_height);

    let scene = bsn! {
        Visibility::Visible
        Transform
        DespawnOnExit<PlaygroundScene>(PlaygroundScene::CustomImageRendering)
        Children [
            Mesh2d(asset_value(RectangleMeshBuilder::new(image_width as f32, image_height as f32)))
            MeshMaterial2d::<CustomRenderMaterial>(asset_value(material))
            Transform,
        ]
    };

    commands.spawn_scene(scene);
}
