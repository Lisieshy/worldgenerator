use bevy::input::mouse::MouseButtonInput;
use bevy::window::PrimaryWindow;
use bevy::{input::mouse::MouseMotion, prelude::*, window::CursorGrabMode};
use bevy_egui::EguiContexts;

use std::f32::consts::PI;

use crate::{AppState, MyAssets};
use crate::debug::{DebugUISet, DebugUIState};
use crate::voxel::Voxel;
use crate::voxel::material::VoxelMaterial;
use crate::voxel::storage::ChunkMap;
use crate::voxel::world::chunks::get_chunk_for_pos;

use super::materials::Air;
use super::{ChunkShape, DirtyChunks};

use bevy_mod_raycast::prelude::*;


#[derive(Default, Component)]
pub struct PlayerController {
    pub yaw: f32,
    pub pitch: f32,
    pub cursor_locked: bool,
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
            mouse_sensitivity: 0.12,
            speed: 12.,
            sprint_speed_mult: 2.,
            use_gamepad: false,
            gamepad:  None,
            gamepad_sensitivity: 0.3,
        }
    }
}

pub fn handle_player_inputs(
    keys: Res<Input<KeyCode>>,
    button_input: Res<Input<GamepadButton>>,
    axes: Res<Axis<GamepadAxis>>,
    time: Res<Time>,
    settings: Res<PlayerSettings>,
    mut proj_q: Query<&mut Projection, With<Camera>>,
    mut query: Query<(&mut PlayerController, &mut Transform)>,
    mut mouse_motion_event_reader: EventReader<MouseMotion>,
    mouse_buttons: Res<Input<MouseButton>>,
    windows: Query<&mut Window>,
    mut chunks: ResMut<ChunkMap<Voxel, ChunkShape>>,
    mut dirty_chunks: ResMut<DirtyChunks>,
    mut raycast: Raycast,

    // under this is only for debug/test purposes. don't forget to remove it later
    debug_ui_state: Res<DebugUIState>,
    // mut painter: ShapePainter,
    mut gizmos: Gizmos,
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

                controller.yaw -= (settings.gamepad_sensitivity * pos.x * time.delta_seconds()).to_radians();
                controller.pitch += (settings.gamepad_sensitivity * pos.y * time.delta_seconds()).to_radians();
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
                velocity += forward * y;
                velocity += right * x;
            }
        }
    } else {
        for mouse_move in mouse_motion_event_reader.read() {
            match window.cursor.grab_mode {
                CursorGrabMode::None => (),
                _ => {
                    controller.yaw -= (settings.mouse_sensitivity * mouse_move.delta.x).to_radians();
                    controller.pitch -= (settings.mouse_sensitivity * mouse_move.delta.y).to_radians();
                }
            }
        }

        if keys.pressed(KeyCode::ShiftLeft) {
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
                        KeyCode::ControlLeft => velocity -= Vec3::Y,
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

    let direction = Quat::from_rotation_y(controller.yaw) * Quat::from_rotation_x(controller.pitch) * Vec3::new(0.0, 0.0, -1.0);
    let ray = Ray3d::new(transform.translation, direction.normalize());
    let hits = match raycast.debug_cast_ray(ray, &default(), &mut gizmos) {
        &[(_entity, ref data), ..] => Some(data),
        &[] => None,
    };

    let in_range_hit = |hit: &IntersectionData| match hit.distance() {
        x if x < 10.0 => Some((hit.position(), hit.normal())),
        _ => None,
    };

    // // draw square on the face of the block the player is looking at
    // if let Some((pos, normal)) = hits.and_then(in_range_hit) {
    //     painter.thickness = 0.1;
    //     painter.thickness_type = ThicknessType::World;
    //     painter.color = Color::ORANGE;
    //     let pos = pos - normal * 0.5;
    //     let mut offset = 0.001;

    //     match normal {
    //         Vec3 { x, .. } if x == 1.0 => {
    //             painter.set_translation(
    //                 pos.trunc(),
    //             );
    //             painter.set_rotation(Quat::from_rotation_y(PI / 2.0));
    //         }
    //         Vec3 { x, .. } if x == -1.0 => {
    //             painter.set_translation(
    //                 pos.trunc() - Vec3::Z,
    //             );
    //             painter.set_rotation(Quat::from_rotation_y(-PI / 2.0));
    //         }
    //         Vec3 { y, .. } if y == 1.0 => {
    //             painter.set_translation(
    //                 pos.trunc() - Vec3::Z,
    //             );
    //             painter.set_rotation(Quat::from_rotation_x(PI / 2.0));
    //             offset = -0.001;
    //         }
    //         Vec3 { y, .. } if y == -1.0 => {
    //             painter.set_translation(
    //                 pos.trunc(),
    //             );
    //             painter.set_rotation(Quat::from_rotation_x(-PI / 2.0));
    //             offset = -0.001;
    //         }
    //         Vec3 { z, .. } if z == 1.0 => {
    //             painter.set_translation(
    //                 pos.trunc() + Vec3::X,
    //             );
    //             painter.set_rotation(Quat::from_rotation_y(PI));
    //             offset = -0.001;
    //         }
    //         Vec3 { z, .. } if z == -1.0 => {
    //             painter.set_translation(
    //                 pos.trunc(),
    //             );
    //             painter.set_rotation(Quat::from_rotation_y(0.0));
    //             offset = -0.001;
    //         }
    //         _ => (),
    //     }

    //     painter.line(Vec3::new(1.0, 0.0, offset), Vec3::new(0.0, 0.0, offset));
    //     painter.line(Vec3::new(0.0, 1.0, offset), Vec3::new(0.0, 0.0, offset));
    //     painter.line(Vec3::new(1.0, 0.0, offset), Vec3::new(1.0, 1.0, offset));
    //     painter.line(Vec3::new(0.0, 1.0, offset), Vec3::new(1.0, 1.0, offset));
    // }

    if mouse_buttons.just_pressed(MouseButton::Right) && window.cursor.grab_mode != CursorGrabMode::None {
        // thanks to Zatmos (https://www.zatmos.xyz) for the monadic style

        let place_block = |(pos, normal)| {
            let pos = pos + normal * 0.5;

            let chunk_pos = get_chunk_for_pos(pos);

            let pos_in_chunk = pos - chunk_pos - Vec3::Y;

            chunks.buffer_at_mut(chunk_pos.as_ivec3())
            .map(|buffer| {
                if buffer.voxel_at([
                    pos_in_chunk.x as u32,
                    pos_in_chunk.y as u32,
                    pos_in_chunk.z as u32,
                ].into()) != Air::into_voxel() {
                    return;
                }
                *buffer.voxel_at_mut([
                    pos_in_chunk.x as u32,
                    pos_in_chunk.y as u32,
                    pos_in_chunk.z as u32,
                ].into()) = Voxel(debug_ui_state.selected_mat);
            }).and_then(|_| {
                dirty_chunks.mark_dirty(chunk_pos.as_ivec3());
                Some(())
            });
        };

        hits.and_then(in_range_hit)
            .map(place_block);
    }

    if mouse_buttons.just_pressed(MouseButton::Left) && window.cursor.grab_mode != CursorGrabMode::None {
        // thanks to Zatmos (https://www.zatmos.xyz) for the monadic style

        let remove_block = |(pos, normal)| {
            let pos = pos - normal * 0.5;

            let chunk_pos = get_chunk_for_pos(pos);

            let pos_in_chunk = pos - chunk_pos;

            chunks.buffer_at_mut(chunk_pos.as_ivec3())
            .map(|buffer| {
                *buffer.voxel_at_mut([
                    pos_in_chunk.x as u32,
                    (pos_in_chunk.y - 1.) as u32,
                    pos_in_chunk.z as u32,
                ].into()) = Air::into_voxel();
            }).and_then(|_| {
                dirty_chunks.mark_dirty(chunk_pos.as_ivec3());
                Some(())
            });
        };

        hits.and_then(in_range_hit)
            .map(remove_block);
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
    mut egui: EguiContexts,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut mouse_button_evr: EventReader<MouseButtonInput>,
    mut settings: ResMut<PlayerSettings>,
) {
    if let Ok(mut window) = windows.get_single_mut() {
        for ev in mouse_button_evr.read() {
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
        if window.cursor.grab_mode != CursorGrabMode::None {
            let width = window.width() / 2.;
            let height = window.height() / 2.;
            window.set_cursor_position(Some(Vec2::new(
                width,
                height,
            )))
        }
    } else {
        unreachable!("No primary window found, something has gone terribly wrong.");
    }
}

pub fn draw_crosshair(
    assets: Res<MyAssets>,
    mut commands: Commands,
) {
    // painter.set_translation(Vec3::NEG_Z);
    // painter.color = Color::ORANGE;
    // painter.circle(10.);
    commands
        .spawn(NodeBundle {
            style: Style {
                height: Val::Percent(100.0),
                width: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceAround,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((NodeBundle {
                    style: Style {
                        width: Val::Px(32.0),
                        height: Val::Px(32.0),
                        ..default()
                    },
                    background_color: Color::rgba(1.0, 1.0, 1.0, 0.5).into(),
                    ..default()
                },
                UiImage::new(assets.crosshair.clone()),
            ))
            .with_children(|parent| {
                parent.spawn((
                    NodeBundle {
                        style: Style {
                            display: Display::None,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    Text::from_section("Crosshair", TextStyle::default()),
                ));
            });
        });
}


#[derive(Hash, Copy, Clone, PartialEq, Eq, Debug, SystemSet)]
/// Systems related to player controls.
pub struct PlayerControllerSet;

pub struct VoxelWorldPlayerControllerPlugin;

impl Plugin for VoxelWorldPlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), init_input)
            .add_systems(Update, grab_cursor.after(DebugUISet::Display).run_if(in_state(AppState::InGame)))
            .add_systems(Update, handle_player_inputs.run_if(in_state(AppState::InGame)))
            .add_systems(OnEnter(AppState::InGame), draw_crosshair);
        // app.add_startup_system(init_input)
        //     .add_system(grab_cursor.in_base_set(CoreSet::Update).after(DebugUISet::Display))
        //     .add_system(handle_player_inputs.in_base_set(CoreSet::Update));
        // app.add_systems(
        //     (handle_player_inputs)
        //         .chain()
        //         .in_base_set(CoreSet::Update)
        //         .after(DebugUISet::Display),
        // );
    }
}