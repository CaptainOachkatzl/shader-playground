use bevy::{mesh::CircleMeshBuilder, prelude::*};

use crate::{
    custom_material::{ATTRIBUTE_BLEND_COLOR, CustomMaterial},
    playgrounds::PlaygroundScene,
};

pub struct TestPlugin;

impl Plugin for TestPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(PlaygroundScene::Test), setup)
            .add_systems(OnExit(PlaygroundScene::Test), cleanup);
    }
}

#[derive(Component)]
struct SceneRoot;

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut custom_materials: ResMut<Assets<CustomMaterial>>,
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
            .map(|_| {
                // use HSV color space to circle between red, green and blue.
                // advancing by 120 degrees shifts to the next base color, starting with red at 0 degrees.
                let color = Into::<LinearRgba>::into(Hsva::new(0.0, 1.0, 1.0, 1.0)).to_f32_array();

                color
            })
            .collect();

        // cube
        let mesh = Mesh::from(Cuboid::new(1.0, 1.0, 1.0))
            .with_inserted_attribute(ATTRIBUTE_BLEND_COLOR, colors);

        parent.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(custom_materials.add(CustomMaterial {
                color: LinearRgba::WHITE,
            })),
            Transform::from_xyz(0.0, 0.5, 0.0),
        ));
        // light
        parent.spawn((
            PointLight {
                shadows_enabled: true,
                ..default()
            },
            Transform::from_xyz(4.0, 8.0, 4.0),
        ));
    });
}

fn cleanup(mut commands: Commands, root: Query<Entity, With<SceneRoot>>) {
    commands.entity(root.single().unwrap()).despawn();
}
