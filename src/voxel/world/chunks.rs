use bevy::{
    math::IVec3,
    prelude::{
        Changed, Commands, Entity, GlobalTransform, IntoSystemConfigs,
        Plugin, Query, Res, ResMut, Resource, SystemSet, With, Vec3, Update, PostUpdate, Last,
    },
    utils::{HashMap, HashSet}, pbr::wireframe::Wireframe, gizmos::AabbGizmo, render::color::Color, ecs::schedule::common_conditions::in_state,
};
use float_ord::FloatOrd;

use super::{player::PlayerController, Chunk, ChunkShape, CHUNK_LENGTH};
use crate::{voxel::storage::ChunkMap, AppState};
use crate::voxel::Voxel;

/// Gets the chunk for any given position.
pub fn get_chunk_for_pos(pos: Vec3) -> Vec3 {
    Vec3::new(
        pos.x.div_euclid(CHUNK_LENGTH as f32) * CHUNK_LENGTH as f32,
        0f32,
        pos.z.div_euclid(CHUNK_LENGTH as f32) * CHUNK_LENGTH as f32,
    )
}

/// Updates the current chunk position for the current player.
fn update_player_pos(
    player: Query<&GlobalTransform, (With<PlayerController>, Changed<GlobalTransform>)>,
    mut chunk_pos: ResMut<CurrentLocalPlayerChunk>,
) {
    if let Ok(ply) = player.get_single() {
        let player_coords = ply.translation();

        let nearest_chunk_origin = get_chunk_for_pos(player_coords);

        chunk_pos.world_pos = player_coords;
        if chunk_pos.chunk_min != nearest_chunk_origin.as_ivec3() {

            chunk_pos.chunk_min = nearest_chunk_origin.as_ivec3();
        }
    }
}

/// Checks for the loaded chunks around the player and schedules loading of new chunks in sight
fn update_view_chunks(
    player_pos: Res<CurrentLocalPlayerChunk>,
    chunk_entities: Res<ChunkEntities>,
    view_radius: Res<ChunkLoadRadius>,
    mut chunk_command_queue: ResMut<ChunkCommandQueue>,
) {
    // quick n dirty circular chunk loading.
    //perf: optimize this.
    for x in -view_radius.horizontal..view_radius.horizontal {
        for z in -view_radius.horizontal..view_radius.horizontal {
            // for y in -view_radius.vertical..view_radius.vertical {
                if x.pow(2) + z.pow(2) >= view_radius.horizontal.pow(2) {
                    continue;
                }

                let chunk_key = {
                    let pos: IVec3 = player_pos.chunk_min
                        + IVec3::new(
                            x * CHUNK_LENGTH as i32,
                            // y * CHUNK_HEIGHT as i32,
                            0,
                            z * CHUNK_LENGTH as i32,
                        );

                    // pos.y = pos.y.max(0);

                    pos
                };

                if chunk_entities.entity(chunk_key).is_none() {
                    chunk_command_queue.create.push(chunk_key);
                }
            // }
        }
    }

    // quick n dirty circular chunk !loading.
    for loaded_chunk in chunk_entities.0.keys() {
        let delta: IVec3 = *loaded_chunk - player_pos.chunk_min;

        // Compiler complains that this is a bug
        #[allow(clippy::suspicious_operation_groupings)]
        if delta.x.pow(2) + delta.z.pow(2)
            > view_radius.horizontal.pow(2) * (CHUNK_LENGTH as i32).pow(2)
            // || delta.y.pow(2) > view_radius.vertical.pow(2) * (CHUNK_HEIGHT as i32).pow(2)
        {
            chunk_command_queue.destroy.push(*loaded_chunk);
        }
    }

    // load chunks starting from the player position
    chunk_command_queue.create.sort_unstable_by_key(|key| {
        FloatOrd(key.as_vec3().distance(player_pos.chunk_min.as_vec3()))
    });
}

/// Creates the requested chunks and attach them an ECS entity.
fn create_chunks(
    mut chunks_command_queue: ResMut<ChunkCommandQueue>,
    mut chunk_entities: ResMut<ChunkEntities>,
    mut cmds: Commands,
) {
    chunks_command_queue
        .create
        .drain(..)
        .for_each(|request|
            chunk_entities
            .attach_entity(
                request,
                cmds.spawn((
                    Chunk(request),
                    AabbGizmo { ..Default::default() },
                    // Wireframe
                )).id()
            )
        );
}

