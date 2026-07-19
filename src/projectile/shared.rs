use bevy::prelude::*;
use lightyear::{
    connection::network_target::NetworkTarget, core::timeline::LocalTimeline, prediction::{Predicted, despawn::PredictionDespawnCommandsExt}, prelude::{InterpolationTarget, PreSpawned, PredictionTarget, Replicate},
};

use crate::{
    enemy::{EnemyHealth, EnemyPosition, shared::ENEMY_COLLISION_RADIUS},
    projectile::protocol::{PlayerProjectile, ProjectileImpact, ProjectilePosition},
    protocol::rooms::GameRoom,
};

pub struct SpawnProjectile {
    pub projectile: PlayerProjectile,
    pub spawn_position: Vec2,
    pub room: GameRoom,
    pub is_authoritative: bool,
}

impl Command for SpawnProjectile {
    type Out = ();

    fn apply(self, world: &mut World) -> Self::Out {
        info!("Spawning projectile");
        let mut binding = world.commands();
        let mut entity = binding.spawn((
            self.projectile,
            ProjectilePosition(self.spawn_position),
            self.room,
            PreSpawned::default(),
            Name::new("Projectile"),
        ));

        if self.is_authoritative {
        let owner = self.projectile.owner;

        entity.insert((
            Replicate::to_clients(NetworkTarget::All),

            // Only the client that fired the projectile predicts it.
            PredictionTarget::to_clients(NetworkTarget::Single(owner)),

            // Every other client receives an interpolated representation.
            InterpolationTarget::to_clients(
            NetworkTarget::AllExceptSingle(owner),
            ),
        ));
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
    position: &ProjectilePosition,
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
        &GameRoom,
    )>,
    mut enemies: Query<(Entity, &EnemyPosition, &mut EnemyHealth)>,
    local_timeline: Res<LocalTimeline>,
) {
    for (projectile_entity, projectile, mut position, room) in &mut projectiles {
        move_projectile(&mut position, projectile);

        let hit_enemy = get_hit_enemy(&mut enemies, projectile, &position);

        if let Some((enemy_entity, enemy_died)) = hit_enemy {
            spawn_projectile_impact(&mut commands, projectile_entity, projectile, &position);

            if enemy_died {
                commands.entity(enemy_entity).despawn();
            }

            continue;
        }

        check_projectile_range(&mut commands, &local_timeline, projectile_entity, projectile, position, room);
    }
}

/// Advances only locally predicted projectiles.
pub fn simulate_predicted_projectiles(
    mut commands: Commands,
    mut projectiles: Query<
        (
            Entity,
            &PlayerProjectile,
            &mut ProjectilePosition,
            &GameRoom,
        ),
        With<Predicted>,
    >,
    local_timeline: Res<LocalTimeline>,
) {
    for (entity, projectile, mut position, room) in &mut projectiles {
        move_projectile(&mut position, projectile);
        check_projectile_range(&mut commands, &local_timeline, entity, projectile, position, room);
    }
}

fn check_projectile_range(commands: &mut Commands<'_, '_>, local_timeline: &Res<'_, LocalTimeline>, projectile_entity: Entity, projectile: &PlayerProjectile, position: Mut<'_, ProjectilePosition>, room: &GameRoom) {
    if projectile
        .expire_time
        .is_expired(&local_timeline.tick())
        || projectile_reached_max_range(&position, projectile)
        || projectile_is_outside_world(position.0, room)
    {
        commands
            .entity(projectile_entity)
            .prediction_despawn();
    }
}

fn spawn_projectile_impact(commands: &mut Commands<'_, '_>, projectile_entity: Entity, projectile: &PlayerProjectile, position: &Mut<'_, ProjectilePosition>) {
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

    commands
        .entity(projectile_entity)
        .prediction_despawn();
}

fn get_hit_enemy(enemies: &mut Query<'_, '_, (Entity, &EnemyPosition, &mut EnemyHealth)>, projectile: &PlayerProjectile, position: &Mut<'_, ProjectilePosition>) -> Option<(Entity, bool)> {
    let mut hit_enemy = None;

    for (enemy_entity, enemy_position, mut health) in enemies {
        if health.current == 0 {
            continue;
        }

        let collision_radius =
            projectile.radius + ENEMY_COLLISION_RADIUS;

        let hit = position
            .0
            .distance_squared(enemy_position.0)
            <= collision_radius * collision_radius;

        if !hit {
            continue;
        }

        health.current =
            health.current.saturating_sub(projectile.damage);

        hit_enemy = Some((
            enemy_entity,
            health.current == 0,
        ));

        break;
    }
    hit_enemy
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
