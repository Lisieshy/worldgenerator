use bevy::input::mouse::MouseButtonInput;
use bevy::window::PrimaryWindow;
use bevy::{input::mouse::MouseMotion, prelude::*, window::CursorGrabMode};
use bevy_egui::EguiContexts;
// use std::f32::consts::FRAC_PI_2;
use std::f32::consts::PI;

use crate::debug::DebugUISet;

// Reusing the player controller impl for now.

// pub const DEFAULT_CAMERA_SENS: f32 = 0.005;

#[derive(Default, Component)]
pub struct PlayerController {
    pub yaw: f32,
    pub pitch: f32,
    pub cursor_locked: bool,
    // pub mouse_motion: ManualEventReader<MouseMotion>,
}


#[derive(Resource)]
pub struct PlayerSettings {
    pub fov: f32,
    pub mouse_sensitivity: f32,
    pub speed: f32,
    pub sprint_speed_mult: f32,
    pub use_gamepad: bool,
    pub gamepad: Option<Gamepad>,
    pub gamepad_sensitivity: f32,
}

impl Default for PlayerSettings {
    fn default() -> Self {
        Self {
            fov: 80f32,
            mouse_sensitivity: 0.00012,
            speed: 12.,
            sprint_speed_mult: 2.,
            use_gamepad: false,
            gamepad:  None,
            gamepad_sensitivity: 0.3,
        }
    }
}

pub fn handle_player_inputs(
    // gamepads: Res<Gamepads>,
    keys: Res<Input<KeyCode>>,
    button_input: Res<Input<GamepadButton>>,
    axes: Res<Axis<GamepadAxis>>,
    time: Res<Time>,
    settings: Res<PlayerSettings>,
    // motion: Res<Events<MouseMotion>>,
    mut proj_q: Query<&mut Projection, With<Camera>>,
    mut query: Query<(&mut PlayerController, &mut Transform)>,
    mut mouse_motion_event_reader: EventReader<MouseMotion>,
    windows: Query<&mut Window>,
    // windows: Query<&Window, With<PrimaryWindow>>,
) {
    let window = windows.single();
    let (mut controller, mut transform) = query.single_mut();
    // wrap the yaw angle between 0 and 2PI
    controller.yaw = (controller.yaw + 2. * PI) % (2. * PI);

    if let Ok(mut projection) = proj_q.get_single_mut() {
        if let Projection::Perspective(ref mut perspective) = *projection {
            perspective.fov = settings.fov.to_radians();
        }
    }

    let mut velocity = Vec3::ZERO;
    let mut acceleration = 1.0f32;

    let forward = Vec3::new(transform.forward().x, 0., transform.forward().z).normalize();
    let right = Vec3::new(transform.right().x, 0., transform.right().z).normalize();

    if settings.use_gamepad {
        if let Some(gamepad) = settings.gamepad {
            if button_input.pressed(GamepadButton::new(gamepad, GamepadButtonType::DPadUp)) {
                velocity += Vec3::Y;
            } else if button_input.pressed(GamepadButton::new(gamepad, GamepadButtonType::DPadDown)) {
                velocity -= Vec3::Y;
            }

            if let (Some(x), Some(y)) = (
                axes.get(GamepadAxis::new(gamepad, GamepadAxisType::RightStickX)),
                axes.get(GamepadAxis::new(gamepad, GamepadAxisType::RightStickY))
            ) {
                let pos = Vec2::new(x, y);
                let window_scale = window.height().min(window.width());

                controller.yaw -= (settings.gamepad_sensitivity * pos.x * window_scale * time.delta_seconds()).to_radians();
                controller.pitch += (settings.gamepad_sensitivity * pos.y * window_scale * time.delta_seconds()).to_radians();
            }

            if button_input.pressed(GamepadButton::new(gamepad, GamepadButtonType::LeftTrigger2)) {
                acceleration *= settings.sprint_speed_mult;
            } else { 
                acceleration = 1.0f32;
            }

            if let (Some(x), Some(y)) = (
                axes.get(GamepadAxis::new(gamepad, GamepadAxisType::LeftStickX)),
                axes.get(GamepadAxis::new(gamepad, GamepadAxisType::LeftStickY))
            ) {
                // if button_input.pressed(GamepadButton::new(gamepad, GamepadButtonType::LeftTrigger2)) {
                //     velocity += forward * y * settings.sprint_speed_mult;
                //     velocity += right * x * settings.sprint_speed_mult;
                // } else {
                    velocity += forward * y;
                    velocity += right * x;
                // }
            }
        }
    } else {
        for mouse_move in mouse_motion_event_reader.iter() {
            match window.cursor.grab_mode {
                CursorGrabMode::None => (),
                _ => {
                    let window_scale = window.height().min(window.width());
                    controller.yaw -= (settings.mouse_sensitivity * mouse_move.delta.x * window_scale).to_radians();
                    controller.pitch -= (settings.mouse_sensitivity * mouse_move.delta.y * window_scale).to_radians();
                }
            }
        }

        if keys.pressed(KeyCode::LShift) {
            acceleration *= settings.sprint_speed_mult;
        } else {
            acceleration = 1.0f32;
        }

        for key in keys.get_pressed() {
            match window.cursor.grab_mode {
                CursorGrabMode::None => (),
                _ => {
                    match key {
                        KeyCode::W => velocity += forward,
                        KeyCode::A => velocity -= right,
                        KeyCode::S => velocity -= forward,
                        KeyCode::D => velocity += right,
                        KeyCode::Space => velocity += Vec3::Y,
                        KeyCode::LControl => velocity -= Vec3::Y,
                        _ => (),
                    }
                }
            }
        }
        velocity = velocity.normalize_or_zero();
    }

    // controller.pitch = controller.pitch.clamp(-FRAC_PI_2, FRAC_PI_2); the frac_pi_2 should work but it doesn't for some reason. Need to fix somehow.
    controller.pitch = controller.pitch.clamp(-1.57, 1.57);
    transform.rotation = Quat::from_axis_angle(Vec3::Y, controller.yaw) * Quat::from_axis_angle(Vec3::X, controller.pitch);
    transform.translation += velocity * settings.speed * acceleration * time.delta_seconds();
}

