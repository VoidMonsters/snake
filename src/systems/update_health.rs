use bevy::prelude::*;

use crate::{
    Snake,
    GameOverEvent,
    HungerRate,
};

pub fn update_health(
    mut snake: Query<&mut Snake>,
    time: Res<Time>,
    mut ev_game_over: EventWriter<GameOverEvent>,
    hunger_rate: Res<HungerRate>,
) {
    let mut snake = snake.single_mut();
    let HungerRate(hunger_rate) = *hunger_rate;
    snake.health -= hunger_rate * time.delta_seconds();
    if snake.health <= 0. {
        ev_game_over.send_default();
    }
}
