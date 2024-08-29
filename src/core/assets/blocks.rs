use bevy::prelude::*;
use bevy_asset_loader::prelude::*;


#[derive(AssetCollection, Resource)]
pub struct BlockTexturesAsset {
    // #[asset(path = "textures/tiles", collection(typed))]
    // tiles: Vec<Handle<Image>>,

    #[asset(path = "textures/blocks/stone.png")]
    pub void: Handle<Image>,
    #[asset(path = "textures/blocks/bedrock.png")]
    pub bedrock: Handle<Image>,
    #[asset(path = "textures/blocks/stone.png")]
    pub rock: Handle<Image>,
    #[asset(path = "textures/blocks/dirt.png")]
    pub dirt: Handle<Image>,
    #[asset(path = "textures/blocks/sand.png")]
    pub sand: Handle<Image>,
    #[asset(path = "textures/blocks/grass_block_top.png")]
    pub grass: Handle<Image>,
    #[asset(path = "textures/blocks/snow.png")]
    pub snow: Handle<Image>,
    #[asset(path = "textures/blocks/water.png")]
    pub water: Handle<Image>,
    #[asset(path = "textures/blocks/sandstone_top.png")]
    pub sandstone: Handle<Image>,
    #[asset(path = "textures/blocks/cactus_top.png")]
    pub cactus: Handle<Image>,
    #[asset(path = "textures/blocks/oak_log.png")]
    pub wood: Handle<Image>,
    #[asset(path = "textures/blocks/oak_leaves.png")]
    pub leaves: Handle<Image>,
    #[asset(path = "textures/blocks/spruce_leaves.png")]
    pub pineleaves: Handle<Image>,
    #[asset(path = "textures/blocks/spruce_log.png")]
    pub pinewood: Handle<Image>,
}