#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    fmt::Write,
    fs::{self, File},
    io::Write as IoWrite,
};

use bevy::{
    app::AppExit,
    prelude::*,
    render::render_resource::{
        ShaderRef, 
        AsBindGroup,
    },
    sprite::{
        MaterialMesh2dBundle,
        Material2dPlugin,
        Material2d,
    },
    ecs::system::SystemId,
    window::{PresentMode, WindowMode},
};

use rand::{random, Rng};

mod constants;
pub use constants::{
    SNAKE_HEAD_RADIUS,
    FOOD_RADIUS,
};

mod systems;
use systems::{
    update_health,
    update_score_output,
    update_coins_output,
    spawn_snake,
    move_food,
};

// visual layers
const PLAYER_LAYER: f32 = 0.;
const FOOD_LAYER: f32 = -1.;

trait RandNormalized {
    fn random() -> Self;
}

impl RandNormalized for Vec3 {
    fn random() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            x: rng.gen(),
            y: rng.gen(),
            z: rng.gen(),
        }
    }
}

#[derive(Resource)]
pub struct DebugSettings {
    output_shown: bool,
}

#[derive(Resource)]
pub struct HighScore;

const HIGHSCORE_FILENAME: &str = "highscore.txt";

impl HighScore {
    fn get(&self) -> usize {
        let scorestr = fs::read_to_string(HIGHSCORE_FILENAME);
        match scorestr {
            Ok(scorestr) => match scorestr.parse() {
                Ok(score) => score,
                Err(_) => 0,
            },
            Err(_) => 0,
        }
    }
    fn save(&self, score: usize) {
        // todo: should this really crash the game, or should it just not save the high score if it
        // can't open the file?
        let mut scorefile =
            File::create(HIGHSCORE_FILENAME).expect("Could not open high score file!");
        let _ = scorefile.write_all(format!("{score}").as_bytes());
    }
}

const BUTTON_FONT_SIZE: f32 = 30.;

fn get_button() -> ButtonBundle {
    ButtonBundle {
        style: Style {
            padding: UiRect::all(Val::Px(5.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        border_color: BorderColor(Color::WHITE),
        background_color: BackgroundColor(Color::NONE),
        ..default()
    }
}

#[derive(Resource)]
pub struct GameFieldSize {
    x: f32,
    y: f32,
}

impl From<GameFieldSize> for Vec2 {
    fn from(gamefieldsize: GameFieldSize) -> Vec2 {
        Vec2 {
            x: gamefieldsize.x,
            y: gamefieldsize.y,
        }
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::AutoVsync,
                    // Tells wasm to resize the window according to the available canvas
                    fit_canvas_to_parent: true,
                    // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                    prevent_default_event_handling: false,
                    ..default()
                }),
                ..default()
            }),
            Material2dPlugin::<HealthbarMaterial>::default(),
            Material2dPlugin::<GameFieldMaterial>::default(),
        ))
        .insert_resource(Game {
            game_over: false,
            score: 0,
            coins: 0.,
        })
        .insert_resource(CameraSettings {
            follow_snake: false,
        })
        .insert_resource(GameOverMenuSelectedButton::Restart)
        .insert_resource(PauseMenuSelectedButton::Quit)
        .insert_resource(PauseState(false)) // game starts unpaused
        .insert_resource(HighScore)
        .insert_resource(DebugSettings {
            output_shown: false,
        })
        .insert_resource(GameFieldSize {
            x: 1920.,
            y: 1080.,
        })
        .add_event::<GameOverEvent>()
        .add_event::<ButtonHighlightedEvent>()
        .add_event::<RestartEvent>()
        .add_event::<PauseGameEvent>()
        .add_event::<UpgradeMenuButtonClickedEvent>()
        .add_systems(
            Startup,
            (
                setup,
                spawn_game_field_quad.after(setup),
                spawn_score_output,
                spawn_coins_output,
                spawn_snake,
                spawn_debug_output,
                spawn_food,
                spawn_pause_menu,
                spawn_game_over_splash,
            ),
        )
        .add_systems(
            Update,
            (
                // can only restart the game if the game is over, atm, this may change in the
                // future so make sure to remove the run condition if it does.
                (
                    restart,
                    menu_navigation,
                    update_high_score,
                    game_over_menu_selected_button_update,
                    on_restart_clicked,
                ).run_if(game_is_over),
                (
                    update_score_output,
                    player_input,
                    update_coins_output,
                    update_health_material,
                    update_health,
                    move_food.run_if(any_with_component::<Food>()),
                ).run_if(not(game_is_over).and_then(not(game_is_paused))),
                collide_with_self.run_if(snake_is_big_enough),
                drag.run_if(not(game_is_paused)),
                update_debug_output.run_if(debug_output_shown),
                spawn_coins
                    .run_if(not(any_with_component::<CoinBag>()).and_then(random_chance(0.02))),
                coinbag_leak.run_if(any_with_component::<CoinBag>().and_then(not(game_is_paused))),
                spawn_food.run_if(any_component_removed::<Food>()),
                consume_items.run_if(any_with_component::<Food>().or_else(any_with_component::<CoinBag>())),
                move_tail.run_if(any_with_component::<SnakeTailNode>()),
                on_quit_clicked,
                show_game_over,
                pause_menu_event_handler,
            ),
        )
        .run();
}

