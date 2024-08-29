use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::core::assets::{BlockTexturesAsset, UiAssets};

pub(super) fn configure(app: &mut App) {
    app
        .add_state::<AppState>()
        .add_loading_state(
            LoadingState::new(AppState::Splash).continue_to_state(AppState::Game),
        )
        .add_collection_to_loading_state::<_, UiAssets>(AppState::Splash)
        .add_collection_to_loading_state::<_, BlockTexturesAsset>(AppState::Splash);
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Splash,
    Game,
}