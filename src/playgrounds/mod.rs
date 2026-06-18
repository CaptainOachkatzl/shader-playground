pub mod basic_coloring;
pub mod paint_cube_face;

use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlaygroundScene {
    #[default]
    BasicColoring,
    PaintCubeFace,
}

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<PlaygroundScene>()
            .add_systems(Update, switch_scenes);
    }
}

fn switch_scenes(
    key_input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<NextState<PlaygroundScene>>,
) {
    if key_input.just_pressed(KeyCode::Digit1) {
        state.set(PlaygroundScene::BasicColoring);
    } else if key_input.just_pressed(KeyCode::Digit2) {
        state.set(PlaygroundScene::PaintCubeFace);
    }
}
