use crate::{app::ServerState, protocol::ProtocolPlugin};
use bevy::prelude::*;

/// World-space size of one 16x16 source-art tile.
pub const TILE_PIXEL_SIZE: f32 = 50.0;

/// Realm of the Mad God realms are approximately 2048 x 2048 logical tiles.
/// Keep the authoritative arena at that scale, but stream only the visible
/// tiles on each GUI process instead of spawning four million sprite entities.
pub const ARENA_WIDTH_IN_TILES: u32 = 2048;
pub const ARENA_HEIGHT_IN_TILES: u32 = 2048;

/// The safezone is a separate, much smaller room. It can be enlarged later
/// without changing the arena's world scale.
pub const SAFEZONE_WIDTH_IN_TILES: u32 = 128;
pub const SAFEZONE_HEIGHT_IN_TILES: u32 = 128;

pub static ARENA_WORLD_BOUNDS: Rect = Rect::from_center_size(
    Vec2::ZERO,
    Vec2::new(
        ARENA_WIDTH_IN_TILES as f32 * TILE_PIXEL_SIZE,
        ARENA_HEIGHT_IN_TILES as f32 * TILE_PIXEL_SIZE,
    ),
);

pub static SAFEZONE_WORLD_BOUNDS: Rect = Rect::from_center_size(
    Vec2::new(
        (ARENA_WIDTH_IN_TILES as f32 * TILE_PIXEL_SIZE * 0.5)
            + (SAFEZONE_WIDTH_IN_TILES as f32 * TILE_PIXEL_SIZE * 0.5)
            + TILE_PIXEL_SIZE * 8.0,
        0.0,
    ),
    Vec2::new(
        SAFEZONE_WIDTH_IN_TILES as f32 * TILE_PIXEL_SIZE,
        SAFEZONE_HEIGHT_IN_TILES as f32 * TILE_PIXEL_SIZE,
    ),
);

pub const GAME_NAME: &str = "Arena of Champions";

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FixedGameplaySet {
    Player,
    Weapon,
    Projectile,
    Persistence,
}

pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ProtocolPlugin);
        app.configure_sets(
            FixedUpdate,
            (
                FixedGameplaySet::Player,
                FixedGameplaySet::Weapon,
                FixedGameplaySet::Projectile,
                FixedGameplaySet::Persistence,
            )
                .chain(),
        );

        #[cfg(feature = "server")]
        app.add_systems(FixedUpdate, update_window_title);
    }
}

fn update_window_title(mut window_query: Query<&mut Window>, state: Res<State<ServerState>>) {
    for mut window in window_query.iter_mut() {
        match state.get() {
            ServerState::Stopped => (),
            ServerState::Hosting => window.title = format!("{}: Server", GAME_NAME),
        }
    }
}
