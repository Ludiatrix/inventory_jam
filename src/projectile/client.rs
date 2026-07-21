use bevy::prelude::*;

use crate::{
    app::ClientState, projectile::shared::simulate_predicted_projectiles, shared::FixedGameplaySet,
};

pub struct ProjectileClientPlugin;

impl Plugin for ProjectileClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            simulate_predicted_projectiles
                .in_set(FixedGameplaySet::Projectile)
                .run_if(in_state(ClientState::Playing)),
        );
    }
}
