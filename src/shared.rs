use crate::{app::ServerState, protocol::ProtocolPlugin};
use bevy::prelude::*;

/// World-space size of one 16x16 source-art tile.
pub const TILE_PIXEL_SIZE: f32 = 50.0;
pub const ARENA_AREA_IN_TILES: Vec2 = Vec2::new(32.0, 24.0);
pub const SAFEZONE_AREA_IN_TILES: Vec2 = Vec2::new(32.0, 24.0);

pub static ARENA_WORLD_BOUNDS: Rect =
    Rect::from_center_size(Vec2::new(0.0, 0.0), Vec2::new(1600.0, 1200.0));
pub static SAFEZONE_WORLD_BOUNDS: Rect =
    Rect::from_center_size(Vec2::new(1700.0, 0.0), Vec2::new(1600.0, 1200.0));

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
