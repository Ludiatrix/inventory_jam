use bevy::prelude::*;

use crate::{
    app::ServerState,
    enemy::{api::SpawnEnemy, server::EnemySpawnSet, EnemyHealth, EnemyPosition},
    protocol::rooms::{GameRoom, GameRooms},
    settings::GameSettings,
};

pub struct WorldServerPlugin;

impl Plugin for WorldServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            run_enemy_spawner
                .in_set(EnemySpawnSet::Request)
                .run_if(in_state(ServerState::Hosting)),
        );
    }
}

fn run_enemy_spawner(
    settings: Res<GameSettings>,
    enemies: Query<(&EnemyPosition, &EnemyHealth, &GameRoom)>,
    mut spawn_messages: MessageWriter<SpawnEnemy>,
) {
    let mut occupied_positions: Vec<Vec2> = enemies
        .iter()
        .filter(|(_, health, room)| health.current > 0 && room.room == GameRooms::Arena)
        .map(|(position, _, _)| position.0)
        .collect();

    let missing_enemy_count = settings.enemy.limit.saturating_sub(occupied_positions.len());
    let spawn_count = missing_enemy_count.min(settings.enemy.max_spawns_per_tick);

    for _ in 0..spawn_count {
        let spawn_position = find_spawn_location_candidate(
            &occupied_positions,
            settings.world.world_radius,
            settings.enemy.spawn_candidate_count,
        );
        occupied_positions.push(spawn_position);
        spawn_messages.write(SpawnEnemy::new(spawn_position));
    }
}

fn find_spawn_location_candidate(
    occupied_positions: &[Vec2],
    world_radius: f32,
    candidate_count: usize,
) -> Vec2 {
    if occupied_positions.is_empty() {
        return random_point_in_circle(world_radius);
    }

    let mut best_candidate = Vec2::ZERO;
    let mut best_nearest_distance_squared = -1.0_f32;

    for _ in 0..candidate_count {
        let candidate = random_point_in_circle(world_radius);
        let nearest_distance_squared = occupied_positions
            .iter()
            .map(|position| candidate.distance_squared(*position))
            .fold(f32::MAX, f32::min);

        if nearest_distance_squared > best_nearest_distance_squared {
            best_nearest_distance_squared = nearest_distance_squared;
            best_candidate = candidate;
        }
    }

    best_candidate
}

fn random_point_in_circle(radius: f32) -> Vec2 {
    let angle = rand::random::<f32>() * std::f32::consts::TAU;
    let distance = rand::random::<f32>().sqrt() * radius;
    Vec2::new(angle.cos(), angle.sin()) * distance
}