#[allow(dead_code)]
struct Upgrade {
    price: f32,
    system: SystemId,
    icon: String, // icon path, to be loaded by the asset loader
}

#[derive(Event)]
pub struct UpgradePurchasedEvent {
    upgrade: Upgrade,
}

pub fn upgrade_purchased(
    mut commands: Commands,
    mut ev_upgrade_purchased: EventReader<UpgradePurchasedEvent>,
) {
    for ev in ev_upgrade_purchased.read() {
        commands.run_system(ev.upgrade.system);
    }
}

pub fn halve_snake_length(
    mut commands: Commands,
    snake_tail_nodes: Query<Entity, With<SnakeTailNode>>,
) {
    let tail_nodes_vec: Vec<_> = snake_tail_nodes.iter().collect();
    let tail_nodes_count = tail_nodes_vec.len();
    for (i, snake_tail_node) in snake_tail_nodes.iter().enumerate() {
        if i > (tail_nodes_count / 2) {
            commands.entity(snake_tail_node).despawn();
        }
    }
}

pub fn update_health_material(
    mut materials: ResMut<Assets<HealthbarMaterial>>,
    snake: Query<&Snake>,
    ) {
    let snake = snake.single();
    for (_, material) in materials.iter_mut() {
        material.health = snake.health / 100.0;
    }
}

pub fn update_high_score(game: Res<Game>, high_score: Res<HighScore>) {
    if game.score > high_score.get() {
        high_score.save(game.score);
    }
}

pub fn random_chance(chance: f32) -> impl FnMut() -> bool {
    move || -> bool { random::<f32>() < chance }
}

#[derive(Component)]
pub struct CoinBag {
    value: f32
}

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

#[derive(Component)]
pub struct PauseMenu;

pub fn debug_output_shown(debug_settings: Res<DebugSettings>) -> bool {
    debug_settings.output_shown
}

pub fn spawn_pause_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                visibility: Visibility::Hidden,
                ..default()
            },
            PauseMenu,
        ))
        .with_children(|parent| {
            // container
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Paused",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 72.0,
                            color: Color::WHITE,
                        },
                    ));
                    parent
                        .spawn(NodeBundle { 
                            style: Style {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            ..default() 
                        })
                        .with_children(|parent| {
                            parent
                                .spawn((get_button(), UpgradesButton, PauseMenu))
                                .with_children(|parent| {
                                    parent.spawn(TextBundle::from_section(
                                        "Upgrades",
                                        TextStyle {
                                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                            font_size: BUTTON_FONT_SIZE,
                                            color: Color::WHITE,
                                        },
                                    ));
                                });
                            parent
                                .spawn((get_button(), QuitButton, PauseMenu))
                                .with_children(|parent| {
                                    parent.spawn(TextBundle::from_section(
                                        "Quit",
                                        TextStyle {
                                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                            font_size: BUTTON_FONT_SIZE,
                                            color: Color::WHITE,
                                        },
                                    ));
                                });
                        });
                });
        });
}

#[derive(Resource)]
pub struct PauseState(bool);

pub fn game_is_paused(paused: Res<PauseState>) -> bool {
    paused.0
}

#[derive(Event, Default)]
pub struct PauseGameEvent;

#[derive(Event, Default)]
pub struct UpgradeMenuButtonClickedEvent;

