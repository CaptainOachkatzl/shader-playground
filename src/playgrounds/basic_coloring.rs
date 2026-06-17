use std::f32::consts::PI;

use bevy::{mesh::CircleMeshBuilder, prelude::*};

use crate::{
    custom_material::{ATTRIBUTE_BLEND_COLOR, CustomMaterial},
    playgrounds::PlaygroundScene,
};

pub struct BasicColoringPlugin;

impl Plugin for BasicColoringPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(PlaygroundScene::BasicColoring), setup)
            .add_systems(
                Update,
                circle_camera.run_if(in_state(PlaygroundScene::BasicColoring)),
            );
    }
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut custom_materials: ResMut<Assets<CustomMaterial>>,
) {
    // circular base
    commands.spawn((
        Mesh3d(meshes.add(CircleMeshBuilder::new(4.0, 256))),
        MeshMaterial3d(standard_materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));

    // The cube mesh has 24 vertices (6 faces, 4 vertices per face), so we insert one BlendColor for each
    let colors: Vec<_> = (0..24)
        .map(|index| {
            // use HSV color space to circle between red, green and blue.
            // advancing by 120 degrees shifts to the next base color, starting with red at 0 degrees.
            let color =
                Into::<LinearRgba>::into(Hsva::new(index as f32 * 120.0 % 360.0, 1.0, 1.0, 1.0))
                    .to_f32_array();
            println!("{:?}", color);

            color
        })
        .collect();

    // cube
    let mesh = Mesh::from(Cuboid::new(1.0, 1.0, 1.0))
        .with_inserted_attribute(ATTRIBUTE_BLEND_COLOR, colors);

    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(custom_materials.add(CustomMaterial {
            color: LinearRgba::WHITE,
        })),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-6.0, 4.5, 0.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn circle_camera(time: Res<Time<Real>>, mut cam_q: Query<&mut Transform, With<Camera3d>>) {
    const CIRCLE_PERIOD: f32 = 30.0;

    for mut transform in cam_q.iter_mut() {
        let rotation = Quat::from_rotation_y(2.0 * PI * time.delta_secs() / CIRCLE_PERIOD);
        *transform = Transform::from_translation(rotation * transform.translation)
            .looking_at(Vec3::ZERO, Vec3::Y);
    }
}
