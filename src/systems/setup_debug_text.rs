use bevy::prelude::*;

use crate::{components::text_change::TextChanges, MyAssets};

pub fn setup_debug_text(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    my_assets: Res<MyAssets>,
) {
    let font = asset_server.get_handle(my_assets.font.clone());

    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "",
                TextStyle {
                    font: font.clone(),
                    font_size: 30.0,
                    color: Color::RED,
                },
            ),
            TextSection::new(
                "",
                TextStyle {
                    font: font.clone(),
                    font_size: 30.0,
                    color: Color::ORANGE_RED,
                },
            ),
            TextSection::new(
                "",
                TextStyle {
                    font: font.clone(),
                    font_size: 30.0,
                    color: Color::MAROON,
                },
            ),
            TextSection::new(
                "",
                TextStyle {
                    font: font.clone(),
                    font_size: 30.0,
                    color: Color::ORANGE,
                },
            ),
            TextSection::new(
                "",
                TextStyle {
                    font: font.clone(),
                    font_size: 30.0,
                    color: Color::BLUE,
                },
            ),
        ]),
        TextChanges,
    ));
}