pub fn pause_menu_event_handler(
    mut pause_state: ResMut<PauseState>,
    mut keys: ResMut<Input<KeyCode>>,
    gamepads: Res<Gamepads>,
    mut buttons: ResMut<Input<GamepadButton>>,
    mut menu_visibility: Query<&mut Visibility, (With<PauseMenu>, Without<QuitButton>, Without<UpgradesButton>)>,
    mut ev_paused: EventReader<PauseGameEvent>,
    mut selected_button: ResMut<PauseMenuSelectedButton>,
    mut ev_quit: EventWriter<AppExit>,
    mut ev_upgrades_menu: EventWriter<UpgradeMenuButtonClickedEvent>,
    mut quit_button_style: Query<&mut Style, (With<PauseMenu>, With<QuitButton>, Without<UpgradesButton>)>,
    mut upgrades_button_style: Query<&mut Style, (With<PauseMenu>, With<UpgradesButton>, Without<QuitButton>)>,
) {
    let mut quit_button_style = quit_button_style.single_mut();
    let mut upgrades_button_style = upgrades_button_style.single_mut();
    match *selected_button {
        PauseMenuSelectedButton::Upgrades => {
            upgrades_button_style.border.bottom = Val::Px(2.);
            upgrades_button_style.margin.bottom = Val::Px(-2.);
            quit_button_style.border.bottom = Val::ZERO;
            quit_button_style.margin.bottom = Val::ZERO;
        }
        PauseMenuSelectedButton::Quit => {
            quit_button_style.border.bottom = Val::Px(2.);
            quit_button_style.margin.bottom = Val::Px(-2.);
            upgrades_button_style.border.bottom = Val::ZERO;
            upgrades_button_style.margin.bottom = Val::ZERO;
        }
        PauseMenuSelectedButton::None => {
            quit_button_style.border.bottom = Val::ZERO;
            quit_button_style.margin.bottom = Val::ZERO;
            upgrades_button_style.border.bottom = Val::ZERO;
            upgrades_button_style.margin.bottom = Val::ZERO;
        }
    }
    let mut menu_visibility = menu_visibility.single_mut();
    if !ev_paused.is_empty() {
        ev_paused.clear();
        *menu_visibility = Visibility::Visible;
        *pause_state = PauseState(true);
    }
    let PauseState(paused) = *pause_state;
    if paused {
        if keys.clear_just_pressed(KeyCode::P) || keys.clear_just_pressed(KeyCode::Escape) {
            *menu_visibility = Visibility::Hidden;
            *pause_state = PauseState(false);
        }
        if keys.clear_just_pressed(KeyCode::Up) {
            *selected_button = match *selected_button {
                PauseMenuSelectedButton::Upgrades => PauseMenuSelectedButton::None,
                PauseMenuSelectedButton::Quit => PauseMenuSelectedButton::Upgrades,
                PauseMenuSelectedButton::None => PauseMenuSelectedButton::Quit,
            };
        }
        if keys.clear_just_pressed(KeyCode::Down) {
            *selected_button = match *selected_button {
                PauseMenuSelectedButton::Upgrades => PauseMenuSelectedButton::Quit,
                PauseMenuSelectedButton::Quit => PauseMenuSelectedButton::None,
                PauseMenuSelectedButton::None => PauseMenuSelectedButton::Upgrades,
            };
        }
        if keys.clear_just_pressed(KeyCode::Return) {
            match *selected_button {
                PauseMenuSelectedButton::Upgrades => {
                    ev_upgrades_menu.send_default();
                }
                PauseMenuSelectedButton::Quit => {
                    ev_quit.send_default();
                }
                PauseMenuSelectedButton::None => {}
            }
        }
        let gamepad = gamepads.iter().next();
        if let Some(gamepad) = gamepad {
            let start_button = GamepadButton {
                gamepad,
                button_type: GamepadButtonType::Start,
            };
            let left_dpad = GamepadButton {
                gamepad,
                button_type: GamepadButtonType::DPadUp,
            };
            let right_dpad = GamepadButton {
                gamepad,
                button_type: GamepadButtonType::DPadDown,
            };
            let a_button = GamepadButton {
                gamepad,
                button_type: GamepadButtonType::South,
            };
            if buttons.clear_just_pressed(start_button) {
                *menu_visibility = Visibility::Hidden;
                *pause_state = PauseState(false);
            }
            if buttons.clear_just_pressed(left_dpad)  {
                *selected_button = match *selected_button {
                    PauseMenuSelectedButton::Upgrades => PauseMenuSelectedButton::None,
                    PauseMenuSelectedButton::Quit => PauseMenuSelectedButton::Upgrades,
                    PauseMenuSelectedButton::None => PauseMenuSelectedButton::Quit,
                };
            }

            if buttons.clear_just_pressed(right_dpad) {
                *selected_button = match *selected_button {
                    PauseMenuSelectedButton::Upgrades => PauseMenuSelectedButton::Quit,
                    PauseMenuSelectedButton::Quit => PauseMenuSelectedButton::None,
                    PauseMenuSelectedButton::None => PauseMenuSelectedButton::Upgrades,
                };
            }
            if buttons.clear_just_pressed(a_button) {
                match *selected_button {
                    PauseMenuSelectedButton::Upgrades => {
                        todo!("Upgrades menu not implemented");
                    }
                    PauseMenuSelectedButton::Quit => {
                        ev_quit.send_default();
                    }
                    PauseMenuSelectedButton::None => {}
                }
            }
        }
    }
}

