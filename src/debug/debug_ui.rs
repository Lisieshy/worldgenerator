use std::fmt::format;

use bevy::{
    diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, DiagnosticsStore},
    input::{keyboard::KeyboardInput, ButtonState, Input},
    prelude::{
        Color, EventReader, IntoSystemConfigs, IntoSystemSetConfigs,
        KeyCode, Plugin, Res, ResMut, Resource, SystemSet, Vec3, IVec3, Transform, Query, Quat, EventWriter, With, Update,
    }, app::AppExit, window::{Window, PrimaryWindow, WindowMode}, gizmos::{self, gizmos::Gizmos, GizmoConfig}, pbr::wireframe::WireframeConfig,
};
use bevy_egui::{
    egui::{self, Rgba, Slider, Button},
    EguiContexts, EguiPlugin, EguiSet,
};
use directories::BaseDirs;

// use bevy_prototype_debug_lines::*;

use crate::voxel::{
    material::VoxelMaterialRegistry, ChunkCommandQueue, ChunkEntities, ChunkLoadRadius,
    CurrentLocalPlayerChunk, DirtyChunks,
    CHUNK_LENGTH, CHUNK_HEIGHT, player::{PlayerSettings, PlayerController}, terraingen::{self, noise::Heightmap}, CHUNK_LENGTH_U, WorldSettings, VoxelWorldPlugin,
};

fn display_debug_stats(mut egui: EguiContexts, diagnostics: Res<DiagnosticsStore>) {
    egui::Window::new("performance stuff").show(egui.ctx_mut(), |ui| {
        ui.label(format!(
            "Avg. FPS: {:.02}",
            diagnostics
                .get(FrameTimeDiagnosticsPlugin::FPS)
                .unwrap()
                .average()
                .unwrap_or_default()
        ));
        ui.label(format!(
            "Total Entity count: {}",
            diagnostics
                .get(EntityCountDiagnosticsPlugin::ENTITY_COUNT)
                .unwrap()
                .average()
                .unwrap_or_default()
        ));
    });
}

fn display_window_settings(
    mut egui: EguiContexts,
    mut ui_state: ResMut<DebugUIState>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut exit: EventWriter<AppExit>,
) {
    egui::Window::new("window stuff").show(egui.ctx_mut(), |ui| {
        ui.heading("Window size");
        ui.label(format!("W x H: {} x {}", windows.single_mut().width(), windows.single_mut().height()));
        ui.label(format!("Physical W x Physical H: {} x {}", windows.single_mut().physical_width(), windows.single_mut().physical_height()));
        ui.separator();
        ui.heading("Window mode");
        ui.label(format!("Window mode: {:?}", windows.single_mut().mode));
        ui.label(format!("Vsync: {:?}", windows.single_mut().present_mode));
        ui.separator();
        ui.horizontal(|ui| {
            ui.radio_value(&mut ui_state.window_mode, WindowMode::Windowed, "Windowed");
            ui.radio_value(&mut ui_state.window_mode, WindowMode::BorderlessFullscreen, "Borderless Fullscreen");
            ui.radio_value(&mut ui_state.window_mode, WindowMode::Fullscreen, "Fullscreen");
        });
        ui.checkbox(&mut ui_state.use_vsync, "Vsync");
        ui.separator();
        if ui.add(Button::new("Quit")).on_hover_text("Exits the game.").clicked() {
            exit.send(AppExit);
        }
    });

    windows.single_mut().mode = ui_state.window_mode;
    windows.single_mut().present_mode = if ui_state.use_vsync {
        bevy::window::PresentMode::AutoVsync
    } else {
        bevy::window::PresentMode::AutoNoVsync
    };
}

