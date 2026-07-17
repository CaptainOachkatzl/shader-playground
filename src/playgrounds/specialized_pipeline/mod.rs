mod custom_rendered_mesh_pipeline;
mod explosion_particle;

use bevy::{
    math::ops::{cos, sin, sqrt},
    prelude::*,
};

use crate::playgrounds::{
    PlaygroundScene,
    specialized_pipeline::{
        custom_rendered_mesh_pipeline::{CustomRenderedEntity, CustomRenderedMeshPipelinePlugin},
        explosion_particle::ExplosionParticle,
    },
};

pub struct SpecializedPipelinePlugin;

impl Plugin for SpecializedPipelinePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CustomRenderedMeshPipelinePlugin)
            .insert_state(ExplosionType::RandomVelocity)
            .add_systems(
                OnEnter(PlaygroundScene::SpecializedPipeline),
                (setup, bevy::asset::handle_internal_asset_events).chain(),
            )
            .add_systems(Update, update);
    }
}

const PARTICLE_COUNT: usize = 1 << 10;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
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
        let initial_velocity = match explosion_type.get() {
            ExplosionType::ConstantExpansion => fibonacci_hemisphere_distribution(i),
            ExplosionType::RandomVelocity => randomized_vec3() * 4.0,
        };

        commands.spawn((
            DespawnOnExit(PlaygroundScene::SpecializedPipeline),
            // We use a marker component to identify the mesh that will be rendered
            // with our specialized pipeline
            CustomRenderedEntity,
            // We need to add the mesh handle to the entity
            Mesh3d(meshes.add(mesh.clone())),
            Transform::from_translation(initial_pos).with_scale(Vec3::splat(0.02)),
            InitialPosition(initial_pos),
            InitialVelocity(initial_velocity),
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
        let time_since_explosion = time.elapsed_secs() % 1.0 * 2.0;
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