pub fn menu_navigation(
    gamepads: Res<Gamepads>,
    selected_button: Res<GameOverMenuSelectedButton>,
    mut keys: ResMut<Input<KeyCode>>,
    mut buttons: ResMut<Input<GamepadButton>>,
    mut ev_button_highlighted: EventWriter<ButtonHighlightedEvent>,
    mut ev_restart: EventWriter<RestartEvent>,
    mut ev_quit: EventWriter<AppExit>,
) {
    let gamepad = gamepads.iter().next();
    let next_button = match *selected_button {
        GameOverMenuSelectedButton::None => GameOverMenuSelectedButton::Quit,
        GameOverMenuSelectedButton::Quit => GameOverMenuSelectedButton::Restart,
        GameOverMenuSelectedButton::Restart => GameOverMenuSelectedButton::Quit,
    };
    let prev_button = match *selected_button {
        GameOverMenuSelectedButton::None => GameOverMenuSelectedButton::Restart,
        GameOverMenuSelectedButton::Quit => GameOverMenuSelectedButton::Restart,
        GameOverMenuSelectedButton::Restart => GameOverMenuSelectedButton::Quit,
    };
    if keys.clear_just_pressed(KeyCode::Right) {
        ev_button_highlighted.send(ButtonHighlightedEvent(next_button));
    }
    if keys.clear_just_pressed(KeyCode::Left) {
        ev_button_highlighted.send(ButtonHighlightedEvent(prev_button));
    }
    if keys.clear_just_pressed(KeyCode::Return) {
        match *selected_button {
            GameOverMenuSelectedButton::Quit => {
                ev_quit.send_default();
            }
            GameOverMenuSelectedButton::Restart => {
                ev_restart.send_default();
            }
            GameOverMenuSelectedButton::None => {
                // no-op
            }
        }
    }
    if let Some(gamepad) = gamepad {
        let left_button = GamepadButton {
            gamepad,
            button_type: GamepadButtonType::DPadLeft,
        };
        let right_button = GamepadButton {
            gamepad,
            button_type: GamepadButtonType::DPadRight,
        };
        let a_button = GamepadButton {
            gamepad,
            button_type: GamepadButtonType::South,
        };
        if buttons.clear_just_pressed(right_button) {
            ev_button_highlighted.send(ButtonHighlightedEvent(next_button));
        }
        if buttons.clear_just_pressed(left_button) {
            ev_button_highlighted.send(ButtonHighlightedEvent(prev_button));
        }
        if buttons.clear_just_pressed(a_button) {
            match *selected_button {
                GameOverMenuSelectedButton::Quit => {
                    ev_quit.send_default();
                }
                GameOverMenuSelectedButton::Restart => {
                    ev_restart.send_default();
                }
                GameOverMenuSelectedButton::None => {
                    // no-op
                }
            }
        }
    }
}

#[derive(Event)]
pub struct ButtonHighlightedEvent(GameOverMenuSelectedButton);

pub fn game_over_menu_selected_button_update(
    mut restart_button: Query<&mut Style, (With<RestartButton>, Without<QuitButton>)>,
    mut quit_button: Query<
        &mut Style,
        (With<QuitButton>, Without<RestartButton>, Without<PauseMenu>),
    >,
    mut highlighted_button: ResMut<GameOverMenuSelectedButton>,
    mut ev_button_highlighted: EventReader<ButtonHighlightedEvent>,
) {
    let mut restart_button = restart_button.single_mut();
    let mut quit_button = quit_button.single_mut();
    for selected_button in ev_button_highlighted.read() {
        match selected_button {
            ButtonHighlightedEvent(GameOverMenuSelectedButton::None) => {
                restart_button.border = UiRect::default();
                restart_button.margin.bottom = Val::Px(0.);
                quit_button.border = UiRect::default();
                quit_button.margin.bottom = Val::Px(0.);
            }
            ButtonHighlightedEvent(GameOverMenuSelectedButton::Restart) => {
                restart_button.border = UiRect::bottom(Val::Px(2.));
                restart_button.margin.bottom = Val::Px(-2.);
                quit_button.border = UiRect::default();
                quit_button.margin.bottom = Val::Px(0.);
            }
            ButtonHighlightedEvent(GameOverMenuSelectedButton::Quit) => {
                restart_button.border = UiRect::default();
                restart_button.margin.bottom = Val::Px(0.);
                quit_button.border = UiRect::bottom(Val::Px(2.0));
                quit_button.margin.bottom = Val::Px(-2.);
            }
        }
        *highlighted_button = selected_button.0.clone();
    }
}

pub fn snake_is_big_enough(tail_nodes: Query<(), With<SnakeTailNode>>) -> bool {
    tail_nodes.iter().collect::<Vec<_>>().len() >= 4
}

