use crate::app::ServerState;
use crate::enemy::{EnemyHealth, EnemyPosition, shared::ENEMY_COLLISION_RADIUS};
use crate::player::protocol::{
    PlayerAimDirection, PlayerBundle, PlayerHealth, PlayerPosition, PlayerVisual,
};
use crate::player::shared::{PLAYER_HALF_SIZE, player_movement};
use crate::protocol::inputs::PlayerAction;
use crate::protocol::rooms::{GameRoom, GameRooms};
use crate::shared::FixedGameplaySet;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use lightyear::connection::client::Connected;
use lightyear::connection::client_of::ClientOf;
use lightyear::prelude::*;
use rand::Rng;

const SPAWN_ATTEMPTS: usize = 64;
const ENEMY_CONTACT_DAMAGE: u32 = 10;
const ENEMY_CONTACT_DAMAGE_INTERVAL_SECONDS: f32 = 0.5;

#[derive(Component, Clone, Copy, Debug, Default)]
struct EnemyContactDamageCooldown {
    remaining_seconds: f32,
}

/// Controls where newly connected players appear.
///
/// `Annulus` chooses a random point between `minimum_radius` and
/// `maximum_radius` from `center`. Set both radii to the same value to spawn at
/// an exact distance from the center.
///
/// `Fixed` always uses the requested position, clamped inside the room bounds.
#[derive(Resource, Clone, Copy, Debug)]
pub enum PlayerSpawnMode {
    Annulus {
        center: Vec2,
        minimum_radius: f32,
        maximum_radius: f32,
    },
    Fixed(Vec2),
}

impl Default for PlayerSpawnMode {
    fn default() -> Self {
        Self::Annulus {
            center: Vec2::ZERO,
            minimum_radius: 350.0,
            maximum_radius: 500.0,
        }
    }
}

pub struct PlayerServerPlugin;

impl Plugin for PlayerServerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerSpawnMode>();
        app.add_observer(handle_connected);

        app.add_systems(
            FixedUpdate,
            (
                player_movement,
                update_player_aim_direction,
                apply_enemy_contact_damage,
            )
                .chain()
                .in_set(FixedGameplaySet::Player)
                .run_if(in_state(ServerState::Hosting)),
        );

        app.add_systems(
            FixedUpdate,
            debug_switch_rooms.run_if(in_state(ServerState::Hosting)),
        );
    }
}

pub(crate) fn handle_connected(
    trigger: On<Add, Connected>,
    query: Query<&RemoteId, With<ClientOf>>,
    spawn_mode: Res<PlayerSpawnMode>,
    mut commands: Commands,
) {
    let Ok(client_id) = query.get(trigger.entity) else {
        return;
    };

    let client_id = client_id.0;
    let room = GameRoom {
        room: GameRooms::Arena,
    };
    let spawn_position = choose_player_spawn(*spawn_mode, room);

    let entity = commands
        .spawn((
            Name::new(format!("Player: {}", client_id)),
            PlayerBundle::new(client_id, spawn_position),
            PlayerVisual,
            EnemyContactDamageCooldown::default(),
            ActionState::<PlayerAction>::default(),
            Replicate::to_clients(NetworkTarget::All),
            PredictionTarget::to_clients(NetworkTarget::Single(client_id)),
            InterpolationTarget::to_clients(NetworkTarget::AllExceptSingle(client_id)),
            ControlledBy {
                owner: trigger.entity,
                lifetime: Default::default(),
            },
        ))
        .id();

    info!(?entity, ?client_id, ?spawn_position, "Created player entity");
}

fn choose_player_spawn(mode: PlayerSpawnMode, room: GameRoom) -> Vec2 {
    let bounds = room.bounds().inflate(-PLAYER_HALF_SIZE);

    match mode {
        PlayerSpawnMode::Fixed(position) => position.clamp(bounds.min, bounds.max),
        PlayerSpawnMode::Annulus {
            center,
            minimum_radius,
            maximum_radius,
        } => {
            let minimum_radius = minimum_radius.max(0.0);
            let maximum_radius = maximum_radius.max(minimum_radius);
            let mut rng = rand::rng();

            for _ in 0..SPAWN_ATTEMPTS {
                let angle = rng.random_range(0.0..std::f32::consts::TAU);
                let radius_squared = rng.random_range(
                    minimum_radius * minimum_radius..=maximum_radius * maximum_radius,
                );
                let radius = radius_squared.sqrt();
                let candidate = center + Vec2::new(angle.cos(), angle.sin()) * radius;

                if bounds.contains(candidate) {
                    return candidate;
                }
            }

            // A malformed or impossible annulus still produces a valid position.
            center.clamp(bounds.min, bounds.max)
        }
    }
}

pub(crate) fn update_player_aim_direction(
    mut players: Query<(&ActionState<PlayerAction>, &mut PlayerAimDirection)>,
) {
    for (actions, mut aim_direction) in &mut players {
        let aim = actions.clamped_axis_pair(&PlayerAction::Aim);

        if aim.length_squared() > 0.0001 {
            aim_direction.0 = aim.normalize_or_zero();
        }
    }
}

fn apply_enemy_contact_damage(
    time: Res<Time>,
    mut players: Query<
        (
            Entity,
            &PlayerPosition,
            &GameRoom,
            &mut PlayerHealth,
            &mut EnemyContactDamageCooldown,
        ),
        With<ControlledBy>,
    >,
    enemies: Query<(&EnemyPosition, &EnemyHealth)>,
) {
    for (player_entity, player_position, room, mut health, mut cooldown) in &mut players {
        cooldown.remaining_seconds =
            (cooldown.remaining_seconds - time.delta_secs()).max(0.0);

        if room.room != GameRooms::Arena
            || health.current == 0
            || cooldown.remaining_seconds > 0.0
        {
            continue;
        }

        let touching_enemy = enemies.iter().any(|(enemy_position, enemy_health)| {
            if enemy_health.current == 0 {
                return false;
            }

            let collision_radius = PLAYER_HALF_SIZE + ENEMY_COLLISION_RADIUS;
            player_position.0.distance_squared(enemy_position.0)
                <= collision_radius * collision_radius
        });

        if !touching_enemy {
            continue;
        }

        health.current = health.current.saturating_sub(ENEMY_CONTACT_DAMAGE);
        cooldown.remaining_seconds = ENEMY_CONTACT_DAMAGE_INTERVAL_SECONDS;

        info!(
            ?player_entity,
            damage = ENEMY_CONTACT_DAMAGE,
            current_health = health.current,
            maximum_health = health.maximum,
            "Player took enemy contact damage"
        );
    }
}

pub(crate) fn debug_switch_rooms(
    mut player_query: Query<(&ActionState<PlayerAction>, &mut GameRoom)>,
) {
    for (actions, mut room) in &mut player_query {
        if actions.just_pressed(&PlayerAction::DebugSwitchRooms) {
            room.room = match room.room {
                GameRooms::Arena => GameRooms::Safezone,
                GameRooms::Safezone => GameRooms::Arena,
            }
        }
    }
}
