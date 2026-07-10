use bevy::prelude::*;
use lightyear::prelude::AppComponentExt;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerProjectile {
    pub direction: Vec2,
    pub speed_per_tick: f32,
}

/// Local simulation state.
///
/// This component is deliberately not registered for replication. The server
/// and each client calculate it independently from the replicated projectile
/// data.
#[derive(Component, Clone, Copy, Debug, PartialEq, Deref, DerefMut)]
pub struct ProjectilePosition(pub Vec2);

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct ProjectileLifetime {
    pub remaining_ticks: u16,
}

pub fn register(app: &mut App) {
    app.component::<PlayerProjectile>().replicate();
}