use bevy::prelude::*;

use crate::core::{storage::VoxelBuffer, SectionShape};

#[derive(Debug, Clone, Copy, Component)]
pub struct SectionComponent;

impl SectionComponent {
    pub fn despawn_orphans(
        query: Query<Entity, (With<SectionComponent>, Without<Parent>)>,
        mut commands: Commands,
    ) {
        for entity in query.iter() {
            warn!("Despawning orphaned section entity: {:?}", entity);
            commands.entity(entity).despawn_recursive();
        }
    }
}

#[derive(Debug, Clone)]
pub struct Section {
    // pub block_count: u16,
    pub data: VoxelBuffer<u32, SectionShape>,
}
