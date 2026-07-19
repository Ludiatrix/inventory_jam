use bevy::{
    ecs::{
        query::{Has, With},
        system::{Commands, Query, Res},
    }, log::info, math::Vec2, state::state::State,
};
use leafwing_input_manager::action_state::ActionState;
use lightyear::{
    core::{tick::TickDuration, timeline::LocalTimeline},
    prediction::Predicted,
};

use crate::{
    app::AppState, player::{PlayerAimDirection, PlayerId, PlayerPosition}, projectile::{
        PlayerProjectile, ProjectileLifetime, SpawnProjectile, projectile_spawn_position,
    }, protocol::{inputs::PlayerAction, rooms::GameRoom}, weapon::protocol::{EquippedWeapon, WeaponCooldown, WeaponKind},
};

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
            projectile_speed_per_tick: 11.8,
            projectile_radius: 0.0,
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

/// Converts player fire input into authoritative replicated projectiles.
///
/// Damage, range, speed, and radius are snapshotted into the projectile when it
/// is fired. Changing weapons afterward cannot retroactively alter an existing
/// projectile.
pub(crate) fn fire_equipped_weapons(
    mut commands: Commands,
    mut players: Query<(
        &PlayerId,
        &PlayerPosition,
        &PlayerAimDirection,
        &GameRoom,
        &ActionState<PlayerAction>,
        &EquippedWeapon,
        &mut WeaponCooldown,
    )>,
    app_state: Res<State<AppState>>,
    local_timeline: Res<LocalTimeline>,
    tick_duration: Res<TickDuration>,
) {
    for (
        player_id,
        player_position,
        aim_direction,
        room,
        actions,
        equipped_weapon,
        mut cooldown,
    ) in &mut players
    {
        // `pressed` allows attack speed to control automatic repeat while the
        // button is held. Use `just_pressed` here instead for semi-auto weapons.
        if !actions.pressed(&PlayerAction::Fire) || !cooldown.is_ready(local_timeline.tick()) {
            continue;
        }

        let direction = aim_direction.0.normalize_or_zero();
        if direction == Vec2::ZERO {
            continue;
        }

        let stats = weapon_stats(equipped_weapon.kind, equipped_weapon.level);
        let spawn_position =
            projectile_spawn_position(player_position.0, direction, stats.projectile_radius);

        let expire_time = ProjectileLifetime::new(
            &local_timeline.tick(),
            stats.range,
            stats.projectile_speed_per_tick,
        );

        let projectile = PlayerProjectile {
            owner: player_id.0,
            weapon: equipped_weapon.kind,
            origin: spawn_position,
            direction,
            speed_per_tick: stats.projectile_speed_per_tick,
            damage: stats.damage,
            max_range: stats.range,
            radius: stats.projectile_radius,
            expire_time,
        };

        info!("fire_equipped_weapons");
        commands.queue(SpawnProjectile {
            projectile,
            spawn_position,
            room: *room,
            is_authoritative: matches!(app_state.get(), AppState::Hosting),
        });

        cooldown.restart(&local_timeline, &tick_duration, stats.attacks_per_second);
    }
}
