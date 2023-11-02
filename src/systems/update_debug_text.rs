// use bevy::{
//     prelude::*,
//     diagnostic::*,
// };

// use crate::components::text_change::TextChanges;

// pub fn update_debug_text(
//     time: Res<Time>,
//     diagnostics: Res<DiagnosticsStore>,
//     mut query: Query<&mut Text, With<TextChanges>>
// ) {
//     for mut text in &mut query {
//         let mut fps = 0.0;
//         if let Some(fps_diagnostic) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
//             if let Some(fps_average) = fps_diagnostic.average() {
//                 fps = fps_average;
//             }
//         }

//         let mut frame_time = time.delta_seconds_f64();
//         if let Some(frame_time_diagnostic) = diagnostics.get(FrameTimeDiagnosticsPlugin::FRAME_TIME)
//         {
//             if let Some(frame_time_smoothed) = frame_time_diagnostic.smoothed() {
//                 frame_time = frame_time_smoothed;
//             }
//         }

//         text.sections[0].value = format!(
//             "FPS: {:.1}\nms: {:.3}\n",
//             fps, frame_time,
//         );
//     }
// }