pub fn collide_with_self(
    snake: Query<&Transform, (With<Snake>, Without<SnakeTailNode>)>,
    tail_nodes: Query<(&Transform, &SnakeTailNode), With<SnakeTailNode>>,
    mut ev_game_over: EventWriter<GameOverEvent>,
) {
    let snake = snake.single();
    for (tail_node, SnakeTailNode(ignore_collision)) in &tail_nodes {
        if *ignore_collision {
            continue;
        }
        if tail_node.translation.distance(snake.translation) < SNAKE_HEAD_RADIUS * 2. {
            ev_game_over.send_default();
        }
    }
}

pub fn show_game_over(
    mut game_over_visibility: Query<&mut Visibility, With<GameOver>>,
    mut ev_game_over: EventReader<GameOverEvent>,
    mut game: ResMut<Game>,
) {
    if !ev_game_over.is_empty() {
        ev_game_over.clear();
        game.game_over = true;
        let mut game_over_visibility = game_over_visibility.single_mut();
        *game_over_visibility = Visibility::Visible;
    }
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

#[derive(Component)]
pub struct UpgradesButton;

pub fn restart(
    mut ev_restart: EventReader<RestartEvent>,
    mut commands: Commands,
    mut game: ResMut<Game>,
    snake_tail: Query<Entity, With<SnakeTailNode>>,
    mut game_over_visibility: Query<&mut Visibility, With<GameOver>>,
    mut snake_head: Query<(&mut Transform, &mut Velocity), With<Snake>>,
    mut snake: Query<&mut Snake>,
    food_entity: Query<Entity, With<Food>>,
    coinbag_entity: Query<Entity, With<CoinBag>>,
    window: Query<&Window>,
) {
    if !ev_restart.is_empty() {
        ev_restart.clear();
        info!("Restarting game");
        game.restart();
        for tail_node in &snake_tail {
            commands.entity(tail_node).despawn();
        }
        let mut game_over_visibility = game_over_visibility.single_mut();
        *game_over_visibility = Visibility::Hidden;
        let window = window.single();
        let mut snake_head_location = Vec3::random();
        let boundary_x = (window.resolution.width() / 2.) - SNAKE_HEAD_RADIUS;
        let boundary_y = (window.resolution.height() / 2.) - SNAKE_HEAD_RADIUS;

        snake_head_location.x -= 0.5;
        snake_head_location.y -= 0.5;

        snake_head_location.x *= boundary_x * 2.;
        snake_head_location.y *= boundary_y * 2.;

        snake_head_location.z = PLAYER_LAYER;
        let snake_head = snake_head.single_mut();
        let (mut snake_head, mut snake_head_velocity) = snake_head;
        snake_head.translation = snake_head_location;
        *snake_head_velocity = Velocity(Vec3::ZERO);

        let mut snake = snake.single_mut();
        snake.health = 100.;

        if !coinbag_entity.is_empty() {
            let coinbag_entity = coinbag_entity.single();
            commands.entity(coinbag_entity).despawn();
        }

        if !food_entity.is_empty() {
            let food_entity = food_entity.single();
            commands.entity(food_entity).despawn();
        }
    }
}

#[derive(Event, Default)]
pub struct RestartEvent;

pub fn on_restart_clicked(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<RestartButton>)>,
    mut ev_select_button: EventWriter<ButtonHighlightedEvent>,
    mut ev_restart: EventWriter<RestartEvent>,
) {
    for interaction in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                ev_restart.send_default();
            }
            Interaction::Hovered => {
                ev_select_button.send(ButtonHighlightedEvent(GameOverMenuSelectedButton::Restart));
            }
            Interaction::None => {
                ev_select_button.send(ButtonHighlightedEvent(GameOverMenuSelectedButton::None));
            }
        }
    }
}
pub fn on_quit_clicked(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<QuitButton>)>,
    mut app_exit_events: EventWriter<AppExit>,
    mut ev_select_button: EventWriter<ButtonHighlightedEvent>,
) {
    for interaction in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                app_exit_events.send_default();
            }
            Interaction::Hovered => {
                ev_select_button.send(ButtonHighlightedEvent(GameOverMenuSelectedButton::Quit));
            }
            Interaction::None => {
                ev_select_button.send(ButtonHighlightedEvent(GameOverMenuSelectedButton::None));
            }
        }
    }
}

#[derive(Resource, Clone, Copy)]
pub enum GameOverMenuSelectedButton {
    Restart,
    Quit,
    None,
}

#[derive(Resource, Clone)]
pub enum PauseMenuSelectedButton {
    Upgrades,
    Quit,
    None,
}

