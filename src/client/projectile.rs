use bevy::prelude::*;

use crate::{
    protocol::{PlayerProjectile, ProjectileLifetime, ProjectilePosition},
    shared,
};

pub(crate) fn initialize_projectile(
    trigger: On<Add, PlayerProjectile>,
    mut commands: Commands,
    projectiles: Query<&PlayerProjectile>,
) {
    let entity = trigger.entity;

    let Ok(projectile) = projectiles.get(entity) else {
        return;
    };

    commands.entity(entity).insert((
        ProjectilePosition(projectile.origin),
        ProjectileLifetime {
            remaining_ticks: shared::PROJECTILE_LIFETIME_TICKS,
        },
    ));
}

pub(crate) fn simulate_client_projectiles(
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

        if lifetime.remaining_ticks == 0 {
            commands.entity(entity).despawn();
        }
    }
}
