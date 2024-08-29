use ilattice::extent::Extent;
use ilattice::glam::UVec3;
use ndshape::{Shape, ConstShape3u32};

use serde::{Serialize, Deserialize, Serializer, ser::SerializeStruct};

use crate::core::{ChunkShape, Voxel};

/// A buffer of typed voxel data stored as a contiguous array in memory.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct VoxelBuffer<V, S: Shape<3, Coord = u32>>
where
    V: Copy + Clone + Default,
{
    data: Box<[V]>,
    shape: S,
}

/// Implementing Serialize for the specific Voxel and ChunkShape types in order to save the buffer to disk.
impl Serialize for VoxelBuffer<Voxel, ChunkShape> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        // let encoded: Vec<u8> = bincode::serialize(&self.data).unwrap();

        let mut voxel_data = serializer.serialize_struct("VoxelBuffer", 1)?;
        voxel_data.serialize_field("data", &self.data)?;
        // voxel_data.serialize_field("shape", &ConstShape3u32)?;
        voxel_data.end()
    }
}

impl<'de> Deserialize<'de> for VoxelBuffer<Voxel, ChunkShape> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Data,
        }

        struct VoxelBufferDataVisitor;

        impl<'de> serde::de::Visitor<'de> for VoxelBufferDataVisitor {
            type Value = VoxelBuffer<Voxel, ChunkShape>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct VoxelBuffer")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<VoxelBuffer<Voxel, ChunkShape>, V::Error>
                where
                    V: serde::de::SeqAccess<'de>,
            {
                let data = seq.next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;

                Ok(VoxelBuffer {
                    data,
                    shape: ConstShape3u32,
                })
            }

            fn visit_map<V>(self, mut map: V) -> Result<VoxelBuffer<Voxel, ChunkShape>, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut data = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Data => {
                            if data.is_some() {
                                return Err(serde::de::Error::duplicate_field("data"));
                            }
                            data = Some(map.next_value()?);
                        }
                    }
                }
                let data = data.ok_or_else(|| serde::de::Error::missing_field("data"))?;

                Ok(VoxelBuffer {
                    data,
                    shape: ConstShape3u32,
                })
            }
        }

        const FIELDS: &[&str] = &["data"];
        deserializer.deserialize_struct("VoxelBuffer", FIELDS, VoxelBufferDataVisitor)

    }
}

#[allow(dead_code)]
impl<V, S: Shape<3, Coord = u32>> VoxelBuffer<V, S>
where
    V: Copy + Clone + Default,
{
    #[inline]
    pub fn new(shape: S, initial_val: V) -> Self {
        Self {
            data: vec![initial_val; shape.size() as usize].into_boxed_slice(),
            shape,
        }
    }

    #[inline]
    pub fn new_empty(shape: S) -> Self {
        Self {
            data: vec![Default::default(); shape.size() as usize].into_boxed_slice(),
            shape,
        }
    }

    // Returns the voxel at the querried position in local space.
    #[inline]
    pub fn voxel_at(&self, pos: UVec3) -> V {
        self.data[self.shape.linearize(pos.to_array()) as usize]
    }

    // Returns a mutable reference to the the voxel at the querried position in local space.
    #[inline]
    pub fn voxel_at_mut(&mut self, pos: UVec3) -> &mut V {
        &mut self.data[self.shape.linearize(pos.to_array()) as usize]
    }

    #[inline]
    pub const fn slice(&self) -> &[V] {
        &self.data
    }

    #[inline]
    pub fn slice_mut(&mut self) -> &mut [V] {
        &mut self.data
    }

    #[inline]
    pub const fn shape(&self) -> &S {
        &self.shape
    }

    /// Fills an extent of this buffer with the specified value.
    #[inline]
    pub fn fill_extent(&mut self, extent: Extent<UVec3>, val: V) {
        ndcopy::fill3(
            extent.shape.to_array(),
            val,
            &mut self.data,
            &self.shape,
            extent.minimum.to_array(),
        );
    }
}