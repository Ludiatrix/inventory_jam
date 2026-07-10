use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use lightyear::prelude::*;

use crate::{
    protocol::{
        PlayerAction, PlayerAimDirection, PlayerPosition, PlayerProjectile, ProjectileLifetime,
        ProjectilePosition,
    },
    shared,
};

pub(crate) fn update_player_aim_direction(
    mut players: Query<(
        &ActionState<PlayerAction>,
        &mut PlayerAimDirection,
        Has<Predicted>,
    )>,
    host_server: Query<(), With<lightyear::connection::host::HostServer>>,
) {
    let is_host_server = !host_server.is_empty();

    for (actions, mut aim_direction, predicted) in &mut players {
        // In host-client mode, do not run authoritative gameplay against the
        // client's predicted copy.
        if is_host_server && predicted {
            continue;
        }

        let aim = actions.clamped_axis_pair(&PlayerAction::Aim);

        if aim.length_squared() > 0.0001 {
            aim_direction.0 = aim.normalize_or_zero();
        }
    }
}

pub(crate) fn fire_player_projectiles(
    mut commands: Commands,
    players: Query<(
        &PlayerPosition,
        &PlayerAimDirection,
        &ActionState<PlayerAction>,
        Has<Predicted>,
    )>,
    host_server: Query<(), With<lightyear::connection::host::HostServer>>,
) {
    let is_host_server = !host_server.is_empty();

    for (player_position, aim_direction, actions, predicted) in &players {
        if is_host_server && predicted {
            continue;
        }

        if !actions.just_pressed(&PlayerAction::Fire) {
            continue;
        }

        let direction = aim_direction.0.normalize_or_zero();

        if direction == Vec2::ZERO {
            continue;
        }

        let spawn_position = player_position.0 + direction * shared::PROJECTILE_SPAWN_OFFSET;

        commands.spawn((
            PlayerProjectile {
                origin: spawn_position,
                direction,
                speed_per_tick: shared::PROJECTILE_SPEED_PER_TICK,
            },
            ProjectilePosition(spawn_position),
            ProjectileLifetime {
                remaining_ticks: shared::PROJECTILE_LIFETIME_TICKS,
            },
            Replicate::to_clients(NetworkTarget::All),
            Name::new("Player Projectile"),
        ));
    }
}

pub(crate) fn simulate_server_projectiles(
    mut commands: Commands,
    mut projectiles: Query<(
        Entity,
        &PlayerProjectile,
        &mut ProjectilePosition,
        &mut ProjectileLifetime,
    )>,
) {
    for (entity, projectile, mut position, mut lifetime) in &mut projectiles {
        shared::move_projectile(&mut position, projectile);

        lifetime.remaining_ticks = lifetime.remaining_ticks.saturating_sub(1);

        if lifetime.remaining_ticks == 0 || projectile_is_outside_world(position.0) {
            commands.entity(entity).despawn();
        }
    }
}

fn projectile_is_outside_world(position: Vec2) -> bool {
    let margin = Vec2::splat(100.0);
    let limit = shared::WORLD_HALF_SIZE + margin;

    position.x.abs() > limit.x || position.y.abs() > limit.y
}
