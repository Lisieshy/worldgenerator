use std::f32::consts::PI;

// use bevy::input::gamepad::GamepadSettings;
use bevy::input::mouse::{MouseMotion, MouseButtonInput};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};

use crate::components::player_camera::PlayerCamera;
use crate::components::text_change::TextChanges;
use crate::resources::user_settings::{PlayerSettings, PlayerInput};

pub fn player_input(
    gamepads: Res<Gamepads>,
    keys: Res<Input<KeyCode>>,
    button_input: Res<Input<GamepadButton>>,
    _button_axes: Res<Axis<GamepadButton>>,
    axes: Res<Axis<GamepadAxis>>,
    time: Res<Time>,
    settings: Res<PlayerSettings>,
    mut state: ResMut<PlayerInput>,
    motion: Res<Events<MouseMotion>>,
    mut cam_query: Query<&mut Transform, With<PlayerCamera>>,
    mut query_text: Query<&mut Text, With<TextChanges>>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok(window) = windows.get_single() {
        let mut delta_state = state.as_mut();
        // wrap the yaw angle between 0 and 2PI
        delta_state.yaw = (delta_state.yaw + 2. * PI) % (2. * PI);

        for mut transform in cam_query.iter_mut() {
            let mut velocity = Vec3::ZERO;

            let forward = Vec3::new(transform.forward().x, 0., transform.forward().z).normalize();
            let right = Vec3::new(transform.right().x, 0., transform.right().z).normalize();

            if settings.use_gamepad { // Controller Input
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

                        delta_state.yaw -= (settings.gamepad_sensitivity * pos.x * window_scale).to_radians();
                        delta_state.pitch += (settings.gamepad_sensitivity * pos.y * window_scale).to_radians();
                    }

                    if let (Some(x), Some(y)) = (
                        axes.get(GamepadAxis::new(gamepad, GamepadAxisType::LeftStickX)),
                        axes.get(GamepadAxis::new(gamepad, GamepadAxisType::LeftStickY))
                    ) {
                        velocity += forward * y;
                        velocity += right * x;
                    }

                    for mut text in query_text.iter_mut() {
                        text.sections[4].value = format!("{}:{}\n", gamepads.name(gamepad).unwrap(), gamepad.id);
                    }
                } else {
                    for mut text in query_text.iter_mut() {
                        text.sections[4].value = "No controller detected\n".to_string();
                    }
                }
            } else { // Keyboard Input
                for mut text in query_text.iter_mut() {
                    text.sections[4].value = "".to_string();
                }

                for ev in delta_state.mouse_motion.iter(&motion) {
                    match window.cursor.grab_mode {
                        CursorGrabMode::None => (),
                        _ => {
                            let window_scale = window.height().min(window.width());
                            delta_state.pitch -= (settings.mouse_sensitivity * ev.delta.y * window_scale).to_radians();
                            delta_state.yaw -= (settings.mouse_sensitivity * ev.delta.x * window_scale).to_radians();
                        }
                    }
                }

                for key in keys.get_pressed() {
                    match window.cursor.grab_mode {
                        CursorGrabMode::None => (),
                        _ => match key {
                            KeyCode::Z => velocity += forward,
                            KeyCode::Q => velocity -= right,
                            KeyCode::S => velocity -= forward,
                            KeyCode::D => velocity += right,
                            KeyCode::Space => velocity += Vec3::Y,
                            KeyCode::LShift => velocity -= Vec3::Y,
                            _ => (),
                        }
                    }
                }
                velocity = velocity.normalize_or_zero();
            }
            
            // Applying rotation
            // Pitch is clamped with an approximation of this : ((-PI / 2.) + EPSILON, (PI / 2.) - EPSILON);
            delta_state.pitch = delta_state.pitch.clamp(-1.57, 1.57);
            transform.rotation = Quat::from_axis_angle(Vec3::Y, delta_state.yaw) * Quat::from_axis_angle(Vec3::X, delta_state.pitch);

            // Applying movement
            transform.translation += velocity * time.delta_seconds() * settings.speed;
            
            for mut text in &mut query_text {
                text.sections[3].value = format!(
                    "XYZ: {:.3} / {:.5} / {:.3}\nLookAt: {:.1} / {:.1}\nVelocity: {:.3} / {:.3} / {:.3}\nForward: {:.3}\nRight: {:.3}\n",
                    transform.translation.x,
                    transform.translation.y,
                    transform.translation.z,
                    delta_state.yaw.to_degrees(),
                    delta_state.pitch.to_degrees(),
                    velocity.x,
                    velocity.y,
                    velocity.z,
                    forward,
                    right,
                )
            }
        }
    } else {
        warn!("No primary window found");
    }
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
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut mouse_button_evr: EventReader<MouseButtonInput>,
    mut settings: ResMut<PlayerSettings>,
) {
    if let Ok(mut window) = windows.get_single_mut() {
        for ev in mouse_button_evr.iter() {
            if ev.button == MouseButton::Left && ev.state.is_pressed() && window.cursor.grab_mode == CursorGrabMode::None {
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
                // Switch the gamepad id to the next one when the key is pressed
                if let Some(gamepad) = gamepads.iter().next() {
                    settings.gamepad = Some(gamepad);
                    info!("Switched to gamepad {} : {}", gamepad.id, gamepads.name(gamepad).unwrap());
                } else {
                    settings.gamepad = None;
                    info!("No gamepad selected/detected");
                }
            }
        }
    } else {
        warn!("No primary window found");
    }
}

// pub fn configure_gamepad(
//     mut settings: ResMut<PlayerMovement>,
//     mut gp_settings: ResMut<GamepadSettings>,
// ) {

//     // Edit gamepad details and settings here
// }