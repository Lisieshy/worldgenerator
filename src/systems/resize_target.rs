// use bevy::{
//     math::*,
//     prelude::*,
//     render::render_resource::Extent3d,
//     sprite::Mesh2dHandle,
//     window::WindowResized,
// };

// use crate::materials::pixelate::PixelateMaterial;
// use crate::components::post_processing_handle::PostProcessingHandle;
// use crate::components::text_change::TextChanges;

// pub fn resize_target(
//     mut events: EventReader<WindowResized>,
//     pph_handle: Query<&PostProcessingHandle>,
//     mut query_handle_2d: Query<&mut Mesh2dHandle, With<PostProcessingHandle>>,
//     mut query_text: Query<&mut Text, With<TextChanges>>,
//     mut images: ResMut<Assets<Image>>,
//     mut materials: ResMut<Assets<PixelateMaterial>>,
//     mut meshes: ResMut<Assets<Mesh>>,
// ) {
//     let mut window_resized = None;
//     for e in events.iter() {
//         window_resized = Some(e);
//     }

//     if let Some(e) = window_resized {
//         let new_pixel_size = e.height as f32 / e.height as f32;
//         for mut text in &mut query_text {
//             text.sections[1].value = format!(
//                 "Logical Window: {}*{}\n",
//                 e.width, e.height,
//             );
//             // text.sections[2].value = format!(
//             //     "pixel shader: {:.2}\n",
//             //     new_pixel_size,
//             // )
//         }
//         let size = Extent3d {
//             width: e.width as u32,
//             height:  e.height as u32,
//             ..default()
//         };
//         if let Ok(mut mesh_handle) = query_handle_2d.get_single_mut() {
//             meshes.remove(mesh_handle.0.clone());
//             mesh_handle.0 = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
//                 size.height as f32 * (16. / 9.),
//                 size.height as f32,
//             ))));
//         }
//         if let Ok(ppmat_handle) = pph_handle.get_single() {
//             if let Some(ppmat) = materials.get_mut(&ppmat_handle.mat_handle) {
//                 ppmat.pixelsize = Vec2::new(new_pixel_size, new_pixel_size);
//                 if let Some(camera_target) = images.get_mut(&ppmat.source_image) {
//                     camera_target.texture_descriptor.size = size;
//                     camera_target.resize(camera_target.texture_descriptor.size);
//                 }
//             }
//         }
//     }
// }