pub fn spawn_game_over_splash(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                visibility: Visibility::Hidden,
                background_color: Color::rgba(0., 0., 0., 0.5).into(),
                ..default()
            },
            GameOver,
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.)),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        TextBundle::from_section(
                            "Game Over",
                            TextStyle {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 72.0,
                                ..default()
                            },
                        )
                        .with_style(Style { ..default() }),
                        Label, // a11y tag
                    ));
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                justify_content: JustifyContent::SpaceBetween,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|parent| {
                            parent
                                .spawn((get_button(), RestartButton))
                                .with_children(|parent| {
                                    parent.spawn(TextBundle::from_section(
                                        "Restart",
                                        TextStyle {
                                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                            font_size: BUTTON_FONT_SIZE,
                                            color: Color::rgb(0.9, 0.9, 0.9),
                                        },
                                    ));
                                });
                            parent
                                .spawn((get_button(), QuitButton))
                                .with_children(|parent| {
                                    parent.spawn(TextBundle::from_section(
                                        "Quit",
                                        TextStyle {
                                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                            font_size: BUTTON_FONT_SIZE,
                                            color: Color::rgb(0.9, 0.9, 0.9),
                                        },
                                    ));
                                });
                        });
                });
        });
    // 3. handle "restart" and "quit" button presses
}

const TAIL_NODE_GAP: f32 = 50.;
const TAIL_CATCHUP_SPEED: f32 = 7.;

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

pub const FOOD_HEALTH: f32 = 30.;

pub fn consume_items(
    mut commands: Commands,
    food: Query<(&Transform, Entity), With<Food>>,
    coins: Query<(&Transform, Entity, &CoinBag), With<CoinBag>>,
    head: Query<&Transform, With<Snake>>,
    tail_nodes: Query<(&Transform, Entity), With<SnakeTailNode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut game: ResMut<Game>,
    mut snake: Query<&mut Snake>,
) {
    let head = head.single();
    let mut snake = snake.single_mut();
    // consume_food
    if !food.is_empty() {
        let food = food.single();
        let (food, food_entity) = food;
        if food.translation.distance(head.translation) < (SNAKE_HEAD_RADIUS + FOOD_RADIUS) {
            // food consumed
            commands.entity(food_entity).despawn();
            game.score += 1;
            snake.health += FOOD_HEALTH;
            snake.health = snake.health.clamp(0., 100.);

            let tail_nodes_vec: Vec<_> = tail_nodes.iter().collect();
            let tail_nodes_count = tail_nodes_vec.len();
            let last_tail_node = tail_nodes_vec.last();
            let offset_origin = match last_tail_node {
                Some((last_tail_node, _)) => last_tail_node.translation,
                None => head.translation,
            };
            let angle_to_offset = if tail_nodes_count >= 2 {
                let penultimate_tail_node = tail_nodes_vec.get(tail_nodes_count - 2).unwrap();
                offset_origin.angle_between(penultimate_tail_node.0.translation)
            } else {
                random()
            };
            let offset_vector = Vec3::new(
                TAIL_NODE_GAP * angle_to_offset.cos(),
                TAIL_NODE_GAP * angle_to_offset.sin(),
                0.0,
            );
            let ignore_collision = tail_nodes_count < 1;
            commands.spawn((
                SnakeTailNode(ignore_collision),
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
    // consume_coins
    if !coins.is_empty() {
        let (coins_transform, coins_entity, coins) = coins.single();
        if coins_transform.translation.distance(head.translation) < (SNAKE_HEAD_RADIUS + FOOD_RADIUS) {
            commands.entity(coins_entity).despawn();
            game.coins += coins.value;
            // game.coins += (rand::thread_rng().gen_range(8.0..12.0f32) * 100.0).round() / 100.0;
        }
    }
}

#[derive(Component)]
pub struct TailNodeCount(usize);

#[derive(Component)]
pub struct DebugOutput;

pub fn update_debug_output(
    mut texts: Query<&mut Text, With<DebugOutput>>,
    snakes: Query<(&Transform, &Velocity), With<Snake>>,
    food: Query<(&Transform, &Velocity), With<Food>>,
    tail_nodes: Query<(), With<SnakeTailNode>>,
    game: Res<Game>,
) {
    let mut text = texts.single_mut();
    let snake = snakes.single();
    let tail_node_count = tail_nodes.iter().count();
    let (snake_transform, snake_velocity) = snake;
    if !food.is_empty() {
        let (food_location, food_velocity) = food.single();
        let food_location = food_location.translation;
        let Velocity(food_velocity) = *food_velocity;
        let mut s = String::new();
        let _ = write!(s, "Food location: {food_location}\n");
        let _ = write!(s, "Food velocity: {food_velocity}\n");
        text.sections[1].value = s;
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
    let mut text_bundle = TextBundle::from_sections([
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
        top: Val::Px(75.0),
        left: Val::Px(15.0),
        ..default()
    });
    text_bundle.visibility = Visibility::Hidden;
    commands.spawn((text_bundle, DebugOutput));
}

const COIN_LEAK_PER_FRAME: f32 = 0.01;

pub fn coinbag_leak(mut commands: Commands, mut coins: Query<(&mut CoinBag, Entity)>) {
    for (mut coins, coins_entity) in &mut coins {
        coins.value -= COIN_LEAK_PER_FRAME;
        if coins.value <= 0. {
            commands.entity(coins_entity).despawn();
        }
    }
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
pub struct CoinsOutput;

#[derive(Component)]
pub struct CoinbagValueOutput;

pub fn spawn_coins_output(mut commands: Commands, game: Res<Game>, asset_server: Res<AssetServer>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                margin: UiRect::all(Val::Px(15.)),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                right: Val::Px(15.),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    format!("Coins: {0}", game.coins),
                    TextStyle {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: 32.0,
                        ..default()
                    },
                ),
                CoinsOutput,
            ));
            parent.spawn((
                TextBundle::from_section(
                    format!("Bag Value"),
                    TextStyle {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: 32.0,
                        ..default()
                    },
                ), 
                CoinbagValueOutput,
            ));
        });
}

#[derive(Asset, AsBindGroup, TypePath, Clone, Default)]
pub struct GameFieldMaterial {
    #[uniform(0)]
    width: f32,
    #[uniform(1)]
    height: f32,
}

impl Material2d for GameFieldMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/gamefield.wgsl".into()
    }
}