fn display_player_settings(
    mut egui: EguiContexts,
    mut settings: ResMut<PlayerSettings>,
    // mut lines: ResMut<DebugLines>,
    mut query: Query<(&mut PlayerController, &mut Transform)>,
) {
    let (controller, _) = query.single_mut();

    // let direction = Quat::from_rotation_y(controller.yaw) * Quat::from_rotation_x(controller.pitch) * Vec3::new(0.0, 0.0, -1.0);

    // let endpoint = transform.translation + direction * 10.0;

    // let up_endpoint = endpoint + Vec3::new(0.0, 1.0, 0.0);
    // let right_endpoint = endpoint + Vec3::new(1.0, 0.0, 0.0);
    // let forward_endpoint = endpoint + Vec3::new(0.0, 0.0, 1.0);

    egui::Window::new("player settings").show(egui.ctx_mut(), |ui| {
        ui.heading("Controller info");
        ui.label(format!("Use gamepad: {}", settings.use_gamepad));
        ui.label(format!("Gamepad: {:?}", settings.gamepad));
        ui.separator();
        ui.heading("sensitivity");
        ui.label(format!("Mouse sensitivity"));
        ui.add(Slider::new(&mut settings.mouse_sensitivity, 0.01..=1.0f32));
        ui.label(format!("Controller sensitivity"));
        ui.add(Slider::new(&mut settings.gamepad_sensitivity, 0.01..=1.0f32));
        ui.separator();
        ui.heading("speed");
        ui.label(format!("Movement speed"));
        ui.add(Slider::new(&mut settings.speed, 1.0..=100.0f32));
        ui.label(format!("Sprint speed multiplier"));
        ui.add(Slider::new(&mut settings.sprint_speed_mult, 1.0..=100.0f32));
        ui.separator();
        ui.heading("camera");
        ui.label(format!("FOV"));
        ui.add(Slider::new(&mut settings.fov, 30.0..=120.0f32));
        ui.label(format!("Camera yaw: {}", controller.yaw.to_degrees()));
        ui.label(format!("Camera pitch: {}", controller.pitch.to_degrees()));
    });

    // lines.line_colored(
    //     endpoint,
    //     up_endpoint,
    //     0.,
    //     Color::GREEN,
    // );
    // lines.line_colored(
    //     endpoint,
    //     right_endpoint,
    //     0.,
    //     Color::RED,
    // );
    // lines.line_colored(
    //     endpoint,
    //     forward_endpoint,
    //     0.,
    //     Color::BLUE,
    // );
}

fn draw_chunk_borders(
    // mut lines: ResMut<DebugLines>,
    // mut shapes: ResMut<DebugShapes>,
    chunk: IVec3,
) {
    let length = CHUNK_LENGTH as f32;
    let height = CHUNK_HEIGHT as f32;
    let cube_x = chunk.x as f32 + length / 2.;
    let cube_y = chunk.y as f32 + height / 2.;
    let cube_z = chunk.z as f32 + length / 2.;

    // shapes
    //     .cuboid()
    //     .position(Vec3::new(cube_x, cube_y, cube_z))
    //     .size(Vec3::new(length, height, length))
    //     .color(Color::RED);

    // for i in 0..=CHUNK_LENGTH {
    //     let i = i as f32;
    //     lines.line_colored(
    //         Vec3::new(chunk.x as f32 + i, chunk.y as f32, chunk.z as f32),
    //         Vec3::new(chunk.x as f32 + i, chunk.y as f32, chunk.z as f32 + length),
    //         0.,
    //         Color::WHITE,
    //     );
    //     lines.line_colored(
    //         Vec3::new(chunk.x as f32, chunk.y as f32, chunk.z as f32 + i),
    //         Vec3::new(chunk.x as f32 + length, chunk.y as f32, chunk.z as f32 + i),
    //         0.,
    //         Color::WHITE,
    //     );
    // }
}

