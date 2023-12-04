use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource)]
pub struct UiAssets {
    #[asset(path = "textures/crosshair.png")]
    pub crosshair: Handle<Image>,
}

