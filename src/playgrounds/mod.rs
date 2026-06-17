pub mod basic_coloring;

use bevy::state::state::States;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlaygroundScene {
    #[default]
    BasicColoring,
}
