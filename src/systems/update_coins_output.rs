use bevy::prelude::*;

use crate::{
    CoinbagValueOutput,
    CoinsOutput,
    CoinBag,
    Game,
};

pub fn update_coins_output(
    mut coins_text: Query<&mut Text, With<CoinsOutput>>, 
    mut bagvalue_text: Query<&mut Text, (With<CoinbagValueOutput>, Without<CoinsOutput>)>,
    bag: Query<&CoinBag>,
    game: Res<Game>,
) {
    let mut coins_text = coins_text.single_mut();
    coins_text.sections[0].value = format!("Coins: {0:>12.2}", game.coins);
    let mut bagvalue_text = bagvalue_text.single_mut();
    bagvalue_text.sections[0].value = if bag.is_empty() {
        format!("")
    } else {
        let bag = bag.single();
        format!("Bag Value: {0:>8.2}", bag.value)
    }
}
