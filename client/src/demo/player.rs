//! Player-specific behavior.

use std::f32::consts::{FRAC_1_PI, FRAC_2_PI};

use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};
use bevy_fog_of_war::prelude::VisionSource;

use crate::{
    AppSystems, PausableSystems, Pause,
    asset_tracking::LoadResource,
    demo::movement::{MovementController, ScreenWrap},
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<PlayerAssets>();

    // Record directional input as movement controls.
    app.add_systems(
        Update,
        (
            record_player_directional_keyboard_input
                .in_set(AppSystems::RecordInput)
                .in_set(PausableSystems),
            record_player_directional_controller_input
                .in_set(AppSystems::RecordInput)
                .in_set(PausableSystems),
            update_player_vision
                .in_set(AppSystems::Update)
                .in_set(PausableSystems),
        ),
    );
}

/// The player character.
pub fn player(
    max_speed: f32,
    player_assets: &PlayerAssets,
    // texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) -> impl Bundle {
    // A texture atlas is a way to split a single image into a grid of related images.
    // You can learn more in this example: https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs
    // TODO: We'll handle animation later
    // let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 6, 2, Some(UVec2::splat(1)), None);
    // let texture_atlas_layout = texture_atlas_layouts.add(layout);
    // let player_animation = PlayerAnimation::new();

    (
        Name::new("Player"),
        Player::default(),
        Sprite::from_image(player_assets.sprite.clone()),
        Transform::from_scale(Vec2::splat(1.0).extend(1.0)).with_translation(Vec3 {
            x: 0.0,
            y: 0.0,
            z: 10.0,
        }),
        MovementController {
            max_speed,
            ..default()
        },
        ScreenWrap,
        // player_animation,

        // Fog of war
        VisionSource::cone(400.0, 0.0, FRAC_2_PI),
        children![(Transform::IDENTITY, VisionSource::circle(100.0),)],
    )
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
struct Player {
    state: PlayerState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
enum PlayerState {
    Idle,
    #[default]
    Walking,
    Running,
}

fn record_player_directional_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut controller_query: Query<&mut MovementController, With<Player>>,
) {
    // Collect directional input.
    let mut intent = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        intent.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        intent.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        intent.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        intent.x += 1.0;
    }

    // Normalize intent so that diagonal movement is the same speed as horizontal / vertical.
    // This should be omitted if the input comes from an analog stick instead.
    let intent = intent.normalize_or_zero();

    // Apply movement intent to controllers.
    for mut controller in &mut controller_query {
        controller.intent = intent;
    }
}

fn record_player_directional_controller_input(
    gamepads: Query<(Entity, &Gamepad)>,
    mut controller_query: Query<(&mut Player, &mut MovementController)>,
) {
    // Collect directional input.
    let mut intent = Vec2::ZERO;
    let mut rotation = Vec2::ZERO;
    let mut new_state = PlayerState::default();

    for (_, gamepad) in &gamepads {
        if gamepad.pressed(GamepadButton::RightTrigger) {
            new_state = PlayerState::Running;
        }

        let left_stick = gamepad.left_stick();
        if left_stick.x.abs() >= 0.1 {
            intent.x = left_stick.x;
        }
        if left_stick.y.abs() >= 0.1 {
            intent.y = left_stick.y;
        }

        let right_stick = gamepad.right_stick();
        if right_stick.x.abs() >= 0.1 {
            rotation.x = right_stick.x;
        }
        if right_stick.y.abs() >= 0.1 {
            rotation.y = right_stick.y;
        }
    }

    if intent == Vec2::ZERO {
        new_state = PlayerState::Idle
    }

    // Normalize intent so that diagonal movement is the same speed as horizontal / vertical.
    // This should be omitted if the input comes from an analog stick instead.
    // let intent = intent;

    // Apply movement intent to controllers.
    for (mut player, mut controller) in &mut controller_query {
        player.state = new_state;
        controller.intent = intent;
        if rotation != Vec2::ZERO {
            controller.facing = rotation;
        }
    }
}

fn update_player_vision(mut query: Query<(&Player, &Transform, &mut VisionSource)>) {
    for (player, transform, mut vision_cone) in &mut query {
        let (axis, angle) = transform.rotation.to_axis_angle();
        vision_cone.direction = axis.z * angle;

        match player.state {
            PlayerState::Idle => {
                vision_cone.range = 600.0;
                vision_cone.angle = FRAC_2_PI * 1.5;
            }
            PlayerState::Walking => {
                vision_cone.range = 400.0;
                vision_cone.angle = FRAC_2_PI * 1.5;
            }
            PlayerState::Running => {
                vision_cone.range = 200.0;
                vision_cone.angle = FRAC_1_PI;
            }
        }
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct PlayerAssets {
    #[dependency]
    sprite: Handle<Image>,
    #[dependency]
    pub steps: Vec<Handle<AudioSource>>,
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            sprite: assets.load_with_settings(
                "images/player/placeholder.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::linear();
                },
            ),
            steps: vec![
                assets.load("audio/sound_effects/step1.ogg"),
                assets.load("audio/sound_effects/step2.ogg"),
                assets.load("audio/sound_effects/step3.ogg"),
                assets.load("audio/sound_effects/step4.ogg"),
            ],
        }
    }
}
