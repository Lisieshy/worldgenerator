use bevy::{
    diagnostic::{Diagnostics, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin},
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::{
        Color, CoreSet, EventReader, IntoSystemConfig, IntoSystemConfigs, IntoSystemSetConfigs,
        KeyCode, Plugin, Res, ResMut, Resource, SystemSet, Vec3, IVec3, Transform, Query, PerspectiveProjection, With, Quat,
    },
};
use bevy_egui::{
    egui::{self, Rgba, Slider},
    EguiContexts, EguiPlugin, EguiSet,
};

use bevy_prototype_debug_lines::*;

use crate::{voxel::{
    material::VoxelMaterialRegistry, ChunkCommandQueue, ChunkEntities, ChunkLoadRadius,
    CurrentLocalPlayerChunk, DirtyChunks,
    CHUNK_LENGTH, player::{PlayerSettings, PlayerController},
}};

fn display_debug_stats(mut egui: EguiContexts, diagnostics: Res<Diagnostics>) {
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

fn display_player_settings(
    mut egui: EguiContexts,
    mut settings: ResMut<PlayerSettings>,
    mut lines: ResMut<DebugLines>,
    mut query: Query<(&mut PlayerController, &mut Transform)>,
) {
    let (controller, transform) = query.single_mut();

    let direction = Quat::from_rotation_y(controller.yaw) * Quat::from_rotation_x(controller.pitch) * Vec3::new(0.0, 0.0, -1.0);

    let endpoint = transform.translation + direction * 10.0;

    let up_endpoint = endpoint + Vec3::new(0.0, 1.0, 0.0);
    let right_endpoint = endpoint + Vec3::new(1.0, 0.0, 0.0);
    let forward_endpoint = endpoint + Vec3::new(0.0, 0.0, 1.0);

    egui::Window::new("player settings").show(egui.ctx_mut(), |ui| {
        ui.heading("Controller info");
        ui.label(format!("Use gamepad: {}", settings.use_gamepad));
        ui.label(format!("Gamepad: {:?}", settings.gamepad));
        ui.separator();
        ui.heading("sensitivity");
        ui.label(format!("Mouse sensitivity"));
        ui.add(Slider::new(&mut settings.mouse_sensitivity, 0.0001..=0.001f32));
        ui.label(format!("Controller sensitivity"));
        ui.add(Slider::new(&mut settings.gamepad_sensitivity, 0.1..=1.0f32));
        ui.separator();
        ui.heading("speed");
        ui.label(format!("Movement speed"));
        ui.add(Slider::new(&mut settings.speed, 1.0..=20.0f32));
        ui.label(format!("Sprint speed multiplier"));
        ui.add(Slider::new(&mut settings.sprint_speed_mult, 1.0..=10.0f32));
        ui.separator();
        ui.heading("camera");
        ui.label(format!("FOV"));
        ui.add(Slider::new(&mut settings.fov, 30.0..=120.0f32));
        ui.label(format!("Camera yaw: {}", controller.yaw.to_degrees()));
        ui.label(format!("Camera pitch: {}", controller.pitch.to_degrees()));
        // ui.label(format!("Camera direction: {:?}", direction));
        ui.label(format!("Camera endpoint: {:?}", endpoint));
    });

    lines.line_colored(
        endpoint,
        up_endpoint,
        0.,
        Color::GREEN,
    );
    lines.line_colored(
        endpoint,
        right_endpoint,
        0.,
        Color::RED,
    );
    lines.line_colored(
        endpoint,
        forward_endpoint,
        0.,
        Color::BLUE,
    );
}

fn draw_chunk_borders(
    mut lines: ResMut<DebugLines>,
    mut shapes: ResMut<DebugShapes>,
    chunk: IVec3,
) {
    let length = CHUNK_LENGTH as f32;
    let cube_x = chunk.x as f32 + length / 2.;
    let cube_y = chunk.y as f32 + length / 2.;
    let cube_z = chunk.z as f32 + length / 2.;

    shapes
        .cuboid()
        .position(Vec3::new(cube_x, cube_y, cube_z))
        .size(Vec3::splat(length))
        .color(Color::RED);

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

fn display_main_stats(
    mut egui: EguiContexts,
    dirty_chunks: Res<DirtyChunks>,
    player_pos: Res<CurrentLocalPlayerChunk>,
    mut chunk_loading_radius: ResMut<ChunkLoadRadius>,
    mut chunk_command_queue: ResMut<ChunkCommandQueue>,
    lines: ResMut<DebugLines>,
    shapes: ResMut<DebugShapes>,
    loaded_chunks: Res<ChunkEntities>,
) {
    egui::Window::new("voxel world stuff").show(egui.ctx_mut(), |ui| {
        ui.heading("Chunks");
        ui.label(format!(
            "Chunks invalidations (per frame):  {}",
            dirty_chunks.num_dirty()
        ));
        ui.label(format!("Loaded chunk count: {}", loaded_chunks.len()));
        ui.separator();
        ui.label("Horizontal chunk loading radius");
        ui.add(Slider::new(&mut chunk_loading_radius.horizontal, 8..=32));
        ui.label("Vertical chunk loading radius");
        ui.add(Slider::new(&mut chunk_loading_radius.vertical, 2..=10));
        ui.separator();

        if ui.button("Clear loaded chunks").clicked() {
            chunk_command_queue.queue_unload(loaded_chunks.iter_keys());
        }
        ui.separator();

        ui.heading("Current player position");
        ui.label(format!("Current position : {}", player_pos.world_pos));
        ui.label(format!("Current chunk : {:?}", player_pos.chunk_min));
    });

    draw_chunk_borders(lines, shapes, player_pos.chunk_min);
}

fn display_debug_ui_criteria(ui_state: Res<DebugUIState>) -> bool {
    ui_state.display_debug_info
}

fn display_mat_debug_ui_criteria(ui_state: Res<DebugUIState>) -> bool {
    ui_state.display_mat_debug
}

fn toggle_debug_ui_displays(
    mut inputs: EventReader<KeyboardInput>,
    mut ui_state: ResMut<DebugUIState>,
) {
    for input in inputs.iter() {
        match input.key_code {
            Some(key_code) if key_code == KeyCode::F3 && input.state == ButtonState::Pressed => {
                ui_state.display_debug_info = !ui_state.display_debug_info;
            }
            Some(key_code) if key_code == KeyCode::F7 && input.state == ButtonState::Pressed => {
                ui_state.display_mat_debug = !ui_state.display_mat_debug;
            }
            _ => {}
        }
    }
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

        let mut selected_mat = materials.get_mut_by_id(ui_state.selected_mat).unwrap();

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
        app.add_plugin(EguiPlugin)
            .add_plugin(FrameTimeDiagnosticsPlugin)
            // .add_plugin(DebugLinesPlugin::with_depth_test(true))
            .add_plugin(DebugLinesPlugin::default())
            .add_plugin(EntityCountDiagnosticsPlugin)
            .add_systems((
                toggle_debug_ui_displays.in_set(DebugUISet::Toggle),
                display_material_editor
                    .in_set(DebugUISet::Display)
                    .run_if(display_mat_debug_ui_criteria),
            ))
            .add_systems(
                (display_debug_stats, display_main_stats, display_player_settings)
                    .in_set(DebugUISet::Display)
                    .distributive_run_if(display_debug_ui_criteria),
            )
            .configure_sets(
                (DebugUISet::Toggle, DebugUISet::Display)
                    .chain()
                    .in_base_set(CoreSet::Update)
                    .after(EguiSet::ProcessInput),
            )
            .init_resource::<DebugUIState>();
    }
}

#[derive(Default, Resource)]
struct DebugUIState {
    display_debug_info: bool,
    display_mat_debug: bool,

    // DD
    pub selected_mat: u8,
}