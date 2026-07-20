use crate::app::game_is_active;
use crate::shared::{ARENA_WORLD_BOUNDS, SAFEZONE_WORLD_BOUNDS, TILE_PIXEL_SIZE};
use bevy::prelude::*;

pub struct WorldRenderPlugin;

impl Plugin for WorldRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_world_tiles);
        app.add_systems(Update, draw_world_boundaries.run_if(game_is_active));
    }
}

#[derive(Component)]
struct WorldTile;

fn setup_world_tiles(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let arena_image = asset_server.load("world_tiles/spr_tileset_arena_main.png");
    let arena_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(16),
        8,
        11,
        None,
        None,
    ));

    let safezone_image = asset_server.load("world_tiles/spr_tileset_safezone.png");
    let safezone_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(16),
        9,
        9,
        None,
        None,
    ));

    spawn_tiled_area(
        &mut commands,
        ARENA_WORLD_BOUNDS,
        &arena_image,
        &arena_layout,
        &[27, 28, 35, 36, 43, 44, 51, 52],
        0.0,
    );

    spawn_tiled_area(
        &mut commands,
        SAFEZONE_WORLD_BOUNDS,
        &safezone_image,
        &safezone_layout,
        &[39, 40, 41, 48, 49, 50],
        0.0,
    );
}

fn spawn_tiled_area(
    commands: &mut Commands,
    bounds: Rect,
    image: &Handle<Image>,
    layout: &Handle<TextureAtlasLayout>,
    tile_indices: &[usize],
    z: f32,
) {
    let columns = (bounds.width() / TILE_PIXEL_SIZE).ceil() as i32;
    let rows = (bounds.height() / TILE_PIXEL_SIZE).ceil() as i32;
    let scale = TILE_PIXEL_SIZE / 16.0;

    for row in 0..rows {
        for column in 0..columns {
            let x = bounds.min.x + TILE_PIXEL_SIZE * (column as f32 + 0.5);
            let y = bounds.min.y + TILE_PIXEL_SIZE * (row as f32 + 0.5);
            let index = tile_indices[((row + column) as usize) % tile_indices.len()];

            commands.spawn((
                WorldTile,
                Sprite::from_atlas_image(
                    image.clone(),
                    TextureAtlas {
                        layout: layout.clone(),
                        index,
                    },
                ),
                Transform {
                    translation: Vec3::new(x, y, z),
                    scale: Vec3::splat(scale),
                    ..default()
                },
                Name::new("World Tile"),
            ));
        }
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
