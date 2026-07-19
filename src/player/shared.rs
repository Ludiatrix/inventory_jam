use bevy::{
    ecs::{query::With, system::Query},
    math::Vec2,
};
use leafwing_input_manager::action_state::ActionState;
use lightyear::prediction::Predicted;

use crate::{
    player::PlayerPosition,
    protocol::{inputs::PlayerAction, rooms::GameRoom},
};

pub const PLAYER_HALF_SIZE: f32 = 25.0;
pub const PLAYER_COLLISION_RADIUS: f32 = PLAYER_HALF_SIZE;
const MOVE_SPEED: f32 = 10.0;

pub fn player_movement(
    mut player_query: Query<
        (&mut PlayerPosition, &GameRoom, &ActionState<PlayerAction>),
        With<Predicted>,
    >,
) {
    for (mut position, room, actions) in player_query.iter_mut() {
        let movement = actions.clamped_axis_pair(&PlayerAction::Move);

        if movement != Vec2::ZERO {
            position.0 += movement * MOVE_SPEED;

            let local_bounds = room.bounds().inflate(-PLAYER_HALF_SIZE);

            position.0 = position.0.clamp(local_bounds.min, local_bounds.max);
        }
    }
}
