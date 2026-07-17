use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};

#[derive(Component)]
pub struct ExplosionParticle(pub usize);

impl ExplosionParticle {
    pub fn mesh() -> Mesh {
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_indices(Indices::U32(vec![0, 1, 2, 3, 2, 1]))
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![
                vec3(0.0, -1.0, 0.0),
                vec3(1.0, 0.0, 0.0),
                vec3(-1.0, 0.0, 0.0),
                vec3(0.0, 1.0, 0.0),
            ],
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_COLOR,
            vec![
                vec4(1.0, 1.0, 0.0, 1.0),
                vec4(1.0, 0.5, 0.0, 1.0),
                vec4(1.0, 0.5, 0.0, 1.0),
                vec4(1.0, 0.0, 0.0, 1.0),
            ],
        )
    }
}
