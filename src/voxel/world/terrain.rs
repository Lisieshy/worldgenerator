use std::path::PathBuf;
use anyhow::Result;

use super::{
    chunks::{ChunkLoadingSet, DirtyChunks},
    Chunk, ChunkShape, WorldSettings,
};
use crate::voxel::{
    storage::{ChunkMap, VoxelBuffer},
    terraingen::TERRAIN_GENERATOR,
    Voxel,
};
use bevy::{
    prelude::{
        Added, Commands, Component, Entity, IntoSystemConfigs, IntoSystemSetConfigs,
        Plugin, Query, ResMut, SystemSet, Update,
    },
    tasks::{AsyncComputeTaskPool, Task}, ecs::{world, system::Res}, log::info, math::IVec3,
};
use directories::BaseDirs;
use futures_lite::future;

pub fn save_chunk_to_disk(
    chunk_data: &VoxelBuffer<Voxel, ChunkShape>,
    key: IVec3,
    world_name: &'static str,
) -> Result<()> {
    // getting the directory
    if let Some(base_dirs) = BaseDirs::new() {
        // creating the saved_worlds + world name directory, nothing happens if it already exists.
        let saves_dir = base_dirs.data_dir().join(".yavafg").join("saved_worlds").join(world_name);
        std::fs::create_dir_all(saves_dir.as_path())?;

        // chunk isn't already saved on disk, so we generate it and save it.
        let encoded_chunk_data: Vec<u8> = bincode::serialize(&chunk_data)?;
        let chunk_path = saves_dir.join(format!("{}.chunk", key));
        info!("saving chunk to {:?}", chunk_path);
        std::fs::write(chunk_path, encoded_chunk_data)?;
        Ok(())
    } else {
        panic!("No valid directory path could be retrieved from the operating system.");
    }
}

pub fn load_chunk_from_disk(
    key: IVec3,
    world_name: &'static str,
) -> Result<Option<VoxelBuffer<Voxel, ChunkShape>>> {
    // getting the directory
    if let Some(base_dirs) = BaseDirs::new() {
        // creating the saved_worlds + world name directory, nothing happens if it already exists.
        let saves_dir = base_dirs.data_dir().join(".yavafg").join("saved_worlds").join(world_name);
        std::fs::create_dir_all(saves_dir.as_path())?;

        // checking if the chunk file exists
        let chunk_path = saves_dir.join(format!("{}.chunk", key));
        if chunk_path.exists() {
            info!("loading chunk from {:?}", chunk_path);
            let encoded_chunk_data = std::fs::read(chunk_path)?;
            let chunk_data: VoxelBuffer<Voxel, ChunkShape> = bincode::deserialize(&encoded_chunk_data)?;
            Ok(Some(chunk_data))
        } else {
            Ok(None)
        }
    } else {
        panic!("No valid directory path could be retrieved from the operating system.");
    }
}


/// Queues the terrain gen async tasks for the newly created chunks.
fn queue_terrain_gen(
    mut commands: Commands,
    new_chunks: Query<(Entity, &Chunk), Added<Chunk>>,
    world_settings: Res<WorldSettings>,
) {
    let task_pool = AsyncComputeTaskPool::get();

    let seed = world_settings.seed;
    let name = world_settings.name;

    let task_gen = |key, seed, name| {
        match load_chunk_from_disk(key, name) {
            Ok(Some(chunk_data)) => {
                chunk_data
            },
            Ok(None) | Err(_) => {
                let mut chunk_data = VoxelBuffer::<Voxel, ChunkShape>::new_empty(ChunkShape {});
                TERRAIN_GENERATOR
                    .read()
                    .unwrap()
                    .generate(key, &mut chunk_data, seed);
                let _ = save_chunk_to_disk(&chunk_data, key, name);
                chunk_data
            }
        }

        // That one deserves its own blogpost
        // b is some but it still enters the unwrap_or
        // Meanwhile the match case works fine

        // let a = load_chunk_from_disk(key, name);
        // info!("a: {:?}", a.is_ok());
        // let b = a.ok().flatten();
        // info!("b: {:?}", b.is_some());
        // b.unwrap_or({
        //         let mut chunk_data = VoxelBuffer::<Voxel, ChunkShape>::new_empty(ChunkShape {});
        //         TERRAIN_GENERATOR
        //             .read()
        //             .unwrap()
        //             .generate(key, &mut chunk_data, seed);
        //         let _ = save_chunk_to_disk(&chunk_data, key, name);
        //         chunk_data
        //     })
    };

    new_chunks
        .iter()
        .map(|(entity, key)| (entity, key.0))
        .map(|(entity, key)| {
            (
                entity,
                (TerrainGenTask(task_pool.spawn(async move {
                    task_gen(key, seed.clone(), name)
                }))),
            )
        })
        .for_each(|(entity, gen_task)| {
            commands.entity(entity).insert(gen_task);
        });
}

fn wrap_up(
    mut chunk_data: ResMut<ChunkMap<Voxel, ChunkShape>>,
    mut commands: Commands,
    mut dirty_chunks: ResMut<DirtyChunks>,
    mut generated_chunks: Query<(Entity, &Chunk, &mut TerrainGenTask)>,
) {
    generated_chunks.for_each_mut(|(entity, chunk, mut gen_task)| {
        if let Some(data) = future::block_on(future::poll_once(&mut gen_task.0)) {
            chunk_data.insert(chunk.0, data);
            dirty_chunks.mark_dirty(chunk.0);
            commands.entity(entity).remove::<TerrainGenTask>();
        }
    });
}

/// Handles terrain generation.
pub struct VoxelWorldTerrainGenPlugin;

// we need to use a whole system stage for this in order to enable the usage of added component querries.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash, SystemSet)]
pub struct TerrainGenSet;

impl Plugin for VoxelWorldTerrainGenPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.configure_sets(
            Update,
            TerrainGenSet
                .after(ChunkLoadingSet),
        )
        .add_systems(
            Update,
            (queue_terrain_gen, wrap_up)
                .chain()
                .in_set(TerrainGenSet),
        );
    }
}

#[derive(Component)]
pub struct TerrainGenTask(Task<VoxelBuffer<Voxel, ChunkShape>>);