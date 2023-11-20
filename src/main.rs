#![allow(
    clippy::type_complexity,
    clippy::manual_clamp,
    clippy::module_inception
)]


use bevy::{
    prelude::*,
    diagnostic::FrameTimeDiagnosticsPlugin, render::{RenderPlugin, settings::{WgpuSettings, WgpuFeatures}}, pbr::wireframe::WireframePlugin
        // Diagnostics,
    // },
    // app::AppExit,
};
// use bevy_embedded_assets::EmbeddedAssetPlugin;
use voxel::player::PlayerSettings;


use bevy::core_pipeline::fxaa::Fxaa;

mod systems;
mod resources;
mod debug;
mod voxel;

fn main() {
    let mut app = App::default();
    app
        // .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        // .add_state::<GameState>()
        // .add_loading_state(
        //     LoadingState::new(GameState::AssetLoading)
        //         .continue_to_state(GameState::GameRunning)
        //         .on_failure_continue_to_state(GameState::AssetError)
        // )
        // .add_collection_to_loading_state::<_, MyAssets>(GameState::AssetLoading)
        .add_plugins((DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "World Generator".to_string(),
                    present_mode: bevy::window::PresentMode::AutoNoVsync,
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest())
            .set(RenderPlugin {
                render_creation: WgpuSettings {
                    features: WgpuFeatures::POLYGON_MODE_LINE,
                    ..default()
                }.into(),
            }),
            // .set(AssetPlugin {
            //     watch_for_changes: true,
            //     ..default()
            // })
            WireframePlugin,
        ))
        .init_resource::<PlayerSettings>()
        .add_plugins(FrameTimeDiagnosticsPlugin)
        // .add_plugin(ProgressPlugin::new(GameState::AssetLoading).continue_to(GameState::GameRunning))
        .add_plugins(voxel::VoxelWorldPlugin)
        .add_plugins(debug::DebugUIPlugins)
        // .add_startup_system(setup_boot_screen)
        .add_systems(Startup, setup)
        // .add_system(asset_error.in_schedule(OnEnter(GameState::AssetError)))
        // .add_system(print_progress)
        // .add_system(update_camera_settings.after(setup))
        .run();
}

fn setup(
    settings: Res<PlayerSettings>,
    mut cmds: Commands
) {
    cmds.spawn(Camera3dBundle {
        projection: bevy::prelude::Projection::Perspective(PerspectiveProjection {
            fov: settings.fov.to_radians(),
            far: 4096.0,
            ..Default::default()
        }),
        camera: Camera {
            order: 1,
            ..default()
        },
        transform: Transform::from_xyz(2.0, 180.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    })
    .insert(voxel::player::PlayerController::default())
    .insert(Fxaa::default())
    .insert(bevy_atmosphere::plugin::AtmosphereCamera::default());

    cmds.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1.0,
    });
}