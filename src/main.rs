#![allow(
    clippy::type_complexity,
    clippy::manual_clamp,
    clippy::module_inception
)]


use bevy::{
    prelude::*,
    sprite::{
        Material2dPlugin,
    },
    diagnostic::{
        Diagnostics,
        FrameTimeDiagnosticsPlugin
    }, pbr::wireframe::WireframePlugin, app::AppExit,
};
use bevy_embedded_assets::EmbeddedAssetPlugin;
use components::player_camera::PlayerCamera;
use voxel::player::PlayerSettings;

use std::f32::consts::PI;

use bevy::{core_pipeline::fxaa::Fxaa, prelude::*};

mod components;
mod systems;
mod materials;
mod resources;
mod debug;
mod voxel;

use crate::materials::pixelate::PixelateMaterial;

use bevy_asset_loader::prelude::*;
use iyes_progress::{ProgressCounter, ProgressPlugin};

fn main() {
    let mut app = App::default();
    app
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "World Gen".to_string(),
                    present_mode: bevy::window::PresentMode::AutoNoVsync,
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest())
            // .set(AssetPlugin {
            //     watch_for_changes: true,
            //     ..default()
            // })
        )
        .init_resource::<PlayerSettings>()
        .add_plugin(voxel::VoxelWorldPlugin)
        .add_plugin(debug::DebugUIPlugins)
        .add_startup_system(setup)
        // .add_system(update_camera_settings.after(setup))
        .run();
}


// fn main() {
//     App::new()
//         .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
//         .add_state::<GameState>()
//         .add_loading_state(
//             LoadingState::new(GameState::AssetLoading)
//                 .continue_to_state(GameState::Game)
//                 .on_failure_continue_to_state(GameState::AssetError)
//         )
//         .add_collection_to_loading_state::<_, MyAssets>(GameState::AssetLoading)
//         .init_resource::<PlayerInput>()
//         .init_resource::<PlayerSettings>()
//         .add_plugins(DefaultPlugins
//             .set(WindowPlugin {
//                 primary_window: Some(Window {
//                     title: "Worldgen".to_string(),
//                     present_mode: bevy::window::PresentMode::AutoNoVsync,
//                     ..default()
//                 }),
//                 ..default()
//             })
//             .set(ImagePlugin::default_nearest())
//             // .set(AssetPlugin {
//             //     watch_for_changes: true,
//             //     ..default()
//             // })
//         )
//         // .build()
//         // .add_before::<bevy::asset::AssetPlugin, _>(EmbeddedAssetPlugin))
//         .add_plugin(Material2dPlugin::<PixelateMaterial>::default())
//         .add_plugin(FrameTimeDiagnosticsPlugin::default())
//         .add_plugin(ProgressPlugin::new(GameState::AssetLoading).continue_to(GameState::Game))
//         .add_plugin(WireframePlugin)
//         .add_startup_system(setup_boot_screen)
//         // .add_system(print_progress.in_set(OnUpdate(GameState::AssetLoading)))
//         .add_system(asset_error.in_schedule(OnEnter(GameState::AssetError)))
//         .add_system(setup.in_schedule(OnEnter(GameState::Game)))
//         .add_system(setup_debug_text.in_schedule(OnEnter(GameState::Game)))
//         .add_system(init_input.in_schedule(OnEnter(GameState::Game)))
//         // .add_system(configure_gamepad.in_schedule(OnEnter(GameState::Game)))
//         .add_system(update_debug_text.in_set(OnUpdate(GameState::Game)))
//         .add_system(player_input.in_set(OnUpdate(GameState::Game)))
//         .add_system(grab_cursor.in_set(OnUpdate(GameState::Game)))
//         .add_system(resize_target.in_set(OnUpdate(GameState::Game)))
//         .add_system(rotate.in_set(OnUpdate(GameState::Game)))
//         .add_system(print_progress)
//         .run();
// }

#[derive(AssetCollection, Resource)]
pub struct MyAssets {
    #[asset(path = "textures/cobble-diffuse.png")]
    base_map: Handle<Image>,
    #[asset(path = "textures/cobble-normal.png")]
    normal_map: Handle<Image>,
    #[asset(path = "fonts/Alkhemikal.ttf")]
    font: Handle<Font>,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameState {
    #[default]
    AssetLoading,
    AssetError,
    Game,
}

fn asset_error(
    mut quit: EventWriter<AppExit>,
) {
    error!("Error while loading assets!");
    quit.send(AppExit);
}

fn print_progress(
    progress: Option<Res<ProgressCounter>>,
    diagnostics: Res<Diagnostics>,
    mut last_done: Local<u32>,
    mut progress_text: Query<&mut Text, With<ProgressText>>,
) {
    if let Some(progress) = progress.map(|counter| counter.progress()) {
        if progress.done > *last_done {
            *last_done = progress.done;
            for mut text in progress_text.iter_mut() {
                text.sections[0].value = format!(
                    "Loading... {:.0}%",
                    progress.done as f32 / progress.total as f32 * 100.
                );
            }
            info!(
                "[Frame {}] Changed progress: {:?}",
                diagnostics
                    .get(FrameTimeDiagnosticsPlugin::FRAME_COUNT)
                    .map(|diagnostic| diagnostic.value().unwrap_or(0.))
                    .unwrap_or(0.),
                progress
            );
        }
    }
}

#[derive(Component)]
pub struct StartText;

#[derive(Component)]
struct ProgressText;

fn setup_boot_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn((TextBundle::from_section(
        "Worldgen",
        TextStyle {
            font: asset_server.load("fonts/Alkhemikal.ttf"),
            font_size: 100.0,
            color: Color::WHITE,
        },
    )
    .with_text_alignment(TextAlignment::Center)
    .with_style(Style {
        position_type: PositionType::Absolute,
        position: UiRect {
            bottom: Val::Px(5.0),
            right: Val::Px(15.0),
            ..default()
        },
        ..default()
    }),
    StartText,
    ));

    commands.spawn((TextBundle::from_section(
        "Loading assets...",
        TextStyle {
            font: asset_server.load("fonts/Alkhemikal.ttf"),
            font_size: 50.0,
            color: Color::WHITE,
        },
    )
    .with_text_alignment(TextAlignment::Center)
    .with_style(Style {
        position_type: PositionType::Absolute,
        position: UiRect {
            bottom: Val::Px(15.0),
            left: Val::Px(15.0),
            ..default()
        },
        ..default()
    }),
    StartText,
    ProgressText,
    ));
}


fn setup(
    settings: Res<PlayerSettings>,
    mut cmds: Commands
) {
    cmds.spawn(Camera3dBundle {
        projection: bevy::prelude::Projection::Perspective(PerspectiveProjection {
            fov: settings.fov.to_radians(),
            far: 2048.0,
            ..Default::default()
        }),
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