use bevy::prelude::*;

/// Storage primitives for storing voxel data
pub mod storage;

/// Manages the asset collection of Bevy Asset Loader
pub mod assets;

/// Utils for managing a voxel world.
mod world;
pub use world::*;

/// Terrain generator.
pub mod terraingen;

/// Systems and utilities for rendering voxels.
pub mod render;

/// Systems for defining voxel materials with physical properties.
pub mod material;

/// rust ports of signed distance field functions for use in world generation.
pub mod sdf;

/// TODO: Delete the voxel module in favor of the blocks module.
mod voxel;
pub use voxel::*;

/// Enums used throughout the project
pub mod enums;

/// Definition of a block and all of its properties.
mod blocks;
pub use blocks::*;

use self::player::PlayerSettings;

pub mod schedule;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        schedule::configure(app);

        blocks::add_systems(app);

        app
            .init_resource::<PlayerSettings>();
    }
}