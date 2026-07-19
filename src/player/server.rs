use crate::app::ServerState;
use crate::player::protocol::{PlayerAimDirection, PlayerBundle};
use crate::player::shared::player_movement;
use crate::protocol::inputs::PlayerAction;
use crate::protocol::rooms::{GameRoom, GameRooms};
use crate::shared::FixedGameplaySet;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use lightyear::connection::client::Connected;
use lightyear::connection::client_of::ClientOf;
use lightyear::connection::host::HostServer;
use lightyear::prelude::*;

pub struct PlayerServerPlugin;

impl Plugin for PlayerServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_connected);

        app.add_systems(
            FixedUpdate,
            (player_movement, update_player_aim_direction)
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
    mut commands: Commands,
) {
    let Ok(client_id) = query.get(trigger.entity) else {
        return;
    };

    let client_id = client_id.0;

    let entity = commands
        .spawn((
            Name::new(format!("Player: {}", client_id)),
            PlayerBundle::new(client_id, Vec2::ZERO),
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

    info!("Create player entity {entity:?} for client {client_id:?}");
}

pub(crate) fn update_player_aim_direction(
    mut players: Query<(
        &ActionState<PlayerAction>,
        &mut PlayerAimDirection,
        Has<Predicted>,
    )>,
    host_server: Query<(), With<HostServer>>,
) {
    let is_host_server = !host_server.is_empty();

    for (actions, mut aim_direction, predicted) in &mut players {
        if is_host_server && predicted {
            continue;
        }

        let aim = actions.clamped_axis_pair(&PlayerAction::Aim);

        if aim.length_squared() > 0.0001 {
            aim_direction.0 = aim.normalize_or_zero();
        }
    }
}

pub(crate) fn debug_switch_rooms(
    mut player_query: Query<(&ActionState<PlayerAction>, &mut GameRoom)>,
) {
    for (actions, mut room) in &mut player_query {
        if actions.just_pressed(&PlayerAction::DebugSwitchRooms) {
            room.room = match room.room {
                GameRooms::Arena => GameRooms::Pit,
                GameRooms::Pit => GameRooms::Arena,
            }
        }
    }
}
