use bevy::prelude::*;

use crate::{
    Snake,
    GameOverEvent,
};

pub const HEALTH_LOSS_PER_SECOND: f32 = 5.;

pub fn update_health(
    mut snake: Query<&mut Snake>,
    time: Res<Time>,
    mut ev_game_over: EventWriter<GameOverEvent>,
) {
    let mut snake = snake.single_mut();
    snake.health -= HEALTH_LOSS_PER_SECOND * time.delta_seconds();
    if snake.health <= 0. {
        ev_game_over.send_default();
    }
}
