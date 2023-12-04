use bevy::prelude::*;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Deref, DerefMut, Resource)]
pub struct WorldSeed(pub i64);