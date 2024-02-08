use bevy::prelude::{AlignItems, Color, Commands, default, FlexDirection, JustifyContent, NodeBundle, Res, Style, TextBundle, TextStyle, Val, Visibility};
use bevy::asset::AssetServer;
use bevy::hierarchy::BuildChildren;
use crate::{BUTTON_FONT_SIZE, PauseMenu, PRIMARY_FONT_NAME, QuitButton, UpgradesButton};

pub fn spawn_pause_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                visibility: Visibility::Hidden,
                ..default()
            },
            PauseMenu,
        ))
        .with_children(|parent| {
            // container
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Paused",
                        TextStyle {
                            font: asset_server.load(PRIMARY_FONT_NAME),
                            font_size: 72.0,
                            color: Color::WHITE,
                        },
                    ));
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|parent| {
                            parent
                                .spawn((crate::get_button(), UpgradesButton, PauseMenu))
                                .with_children(|parent| {
                                    parent.spawn(TextBundle::from_section(
                                        "Upgrades",
                                        TextStyle {
                                            font: asset_server.load(PRIMARY_FONT_NAME),
                                            font_size: BUTTON_FONT_SIZE,
                                            color: Color::WHITE,
                                        },
                                    ));
                                });
                            parent
                                .spawn((crate::get_button(), QuitButton, PauseMenu))
                                .with_children(|parent| {
                                    parent.spawn(TextBundle::from_section(
                                        "Quit",
                                        TextStyle {
                                            font: asset_server.load(PRIMARY_FONT_NAME),
                                            font_size: BUTTON_FONT_SIZE,
                                            color: Color::WHITE,
                                        },
                                    ));
                                });
                        });
                });
        });
}
