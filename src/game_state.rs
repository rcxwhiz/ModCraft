use bevy::prelude::*;

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy, Hash, States)]
pub enum GameState {
    #[default]
    Playing,
    EscapeMenu,
}
