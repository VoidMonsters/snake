use bevy::prelude::*;

use crate::SnakeSpeed;

pub fn increase_speed(
    mut speed: ResMut<SnakeSpeed>,
) {
    speed.analog += 50.;
    speed.discrete += 1.;
}