fn display_world_info(
    mut egui: EguiContexts,
    dirty_chunks: Res<DirtyChunks>,
    player_pos: Res<CurrentLocalPlayerChunk>,
    mut chunk_loading_radius: ResMut<ChunkLoadRadius>,
    mut chunk_command_queue: ResMut<ChunkCommandQueue>,
    world_settings: Res<WorldSettings>,
    // lines: ResMut<DebugLines>,
    // shapes: ResMut<DebugShapes>,
    loaded_chunks: Res<ChunkEntities>,
) {
    let chunk_continentalness = terraingen::noise::get_chunk_continentalness(player_pos.chunk_min, CHUNK_LENGTH_U, world_settings.seed);
    let continentalness = Heightmap::<CHUNK_LENGTH_U, CHUNK_LENGTH_U>::from_slice(&chunk_continentalness);
    let chunk_erosion = terraingen::noise::get_chunk_erosion(player_pos.chunk_min, CHUNK_LENGTH_U, world_settings.seed);
    let erosion = Heightmap::<CHUNK_LENGTH_U, CHUNK_LENGTH_U>::from_slice(&chunk_erosion);
    let chunk_peaks_valleys = terraingen::noise::get_chunk_peaks_valleys(player_pos.chunk_min, CHUNK_LENGTH_U, world_settings.seed);
    let peaks_valleys = Heightmap::<CHUNK_LENGTH_U, CHUNK_LENGTH_U>::from_slice(&chunk_peaks_valleys);

    let pos_in_chunk = player_pos.world_pos - player_pos.chunk_min.as_vec3();

    egui::Window::new(format!("{} info", world_settings.name)).show(egui.ctx_mut(), |ui| {
        ui.heading("Chunks");
        ui.label(format!(
            "Chunks invalidations (per frame):  {}",
            dirty_chunks.num_dirty()
        ));
        ui.label(format!("Loaded chunk count: {}", loaded_chunks.len()));
        ui.separator();
        ui.label("Horizontal chunk loading radius");
        ui.add(Slider::new(&mut chunk_loading_radius.horizontal, 2..=32));
        ui.separator();

        if ui.button("Clear loaded chunks").clicked() {
            chunk_command_queue.queue_unload(loaded_chunks.iter_keys());
        }
        ui.separator();

        ui.heading("Current player position");
        ui.label(format!("Current position : X: {:.3}, Y: {:.3}, Z: {:.3}", player_pos.world_pos.x, player_pos.world_pos.y, player_pos.world_pos.z));
        ui.label(format!("Current chunk : X: {:.3}, Y: {:.3}, Z: {:.3}", player_pos.chunk_min.x, player_pos.chunk_min.y, player_pos.chunk_min.z));
        ui.label(format!("Current position in chunk : X: {:.3}, Y: {:.3}, Z: {:.3}", pos_in_chunk.x, pos_in_chunk.y, pos_in_chunk.z));
        ui.separator();
        ui.heading("Noise info");
        ui.label(format!("Seed: {}", world_settings.seed));
        ui.label(format!("Continentalness : {}", continentalness.getf([pos_in_chunk.x as u32, pos_in_chunk.z as u32])));
        ui.label(format!("Erosion : {}", erosion.getf([pos_in_chunk.x as u32, pos_in_chunk.z as u32])));
        ui.label(format!("Peaks&Valleys : {}", peaks_valleys.getf([pos_in_chunk.x as u32, pos_in_chunk.z as u32])));
        // ui.label(format!("Current biome : {}", biome.name()));
    });

    // draw_chunk_borders(shapes, player_pos.chunk_min);
}

fn display_misc_info(
    mut egui: EguiContexts,
) {

    egui::Window::new("Misc.").show(egui.ctx_mut(), |ui| {
        if let Some(base_dirs) = BaseDirs::new() {
            let data_dir = base_dirs.data_dir().join(".yavafg");
            let saves_dir = base_dirs.data_dir().join(".yavafg").join("saved_worlds");

            ui.heading("Data Dirs");
            ui.label(format!("Data dir: {}", data_dir.display()));
            ui.label(format!("Saves dir: {}", saves_dir.display()));
        }
    });
}

fn display_debug_ui_criteria(ui_state: Res<DebugUIState>) -> bool {
    ui_state.display_debug_info
}

fn display_mat_debug_ui_criteria(ui_state: Res<DebugUIState>) -> bool {
    ui_state.display_mat_debug
}

fn toggle_debug_ui_displays(
    keys: Res<Input<KeyCode>>,
    mut ui_state: ResMut<DebugUIState>,
    mut gizmos_config: ResMut<GizmoConfig>,
    mut _wireframe_config: ResMut<WireframeConfig>,
) {
    let f3 = keys.pressed(KeyCode::F3);
    let _f7 = keys.pressed(KeyCode::F7);
    let _shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

    if keys.just_pressed(KeyCode::F3) {
        ui_state.display_debug_info = !ui_state.display_debug_info;
    };

    if keys.just_pressed(KeyCode::F7) {
        ui_state.display_mat_debug = !ui_state.display_mat_debug;
    };

    if f3 && keys.just_pressed(KeyCode::X) {
        gizmos_config.enabled = !gizmos_config.enabled;
    };
}

