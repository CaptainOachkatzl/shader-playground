use std::f32::consts::PI;

use bevy::{mesh::CircleMeshBuilder, prelude::*};

pub struct ExampleScenePlugin;

impl Plugin for ExampleScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, circle_camera);
    }
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn((
        Mesh3d(meshes.add(CircleMeshBuilder::new(4.0, 256))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    // cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
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
    const CIRCLE_PERIOD: f32 = 10.0;

    for mut transform in cam_q.iter_mut() {
        let rotation = Quat::from_rotation_y(2.0 * PI * time.delta_secs() / CIRCLE_PERIOD);
        *transform = Transform::from_translation(rotation * transform.translation)
            .looking_at(Vec3::ZERO, Vec3::Y);
    }
}
