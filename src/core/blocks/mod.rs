use bevy::{prelude::{App, AssetServer, Commands, Res, Resource, Startup}, ecs::schedule::{IntoSystemConfigs, common_conditions::in_state}};

pub use self::{block::Blocks, state::BlockStates};

pub mod block;
pub mod state;

#[derive(Debug, Clone, Resource)]
pub struct BlockData {
    pub blocks: Blocks,
    pub states: BlockStates,
}

impl BlockData {
    fn create(asset_serve: Res<AssetServer>, mut commands: Commands) {
        let blocks = Blocks::create();
        let states = BlockStates::create(&asset_serve);

        commands.insert_resource(BlockData { blocks, states });
    }
}

pub(super) fn add_systems(app: &mut App) {
    app.add_systems(Startup, BlockData::create);
}