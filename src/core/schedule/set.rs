use bevy::prelude::*;

use super::state::AppState;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct SplashSet;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct GameSet;

pub(super) fn configure(app: &mut App) {
    app.configure_sets(
        Update,
        (
            SplashSet.run_if(
                in_state(AppState::Splash)
            ),
            GameSet.run_if(
                in_state(AppState::Game)
            ),
        )
    );
}