pub struct CameraPlugin;

use std::f32::consts::PI;

use bevy::prelude::*;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
            .add_systems(Update, circle_camera);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-6.0, 4.5, 0.).looking_at(Vec3::ZERO, Vec3::Y),
        Msaa::Off,
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
