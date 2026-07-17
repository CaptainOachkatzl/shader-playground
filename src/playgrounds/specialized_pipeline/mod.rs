mod custom_rendered_mesh_pipeline;
mod explosion_particle;
mod ui;

use bevy::{
    math::ops::{cos, sin, sqrt},
    prelude::*,
};

use crate::playgrounds::{
    PlaygroundScene,
    specialized_pipeline::{
        custom_rendered_mesh_pipeline::{CustomRenderedEntity, CustomRenderedMeshPipelinePlugin},
        explosion_particle::ExplosionParticle,
        ui::UiPlugin,
    },
};

pub struct SpecializedPipelinePlugin;

impl Plugin for SpecializedPipelinePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((CustomRenderedMeshPipelinePlugin, UiPlugin))
            .insert_state(ExplosionType::RandomVelocity)
            .add_systems(
                OnEnter(PlaygroundScene::SpecializedPipeline),
                (setup, bevy::asset::handle_internal_asset_events).chain(),
            )
            .add_systems(
                Update,
                update.run_if(in_state(PlaygroundScene::SpecializedPipeline)),
            )
            .add_systems(
                Update,
                on_change_explosion_type.run_if(state_changed::<ExplosionType>),
            );
    }
}

const PARTICLE_COUNT: usize = 1 << 10;

#[derive(States, strum::Display, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ExplosionType {
    #[default]
    ConstantExpansion,
    RandomVelocity,
}

#[derive(Component)]
struct InitialPosition(pub Vec3);

#[derive(Component)]
struct InitialVelocity(pub Vec3);

/// Spawns the objects in the scene.
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    explosion_type: Res<State<ExplosionType>>,
) {
    let mesh = ExplosionParticle::mesh();

    for i in 0..PARTICLE_COUNT {
        let initial_pos = 0.1 * fibonacci_hemisphere_distribution(i);

        commands.spawn((
            ExplosionParticle(i),
            DespawnOnExit(PlaygroundScene::SpecializedPipeline),
            // We use a marker component to identify the mesh that will be rendered
            // with our specialized pipeline
            CustomRenderedEntity,
            // We need to add the mesh handle to the entity
            Mesh3d(meshes.add(mesh.clone())),
            Transform::from_translation(initial_pos).with_scale(Vec3::splat(0.02)),
            InitialPosition(initial_pos),
            InitialVelocity(get_initial_velocity(i, &explosion_type)),
        ));
    }
}

fn update(
    time: Res<Time>,
    mut particles: Query<
        (&mut Transform, &InitialPosition, &InitialVelocity),
        With<CustomRenderedEntity>,
    >,
) {
    for (mut pos, InitialPosition(pos0), InitialVelocity(v0)) in particles.iter_mut() {
        let time_since_explosion = time.elapsed_secs() % 1.0;
        pos.translation = pos0 + v0 * time_since_explosion;
    }
}

fn fibonacci_hemisphere_distribution(particle_index: usize) -> Vec3 {
    const GOLDEN_ANGLE: f32 = 2.39996322972865332;

    let y = (particle_index as f32 + 0.5) / PARTICLE_COUNT as f32; // 0..1

    let phi = GOLDEN_ANGLE * particle_index as f32;

    let r = sqrt(1.0 - y * y);

    Vec3::new(r * sin(phi), y, r * cos(phi))
}

fn randomized_vec3() -> Vec3 {
    fn random_f32_neg_1_to_pos_1() -> f32 {
        rand::random::<f32>() * 2.0 - 1.0
    }

    loop {
        if let Some(normalized) = Vec3::new(
            random_f32_neg_1_to_pos_1(),
            rand::random::<f32>(), // y >= 0 to only move to northern hemisphere
            random_f32_neg_1_to_pos_1(),
        )
        .try_normalize()
        {
            return normalized * rand::random::<f32>();
        }
    }
}

fn get_initial_velocity(particle_index: usize, explosion_type: &ExplosionType) -> Vec3 {
    2.0 * match explosion_type {
        ExplosionType::ConstantExpansion => fibonacci_hemisphere_distribution(particle_index),
        ExplosionType::RandomVelocity => randomized_vec3(),
    }
}

fn on_change_explosion_type(
    explosion_type: Res<State<ExplosionType>>,
    particle_q: Query<(&ExplosionParticle, &mut InitialVelocity)>,
) {
    for (ExplosionParticle(index), mut initial_velocity) in particle_q {
        *initial_velocity = InitialVelocity(get_initial_velocity(*index, &explosion_type))
    }
}
