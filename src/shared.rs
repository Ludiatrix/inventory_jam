use crate::protocol::ProtocolPlugin;
use bevy::prelude::*;

pub static ARENA_WORLD_BOUNDS: Rect =
    Rect::from_center_size(Vec2::new(0.0, 0.0), Vec2::new(1600.0, 1200.0));
pub static SHOP_WORLD_BOUNDS: Rect =
    Rect::from_center_size(Vec2::new(1700.0, 0.0), Vec2::new(1600.0, 1200.0));

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
    }
}
