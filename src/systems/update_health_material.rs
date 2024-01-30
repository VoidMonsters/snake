use bevy::prelude::*;

use crate::{
    HealthbarMaterial,
    Snake,
};

pub fn update_health_material(
    mut materials: ResMut<Assets<HealthbarMaterial>>,
    snake: Query<&Snake>,
) {
    let snake = snake.single();
    for (_, material) in materials.iter_mut() {
        material.health = snake.health / 100.0;
    }
}
