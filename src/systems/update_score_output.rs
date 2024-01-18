use bevy::prelude::*;

use crate::{
    Game,
    ScoreOutput,
    HighScore,
};

pub fn update_score_output(
    mut score_text: Query<&mut Text, With<ScoreOutput>>,
    game: Res<Game>,
    high_score: Res<HighScore>,
) {
    let mut score_text = score_text.single_mut();
    score_text.sections[0].value = format!(
        "Last High Score: {0}\nScore: {1}",
        high_score.get(),
        game.score
    );
}
