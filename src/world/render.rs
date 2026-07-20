use crate::shared::{ARENA_WORLD_BOUNDS, SAFEZONE_WORLD_BOUNDS, TILE_PIXEL_SIZE};
use bevy::camera::Camera2d;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

/// Number of tiles retained around the camera. This is intentionally larger
/// than a typical viewport so camera movement never exposes an unloaded edge.
const STREAM_HALF_WIDTH: i32 = 28;
const STREAM_HALF_HEIGHT: i32 = 20;

const ARENA_FLOOR_TILES: &[usize] = &[35, 36, 37, 43, 44, 45, 51, 52, 53];

pub struct WorldRenderPlugin;

impl Plugin for WorldRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoadedWorldTiles>();
        app.add_systems(Startup, load_world_tile_assets);
        app.add_systems(Update, stream_world_tiles);
        app.add_systems(Update, draw_world_boundaries);
    }
}

#[derive(Component)]
struct WorldTile;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum TileRoom {
    Arena,
    Safezone,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct TileKey {
    room: TileRoom,
    x: i32,
    y: i32,
}

#[derive(Resource)]
struct WorldTileAssets {
    arena_image: Handle<Image>,
    arena_layout: Handle<TextureAtlasLayout>,
    safezone_image: Handle<Image>,
    safezone_layout: Handle<TextureAtlasLayout>,
}

#[derive(Resource, Default)]
struct LoadedWorldTiles {
    entities: HashMap<TileKey, Entity>,
    last_camera_tile: Option<IVec2>,
}

fn load_world_tile_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.insert_resource(WorldTileAssets {
        arena_image: asset_server.load("world_tiles/spr_tileset_arena_main.png"),
        arena_layout: layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(16),
            8,
            11,
            None,
            None,
        )),
        safezone_image: asset_server.load("world_tiles/spr_tileset_safezone.png"),
        safezone_layout: layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(16),
            9,
            9,
            None,
            None,
        )),
    });
}

fn stream_world_tiles(
    mut commands: Commands,
    assets: Option<Res<WorldTileAssets>>,
    cameras: Query<&GlobalTransform, With<Camera2d>>,
    mut loaded: ResMut<LoadedWorldTiles>,
) {
    let Some(assets) = assets else {
        return;
    };

    let Some(camera_transform) = cameras.iter().next() else {
        return;
    };

    let camera_position = camera_transform.translation().truncate();
    let camera_tile = IVec2::new(
        (camera_position.x / TILE_PIXEL_SIZE).floor() as i32,
        (camera_position.y / TILE_PIXEL_SIZE).floor() as i32,
    );

    if loaded.last_camera_tile == Some(camera_tile) {
        return;
    }
    loaded.last_camera_tile = Some(camera_tile);

    let mut desired = HashSet::new();

    for y in (camera_tile.y - STREAM_HALF_HEIGHT)..=(camera_tile.y + STREAM_HALF_HEIGHT) {
        for x in (camera_tile.x - STREAM_HALF_WIDTH)..=(camera_tile.x + STREAM_HALF_WIDTH) {
            let world_position = Vec2::new(
                (x as f32 + 0.5) * TILE_PIXEL_SIZE,
                (y as f32 + 0.5) * TILE_PIXEL_SIZE,
            );

            let Some(room) = room_for_position(world_position) else {
                continue;
            };

            let key = TileKey { room, x, y };
            desired.insert(key);

            if loaded.entities.contains_key(&key) {
                continue;
            }

            let entity = spawn_world_tile(&mut commands, &assets, key, world_position);
            loaded.entities.insert(key, entity);
        }
    }

    let stale: Vec<(TileKey, Entity)> = loaded
        .entities
        .iter()
        .filter_map(|(key, entity)| (!desired.contains(key)).then_some((*key, *entity)))
        .collect();

    for (key, entity) in stale {
        commands.entity(entity).despawn();
        loaded.entities.remove(&key);
    }
}

fn spawn_world_tile(
    commands: &mut Commands,
    assets: &WorldTileAssets,
    key: TileKey,
    world_position: Vec2,
) -> Entity {
    let scale = TILE_PIXEL_SIZE / 16.0;

    match key.room {
        TileRoom::Arena => {
            let index = arena_floor_index(key.x, key.y);
            commands
                .spawn((
                    WorldTile,
                    Sprite::from_atlas_image(
                        assets.arena_image.clone(),
                        TextureAtlas {
                            layout: assets.arena_layout.clone(),
                            index,
                        },
                    ),
                    Transform {
                        translation: world_position.extend(0.0),
                        scale: Vec3::splat(scale),
                        ..default()
                    },
                    Name::new("Arena Floor Tile"),
                ))
                .id()
        }
        TileRoom::Safezone => {
            commands
                .spawn((
                    WorldTile,
                    Sprite::from_color(Color::srgb(0.105, 0.12, 0.13), Vec2::splat(16.0)),
                    Transform {
                        translation: world_position.extend(0.0),
                        scale: Vec3::splat(scale),
                        ..default()
                    },
                    Name::new("Safezone Floor Tile"),
                ))
                .id()
        }
    }
}

fn arena_floor_index(x: i32, y: i32) -> usize {
    // Deterministic integer hash: visually varied, stable between frames and
    // clients, and avoids obvious row/column striping.
    let mut value = (x as u32).wrapping_mul(0x9E37_79B9)
        ^ (y as u32).wrapping_mul(0x85EB_CA6B);
    value ^= value >> 16;
    value = value.wrapping_mul(0x7FEB_352D);
    value ^= value >> 15;

    ARENA_FLOOR_TILES[value as usize % ARENA_FLOOR_TILES.len()]
}

fn room_for_position(position: Vec2) -> Option<TileRoom> {
    if ARENA_WORLD_BOUNDS.contains(position) {
        Some(TileRoom::Arena)
    } else if SAFEZONE_WORLD_BOUNDS.contains(position) {
        Some(TileRoom::Safezone)
    } else {
        None
    }
}

fn draw_world_boundaries(mut gizmos: Gizmos) {
    for bounds in [ARENA_WORLD_BOUNDS, SAFEZONE_WORLD_BOUNDS] {
        gizmos.rect_2d(
            Isometry2d::from_translation(bounds.center()),
            bounds.size(),
            Color::srgb(0.95, 0.25, 0.2),
        );
    }
}
