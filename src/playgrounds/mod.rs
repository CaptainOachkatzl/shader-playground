pub mod basic_coloring;
pub mod paint_cube_face;
pub mod rainbow_cube;
pub mod hologram;
pub mod custom_render_phase;

use bevy::prelude::*;
use strum::{EnumIter, IntoEnumIterator};

#[derive(States, EnumIter, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlaygroundScene {
    #[default]
    BasicColoring,
    PaintCubeFace,
    RainbowCube,
    Hologram,
    CustomRenderPhase,
}

impl PlaygroundScene {
    fn get_key_code(&self) -> KeyCode {
        match self {
            PlaygroundScene::BasicColoring => KeyCode::Digit1,
            PlaygroundScene::PaintCubeFace => KeyCode::Digit2,
            PlaygroundScene::RainbowCube => KeyCode::Digit3,
            PlaygroundScene::Hologram => KeyCode::Digit4,
            PlaygroundScene::CustomRenderPhase => KeyCode::Digit5,
        }
    }
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
    for scene in PlaygroundScene::iter() {
        if key_input.just_pressed(scene.get_key_code()) {
            NextState::set_if_neq(&mut state, scene);
            return;
        }
    }
}
