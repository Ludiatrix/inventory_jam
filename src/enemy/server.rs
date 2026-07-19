use crate::app::AppState;
use crate::enemy::api::SpawnEnemy;
use crate::enemy::protocol::{EnemyHealth, EnemyPosition};
use bevy::prelude::*;
use lightyear::prelude::*;
use rand::Rng;

const ENEMY_SPAWN_RADIUS: f32 = 150.0;
const ENEMY_MAX_HEALTH: u32 = 100;

pub struct EnemyServerPlugin;

impl Plugin for EnemyServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (
            spawn_debug_enemy.run_if(in_state(AppState::Hosting)),
            spawn_requested_enemy.run_if(in_state(AppState::Playing)),
        ));
        app.add_message::<SpawnEnemy>();
    }
}

/// The only system that converts pool requests into authoritative replicated
/// fragment entities.
pub(crate) fn spawn_requested_enemy(
    mut commands: Commands,
    mut requests: MessageReader<SpawnEnemy>,
) {
    for request in requests.read() {
        spawn_enemy(
            &mut commands,
            request.position,
        );
    }
}

/// Temporary server debug spawner. Press N to create a target.
fn spawn_debug_enemy(input: Option<Res<ButtonInput<KeyCode>>>, mut commands: Commands) {
    if input.is_some_and(|input| input.just_pressed(KeyCode::KeyN)) {
        
        let position = get_random_position();

        let entity = spawn_enemy(&mut commands, position);

        info!(?entity, ?position, "Created debug enemy");
    }
}

fn spawn_enemy(commands: &mut Commands, spawn_position: Vec2) -> Entity {
    
    let entity = commands
        .spawn((
            EnemyPosition(spawn_position),
            EnemyHealth::new(ENEMY_MAX_HEALTH),
            Replicate::to_clients(NetworkTarget::All),
            InterpolationTarget::to_clients(NetworkTarget::All),
            Name::new("Enemy"),
        ))
        .id();

    entity
}

fn get_random_position() -> Vec2 {
    let mut rng = rand::rng();
    let angle = rng.random_range(0.0..std::f32::consts::TAU);
    let radius = rng
        .random_range(0.0..=ENEMY_SPAWN_RADIUS * ENEMY_SPAWN_RADIUS)
        .sqrt();
    let position = Vec2::new(angle.cos(), angle.sin()) * radius;

    position
}