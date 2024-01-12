use std::fmt::Write;

use bevy::{
    prelude::*,
    app::AppExit,
    sprite::MaterialMesh2dBundle,
};

use rand::{random, Rng};

// visual layers
#[allow(dead_code)]
const PLAYER_LAYER: f32 = 0.;
const FOOD_LAYER: f32 = -1.;

fn random_normalized_vector() -> Vec3 {
    let mut rng = rand::thread_rng();
    Vec3 {
        x: rng.gen(),
        y: rng.gen(),
        z: rng.gen(),
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Game { game_over: false, score: 0 })
        .add_event::<GameOverEvent>()
        .add_systems(
            Startup,
            (
                setup,
                spawn_snake,
                spawn_debug_output,
                spawn_food,
            ),
        )
        .add_systems(
            Update,
            (
                game_over_splash.run_if(game_is_over),
                head_movement.run_if(not(game_is_over)),
                bevy::window::close_on_esc,
                drag,
                update_debug_output,
                spawn_food.run_if(any_component_removed::<Food>()),
                consume_food.run_if(any_with_component::<Food>()),
                move_tail.run_if(any_with_component::<SnakeTailNode>()),
                on_quit_clicked.run_if(game_is_over),
                on_restart_clicked.run_if(game_is_over),
            ),
        )
        .run();
}

pub fn game_is_over(game: Res<Game>) -> bool {
    game.game_over
}

// marker for "Game Over" splash screen and related components
#[derive(Component)]
pub struct GameOver;

#[derive(Event, Default)]
pub struct GameOverEvent;

#[derive(Component)]
pub struct RestartButton;

#[derive(Component)]
pub struct QuitButton;


pub fn on_restart_clicked(
    mut commands: Commands,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
        ),
        (Changed<Interaction>, With<RestartButton>),
    >,
    mut game: ResMut<Game>,
    snake_tail: Query<Entity, With<SnakeTailNode>>,
    game_over_screen: Query<Entity, With<GameOver>>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                info!("Restarting game");
                game.score = 0;
                game.game_over = false;
                for tail_node in &snake_tail {
                    commands.entity(tail_node).despawn();
                }
                let game_over_screen = game_over_screen.single();
                commands.entity(game_over_screen).despawn();
            }
            Interaction::Hovered => {
                color.0 = Color::RED;
            }
            Interaction::None => {
                color.0 = Color::BLUE;
            }
        }
    }
}
pub fn on_quit_clicked(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
        ),
        (Changed<Interaction>, With<QuitButton>),
    >,
    mut app_exit_events: EventWriter<AppExit>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                app_exit_events.send_default();
            }
            Interaction::Hovered => {
                color.0 = Color::RED;
            }
            Interaction::None => {
                color.0 = Color::BLUE;
            }
        }
    }
}

pub fn game_over_splash(
    mut commands: Commands,
    mut ev_gameover: EventReader<GameOverEvent>,
    asset_server: Res<AssetServer>,
) {
    if !ev_gameover.is_empty() {
        ev_gameover.clear();
        info!("Drawing Game Over splash screen");
        // 1. draw "Game Over" splash
        commands.spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::rgba(0.,0.,0.,0.5).into(),
                ..default()
            },
            GameOver
        )).with_children(|parent| {
            parent.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.)),
                    ..default()
                },
                border_color: Color::RED.into(),
                ..default()
            }).with_children(|parent| {
                parent.spawn((
                    TextBundle::from_section(
                        "Game Over",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 72.0,
                            ..default()
                        },
                    ).with_style(Style {
                        ..default()
                    }),
                    Label, // a11y tag
                ));
                parent.spawn(NodeBundle {
                    style: Style {
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                }).with_children(|parent| {
                    let button = 
                        ButtonBundle {
                            style: Style {
                                // width: Val::Px(150.0),
                                // height: Val::Px(65.0),
                                padding: UiRect::all(Val::Px(5.0)),
                                border: UiRect::all(Val::Px(5.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            border_color: BorderColor(Color::BLACK),
                            background_color: Color::rgb(0.2, 0.4, 0.3).into(),
                            ..default()
                        };
                    parent.spawn(
                        (button.clone(), RestartButton)
                    ).with_children(|parent| {
                        parent.spawn(TextBundle::from_section(
                                "Restart",
                                TextStyle {
                                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: 30.0,
                                    color: Color::rgb(0.9, 0.9, 0.9),
                                },
                        ));
                    });
                    parent.spawn(
                        (button.clone(), QuitButton)
                    ).with_children(|parent| {
                        parent.spawn(TextBundle::from_section(
                                "Quit",
                                TextStyle {
                                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: 30.0,
                                    color: Color::rgb(0.9, 0.9, 0.9),
                                },
                        ));
                    });
                });
            });
        });
        // 3. handle "restart" and "quit" button presses
    }
}

