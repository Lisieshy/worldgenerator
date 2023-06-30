use bevy::prelude::*;

use crate::materials::pixelate::PixelateMaterial;

#[derive(Component)]
pub struct PostProcessingHandle {
    pub mat_handle: Handle<PixelateMaterial>
}
