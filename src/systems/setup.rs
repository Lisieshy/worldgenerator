use bevy::{
    prelude::*,
    pbr::wireframe::{
        Wireframe,
        WireframeConfig,
    },
    core_pipeline::clear_color::ClearColorConfig,
    render::{
        mesh::{
            Indices,
            PrimitiveTopology,
        },
        // Uncomment to re-enable post processing shader layer
        // camera::{
        //     RenderTarget,
        // },
        render_resource::{
            AddressMode,
            SamplerDescriptor,
            TextureDescriptor,
            TextureUsages,
            Extent3d,
            TextureDimension,
            TextureFormat,
        },
        texture::{
            BevyDefault,
            ImageSampler,
        },
        view::RenderLayers,
    },
    sprite::{
        MaterialMesh2dBundle,
    }, window::PrimaryWindow
};

use crate::{components::shape::Shape, resources::user_settings::PlayerSettings};
use crate::components::player_camera::PlayerCamera;
use crate::components::post_processing_handle::PostProcessingHandle;
use crate::materials::pixelate::PixelateMaterial;
use crate::MyAssets;
use crate::StartText;

pub fn setup(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut post_processing_materials: ResMut<Assets<PixelateMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut wireframe_config: ResMut<WireframeConfig>,
    settings: Res<PlayerSettings>,
    asset_server: Res<AssetServer>,
    my_assets: Res<MyAssets>,
    mut start_text: Query<&mut Text, With<StartText>>,
) {

    // let mut loaded = false;

    // while !loaded {
    //     match asset_server.get_group_load_state(loading.0.iter().map(|h| h.id)) {
    //         LoadState::Loaded => {
    //             loaded = true;
    //         }
    //         _ => {
    //             continue;
    //         }
    //     }
    // }

    for mut text in &mut start_text {
        text.sections.clear();
    }

    wireframe_config.global = false;
    let Ok(window) = windows.get_single() else {
        return;
    };
    let size = Extent3d {
        width: window.physical_width(),
        height: window.physical_height(),
        ..default()
    };

    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::bevy_default(),
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    image.resize(size);
    let image_handle = images.add(image);

    // let texture = asset_server.get_handle("textures/base-map.png");
    let panel2 = asset_server.get_handle(my_assets.normal_map.clone());
    let panel = asset_server.get_handle(my_assets.base_map.clone());

    if let Some(panel_image) = images.get_mut(&panel) {
        match &mut panel_image.sampler_descriptor {
            ImageSampler::Default => {
                panel_image.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
                    address_mode_u: AddressMode::Repeat,
                    address_mode_v: AddressMode::Repeat,
                    address_mode_w: AddressMode::Repeat,
                    ..default()
                });
            }
            ImageSampler::Descriptor(sampler_descriptor) => {
                sampler_descriptor.address_mode_u = AddressMode::Repeat;
                sampler_descriptor.address_mode_v = AddressMode::Repeat;
                sampler_descriptor.address_mode_w = AddressMode::Repeat;
            }
        };
    }
    if let Some(panel2_image) = images.get_mut(&panel2) {
        match &mut panel2_image.sampler_descriptor {
            ImageSampler::Default => {
                panel2_image.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
                    address_mode_u: AddressMode::Repeat,
                    address_mode_v: AddressMode::Repeat,
                    address_mode_w: AddressMode::Repeat,
                    ..default()
                });
            }
            ImageSampler::Descriptor(sampler_descriptor) => {
                sampler_descriptor.address_mode_u = AddressMode::Repeat;
                sampler_descriptor.address_mode_v = AddressMode::Repeat;
                sampler_descriptor.address_mode_w = AddressMode::Repeat;
            }
        };
    }

    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(panel.clone()),
        normal_map_texture: Some(panel2.clone()),
        flip_normal_map_y: true,
        alpha_mode: AlphaMode::Opaque,
        ..default()
    });

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            // top      (+y)
            [-0.5, 0.5, -0.5], // index 0
            [0.5, 0.5, -0.5],
            [0.5, 0.5, 0.5],
            [-0.5, 0.5, 0.5],
            // bottom   (-y)
            [-0.5, -0.5, -0.5],
            [0.5, -0.5, -0.5],
            [0.5, -0.5, 0.5],
            [-0.5, -0.5, 0.5],
            // right    (+x)
            [0.5, -0.5, -0.5],
            [0.5, -0.5, 0.5],
            [0.5, 0.5, 0.5],
            [0.5, 0.5, -0.5],
            // left     (-x)
            [-0.5, -0.5, -0.5],
            [-0.5, -0.5, 0.5],
            [-0.5, 0.5, 0.5],
            [-0.5, 0.5, -0.5],
            // back     (+z)
            [-0.5, -0.5, 0.5],
            [-0.5, 0.5, 0.5],
            [0.5, 0.5, 0.5],
            [0.5, -0.5, 0.5],
            // forward  (-z)
            [-0.5, -0.5, -0.5],
            [-0.5, 0.5, -0.5],
            [0.5, 0.5, -0.5],
            [0.5, -0.5, -0.5],
        ],
    );

    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vec![
            // [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0] = default UV coords
            [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
            [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
            [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
            [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
            [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
            [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
            // UV Coords for dice texture
            // Assigning the UV coords for the top side.
            // [0.25, 0.33], [0.5, 0.33], [0.5, 0.0], [0.25, 0.0],
            // Assigning the UV coords for the bottom side.
            // [0.25, 0.66], [0.5, 0.66], [0.5, 1.0], [0.25, 1.0],
            // Assigning the UV coords for the right side.
            // [0.5, 0.66], [0.75, 0.66], [0.75, 0.33], [0.5, 0.33],
            // Assigning the UV coords for the left side. 
            // [0.0, 0.66], [0.25, 0.66], [0.25, 0.33], [0.0, 0.33],
            // Assigning the UV coords for the back side.
            // [0.25, 0.66], [0.5, 0.66], [0.5, 0.33], [0.25, 0.33],
            // Assigning the UV coords for the forward side.
            // [0.75, 0.66], [1.0, 0.66], [1.0, 0.33], [0.75, 0.33],
        ]
    );

    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        vec![
            // Normals for the top side (towards +y)
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            // Normals for the bottom side (towards -y)
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            // Normals for the right side (towards +x)
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            // Normals for the left side (towards -x)
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            // Normals for the back side (towards +z)
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            // Normals for the forward side (towards -z)
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
        ]
    );

    mesh.set_indices(Some(Indices::U32(vec![
        0,3,1 , 1,3,2, // top (+y)
        4,5,7 , 5,6,7, // bottom (-y)
        8,11,9 , 9,11,10, // right (+x)
        12,13,15 , 13,14,15, // left (-x)
        16,19,17 , 17,19,18, // back (+z)
        20,21,23 , 21,22,23 // forward (-z)
    ])));

    let mut _tangents = mesh.generate_tangents();


    // commands.spawn((
    //     PbrBundle {
    //         mesh: meshes.add(mesh),
    //         material: material_handle,
    //         transform: Transform::from_xyz(0.0, 0.0, 0.0),
    //         ..default()
    //     },
    //     Wireframe,
    // ));

    // creates a 10x10x10 cube of cubes

    for x in 0..40 {
        for y in 0..1 {
            for z in 0..40 {
                commands.spawn((
                    PbrBundle {
                        mesh: meshes.add(mesh.clone()),
                        // material: materials.add(StandardMaterial {
                        //     base_color: Color::rgb(0.0, 0.0, 0.0),
                        //     ..default()
                        // }),
                        material: material_handle.clone(),
                        transform: Transform::from_xyz(x as f32, y as f32, z as f32),
                        ..default()
                    },
                    Wireframe,
                ));
            }
        }
    }

    commands.spawn((
        PointLightBundle {
            point_light: PointLight {
                intensity: 4000.0,
                range: 100.,
                shadows_enabled: false,
                ..default()
            },
            transform: Transform::from_xyz(4.0, 4.0, 4.0),
            ..default()
        },
        Shape,
    ));

    commands.spawn((
        Camera3dBundle {
            camera_3d: Camera3d {
                clear_color: ClearColorConfig::Custom(Color::rgb(0.7, 0.7, 0.7)),
                ..default()
            },
            camera: Camera {
                // Uncomment to re-enable post processing shader layer
                // target: RenderTarget::Image(image_handle.clone()),
                order: 1,
                ..default()
            },
            
            projection: Projection::Perspective(PerspectiveProjection {
                fov: settings.fov,
                near: 0.01,
                far: 1000.0,
                aspect_ratio: 1.0,
            }),
            transform: Transform::from_xyz(0.0, 0.0, 2.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
            ..default()
        },
        // UiCameraConfig { show_ui: false },
        PlayerCamera,
    ));

    let post_processing_pass_layer = RenderLayers::layer((RenderLayers::TOTAL_LAYERS - 1) as u8);
    
    let material_handle = post_processing_materials.add(PixelateMaterial {
        source_image: image_handle,
        pixelsize: Vec2::new(0.1, 0.1),
    });

    let quad_handle = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
        size.width as f32,
        size.height as f32,
    ))));

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: quad_handle.into(),
            material: material_handle.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 1.5),
                ..default()
            },
            ..default()
        },
        PostProcessingHandle {
            mat_handle: material_handle
        },
        post_processing_pass_layer,
    ));

    // Uncomment to re-enable post processing shader layer
    // commands.spawn((
    //     Camera2dBundle {
    //         camera: Camera {
    //             order: 1,
    //             ..default()
    //         },
    //         ..Camera2dBundle::default()
    //     },
    //     post_processing_pass_layer,
    // ));
}
