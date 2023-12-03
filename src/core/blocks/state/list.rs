use bevy::prelude::AssetServer;

use super::{model::BlockModel, textures::BlockTextures, BlockState, StatesMap};

macro_rules! register_state {
    ($states:expr, $asset_server:expr, $block_id:expr, $state_id:expr, $textures:expr) => {
        register_state!(
            $states,
            $asset_server,
            $block_id,
            $state_id,
            $textures,
            match $textures.is_empty() {
                true => BlockModel::None,
                false => BlockModel::Standard,
            }
        );
    };
    ($states:expr, $asset_server:expr, $block_id:expr, $state_id:expr, $textures:expr, $model:expr) => {
        $states.insert(
            $state_id,
            BlockState {
                block_id: $block_id,
                state_id: $state_id,
                textures: match $textures.is_empty() {
                    true => BlockTextures::NONE,
                    false => BlockTextures::new($textures, $asset_server),
                },
                model: $model,
            }
        )
    };
}

macro_rules! register_state_range {
    ($states:expr, $asset_server:expr, $block_id:expr, $state_id_range:expr, $textures:expr) => {
        for state_id in $state_id_range {
            register_state!($states, $asset_server, $block_id, state_id, $textures);
        }
    };
    ($states:expr, $asset_server:expr, $block_id:expr, $state_id_range:expr, $textures:expr, $models:expr) => {
        for ((state_id, model), textures) in $state_id_range.zip_eq($models).zip_eq($textures) {
            register_state!($states, $asset_server, $block_id, state_id, textures, model);
        }
    };
}

static EMPTY: &[&str] = &[];

pub(super) fn create_states(states: &mut StatesMap, asset_server: &AssetServer) {
    register_state!(states, asset_server, 0u32, 0u32, EMPTY);
    register_state!(states, asset_server, 1u32, 1u32, &["stone.png"]);
    register_state!(states, asset_server, 2u32, 2u32, &["dirt.png"]);
    register_state_range!(
        states,
        asset_server,
        3u32,
        3u32..=4u32,
        &["grass_block_top.png", "dirt.png", "grass_block_side.png"]
    );
    register_state_range!(
        states,
        asset_server,
        4u32,
        5u32..=6u32,
        &["oak_log_top.png", "oak_log.png"]
    );
    
}