use bevy::prelude::*;

use crate::{
    UpgradeMenuButtonClickedEvent,
    UpgradesMenu,
    PauseMenu,
    QuitButton,
    UpgradesButton,
    Systems,
};

pub fn upgrade_menu_handler(
    mut commands: Commands,
    mut ev_upgrade_menu_pressed: EventReader<UpgradeMenuButtonClickedEvent>,
    mut upgrades_menu: Query<&mut Visibility, With<UpgradesMenu>>,
    mut pause_menu: Query<
        &mut Visibility,
        (
            With<PauseMenu>,
            Without<UpgradesMenu>,
            Without<QuitButton>,
            Without<UpgradesButton>,
        ),
    >,
    systems: Res<Systems>,
) {
    for _ in ev_upgrade_menu_pressed.read() {
        let mut pause_menu = pause_menu.single_mut();
        if upgrades_menu.is_empty() {
            commands.run_system(systems.spawn_upgrades_menu);
            *pause_menu = Visibility::Hidden;
        } else {
            let mut upgrades_menu = upgrades_menu.single_mut();
            *upgrades_menu = Visibility::Visible;
            *pause_menu = Visibility::Hidden;
        }
    }
}
