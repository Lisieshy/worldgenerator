use block_mesh::{MergeVoxel, Voxel as MeshableVoxel};
use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Hash, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Voxel(pub u16);

impl Voxel {
    pub const EMPTY_VOXEL: Self = Self(0);
}

impl Default for Voxel {
    fn default() -> Self {
        Self::EMPTY_VOXEL
    }
}

impl MeshableVoxel for Voxel {
    #[inline]
    fn get_visibility(&self) -> block_mesh::VoxelVisibility {
        match *self {
            // Switch the EMPTY_VOXEL to Opaque to hide chunk borders.
            // Will be useful later when chunk meshing will be handled better.
            Self::EMPTY_VOXEL => block_mesh::VoxelVisibility::Empty,
            Voxel(1) => block_mesh::VoxelVisibility::Empty, // Air Voxel type has ID 1.
            Voxel(8) => block_mesh::VoxelVisibility::Translucent, // Water Voxel type has ID 6.
            _ => block_mesh::VoxelVisibility::Opaque,
        }
    }
}

impl MergeVoxel for Voxel {
    type MergeValue = u16;

    #[inline]
    fn merge_value(&self) -> Self::MergeValue {
        self.0
    }
}

pub trait MaterialVoxel: MergeVoxel + MeshableVoxel {
    fn as_mat_id(&self) -> u16;
}

impl MaterialVoxel for Voxel {
    fn as_mat_id(&self) -> u16 {
        self.0
    }
}