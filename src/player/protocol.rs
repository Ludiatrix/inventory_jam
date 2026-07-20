#[cfg(feature = "server")]
use crate::protocol::rooms::GameRooms;
use crate::protocol::{inputs::PlayerAction, rooms::GameRoom};
use bevy::math::Curve;
use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionState;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use crate::app::temporary_username;

pub struct PlayerProtocolPlugin;

impl Plugin for PlayerProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.component::<PlayerId>().replicate();

        app.add_systems(PostUpdate, debug_player_position);
        app.component::<PlayerPosition>()
            .replicate()
            .predict()
            .add_linear_interpolation();

        app.component::<PlayerAimDirection>().replicate().predict();

        app.component::<PlayerColor>().replicate();

        app.component::<GameRoom>().replicate();

        app.component::<PlayerUsername>().replicate();

        app.component::<PlayerHealth>().replicate().predict();
    }
}

#[allow(unused)]
fn debug_player_position(
    q: Query<(Entity, &PlayerPosition, &ActionState<PlayerAction>)>,
    local_timeline: Res<LocalTimeline>,
) {
    for (entity, position, actions) in q.iter() {
        let movement = actions.clamped_axis_pair(&PlayerAction::Move);

        if movement != Vec2::ZERO {
            // info!(
            //     "Player Pos: {:?} {:?} {:?}",
            //     local_timeline.tick(),
            //     position,
            //     entity
            // );
        }
    }
}

#[cfg(feature = "server")]
#[derive(Bundle)]
pub(crate) struct PlayerBundle {
    id: PlayerId,
    position: PlayerPosition,
    color: PlayerColor,
    aim_direction: PlayerAimDirection,
    game_room: GameRoom,
    username: PlayerUsername,
    health: PlayerHealth,
}

#[cfg(feature = "server")]
impl PlayerBundle {
    pub(crate) fn new(id: PeerId, position: Vec2) -> Self {
        let h = (((id.to_bits().wrapping_mul(30)) % 360) as f32) / 360.0;
        let color = Color::hsl(h, 0.8, 0.5);

        Self {
            id: PlayerId(id),
            position: PlayerPosition(position),
            color: PlayerColor(color),
            aim_direction: PlayerAimDirection::default(),
            game_room: GameRoom {
                room: GameRooms::Arena,
            },
            username: PlayerUsername(temporary_username(id.to_bits())),
            health: PlayerHealth::new(100),
        }
    }
}

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerId(pub PeerId);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect, Deref, DerefMut)]
pub struct PlayerPosition(pub Vec2);

impl Ease for PlayerPosition {
    fn interpolating_curve_unbounded(start: Self, end: Self) -> impl Curve<Self> {
        FunctionCurve::new(Interval::UNIT, move |t| {
            PlayerPosition(Vec2::lerp(start.0, end.0, t))
        })
    }
}

#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Reflect)]
pub struct PlayerAimDirection(pub Vec2);

impl Default for PlayerAimDirection {
    fn default() -> Self {
        let default_direction = Vec2 { x: 5.0, y: 4.0 };
        Self(default_direction) // Defaults to [5,4] for debug since it's a weird direction
    }
}

#[derive(Component, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct PlayerColor(pub(crate) Color);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Deref)]
pub struct PlayerUsername(pub String);

#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerHealth {
    pub current: u32,
    pub maximum: u32,
}

impl PlayerHealth {
    pub const fn new(maximum: u32) -> Self {
        Self {
            current: maximum,
            maximum,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub(crate) struct PlayerVisual;

#[derive(Resource, Clone, Copy, Debug)]
pub(crate) struct LocalAimInput(pub Vec2);

impl Default for LocalAimInput {
    fn default() -> Self {
        Self(Vec2::X)
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct SmoothedAimDirection(pub Vec2);

impl Default for SmoothedAimDirection {
    fn default() -> Self {
        Self(Vec2::X)
    }
}
