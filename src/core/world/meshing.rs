use std::cell::RefCell;

use super::{
    chunks::{ChunkEntities, ChunkLoadingSet, DirtyChunks},
    terrain::{TerrainGenSet, save_chunk_to_disk},
    Chunk, ChunkShape, Voxel, CHUNK_LENGTH, CHUNK_HEIGHT, WorldSettings,
};
use crate::{core::{
    render::{mesh_buffer, MeshBuffers, ChunkMaterialSingleton },
    storage::ChunkMap,
}, AppState};
use bevy::{
    prelude::*,
    render::{primitives::Aabb, render_resource::PrimitiveTopology },
    tasks::{AsyncComputeTaskPool, Task},
};
use futures_lite::future;
use once_cell::sync::Lazy;
use thread_local::ThreadLocal;

/// Attaches to the newly inserted chunk entities components required for rendering.
pub fn prepare_chunks(
    chunks: Query<(Entity, &Chunk), Added<Chunk>>,
    mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<GpuTerrainMaterial>>,
    material: Res<ChunkMaterialSingleton>,
    mut cmds: Commands,
    // assets: Res<MyAssets>,
    // asset_server: Res<AssetServer>,
) {
    for (chunk, chunk_key) in chunks.iter() {
        let mut entity_commands = cmds.entity(chunk);
        entity_commands.insert((
            MaterialMeshBundle {
                material: (**material).clone(),
                mesh: meshes.add(Mesh::new(PrimitiveTopology::TriangleList)),
                transform: Transform::from_translation(chunk_key.0.as_vec3() - Vec3::new(1.0, 1.0, 1.0)),
                visibility: Visibility::Hidden,
                ..Default::default()
            },
            Aabb::from_min_max(Vec3::new(1., 1., 1.), Vec3::new(CHUNK_LENGTH as f32 + 1., CHUNK_HEIGHT as f32 + 1., CHUNK_LENGTH as f32 + 1.)),
        ));
    }
}

// a pool of mesh buffers shared between meshing tasks.
static SHARED_MESH_BUFFERS: Lazy<ThreadLocal<RefCell<MeshBuffers<Voxel, ChunkShape>>>> =
    Lazy::new(ThreadLocal::default);

/// Queues meshing tasks for the chunks in need of a remesh.
fn queue_mesh_tasks(
    mut commands: Commands,
    dirty_chunks: Res<DirtyChunks>,
    chunk_entities: Res<ChunkEntities>,
    chunks: Res<ChunkMap<Voxel, ChunkShape>>,
    world_settings: Res<WorldSettings>,
) {
    let task_pool = AsyncComputeTaskPool::get();

    let name = world_settings.name;

    let mesh_gen = |buffer, key, name| {
        let _ = save_chunk_to_disk(&buffer, key, name);

        let mut mesh_buffers = SHARED_MESH_BUFFERS
        .get_or(|| {
            RefCell::new(MeshBuffers::<Voxel, ChunkShape>::new(ChunkShape {}))
        })
        .borrow_mut();

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh_buffer(&buffer, &mut mesh_buffers, &mut mesh, 1.0);

        mesh
    };

    dirty_chunks
        .iter_dirty()
        .filter_map(|key| chunk_entities.entity(*key).map(|entity| (key, entity)))
        .filter_map(|(key, entity)| {
            chunks
                .buffer_at(*key)
                .map(|buffer| (buffer.clone(), entity, *key))
        })
        .map(|(buffer, entity, key)| {
            (
                entity,
                ChunkMeshingTask(task_pool.spawn(async move {
                    mesh_gen(buffer, key, name)
                })),
            )
        })
        .for_each(|(entity, task)| {
            commands.entity(entity).insert(task);
        });
}

/// Polls and process the generated chunk meshes
fn process_mesh_tasks(
    mut meshes: ResMut<Assets<Mesh>>,
    mut chunk_query: Query<(Entity, &Handle<Mesh>, &mut ChunkMeshingTask), With<Chunk>>,
    mut commands: Commands,
) {
    chunk_query.for_each_mut(|(entity, handle, mut mesh_task)| {
        if let Some(mesh) = future::block_on(future::poll_once(&mut mesh_task.0)) {
            *meshes.get_mut(handle).unwrap() = mesh;
            commands.entity(entity)
                .remove::<ChunkMeshingTask>();
        }
    });
}

/// The set of systems which asynchronusly mesh the chunks.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash, SystemSet)]
pub struct ChunkMeshingSet;

/// Handles the meshing of the chunks.
pub struct VoxelWorldMeshingPlugin;

impl Plugin for VoxelWorldMeshingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.configure_sets(
            Update,
            ChunkMeshingSet
                .after(TerrainGenSet)
                .after(ChunkLoadingSet),
        )
        .add_systems(
            Update,
            (prepare_chunks, queue_mesh_tasks, process_mesh_tasks)
                .chain()
                .in_set(ChunkMeshingSet)
                .run_if(in_state(AppState::InGame)),
        );
    }
}

#[derive(Component)]
pub struct ChunkMeshingTask(Task<Mesh>);