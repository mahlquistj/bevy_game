//! Spawn the main level.

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_fog_of_war::prelude::Capturable;

use crate::{
    asset_tracking::LoadResource,
    audio::music,
    demo::player::{PlayerAssets, player},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<LevelAssets>();
    app.add_plugins((TilemapPlugin,));
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[dependency]
    music: Handle<AudioSource>,
    #[dependency]
    ground_texture: Handle<Image>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            music: assets.load("audio/music/Fluffing A Duck.ogg"),
            ground_texture: assets.load("images/terrain/ground.png"),
        }
    }
}

/// A system that spawns the main level.
pub fn spawn_level(
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
    player_assets: Res<PlayerAssets>,
    // mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let tilemap_entity = spawn_terrain(
        &mut commands,
        level_assets.ground_texture.clone(),
        TilemapSize { x: 10, y: 10 },
    );

    let player = player(400.0, &player_assets);
    let level_entity = commands
        .spawn((
            Name::new("Level"),
            Transform::default(),
            Visibility::default(),
            DespawnOnExit(Screen::Gameplay),
            children![
                player,
                // (
                //     Name::new("Gameplay Music"),
                //     music(level_assets.music.clone())
                // )
            ],
        ))
        .id();

    commands.entity(level_entity).add_child(tilemap_entity);
}

/// Spawns a tilemap and returns its entity, which can then be added as a child of another entity.
/// Tile entities are parented to the tilemap entity so they despawn with it.
pub fn spawn_terrain(commands: &mut Commands, texture: Handle<Image>, size: TilemapSize) -> Entity {
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(size);

    for x in 0..size.x {
        for y in 0..size.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    ..default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
            commands.entity(tilemap_entity).add_child(tile_entity);
        }
    }

    let tile_size = TilemapTileSize {
        x: 2048.0,
        y: 2048.0,
    };
    let grid_size = tile_size.into();
    let map_type = TilemapType::Square;

    commands.entity(tilemap_entity).insert((
        Name::new("Tilemap"),
        TilemapBundle {
            grid_size,
            map_type,
            size,
            storage: tile_storage,
            texture: TilemapTexture::Single(texture),
            tile_size,
            anchor: TilemapAnchor::Center,
            transform: Transform::from_scale(Vec3::splat(0.3)),
            ..default()
        },
    ));

    tilemap_entity
}
