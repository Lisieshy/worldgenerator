use std::marker::PhantomData;

use crate::voxel::{storage::VoxelBuffer, MaterialVoxel};
use bevy::{
    prelude::Mesh,
    render::mesh::{Indices, VertexAttributeValues},
};
use block_mesh::{greedy_quads, GreedyQuadsBuffer, RIGHT_HANDED_Y_UP_CONFIG};
use ndcopy::copy3;
use ndshape::{RuntimeShape, Shape};

use super::VoxelTerrainMesh;

// const UV_SCALE: f32 = 1.0;

/// Intermediate buffers for greedy meshing of voxel data which are reusable between frames to not allocate.
pub struct MeshBuffers<T, S: Shape<3, Coord = u32>>
where
    T: Copy + Default + MaterialVoxel,
{
    // A padded buffer to run greedy meshing algorithm on
    scratch_buffer: VoxelBuffer<T, RuntimeShape<u32, 3>>,
    greedy_buffer: GreedyQuadsBuffer,
    _phantom: PhantomData<S>,
}

impl<T, S: Shape<3, Coord = u32>> MeshBuffers<T, S>
where
    T: Copy + Default + MaterialVoxel,
{
    pub fn new(shape: S) -> Self {
        let padded_shape = RuntimeShape::<u32, 3>::new(shape.as_array().map(|x| x + 2));

        Self {
            greedy_buffer: GreedyQuadsBuffer::new(padded_shape.size() as usize),
            scratch_buffer: VoxelBuffer::<T, RuntimeShape<u32, 3>>::new_empty(padded_shape),
            _phantom: Default::default(),
        }
    }
}

// Processes the voxel data buffer specified as a parameter and generate.
//todo: don't populate mesh directly, introduce a meshbuilding system.
pub fn mesh_buffer<T, S>(
    buffer: &VoxelBuffer<T, S>,
    mesh_buffers: &mut MeshBuffers<T, S>,
    render_mesh: &mut Mesh,
    scale: f32,
) where
    T: Copy + Default + MaterialVoxel,
    S: Shape<3, Coord = u32>,
{
    mesh_buffers
        .greedy_buffer
        .reset(buffer.shape().size() as usize);

    let dst_shape = mesh_buffers.scratch_buffer.shape().clone();

    copy3(
        buffer.shape().as_array(),
        buffer.slice(),
        buffer.shape(),
        [0; 3],
        mesh_buffers.scratch_buffer.slice_mut(),
        &dst_shape,
        [1; 3],
    );

    // Extent::from_min_and_max(UVec2::ZERO, UVec2::splat(mesh_buffers.scratch_buffer.shape().size()))
    // .iter2()
    // .for_each(|pos| {
    //     *mesh_buffers.scratch_buffer.voxel_at_mut([pos.x, 0, pos.y].into()) = T::default();
    //     *mesh_buffers.scratch_buffer.voxel_at_mut([pos.x, CHUNK_HEIGHT, pos.y].into()) = T::default();
    // });

    greedy_quads(
        mesh_buffers.scratch_buffer.slice(),
        mesh_buffers.scratch_buffer.shape(),
        [0; 3],
        mesh_buffers
            .scratch_buffer
            .shape()
            .as_array()
            .map(|axis| axis - 1),
        &RIGHT_HANDED_Y_UP_CONFIG.faces,
        &mut mesh_buffers.greedy_buffer,
    );

    let num_indices = mesh_buffers.greedy_buffer.quads.num_quads() * 6;
    let num_vertices = mesh_buffers.greedy_buffer.quads.num_quads() * 4;
    let mut indices = Vec::with_capacity(num_indices);
    let mut positions = Vec::with_capacity(num_vertices);
    let mut normals = Vec::with_capacity(num_vertices);
    let mut tex_coords = Vec::with_capacity(num_vertices);
    let mut data = Vec::with_capacity(num_vertices);

    for (_, (group, face)) in mesh_buffers
        .greedy_buffer
        .quads
        .groups
        .as_ref()
        .iter()
        .zip(RIGHT_HANDED_Y_UP_CONFIG.faces.into_iter())
        .enumerate() {
        for quad in group.into_iter() {
            indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
            positions.extend_from_slice(&face.quad_mesh_positions(&quad, scale));
            normals.extend_from_slice(&face.quad_mesh_normals());
            tex_coords.extend_from_slice(&face.tex_coords(
                RIGHT_HANDED_Y_UP_CONFIG.u_flip_face,
                true,
                &quad,
            ));
            data.extend_from_slice(
                &[buffer
                        .voxel_at(quad.minimum.map(|x| x - 1).into())
                        .as_mat_id() as u32; 4],
            );

            // let mat_index = (block_face_normal_index as u32) << 8u32 | buffer
            //     .voxel_at(quad.minimum.map(|x| x - 1).into())
            //     .as_mat_id() as u32;

            // let uvs = &face.tex_coords(
            //     RIGHT_HANDED_Y_UP_CONFIG.u_flip_face,
            //     true,
            //     &quad,
            // );

            // convert mat_index to a float and keep the bits intact
            // let mat_index = [f32::from_bits(mat_index); 3];

            // data.extend_from_slice(
            //     &[mat_index; 3],
            // );

            // info!("mat_index: {:#034b}", mat_index);
            // info!("voxel_data: {:#034b}", data.last().unwrap());
        }
    }

    // for uv in tex_coords.iter_mut() {
    //     for c in uv.iter_mut() {
    //         *c *= UV_SCALE;
    //     }
    // }

    render_mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(positions),
    );

    render_mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::Float32x3(normals),
    );

    render_mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        VertexAttributeValues::Float32x2(tex_coords),
    );

    render_mesh.insert_attribute(
        VoxelTerrainMesh::ATTRIBUTE_MATERIAL_INDEX,
        VertexAttributeValues::Uint32(data),
    );

    // render_mesh.insert_attribute(
    //     VoxelTerrainMesh::ATTRIBUTE_DATA,
    //     VertexAttributeValues::Float32x3(data),
    // );

    render_mesh.set_indices(Some(Indices::U32(indices.clone())));
}