use bevy::prelude::*;

use crate::{
    app::ServerState,
    projectile::shared::{expire_server_impacts, simulate_server_projectiles},
    shared::FixedGameplaySet,
};

pub struct ProjectileServerPlugin;

impl Plugin for ProjectileServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (simulate_server_projectiles, expire_server_impacts)
                .chain()
                .in_set(FixedGameplaySet::Projectile)
                .run_if(in_state(ServerState::Hosting)),
        );
    }
}
