use std::sync::Arc;

use bevy::prelude::*;
use parking_lot::RwLock;

use crate::core::SECTION_COUNT;

use super::section::Section;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deref, DerefMut)]
pub struct ChunkEntity(pub Entity);

pub type ChunkSections = Arc<RwLock<[Section; SECTION_COUNT]>>;

#[derive(Debug, Clone, Component)]
pub struct Chunk {
    pub sections: ChunkSections,
    pub position: IVec2,
}

impl Chunk {
    
}