fn destroy_chunks(
    mut chunks_command_queue: ResMut<ChunkCommandQueue>,
    mut chunks: ResMut<ChunkMap<Voxel, ChunkShape>>,
    mut chunk_entities: ResMut<ChunkEntities>,
    mut cmds: Commands,
) {
    for command in chunks_command_queue.destroy.drain(..) {
        cmds.entity(chunk_entities.detach_entity(command).unwrap())
            .despawn();
        chunks.remove(command);
    }
}

fn clear_dirty_chunks(mut dirty_chunks: ResMut<DirtyChunks>) {
    dirty_chunks.0.clear();
}

/// Label for the stage housing the chunk loading systems.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash, SystemSet)]
pub struct ChunkLoadingSet;

/// Handles dynamically loading / unloading regions (aka chunks) of the world according to camera position.
pub struct VoxelWorldChunkingPlugin;

/// Stores the Entity <-> Chunk voxel data buffer mapping
#[derive(Default, Resource)]
pub struct ChunkEntities(HashMap<IVec3, Entity>);

impl ChunkEntities {
    /// Returns the entity attached to the chunk.
    pub fn entity(&self, pos: IVec3) -> Option<Entity> {
        self.0.get(&pos).copied()
    }

    /// Attaches the specified entity to the chunk data.
    pub fn attach_entity(&mut self, pos: IVec3, entity: Entity) {
        self.0.insert(pos, entity);
    }

    /// Detaches the specified entity to the chunk data.
    pub fn detach_entity(&mut self, pos: IVec3) -> Option<Entity> {
        self.0.remove(&pos)
    }

    /// Returns an iterator iterating over the loaded chunk keys.
    pub fn iter_keys(&self) -> impl Iterator<Item = &IVec3> {
        self.0.keys()
    }

    /// Return the number of loaded chunks.
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

/// Holds the dirty chunk for the current frame.
#[derive(Default, Resource)]
pub struct DirtyChunks(HashSet<IVec3>);

#[allow(dead_code)]
impl DirtyChunks {
    pub fn mark_dirty(&mut self, chunk: IVec3) {
        self.0.insert(chunk);
    }

    pub fn iter_dirty(&self) -> impl Iterator<Item = &IVec3> {
        self.0.iter()
    }

    pub fn num_dirty(&self) -> usize {
        self.0.len()
    }
}

/// Resource storing the current chunk the player is in as well as its current coords.
#[derive(Resource)]
pub struct CurrentLocalPlayerChunk {
    pub chunk_min: IVec3,
    pub world_pos: Vec3,
}

// Resource holding the view distance.
#[derive(Resource)]
pub struct ChunkLoadRadius {
    pub horizontal: i32,
    // pub vertical: i32,
}

/// A queue tracking the creation / destroy commands for chunks.
#[derive(Default, Resource)]
pub struct ChunkCommandQueue {
    create: Vec<IVec3>,
    destroy: Vec<IVec3>,
}

impl ChunkCommandQueue {
    pub fn queue_unload<'a>(&mut self, region: impl Iterator<Item = &'a IVec3>) {
        self.destroy.extend(region);
    }
}

impl Plugin for VoxelWorldChunkingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource::<ChunkLoadRadius>(ChunkLoadRadius {
            horizontal: 8,
            // vertical: 1,
        })
        .init_resource::<ChunkEntities>()
        .insert_resource(CurrentLocalPlayerChunk {
            chunk_min: IVec3::ZERO,
            world_pos: Vec3::ZERO,
        })
        .init_resource::<ChunkCommandQueue>()
        .init_resource::<DirtyChunks>()
        .configure_sets(
            Update,
            ChunkLoadingSet
        )
        .add_systems(
            Update,
            (update_player_pos, update_view_chunks, create_chunks)
                .chain()
                .in_set(ChunkLoadingSet)
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            PostUpdate,
            destroy_chunks.run_if(in_state(AppState::InGame))
        )
        .add_systems(
            Last,
            clear_dirty_chunks.run_if(in_state(AppState::InGame))
        );
    }
}