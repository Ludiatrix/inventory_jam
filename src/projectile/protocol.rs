use crate::weapon::protocol::WeaponKind;
use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

pub struct ProjectileProtocolPlugin;

impl Plugin for ProjectileProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.component::<PlayerProjectile>().replicate();
        app.component::<ProjectileImpact>().replicate();
        app.component::<ProjectilePosition>()
            .replicate()
            .predict()
            .add_linear_interpolation();
    }
}

/// Authoritative attack snapshot carried by a projectile.
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct PlayerProjectile {
    pub owner: PeerId,
    pub weapon: WeaponKind,
    pub origin: Vec2,
    pub direction: Vec2,
    pub speed_per_tick: f32,
    pub damage: u32,
    pub max_range: f32,
    pub radius: f32,
}

/// Short-lived replicated impact event represented as an entity.
///
/// Clients use this to create local hit VFX. The server has already resolved
/// the hit and applied damage before this component is replicated.
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct ProjectileImpact {
    pub position: Vec2,
    pub weapon: WeaponKind,
    pub damage: u32,
}

/// If a projectile is "smart" (e.g. homing projectile), we want to
/// synchronize the projectile position to avoid desync.
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Deref, DerefMut)]
pub struct ProjectilePosition(pub Vec2);

impl Ease for ProjectilePosition {
    fn interpolating_curve_unbounded(start: Self, end: Self) -> impl Curve<Self> {
        FunctionCurve::new(Interval::UNIT, move |t| {
            ProjectilePosition(Vec2::lerp(start.0, end.0, t))
        })
    }
}
