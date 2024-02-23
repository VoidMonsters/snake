use bevy::{
    prelude::*,
    sprite::MaterialMesh2dBundle,
};

use crate::{
    HealthbarMaterial,
    Snake,
    Velocity,
    SNAKE_HEAD_RADIUS,
};

pub fn spawn_snake(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    mut healthbar_materials: ResMut<Assets<HealthbarMaterial>>,
) {
    commands.spawn((
        Snake {
            health: 100.
        },
        MaterialMesh2dBundle {
            mesh: meshes
                .add(shape::Circle::new(SNAKE_HEAD_RADIUS).into())
                .into(),
            material: color_materials.add(ColorMaterial::from(Color::GREEN)),
            transform: Transform::from_translation(Vec3::new(-0., 0., 0.)),
            ..default()
        },
        Velocity(Vec3::ZERO),
    )).with_children(|parent| {
        // snake health bar
        parent.spawn(MaterialMesh2dBundle {
            mesh: meshes.add(shape::Quad::new(Vec2::new(SNAKE_HEAD_RADIUS * 2.0, 10.)).into()).into(),
            // material: color_materials.add(ColorMaterial::from(Color::BLACK)),
            material: healthbar_materials.add(HealthbarMaterial {
                health: 1.
            }),
            transform: Transform {
                translation: Vec3::new(0., SNAKE_HEAD_RADIUS + 20., 1.),
                ..default()
            },
            ..default()
        });
    });
}