pub fn spawn_game_field_quad(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<GameFieldMaterial>>,
    gamefield_size: Res<GameFieldSize>,
    ) {

    let gamefield_size = Vec2 {
        x: gamefield_size.x,
        y: gamefield_size.y,
    };

    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(shape::Quad::new(gamefield_size).into()).into(),
        material: materials.add(GameFieldMaterial {
            width: gamefield_size.x,
            height: gamefield_size.y,
        }),
        transform: Transform::from_translation(Vec3::new(0., 0., -1.)),
        ..default()
    });
}

#[derive(Component)]
pub struct ScoreOutput;

pub fn spawn_score_output(mut commands: Commands, game: Res<Game>, asset_server: Res<AssetServer>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                margin: UiRect::all(Val::Px(15.)),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    format!("Score: {0}", game.score),
                    TextStyle {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: 32.0,
                        ..default()
                    },
                ),
                ScoreOutput,
            ));
        });
}


#[derive(Component)]
pub struct Snake {
    health: f32,
}

#[derive(Component)]
pub struct SnakeTailNode(bool);

#[derive(Asset, AsBindGroup, TypePath, Clone)]
pub struct HealthbarMaterial {
    #[uniform(0)]
    health: f32,
}

impl Material2d for HealthbarMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/healthbar.wgsl".into()
    }
}

#[derive(Component)]
pub struct Velocity(Vec3);

#[derive(Component)]
pub struct Food;

fn setup(mut commands: Commands, mut window: Query<&mut Window>) {
    commands.spawn(Camera2dBundle::default());
    let mut window = window.single_mut();
    // set the window to fullscreen on startup
    // reason we defer it until now is so that it will appear on the _current_ monitor
    // for players with a multi-monitor setup.  Without the delay, it appears on the primary
    // monitor instead which is maybe not always what the player wanted/expected.
    let _ = window.mode.set(Box::new(WindowMode::BorderlessFullscreen));
}

