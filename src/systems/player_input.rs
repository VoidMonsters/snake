use bevy::prelude::*;

use crate::{
    Velocity,
    Snake,
    GameOverEvent,
    PauseGameEvent,
    DebugSettings,
    DebugOutput,
    GameFieldSize,
    CameraSettings,
    SNAKE_HEAD_RADIUS,
};

pub fn player_input(
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
