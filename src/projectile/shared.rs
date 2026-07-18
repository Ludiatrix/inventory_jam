use bevy::prelude::*;
use lightyear::{connection::network_target::NetworkTarget, prelude::{InterpolationTarget, Replicate}};

use crate::{
    enemy::{EnemyHealth, EnemyPosition, shared::ENEMY_COLLISION_RADIUS},
    projectile::protocol::{PlayerProjectile, ProjectileImpact, ProjectilePosition},
    protocol::rooms::GameRoom,
};

pub struct SpawnProjectile {
    pub projectile: PlayerProjectile,
    pub spawn_position: Vec2,
    pub room: GameRoom,
}

impl Command for SpawnProjectile {
    type Out = ();

    fn apply(self, world: &mut World) -> Self::Out {
        world.commands().spawn((
            self.projectile,
            ProjectilePosition(self.spawn_position),
            ProjectileLifetime::from_projectile(&self.projectile),
            self.room,
            Replicate::to_clients(NetworkTarget::All),
            InterpolationTarget::to_clients(NetworkTarget::All),
            Name::new("Projectile"),
        ));
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct ProjectileLifetime {
    pub remaining_ticks: u16,
}

impl ProjectileLifetime {
    pub fn from_projectile(projectile: &PlayerProjectile) -> Self {
        let travel_ticks =
            (projectile.max_range / projectile.speed_per_tick.max(0.001)).ceil() as u32;

        Self {
            remaining_ticks: travel_ticks.saturating_add(2).min(u16::MAX as u32) as u16,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct ImpactLifetime {
    pub remaining_ticks: u16,
}

pub const IMPACT_LIFETIME_TICKS: u16 = 12;

pub fn projectile_spawn_position(
    player_position: Vec2,
    direction: Vec2,
    projectile_radius: f32,
) -> Vec2 {
    const PLAYER_HALF_SIZE: f32 = 25.0;
    const SPAWN_GAP: f32 = 2.0;

    player_position
        + direction.normalize_or_zero()
            * (PLAYER_HALF_SIZE + projectile_radius.max(0.0) + SPAWN_GAP)
}

pub fn move_projectile(position: &mut ProjectilePosition, projectile: &PlayerProjectile) {
    position.0 += projectile.direction.normalize_or_zero() * projectile.speed_per_tick;
}

pub fn projectile_reached_max_range(
    position: ProjectilePosition,
    projectile: &PlayerProjectile,
) -> bool {
    position.0.distance_squared(projectile.origin) >= projectile.max_range * projectile.max_range
}

/// Simulates authoritative projectile motion, hit detection, damage, and death.
pub fn simulate_server_projectiles(
    mut commands: Commands,
    mut projectiles: Query<(
        Entity,
        &PlayerProjectile,
        &mut ProjectilePosition,
        &mut ProjectileLifetime,
        &GameRoom,
    )>,
    mut enemies: Query<(Entity, &EnemyPosition, &mut EnemyHealth)>,
) {
    for (projectile_entity, projectile, mut position, mut lifetime, room) in &mut projectiles {
        move_projectile(&mut position, projectile);

        let mut hit_enemy = None;

        for (enemy_entity, enemy_position, mut health) in &mut enemies {
            if health.current == 0 {
                continue;
            }

            let collision_radius = projectile.radius + ENEMY_COLLISION_RADIUS;
            let hit = position.0.distance_squared(enemy_position.0)
                <= collision_radius * collision_radius;

            if !hit {
                continue;
            }

            health.current = health.current.saturating_sub(projectile.damage);
            hit_enemy = Some((enemy_entity, health.current == 0));
            break;
        }

        if let Some((enemy_entity, enemy_died)) = hit_enemy {
            commands.spawn((
                ProjectileImpact {
                    position: position.0,
                    weapon: projectile.weapon,
                    damage: projectile.damage,
                },
                ImpactLifetime {
                    remaining_ticks: IMPACT_LIFETIME_TICKS,
                },
                Replicate::to_clients(NetworkTarget::All),
                Name::new("Server Projectile Impact"),
            ));

            commands.entity(projectile_entity).despawn();

            if enemy_died {
                commands.entity(enemy_entity).despawn();
            }

            continue;
        }

        lifetime.remaining_ticks = lifetime.remaining_ticks.saturating_sub(1);

        if lifetime.remaining_ticks == 0
            || projectile_reached_max_range(*position, projectile)
            || projectile_is_outside_world(position.0, room)
        {
            commands.entity(projectile_entity).despawn();
        }
    }
}

pub fn expire_server_impacts(
    mut commands: Commands,
    mut impacts: Query<(Entity, &mut ImpactLifetime)>,
) {
    for (entity, mut lifetime) in &mut impacts {
        lifetime.remaining_ticks = lifetime.remaining_ticks.saturating_sub(1);

        if lifetime.remaining_ticks == 0 {
            commands.entity(entity).despawn();
        }
    }
}

fn projectile_is_outside_world(position: Vec2, room: &GameRoom) -> bool {
    !room.bounds().contains(position)
}
