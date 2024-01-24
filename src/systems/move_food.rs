use bevy::prelude::*;

use crate::{
    Food,
    Velocity,
    RandNormalized,
    GameFieldSize,
    FOOD_RADIUS,
    Snake,
};

use rand::random;

const FOOD_MOVE_SPEED: f32 = 200.;

fn should_change_direction(chance: f32) -> bool {
    random::<f32>() < chance
}

// this doesn't work as expected and I have no idea why, it's behaving very bizarrely
pub fn move_food(
    mut food: Query<(&mut Transform, &mut Velocity), With<Food>>,
    mut snake: Query<&Transform, (With<Snake>, Without<Food>)>,
    time: Res<Time>,
    gamefield_size: Res<GameFieldSize>,
) {
    let snake = snake.single_mut();
    let (mut food_transform, mut food_velocity) = food.single_mut();
    let Velocity(ref mut food_velocity) = *food_velocity;
    let distance = food_transform.translation.distance(snake.translation);
    if should_change_direction((1./distance) * 10.) {
        *food_velocity = Vec3::random();
        food_velocity.x -= 0.5;
        food_velocity.y -= 0.5;
        *food_velocity *= FOOD_MOVE_SPEED * 2.;
        food_velocity.z = 0.;
        let rotate_to_velocity = Quat::from_rotation_arc(Vec3::Z, *food_velocity);
        food_transform.rotation = rotate_to_velocity;
    }
    let next_loc = food_transform.translation + *food_velocity;
    let mut boundary = Vec2::new(gamefield_size.x, gamefield_size.y);
    boundary.x /= 2.;
    boundary.y /= 2.;
    if 
        next_loc.x < -boundary.x + FOOD_RADIUS ||
        next_loc.x > boundary.x - FOOD_RADIUS  ||
        next_loc.y < -boundary.y + FOOD_RADIUS ||
        next_loc.y > boundary.y - FOOD_RADIUS
    {
        // food should "bump" into the wall
        *food_velocity = Vec3::ZERO;
    }
    food_transform.translation += *food_velocity * time.delta_seconds();
}
