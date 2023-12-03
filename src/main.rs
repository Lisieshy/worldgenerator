#![allow(
    clippy::type_complexity,
    clippy::manual_clamp,
    clippy::module_inception
)]

use bevy::{
    prelude::*,
    diagnostic::FrameTimeDiagnosticsPlugin, render::{RenderPlugin, settings::{WgpuSettings, WgpuFeatures}}, pbr::wireframe::WireframePlugin,
        // Diagnostics,
    // },
    // app::AppExit,
};

// use bevy_embedded_assets::EmbeddedAssetPlugin;
use core::player::PlayerSettings;

use bevy::core_pipeline::fxaa::Fxaa;

use bevy_asset_loader::prelude::*;

use directories::BaseDirs;

mod systems;
mod debug;
mod core;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Loading,
    InGame,
}


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
            // TemporalAntiAliasPlugin,
            WireframePlugin,
        ))
        .add_state::<AppState>()
        .add_loading_state(
            LoadingState::new(AppState::Loading).continue_to_state(AppState::InGame),
        )
        .add_collection_to_loading_state::<_, MyAssets>(AppState::Loading)
        .add_collection_to_loading_state::<_, BlockTextures>(AppState::Loading)
        .init_resource::<PlayerSettings>()
        .add_plugins(FrameTimeDiagnosticsPlugin)
        // .add_plugin(ProgressPlugin::new(GameState::AssetLoading).continue_to(GameState::GameRunning))
        .add_plugins(core::VoxelWorldPlugin)
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
    // basically useless here for now, might be useful when a world selection screen/menus are added.
    if let Some(base_dirs) = BaseDirs::new() {
        let data_dir = base_dirs.data_dir().join(".yavafg"); // yet another voxel and fantasy game, root folder
        info!("root data directory: {}", data_dir.display());

        let saves_dir = base_dirs.data_dir().join(".yavafg").join("saved_worlds");
        info!("saves directory: {}", saves_dir.display());

        std::fs::create_dir_all(data_dir.as_path()).unwrap();
        std::fs::create_dir_all(saves_dir.as_path()).unwrap();

    } else {
        panic!("No valid directory path could be retrieved from the operating system.");
    }


    cmds.spawn(Camera3dBundle {
        projection: bevy::prelude::Projection::Perspective(PerspectiveProjection {
            fov: settings.fov.to_radians(),
            far: 4096.0,
            ..Default::default()
        }),
        camera: Camera {
            // hdr: true,
            order: 1,
            ..default()
        },
        // todo: Serialize player data to file and load it before spawning everything.
        // will most likely need a complete rewrite of the app startup process because
        // it will inevitably lead to the creation of some kind of main menu and world
        // selection screen.
        transform: Transform::from_xyz(2.0, 180.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    })
    .insert(core::player::PlayerController::default())
    // .insert(ScreenSpaceAmbientOcclusionBundle::default())
    // .insert(TemporalAntiAliasBundle::default())
    .insert(Fxaa::default())
    .insert(bevy_atmosphere::plugin::AtmosphereCamera::default());

    cmds.insert_resource(AmbientLight {
        brightness: 1.0,
        ..Default::default()
    });
}

#[derive(AssetCollection, Resource)]
struct MyAssets {
    // #[asset(path = "textures/tiles", collection(typed))]
    // tiles: Vec<Handle<Image>>,

    #[asset(path = "textures/crosshair.png")]
    crosshair: Handle<Image>,
}


#[derive(AssetCollection, Resource)]
struct BlockTextures {
    #[asset(path = "textures/blocks/0_void.png")]
    void: Handle<Image>,
    #[asset(path = "textures/blocks/bedrock.png")]
    bedrock: Handle<Image>,
    #[asset(path = "textures/blocks/stone.png")]
    rock: Handle<Image>,
    #[asset(path = "textures/blocks/dirt.png")]
    dirt: Handle<Image>,
    #[asset(path = "textures/blocks/sand.png")]
    sand: Handle<Image>,
    #[asset(path = "textures/blocks/grass_block_top.png")]
    grass: Handle<Image>,
    #[asset(path = "textures/blocks/snow.png")]
    snow: Handle<Image>,
    #[asset(path = "textures/blocks/water.png")]
    water: Handle<Image>,
    #[asset(path = "textures/blocks/sandstone_top.png")]
    sandstone: Handle<Image>,
    #[asset(path = "textures/blocks/cactus_top.png")]
    cactus: Handle<Image>,
    #[asset(path = "textures/blocks/oak_log.png")]
    wood: Handle<Image>,
    #[asset(path = "textures/blocks/oak_leaves.png")]
    leaves: Handle<Image>,
    #[asset(path = "textures/blocks/spruce_leaves.png")]
    pineleaves: Handle<Image>,
    #[asset(path = "textures/blocks/spruce_log.png")]
    pinewood: Handle<Image>,
}