const TAIL_NODE_GAP: f32 = 60.;
const TAIL_CATCHUP_SPEED: f32 = 5.;

pub fn move_tail(
    time: Res<Time>,
    mut tail_nodes: Query<&mut Transform, With<SnakeTailNode>>,
    snake: Query<&Transform, (With<Snake>, Without<SnakeTailNode>)>,
) {
    let snake = snake.single();
    let mut target_point = snake.translation;
    for mut tail_node in &mut tail_nodes {
        if tail_node.translation.distance(target_point) >= TAIL_NODE_GAP {
            let bearing = target_point - tail_node.translation;
            tail_node.translation += bearing * time.delta_seconds() * TAIL_CATCHUP_SPEED;
        }

        target_point = tail_node.translation;
    }
}

pub fn consume_food(
    mut commands: Commands,
    food: Query<(&Transform, Entity), With<Food>>,
    head: Query<(&Transform, Entity), With<Snake>>,
    tail_nodes: Query<(&Transform, Entity), With<SnakeTailNode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut game: ResMut<Game>,
) {
    let food = food.single();
    let head = head.single();
    let (food, food_entity) = food;
    let (head, _) = head;
    if food.translation.distance(head.translation) < (SNAKE_HEAD_RADIUS + FOOD_RADIUS) {
        // food consumed
        commands.entity(food_entity).despawn();
        game.score += 1;
        let tail_nodes_vec: Vec<_> = tail_nodes.iter().collect();
        let tail_nodes_count = tail_nodes_vec.len();
        let last_tail_node = tail_nodes_vec.first();
        let offset_origin = match last_tail_node {
            Some((last_tail_node, _)) => last_tail_node.translation,
            None => head.translation,
        };
        let angle_to_offset = if tail_nodes_count >= 2 {
            let penultimate_tail_node = tail_nodes_vec.get(tail_nodes_count - 2);
            match penultimate_tail_node {
                Some((penultimate_tail_node, _)) => {
                    offset_origin.angle_between(penultimate_tail_node.translation)
                }
                None => random(),
            }
        } else {
            random()
        };
        let offset_vector = Vec3::new(
            TAIL_NODE_GAP * angle_to_offset.cos(),
            TAIL_NODE_GAP * angle_to_offset.sin(),
            0.0,
        );
        commands.spawn((
            SnakeTailNode,
            MaterialMesh2dBundle {
                mesh: meshes
                    .add(shape::Circle::new(SNAKE_HEAD_RADIUS).into())
                    .into(),
                material: materials.add(ColorMaterial::from(Color::GREEN)),
                transform: Transform::from_translation(offset_origin + offset_vector),
                ..default()
            },
            TailNodeCount(tail_nodes_count),
        ));
    }
}

#[derive(Component)]
pub struct TailNodeCount(usize);

#[derive(Component)]
pub struct DebugOutput;

pub fn update_debug_output(
    mut texts: Query<&mut Text, With<DebugOutput>>,
    snakes: Query<(&Transform, &Velocity), With<Snake>>,
    food_location: Query<&Transform, With<Food>>,
    tail_nodes: Query<(), With<SnakeTailNode>>,
    game: Res<Game>,
) {
    let mut text = texts.single_mut();
    let snake = snakes.single();
    let tail_node_count = tail_nodes.iter().count();
    let (snake_transform, snake_velocity) = snake;
    if !food_location.is_empty() {
        let food_location = food_location.single();
        let food_location = food_location.translation;
        text.sections[1].value = format!("Food location: {food_location}\n");
    }
    let Velocity(velocity) = *snake_velocity;
    let position = snake_transform.translation;
    let mut s = String::new();
    let _ = write!(s, "Snake head velocity: {velocity}\n");
    let _ = write!(s, "Snake head position: {position}\n");
    let _ = write!(s, "Snake tail sections: {tail_node_count}\n");
    let _ = write!(s, "Score: {0}\n", game.score);
    text.sections[0].value = s;
}

pub fn spawn_debug_output(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraMono-Medium.ttf");
    commands.spawn((
        TextBundle::from_sections([
            TextSection::from_style(TextStyle {
                font: font.clone(),
                font_size: 20.0,
                color: Color::FUCHSIA,
                ..default()
            }),
            TextSection::from_style(TextStyle {
                font: font.clone(),
                font_size: 20.0,
                color: Color::FUCHSIA,
                ..default()
            }),
        ])
        .with_style(Style {
            flex_direction: FlexDirection::Column,
            top: Val::Px(5.0),
            left: Val::Px(15.0),
            ..default()
        }),
        DebugOutput,
    ));
}

const DRAG_COEFFICIENT: f32 = 0.98;
const MAX_VELOCITY: f32 = 10.0;

