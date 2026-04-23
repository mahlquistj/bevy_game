use bevy::prelude::*;

use crate::{AppSystems, PausableSystems, WorldCamera, demo::player::Player};

const CONTROLLER_STICK_DEADZONE: f32 = 0.1; // 10%

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        control_player
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
    app.add_systems(
        Update,
        record_input
            .in_set(AppSystems::RecordInput)
            .in_set(PausableSystems),
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
pub(super) enum PlayerState {
    Idle,
    #[default]
    Walking,
    Running,
}

#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub(super) struct PlayerController {
    pub state: PlayerState,
    pub intent: Vec2,
    pub facing: Vec2,
    pub max_walking_speed: f32,
    pub max_running_speed: f32,
}

impl Default for PlayerController {
    fn default() -> Self {
        Self {
            state: PlayerState::Idle,
            intent: Vec2::ZERO,
            facing: Vec2::X,
            max_walking_speed: 400.0,
            max_running_speed: 600.0,
        }
    }
}

fn record_input(
    camera: Single<(&Camera, &GlobalTransform), With<WorldCamera>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut mouse_events: MessageReader<CursorMoved>,
    mut player: Single<(&GlobalTransform, &mut PlayerController), With<Player>>,
    #[allow(unused)]
    #[cfg(feature = "dev")]
    mut gizmos: Gizmos,
) {
    let (player_transform, mut controller) = player.into_inner();
    let mut intent = Vec2::ZERO;
    let mut facing = controller.facing;
    let mut new_state = PlayerState::default();

    // Start by checking gamepad(s)
    for gamepad in &gamepads {
        if gamepad.pressed(GamepadButton::RightTrigger) {
            new_state = PlayerState::Running;
        }

        let left_stick = gamepad.left_stick();
        if stick_activated(left_stick) {
            intent.x = left_stick.x;
            intent.y = left_stick.y;
        }

        let right_stick = gamepad.right_stick();
        if stick_activated(right_stick) {
            facing.x = right_stick.x;
            facing.y = right_stick.y;
        }
    }

    // Then we check keyboards - We always overwrite gamepad values, if the keyboard is used!
    if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight) {
        new_state = PlayerState::Running;
    }

    let mut keyboard_intent = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        keyboard_intent.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        keyboard_intent.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        keyboard_intent.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        keyboard_intent.x += 1.0;
    }

    // Keyboard movement overwrites controller, if used
    if keyboard_intent != Vec2::ZERO {
        // Also, normalize keyboard movement, so we don't double the speed when moving
        // diagonally
        intent = keyboard_intent.normalize_or_zero();
    }

    if intent == Vec2::ZERO {
        new_state = PlayerState::Idle;
    }

    if let Some(cursor_event) = mouse_events.read().last() {
        let (camera, camera_transform) = camera.into_inner();
        if let Ok(cursor_world) =
            camera.viewport_to_world_2d(camera_transform, cursor_event.position)
        {
            let player_world = player_transform.translation().truncate();
            let mouse_direction = cursor_world - player_world;
            mouse_events.clear();

            if mouse_direction.length_squared() > 0.0 {
                facing = mouse_direction.normalize();
            }
        };
    }

    // Apply new intents to controller
    controller.intent = intent;
    controller.state = new_state;
    controller.facing = facing;

    // TODO: Draw gizmos for movement?
}

fn control_player(
    time: Res<Time>,
    movement_query: Single<(&PlayerController, &mut Transform), With<Player>>,
    #[cfg(feature = "dev")] mut gizmos: Gizmos,
) {
    let (controller, mut transform) = movement_query.into_inner();

    // Calculations
    let intent = match controller.state {
        PlayerState::Idle => Vec2::ZERO,
        PlayerState::Walking => controller.max_walking_speed * controller.intent,
        PlayerState::Running => controller.max_running_speed * controller.intent,
    };
    let velocity = intent.extend(0.0) * time.delta_secs();

    // Assignments
    transform.translation += velocity;
    if controller.facing.length_squared() > 0.0 {
        let angle = Vec2::X.angle_to(controller.facing);
        transform.rotation = Quat::from_rotation_z(angle);
    }

    #[cfg(feature = "dev")]
    {
        use bevy::color::palettes::css;

        // Velocity arrow
        gizmos.arrow_2d(
            transform.translation.truncate(),
            transform.translation.truncate() + (controller.intent * Vec2::splat(100.0)),
            css::BLUE,
        );

        // Facing arrow
        gizmos.arrow_2d(
            transform.translation.truncate(),
            transform.translation.truncate() + (controller.facing * Vec2::splat(100.0)),
            css::GREEN,
        );
    }
}

// HELPERS

/// Checks if the value is outside the deadzone of the controller
fn stick_activated(axes: Vec2) -> bool {
    // Radial deadzone
    axes.length_squared() > CONTROLLER_STICK_DEADZONE * CONTROLLER_STICK_DEADZONE
}
