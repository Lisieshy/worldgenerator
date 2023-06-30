// use crate::resources::my_gamepad::MyGamepad;

// use bevy::prelude::*;
// use bevy::input::gamepad::{
//     GamepadAxisChangedEvent,
//     GamepadButtonChangedEvent,
//     GamepadConnectionEvent,
//     GamepadEvent
// };

// use bevy::input::gamepad::GamepadEvent;

// pub fn gamepad_connections(
//     mut commands: Commands,
//     my_gamepad: Option<Res<MyGamepad>>,
//     mut gamepad_evr: EventReader<GamepadEvent>,
// ) {
//     for ev in gamepad_evr.iter() {
//         let id = ev.gamepad;
//         match &ev.event_type {
//             GamepadEventType::Connected(_info) => {
//                 if my_gamepad.is_none() {
//                     commands.insert_resource(MyGamepad(id));
//                 }
//             }
//             GamepadEventType::Disconnected => {
//                 if let Some(MyGamepad(old_id)) = my_gamepad.as_deref() {
//                     if *old_id == id {
//                         commands.remove_resource::<MyGamepad>();
//                     }
//                 }
//             }
//             _ => {}
//         }
//     }
// }

// pub fn gamepad_connections(
//     mut commands: Commands,
//     my_gamepad: Option<Res<MyGamepad>>,
//     mut gamepad_evr: EventReader<GamepadConnectionEvent>,
// ) {
//     for ev in gamepad_evr.iter() {
//         ev.
//     }
// }