fn display_material_editor(
    mut egui: EguiContexts,
    mut ui_state: ResMut<DebugUIState>,
    mut materials: ResMut<VoxelMaterialRegistry>,
) {
    egui::Window::new("material editor").show(egui.ctx_mut(), |ui| {
        ui.heading("Select material");
        egui::containers::ComboBox::from_label("Material")
            .selected_text(
                materials
                    .get_by_id(ui_state.selected_mat)
                    .unwrap()
                    .name
                    .to_string(),
            )
            .show_ui(ui, |content| {
                materials
                    .iter_mats()
                    .enumerate()
                    .for_each(|(mat_index, mat)| {
                        content.selectable_value(
                            &mut ui_state.selected_mat,
                            mat_index as u8,
                            mat.name,
                        );
                    })
            });

        ui.heading("Material properties");

        // base_color
        ui.label("Base color");

        let selected_mat = materials.get_mut_by_id(ui_state.selected_mat).unwrap();

        let mut editable_color = Rgba::from_rgba_unmultiplied(
            selected_mat.base_color.r(),
            selected_mat.base_color.g(),
            selected_mat.base_color.b(),
            selected_mat.base_color.a(),
        );
        egui::widgets::color_picker::color_edit_button_rgba(
            ui,
            &mut editable_color,
            egui::color_picker::Alpha::Opaque,
        );
        selected_mat.base_color = Color::from(editable_color.to_array());
        ui.label("Perceptual Roughness");
        ui.add(Slider::new(
            &mut selected_mat.perceptual_roughness,
            0.0..=1.0f32,
        ));
        ui.label("Metallic");
        ui.add(Slider::new(&mut selected_mat.metallic, 0.0..=1.0f32));
        ui.label("Reflectance");
        ui.add(Slider::new(&mut selected_mat.reflectance, 0.0..=1.0f32));
        ui.label("Emissive");

        let mut editable_emissive = Rgba::from_rgba_unmultiplied(
            selected_mat.emissive.r(),
            selected_mat.emissive.g(),
            selected_mat.emissive.b(),
            selected_mat.emissive.a(),
        );
        egui::widgets::color_picker::color_edit_button_rgba(
            ui,
            &mut editable_emissive,
            egui::color_picker::Alpha::Opaque,
        );
        selected_mat.emissive = Color::from(editable_emissive.to_array());

        // ui.label("Opacity");
        // ui.add(Slider::new(&mut selected_mat.opacity, 0.0..=1.0f32));
    });
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, SystemSet)]
/// Systems related to the debug UIs.
pub enum DebugUISet {
    Toggle,
    Display,
}

pub struct DebugUIPlugins;

impl Plugin for DebugUIPlugins {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(EguiPlugin)
            // .add_plugin(FrameTimeDiagnosticsPlugin)
            // .add_plugin(DebugLinesPlugin::with_depth_test(true))
            // .add_plugins(DebugLinesPlugin::default())
            .add_plugins(EntityCountDiagnosticsPlugin)
            .add_systems(Update, (
                toggle_debug_ui_displays.in_set(DebugUISet::Toggle),
                display_material_editor
                    .in_set(DebugUISet::Display)
                    .run_if(display_mat_debug_ui_criteria),
            ))
            .add_systems(
                Update,
                (
                    display_debug_stats,
                    display_world_info,
                    display_player_settings,
                    display_misc_info,
                    display_window_settings
                )
                    .in_set(DebugUISet::Display)
                    .distributive_run_if(display_debug_ui_criteria),
            )
            .configure_sets(
                Update,
                (DebugUISet::Toggle, DebugUISet::Display)
                    .chain()
                    .after(EguiSet::ProcessInput),
            )
            // .init_resource::<DebugUIState>();
            .insert_resource(DebugUIState {
                display_debug_info: true,
                display_mat_debug: false,
                selected_mat: 4,
                window_mode: WindowMode::Windowed,
                use_vsync: true,
            });
    }
}

#[derive(Resource)]
struct DebugUIState {
    display_debug_info: bool,
    display_mat_debug: bool,

    // DD
    pub selected_mat: u8,
    pub window_mode: WindowMode,
    pub use_vsync: bool,
}