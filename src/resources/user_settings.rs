use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::ecs::event::ManualEventReader;

#[derive(Resource, Default)]
pub struct PlayerInput {
    pub mouse_motion: ManualEventReader<MouseMotion>,
    pub pitch: f32,
    pub yaw: f32,
}

#[derive(Resource)]
pub struct PlayerSettings {
    pub fov: f32,
    pub mouse_sensitivity: f32,
    pub speed: f32,
    pub use_gamepad: bool,
    pub gamepad: Option<Gamepad>,
    pub gamepad_sensitivity: f32,
}

impl Default for PlayerSettings {
    fn default() -> Self {
        Self {
            fov: 80f32.to_radians(),
            mouse_sensitivity: 0.00012,
            speed: 3.,
            use_gamepad: false,
            gamepad:  None,
            gamepad_sensitivity: 0.1,
        }
    }
}