fn spawn_food(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    gamefield_size: Res<GameFieldSize>,
) {
    let mut food_location = Vec3::random();
    let boundary_x = (gamefield_size.x / 2.) - FOOD_RADIUS;
    let boundary_y = (gamefield_size.y / 2.) - FOOD_RADIUS;

    food_location.x -= 0.5;
    food_location.y -= 0.5;

    food_location.x *= boundary_x * 2.;
    food_location.y *= boundary_y * 2.;

    food_location.z = FOOD_LAYER;

    commands.spawn((
        Food,
        Velocity(Vec3::ZERO),
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
    coins: f32,
}

impl Game {
    pub fn new() -> Self {
        Self {
            game_over: false,
            score: 0,
            coins: 0.,
        }
    }
    pub fn restart(&mut self) {
        *self = Self::new()
    }
}

#[derive(Resource)]
pub struct CameraSettings {
    follow_snake: bool,
}

fn player_input(
    time: Res<Time>,
    mut snake: Query<(&mut Transform, &mut Velocity), With<Snake>>,
    mut keys: ResMut<Input<KeyCode>>,
    // mut window: Query<&mut Window>,
    gamepads: Res<Gamepads>,
    axes: Res<Axis<GamepadAxis>>,
    button_axes: Res<Axis<GamepadButton>>,
    mut buttons: ResMut<Input<GamepadButton>>,
    mut ev_gameover: EventWriter<GameOverEvent>,
    mut ev_pause: EventWriter<PauseGameEvent>,
    mut debug_settings: ResMut<DebugSettings>,
    mut debug_output_visibility: Query<&mut Visibility, With<DebugOutput>>,
    mut camera_projection: Query<(&mut OrthographicProjection, &mut Transform), (With<Camera2d>, Without<Snake>)>,
    gamefield_size: Res<GameFieldSize>,
    mut camera_settings: ResMut<CameraSettings>,
) {
    // let mut window = window.single_mut();
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
        let axis_rx = GamepadAxis {
            gamepad,
            axis_type: GamepadAxisType::RightStickX,
        };
        let axis_ry = GamepadAxis {
            gamepad,
            axis_type: GamepadAxisType::RightStickY,
        };
        let axis_rt = GamepadButton {
            gamepad,
            button_type: GamepadButtonType::RightTrigger2,
        };
        let axis_lt = GamepadButton {
            gamepad,
            button_type: GamepadButtonType::LeftTrigger2,
        };
        if let (Some(x), Some(y)) = (axes.get(axis_x), axes.get(axis_y)) {
            // combine X and Y into one vector
            let left_stick_pos = Vec2::new(x, y);
            let mut player_requested_velocity = Vec3::from((left_stick_pos, 0.));
            player_requested_velocity *= time.delta_seconds() * analog_accel_factor;
            *head_velocity = *head_velocity + player_requested_velocity;
        }
        let (mut camera_projection, mut camera_transform) = camera_projection.single_mut();
        if let (Some(x), Some(y)) = (axes.get(axis_rx), axes.get(axis_ry)) {
            let right_stick_pos = Vec3::new(x, y, 0.);
            camera_transform.translation += right_stick_pos * 100.;
        }
        if let (Some(rt), Some(lt)) = (button_axes.get(axis_rt), button_axes.get(axis_lt)) {
            let total_zoom = (lt - rt).clamp(-0.7, 1.0);
            camera_projection.scale = 1.0 + total_zoom;
        }
        if camera_settings.follow_snake || camera_projection.scale != 1.0 {
            let camera_origin = head_transform.translation;
            camera_transform.translation = camera_origin;
        } 
        let start_button = GamepadButton {
            gamepad,
            button_type: GamepadButtonType::Start,
        };
        let select_button = GamepadButton {
            gamepad,
            button_type: GamepadButtonType::Select,
        };
        if buttons.clear_just_pressed(select_button) {
            camera_settings.follow_snake = !camera_settings.follow_snake;
            if !camera_settings.follow_snake {
                camera_transform.translation = Vec3::ZERO;
            }
        }
        if buttons.clear_just_pressed(start_button) {
            ev_pause.send_default();
        }
    }
    if keys.clear_just_pressed(KeyCode::F3) {
        debug_settings.output_shown = !debug_settings.output_shown;
        let mut debug_output_visibility = debug_output_visibility.single_mut();
        *debug_output_visibility = match debug_settings.output_shown {
            true => Visibility::Visible,
            false => Visibility::Hidden,
        }
    }
    let mut head_velocity_delta = Vec3::ZERO;
    if keys.pressed(KeyCode::W) || keys.pressed(KeyCode::Up) {
        head_velocity_delta.y += 1.;
    }
    if keys.pressed(KeyCode::S) || keys.pressed(KeyCode::Down) {
        head_velocity_delta.y -= 1.;
    }
    if keys.pressed(KeyCode::D) || keys.pressed(KeyCode::Right) {
        head_velocity_delta.x += 1.;
    }
    if keys.pressed(KeyCode::A) || keys.pressed(KeyCode::Left) {
        head_velocity_delta.x -= 1.;
    }
    *head_velocity = *head_velocity + (head_velocity_delta.normalize_or_zero() * accel_factor);

    if keys.clear_just_pressed(KeyCode::P) || keys.clear_just_pressed(KeyCode::Escape) {
        ev_pause.send_default();
    }
    head_transform.translation += *head_velocity * time.delta_seconds();
    let mut boundary_x = gamefield_size.x / 2.;
    let mut boundary_y = gamefield_size.y / 2.;
    boundary_x -= SNAKE_HEAD_RADIUS;
    boundary_y -= SNAKE_HEAD_RADIUS;
    if head_transform.translation.x > boundary_x
        || head_transform.translation.y > boundary_y
        || head_transform.translation.x < -boundary_x
        || head_transform.translation.y < -boundary_y
    {
        // game over
        ev_gameover.send_default();
    }
    head_transform.translation.x = head_transform.translation.x.clamp(-boundary_x, boundary_x);
    head_transform.translation.y = head_transform.translation.y.clamp(-boundary_y, boundary_y);
}