pub fn init_input(
    mut windows: Query<&mut Window>,
    gamepads: Res<Gamepads>,
    mut settings: ResMut<PlayerSettings>,

) {
    if let Ok(mut window) = windows.get_single_mut() {
        window.cursor.grab_mode = CursorGrabMode::Confined;
        window.cursor.visible = false;

        if let Some(gamepad) = gamepads.iter().next() {
            settings.gamepad = Some(gamepad);
            info!("Switched to default gamepad {} : {}", gamepad.id, gamepads.name(gamepad).unwrap());
        }
    } else {
        warn!("No primary window found");
    }
}

pub fn toggle_grab_mode(window: &mut Window) {
    match window.cursor.grab_mode {
        CursorGrabMode::None => {
            window.cursor.grab_mode = CursorGrabMode::Confined;
            window.cursor.visible = false;
            window.set_cursor_position(Some(Vec2::new(
                window.width() / 2.,
                window.height() / 2.,
            )));
        }
        _ => {
            window.cursor.grab_mode = CursorGrabMode::None;
            window.cursor.visible = true;
            window.set_cursor_position(Some(Vec2::new(
                window.width() / 2.,
                window.height() / 2.,
            )));
        }
    }
}

pub fn grab_cursor(
    keys: Res<Input<KeyCode>>,
    gamepads: Res<Gamepads>,
    mut egui: EguiContexts,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut mouse_button_evr: EventReader<MouseButtonInput>,
    mut settings: ResMut<PlayerSettings>,
) {
    if let Ok(mut window) = windows.get_single_mut() {
        for ev in mouse_button_evr.iter() {
            if ev.button == MouseButton::Left && ev.state.is_pressed() && window.cursor.grab_mode == CursorGrabMode::None  && !egui.ctx_mut().wants_pointer_input() {
                toggle_grab_mode(&mut window);
            }
        }
        if keys.just_pressed(KeyCode::Escape) {
            toggle_grab_mode(&mut window);
        }
        if keys.just_pressed(KeyCode::K) {
            settings.use_gamepad = !settings.use_gamepad;
            info!("Toggled use_controller to {}", settings.use_gamepad);
        }
        if settings.use_gamepad {
            if keys.just_pressed(KeyCode::L) {
                // Iterates through the gamepad list and changes the settings.gamepad to the next one in the iteration
                // let mut gamepad_iter = gamepads.iter();
                // let mut gamepad = gamepad_iter.next();
                // while gamepad.is_some() && gamepad != settings.gamepad {
                //     gamepad = gamepad_iter.next();
                // }
                if let Some(gamepad) = gamepads.iter().next() {
                    settings.gamepad = Some(gamepad);
                    info!("Switched to gamepad {} : {}", gamepad.id, gamepads.name(gamepad).unwrap());
                } else {
                    settings.gamepad = None;
                    info!("No gamepad detected");
                }
            }
        }
    } else {
        warn!("No primary window found");
    }
}



#[derive(Hash, Copy, Clone, PartialEq, Eq, Debug, SystemSet)]
/// Systems related to player controls.
pub struct PlayerControllerSet;

pub struct VoxelWorldPlayerControllerPlugin;

impl Plugin for VoxelWorldPlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(init_input)
            .add_system(grab_cursor.in_base_set(CoreSet::Update).after(DebugUISet::Display))
            .add_system(handle_player_inputs.in_base_set(CoreSet::Update));
        // app.add_systems(
        //     (handle_player_inputs)
        //         .chain()
        //         .in_base_set(CoreSet::Update)
        //         .after(DebugUISet::Display),
        // );
    }
}