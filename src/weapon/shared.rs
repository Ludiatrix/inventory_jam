use bevy::prelude::*;
use lightyear::core::{
    tick::{Tick, TickDuration},
    timeline::LocalTimeline,
};

use crate::weapon::protocol::WeaponKind;

/// Authoritative combat values resolved from an equipped weapon.
///
/// Keep these values in shared code so server simulation and client-side
/// presentation can agree on geometry. Only the server may use them to decide
/// whether damage is applied.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WeaponStats {
    pub damage: u32,
    pub range: f32,
    pub attacks_per_second: f32,
    pub projectile_speed_per_tick: f32,
    pub projectile_radius: f32,
}

/// Server-only firing state attached to an authoritative player entity.
#[derive(Component, Clone, Copy, Debug, Default)]
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

/// Returns the final stats for a weapon and level.
///
/// Level scaling is intentionally centralized here. Do not store duplicated
/// damage/range/attack-speed fields on each player.
pub fn weapon_stats(kind: WeaponKind, level: u32) -> WeaponStats {
    let level = level.max(1);
    let bonus_levels = level.saturating_sub(1);

    match kind {
        WeaponKind::Sword => WeaponStats {
            damage: 20 + bonus_levels * 4,
            range: 360.0 + bonus_levels as f32 * 12.0,
            attacks_per_second: 2.5 + bonus_levels as f32 * 0.08,
            projectile_speed_per_tick: 18.0,
            projectile_radius: 18.0,
        },
        WeaponKind::Spear => WeaponStats {
            damage: 32 + bonus_levels * 6,
            range: 520.0 + bonus_levels as f32 * 16.0,
            attacks_per_second: 1.5 + bonus_levels as f32 * 0.05,
            projectile_speed_per_tick: 22.0,
            projectile_radius: 12.0,
        },
        WeaponKind::Staff => WeaponStats {
            damage: 16 + bonus_levels * 3,
            range: 460.0 + bonus_levels as f32 * 14.0,
            attacks_per_second: 3.2 + bonus_levels as f32 * 0.1,
            projectile_speed_per_tick: 16.0,
            projectile_radius: 14.0,
        },
    }
}
