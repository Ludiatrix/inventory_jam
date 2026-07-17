use bevy::prelude::*;
use lightyear::{core::tick::TickDuration, prelude::*};
use serde::{Deserialize, Serialize};

/// Installs weapon component registration on every peer before connections are
/// spawned.
pub struct WeaponProtocolPlugin;

impl Plugin for WeaponProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.component::<EquippedWeapon>().replicate();
        app.component::<WeaponCooldown>().replicate();
    }
}

/// Stable gameplay identifier replicated over the network.
///
/// Clients map this ID to local art and audio. Asset paths are deliberately not
/// replicated because they are presentation data, not authoritative state.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WeaponKind {
    Sword,
    Spear,
    Staff,
}

/// The weapon currently equipped by a player.
///
/// The server owns this component and replicates it to every client. Weapon
/// stats are resolved from `WeaponKind` and `level` on the server.
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct EquippedWeapon {
    pub kind: WeaponKind,
    pub level: u32,
}

impl EquippedWeapon {
    pub const fn new(kind: WeaponKind, level: u32) -> Self {
        Self { kind, level }
    }
}

/// Server-only firing state attached to an authoritative player entity.
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug, Default)]
pub struct WeaponCooldown {
    pub cooldown_tick: Tick,
}

impl WeaponCooldown {
    pub fn is_ready(&self, current_tick: Tick) -> bool {
        self.cooldown_tick < current_tick
    }

    pub fn restart(
        &mut self,
        local_timeline: &LocalTimeline,
        tick_duration: &TickDuration,
        attacks_per_second: f32,
    ) {
        let ticks_per_attack = 1.0 / (attacks_per_second * tick_duration.0.as_secs_f32());
        self.cooldown_tick = local_timeline.tick() + Tick(ticks_per_attack.ceil() as u32);
    }
}
