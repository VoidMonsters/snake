use bevy::prelude::*;
use crate::{
    UpgradeIcon,
    UpgradeIconClickedEvent,
    Upgrades,
    IconHoverEffectMaterial,
    Game,
};

pub fn upgrade_menu_event_handler(
    upgrade_icons: Query<(&UpgradeIcon, &Interaction), (With<UpgradeIcon>, Changed<Interaction>)>,
    mut ev_upgrade_icon_clicked: EventWriter<UpgradeIconClickedEvent>,
    mut upgrades: ResMut<Upgrades>,
    mut materials: ResMut<Assets<IconHoverEffectMaterial>>,
    game: Res<Game>,
) {
    for (icon, interaction) in &upgrade_icons {
        let hover_effect_material = materials.iter_mut().find(|(_,m)| m.upgrade_id == icon.upgrade.id);
        match *interaction {
            Interaction::Pressed => {
                if let Some((_, hover_effect_material)) = hover_effect_material {
                    hover_effect_material.color = if game.coins > icon.upgrade.price {
                        Color::GREEN.into()
                    } else {
                        Color::RED.into()
                    }
                }
                ev_upgrade_icon_clicked.send(UpgradeIconClickedEvent { icon: icon.clone() });
            }
            Interaction::Hovered => {
                let index = upgrades.index_of(&icon.upgrade);
                if let Some(index) = index {
                    upgrades.selected_index = index;
                    if let Some((_, hover_effect_material)) = hover_effect_material {
                        hover_effect_material.highlight = 1;
                    }
                } else {
                    // unreachable
                    panic!("Somehow the user has hovered over an upgrade that doesn't exist!");
                }
            }
            Interaction::None => {
                let index = upgrades.index_of(&icon.upgrade);
                if let Some(index) = index {
                    upgrades.selected_index = index;
                    if let Some((_, hover_effect_material)) = hover_effect_material {
                        hover_effect_material.highlight = 0;
                        hover_effect_material.color = Color::WHITE.into();
                    }
                } else {
                    // unreachable
                    panic!("Somehow the user has hovered over an upgrade that doesn't exist!");
                }
            }
        }
    }
}
