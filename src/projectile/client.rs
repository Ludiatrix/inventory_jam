use crate::app::AppState;
use crate::projectile::shared::{expire_server_impacts, simulate_server_projectiles};
use crate::shared::FixedGameplaySet;
use bevy::prelude::*;

pub struct ProjectileClientPlugin;

impl Plugin for ProjectileClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (simulate_server_projectiles, expire_server_impacts)
                .chain()
                .in_set(FixedGameplaySet::Projectile)
                .run_if(in_state(AppState::Playing)),
        );
    }
}
