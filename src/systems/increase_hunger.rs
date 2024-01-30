use bevy::prelude::*;

use crate::SnakeMaxHealth;

pub fn increase_hunger(
    mut max_health: ResMut<SnakeMaxHealth>,
) {
    max_health.0 += 50.;
}