pub fn drag(mut velocities: Query<&mut Velocity>) {
    for mut velocity in &mut velocities {
        let Velocity(ref mut velocity) = *velocity;
        *velocity *= DRAG_COEFFICIENT;
        if velocity.length() < 1.0 {
            *velocity = Vec3::ZERO;
        }
        velocity.clamp_length(0.0, MAX_VELOCITY);
    }
}

#[derive(Component)]
pub struct Snake;

#[derive(Component)]
pub struct SnakeTailNode;

const FOOD_RADIUS: f32 = 15.0;
const SNAKE_HEAD_RADIUS: f32 = 30.0;

pub fn spawn_snake(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Snake,
        MaterialMesh2dBundle {
            mesh: meshes
                .add(shape::Circle::new(SNAKE_HEAD_RADIUS).into())
                .into(),
            material: materials.add(ColorMaterial::from(Color::GREEN)),
            transform: Transform::from_translation(Vec3::new(-150., 0., 0.)),
            ..default()
        },
        Velocity(Vec3::ZERO),
    ));
}

#[derive(Component)]
pub struct Velocity(Vec3);

#[derive(Component)]
pub struct Food;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_food(
    window: Query<&Window>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut food_location: Vec3 = random_normalized_vector();
    let window = window.single();
    let boundary_x = (window.resolution.width() / 2.) - FOOD_RADIUS;
    let boundary_y = (window.resolution.height() / 2.) - FOOD_RADIUS;

    food_location.x -= 0.5;
    food_location.y -= 0.5;

    food_location.x *= boundary_x * 2.;
    food_location.y *= boundary_y * 2.;

    food_location.z = FOOD_LAYER;

    commands.spawn((
        Food,
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(FOOD_RADIUS).into()).into(),
            material: materials.add(ColorMaterial::from(Color::RED)),
            transform: Transform::from_translation(food_location),
            ..default()
        },
    ));
}

#[derive(Resource)]
pub struct Game {
    game_over: bool,
    score: usize,
}

impl Game {
    pub fn new() -> Self {
        Self {
            game_over: false,
            score: 0,
        }
    }
}

fn head_movement(
    time: Res<Time>,
    mut snake: Query<(&mut Transform, &mut Velocity), With<Snake>>,
    keys: Res<Input<KeyCode>>,
    window: Query<&Window>,
    gamepads: Res<Gamepads>,
    axes: Res<Axis<GamepadAxis>>,
    mut game: ResMut<Game>,
    mut ev_gameover: EventWriter<GameOverEvent>,
) {
    let gamepad = gamepads.iter().next();
    let (mut head_transform, mut head_velocity) = snake.single_mut();
    let Velocity(ref mut head_velocity) = *head_velocity;
    let accel_factor = 10.;
    let analog_accel_factor = 500.;
    if let Some(gamepad) = gamepad {
        let axis_x = GamepadAxis {
            gamepad,
            axis_type: GamepadAxisType::LeftStickX,
        };
        let axis_y = GamepadAxis {
            gamepad,
            axis_type: GamepadAxisType::LeftStickY,
        };
        if let (Some(x), Some(y)) = (axes.get(axis_x), axes.get(axis_y)) {
            // combine X and Y into one vector
            let left_stick_pos = Vec2::new(x, y);
            let mut player_requested_velocity = Vec3::from((left_stick_pos, 0.));
            player_requested_velocity *= time.delta_seconds() * analog_accel_factor;
            *head_velocity = *head_velocity + player_requested_velocity;
        }
    }
    if keys.pressed(KeyCode::W) {
        head_velocity.y += accel_factor;
    }
    if keys.pressed(KeyCode::S) {
        head_velocity.y -= accel_factor;
    }
    if keys.pressed(KeyCode::D) {
        head_velocity.x += accel_factor;
    }
    if keys.pressed(KeyCode::A) {
        head_velocity.x -= accel_factor;
    }
    head_transform.translation += *head_velocity * time.delta_seconds();
    let window = window.single();
    let mut boundary_x = window.resolution.width() / 2.;
    let mut boundary_y = window.resolution.height() / 2.;
    boundary_x -= SNAKE_HEAD_RADIUS;
    boundary_y -= SNAKE_HEAD_RADIUS;
    if head_transform.translation.x > boundary_x
        || head_transform.translation.y > boundary_y
        || head_transform.translation.x < -boundary_x
        || head_transform.translation.y < -boundary_y
    {
        // game over
        game.game_over = true;
        ev_gameover.send_default();
    }
    head_transform.translation.x = head_transform.translation.x.clamp(-boundary_x, boundary_x);
    head_transform.translation.y = head_transform.translation.y.clamp(-boundary_y, boundary_y);
}
