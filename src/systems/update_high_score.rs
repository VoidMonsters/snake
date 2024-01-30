use bevy::prelude::*;

use crate::{
    HighScore,
    Game,
};

pub fn update_high_score(game: Res<Game>, high_score: Res<HighScore>) {
    if game.score > high_score.get() {
        high_score.save(game.score);
    }
}
