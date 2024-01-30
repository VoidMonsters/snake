use bevy::prelude::*;

use crate::{
    FOOD_LAYER,
    CoinBag,
    RandNormalized,
};

use rand::Rng;

// the minimum distance from the edge of the screen that coins spawn at
const COIN_BOUNDARY: f32 = 128.;

pub fn spawn_coins(mut commands: Commands, asset_server: Res<AssetServer>, window: Query<&Window>) {
    let mut coins_location = Vec3::random();
    let window = window.single();
    let boundary_x = (window.resolution.width() / 2.) - COIN_BOUNDARY;
    let boundary_y = (window.resolution.height() / 2.) - COIN_BOUNDARY;

    coins_location.x -= 0.5;
    coins_location.y -= 0.5;

    coins_location.x *= boundary_x * 2.;
    coins_location.y *= boundary_y * 2.;

    coins_location.z = FOOD_LAYER;
    commands.spawn((
        CoinBag {
            value: (rand::thread_rng().gen_range(8.0..12.0f32) * 100.0).round() / 100.0,
        },
        SpriteBundle {
            texture: asset_server.load("sprites/coinbag.png"),
            transform: Transform {
                translation: coins_location,
                scale: Vec3::new(0.3, 0.3, 1.0),
                ..default()
            },
            ..default()
        },
    ));
}
