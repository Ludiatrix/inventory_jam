use bevy::prelude::*;
use lightyear::{
    connection::network_target::NetworkTarget,
    core::timeline::LocalTimeline,
    prediction::{Predicted, despawn::PredictionDespawnCommandsExt},
    prelude::{InterpolationTarget, PreSpawned, PredictionTarget, Replicate},
};

use crate::player::protocol::{PlayerAristeia, PlayerHealth};
use crate::world::api::AddGlobalAristeia;
use crate::{
    enemy::{EnemyHealth, EnemyPosition},
    fragment::api::SpawnFragmentPool,
    player::api::AddKillsToAristeia,
    player::{PlayerId, PlayerPosition},
    projectile::protocol::{PlayerProjectile, ProjectileImpact, ProjectilePosition},
    protocol::rooms::GameRoom,
    settings::GameSettings,
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
                PredictionTarget::to_clients(NetworkTarget::Single(owner)),
                InterpolationTarget::to_clients(NetworkTarget::AllExceptSingle(owner)),
            ));
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct ImpactLifetime {
    pub remaining_ticks: u16,
}

pub fn projectile_spawn_position(
    player_position: Vec2,
    direction: Vec2,
    projectile_radius: f32,
    settings: &GameSettings,
) -> Vec2 {
    player_position
        + direction.normalize_or_zero()
            * (settings.player.half_size
                + projectile_radius.max(0.0)
                + settings.projectile.spawn_gap)
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
    mut players: Query<(
        Entity,
        &PlayerId,
        &PlayerPosition,
        &GameRoom,
        &mut PlayerHealth,
        &PlayerAristeia,
    )>,
    mut enemies: Query<(Entity, &EnemyPosition, &GameRoom, &mut EnemyHealth)>,
    local_timeline: Res<LocalTimeline>,
    mut fragment_drops: MessageWriter<SpawnFragmentPool>,
    mut aristeia_tracking: MessageWriter<AddKillsToAristeia>,
    mut global_aristeia: MessageWriter<AddGlobalAristeia>,
    settings: Res<GameSettings>,
) {
    for (projectile_entity, projectile, mut position, room) in &mut projectiles {
        move_projectile(&mut position, projectile);

        if get_hit_player(&mut players, projectile, &position, room, &settings).is_some() {
            spawn_projectile_impact(
                &mut commands,
                projectile_entity,
                projectile,
                &position,
                &settings,
            );
            continue;
        }

        if let Some((enemy_entity, enemy_died)) =
            get_hit_enemy(&mut enemies, projectile, &position, room, &settings)
        {
            spawn_projectile_impact(
                &mut commands,
                projectile_entity,
                projectile,
                &position,
                &settings,
            );
            if enemy_died {
                let personal_aristeia = players
                    .iter()
                    .find(|(_, player_id, _, _, _, _)| player_id.0 == projectile.owner)
                    .map(|(_, _, _, _, _, aristeia)| aristeia.current)
                    .unwrap_or(0);
                let bonus = u32::from(settings.fragment.bonus_drops_per_aristeia)
                    .saturating_mul(personal_aristeia);
                let drop_count = u32::from(settings.fragment.base_drop_count)
                    .saturating_add(bonus)
                    .min(u32::from(settings.fragment.maximum_drop_count))
                    as u16;

                fragment_drops.write(SpawnFragmentPool::new(position.0, drop_count, *room));
                commands.entity(enemy_entity).despawn();
                aristeia_tracking.write(AddKillsToAristeia::new(projectile.owner, 1));
                global_aristeia.write(AddGlobalAristeia(
                    settings.global_aristeia.contribution_per_kill,
                ));
            }
            continue;
        }

        check_projectile_range(
            &mut commands,
            &local_timeline,
            projectile_entity,
            projectile,
            &position,
            room,
            &settings,
        );
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
    settings: Res<GameSettings>,
) {
    for (entity, projectile, mut position, room) in &mut projectiles {
        move_projectile(&mut position, projectile);
        check_projectile_range(
            &mut commands,
            &local_timeline,
            entity,
            projectile,
            &position,
            room,
            &settings,
        );
    }
}

fn check_projectile_range(
    commands: &mut Commands<'_, '_>,
    local_timeline: &Res<'_, LocalTimeline>,
    projectile_entity: Entity,
    projectile: &PlayerProjectile,
    position: &ProjectilePosition,
    room: &GameRoom,
    settings: &GameSettings,
) {
    if projectile.expire_time.is_expired(&local_timeline.tick())
        || projectile_reached_max_range(position, projectile)
        || projectile_is_outside_world(position.0, room, settings)
    {
        commands.entity(projectile_entity).prediction_despawn();
    }
}

fn spawn_projectile_impact(
    commands: &mut Commands<'_, '_>,
    projectile_entity: Entity,
    projectile: &PlayerProjectile,
    position: &ProjectilePosition,
    settings: &GameSettings,
) {
    commands.spawn((
        ProjectileImpact {
            position: position.0,
            weapon: projectile.weapon,
            damage: projectile.damage,
        },
        ImpactLifetime {
            remaining_ticks: settings.projectile.impact_lifetime_ticks,
        },
        Replicate::to_clients(NetworkTarget::All),
        Name::new("Server Projectile Impact"),
    ));

    commands.entity(projectile_entity).prediction_despawn();
}

fn get_hit_player(
    players: &mut Query<
        '_,
        '_,
        (
            Entity,
            &PlayerId,
            &PlayerPosition,
            &GameRoom,
            &mut PlayerHealth,
            &PlayerAristeia,
        ),
    >,
    projectile: &PlayerProjectile,
    position: &ProjectilePosition,
    projectile_room: &GameRoom,
    settings: &GameSettings,
) -> Option<Entity> {
    for (player_entity, player_id, player_position, player_room, mut health, _) in players {
        if player_id.0 == projectile.owner || player_room != projectile_room || health.current == 0
        {
            continue;
        }

        let collision_radius = projectile.radius + settings.player.collision_radius;
        let hit =
            position.0.distance_squared(player_position.0) <= collision_radius * collision_radius;

        if !hit {
            continue;
        }

        health.current = health.current.saturating_sub(projectile.damage);

        info!(
            ?player_entity,
            current_health = health.current,
            maximum_health = health.maximum,
            "Player took projectile damage"
        );

        return Some(player_entity);
    }

    None
}

fn get_hit_enemy(
    enemies: &mut Query<'_, '_, (Entity, &EnemyPosition, &GameRoom, &mut EnemyHealth)>,
    projectile: &PlayerProjectile,
    position: &ProjectilePosition,
    projectile_room: &GameRoom,
    settings: &GameSettings,
) -> Option<(Entity, bool)> {
    for (enemy_entity, enemy_position, enemy_room, mut health) in enemies {
        if enemy_room != projectile_room || health.current == 0 {
            continue;
        }

        let collision_radius = projectile.radius + settings.enemy.collision_radius;
        let hit =
            position.0.distance_squared(enemy_position.0) <= collision_radius * collision_radius;

        if !hit {
            continue;
        }

        health.current = health.current.saturating_sub(projectile.damage);
        return Some((enemy_entity, health.current == 0));
    }

    None
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

fn projectile_is_outside_world(position: Vec2, room: &GameRoom, settings: &GameSettings) -> bool {
    !room.bounds(&settings.world).contains(position)
}
