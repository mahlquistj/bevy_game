//! Player-specific behavior.

use std::f32::consts::FRAC_2_PI;

use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};
use bevy_fog_of_war::prelude::VisionSource;

use crate::{
    AppSystems, PausableSystems,
    asset_tracking::LoadResource,
    demo::{
        movement::MovementController,
        player::controller::{PlayerController, PlayerState},
    },
};

mod controller;

const FOW_CONE_BASE_ANGLE: f32 = FRAC_2_PI * 2.0;
const FOW_CONE_BASE_RANGE: f32 = 400.0;
const FOW_IDLE_MULTIPLIER: f32 = 1.2;
const FOW_RUNNING_MULTIPLIER: f32 = 0.9;
const FOW_CIRCLE_BASE_RANGE: f32 = 120.0;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(controller::plugin);
    app.load_resource::<PlayerAssets>();
    // Record directional input as movement controls.
    app.add_systems(
        Update,
        (update_player_vision
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),),
    );
}

/// The player character.
pub fn player(
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
        Player,
        Sprite::from_image(player_assets.sprite.clone()),
        Transform::from_scale(Vec2::splat(1.0).extend(1.0)).with_translation(Vec3 {
            x: 0.0,
            y: 0.0,
            z: 10.0,
        }),
        PlayerController::default(),
        // player_animation,

        // Fog of war
        VisionSource::cone(FOW_CONE_BASE_RANGE, 0.0, FOW_CONE_BASE_ANGLE),
        children![(
            Transform::IDENTITY,
            VisionSource::circle(FOW_CIRCLE_BASE_RANGE),
        )],
    )
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
struct Player;

fn update_player_vision(query: Single<(&PlayerController, &Transform, &mut VisionSource)>) {
    let (controller, transform, mut vision_cone) = query.into_inner();
    let (axis, angle) = transform.rotation.to_axis_angle();
    vision_cone.direction = axis.z * angle;

    match controller.state {
        PlayerState::Idle => {
            vision_cone.range = FOW_CONE_BASE_RANGE * FOW_IDLE_MULTIPLIER;
            vision_cone.angle = FOW_CONE_BASE_ANGLE * FOW_IDLE_MULTIPLIER;
        }
        PlayerState::Walking => {
            vision_cone.range = FOW_CONE_BASE_RANGE;
            vision_cone.angle = FOW_CONE_BASE_ANGLE;
        }
        PlayerState::Running => {
            vision_cone.range = FOW_CONE_BASE_RANGE * FOW_RUNNING_MULTIPLIER;
            vision_cone.angle = FOW_CONE_BASE_ANGLE * FOW_RUNNING_MULTIPLIER;
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
