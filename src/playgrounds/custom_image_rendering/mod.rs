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
            RunInState(Update, change_data.run_if(trigger_data_change)),
        );
    }
}

#[derive(Resource)]
struct ImageHandle(Handle<Image>);

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    camera_q: Query<Entity, With<Camera3d>>,
) {
    if let Ok(cam_entity) = camera_q.single() {
        commands.entity(cam_entity).despawn();
    }

    let mut projection = OrthographicProjection::default_2d();
    projection.scale = 1. / 16.;
    commands.spawn((
        Camera2d,
        Projection::Orthographic(projection),
        DespawnOnExit(PlaygroundScene::CustomImageRendering),
    ));

    let image_width = 21;
    let image_height = 21;
    let material = CustomRenderMaterial::new(&mut images, image_width, image_height);
    let handle = material.data.clone();

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
    commands.insert_resource(ImageHandle(handle));
}

fn trigger_data_change(keyboard: Res<ButtonInput<KeyCode>>) -> bool {
    keyboard.just_pressed(KeyCode::Space)
}

fn change_data(mut images: ResMut<Assets<Image>>, handle: Res<ImageHandle>) {
    let mut image = images.get_mut(&handle.0).unwrap();
    image
        .data
        .as_mut()
        .unwrap()
        .iter_mut()
        .enumerate()
        .for_each(|(i, cell)| *cell = i as u8 % 2);
}
