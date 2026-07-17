mod custom_rendered_mesh_pipeline;
mod explosion_particle;

use bevy::{
    math::ops::{cos, sin, sqrt},
    prelude::*,
};

use crate::playgrounds::{
    PlaygroundScene, specialized_pipeline::{
        custom_rendered_mesh_pipeline::{CustomRenderedEntity, CustomRenderedMeshPipelinePlugin}, explosion_particle::ExplosionParticle,
    },
};

pub struct SpecializedPipelinePlugin;

impl Plugin for SpecializedPipelinePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CustomRenderedMeshPipelinePlugin)
            .insert_state(ExplosionType::default())
            .add_systems(
                OnEnter(PlaygroundScene::SpecializedPipeline),
                (setup, bevy::asset::handle_internal_asset_events).chain(),
            )
            .add_systems(
                Update,
                update_constant_expansion.run_if(in_state(ExplosionType::ConstantExpansion)),
            );
    }
}



const PARTICLE_COUNT: usize = 1 << 10;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ExplosionType {
    #[default]
    ConstantExpansion,
}

/// Spawns the objects in the scene.
fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mesh = ExplosionParticle::mesh();

    for i in 0..PARTICLE_COUNT {
        commands.spawn((
            DespawnOnExit(PlaygroundScene::SpecializedPipeline),
            // We use a marker component to identify the mesh that will be rendered
            // with our specialized pipeline
            CustomRenderedEntity,
            // We need to add the mesh handle to the entity
            Mesh3d(meshes.add(mesh.clone())),
            Transform::from_translation(calculate_particle_position(i, 0.0))
                .with_scale(Vec3::splat(0.02)),
        ));
    }
}

fn update_constant_expansion(
    time: Res<Time>,
    mut particles: Query<&mut Transform, With<CustomRenderedEntity>>,
) {
    for (i, mut pos) in particles.iter_mut().enumerate() {
        pos.translation = calculate_particle_position(i, time.elapsed_secs() % 1.0 * 2.0);
    }
}

// time_since_explosion is between 0.0 and 1.0
fn calculate_particle_position(particle_index: usize, time_since_explosion: f32) -> Vec3 {
    let d0 = initial_velocity(particle_index);

    d0 * time_since_explosion
}

fn initial_velocity(particle_index: usize) -> Vec3 {
    const GOLDEN_ANGLE: f32 = 2.39996322972865332;

    let y = (particle_index as f32 + 0.5) / PARTICLE_COUNT as f32; // 0..1

    let phi = GOLDEN_ANGLE * particle_index as f32;

    let r = sqrt(1.0 - y * y);

    Vec3::new(r * sin(phi), y, r * cos(phi))
}
