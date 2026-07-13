use crate::app::AppState;
use crate::player::protocol::{CachedCursorAim, SmoothedAimDirection};
use crate::protocol::inputs::PlayerAction;
use crate::protocol::player::{PlayerAimDirection, PlayerPosition};
use bevy::app::{App, Plugin, Update};
use bevy::camera::{Camera, Camera2d};
use bevy::color::Color;
use bevy::math::{Isometry2d, StableInterpolate, Vec2};
use bevy::prelude::{
    Gizmos, GlobalTransform, IntoScheduleConfigs, Query, Res, Single, Time, Transform, Window,
    With, in_state,
};
use bevy::window::PrimaryWindow;
use leafwing_input_manager::input_map::InputMap;
use lightyear::prediction::Predicted;
use lightyear::prelude::Controlled;

/// How quickly should the camera snap to the desired location
const CAMERA_DECAY_RATE: f32 = 2.;

pub struct PlayerRenderPlugin;

impl Plugin for PlayerRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            draw_local_aimstick.run_if(in_state(AppState::Playing)),
        );
        app.add_systems(
            Update,
            (update_camera, sample_cursor_aim)
                .chain()
                .run_if(in_state(AppState::Playing)),
        );
        app.add_systems(
            Update,
            (smooth_local_aim_visual)
                .chain()
                .run_if(in_state(AppState::Playing)),
        );
    }
}

/// Client-only system that smoothly moves the camera to the center of the Player's position.
fn update_camera(
    mut camera: Single<&mut Transform, With<Camera2d>>,
    player: Single<&PlayerPosition, With<Predicted>>,
    time: Res<Time>,
) {
    let target = player.0.extend(camera.translation.z);

    camera
        .translation
        .smooth_nudge(&target, CAMERA_DECAY_RATE, time.delta_secs());
}

fn sample_cursor_aim(
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
    predicted_player: Single<&PlayerPosition, With<Predicted>>,
    mut input_entity: Single<
        &mut CachedCursorAim,
        (With<Controlled>, With<InputMap<PlayerAction>>),
    >,
) {
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let (camera, camera_transform) = camera.into_inner();

    let Ok(cursor_world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position)
    else {
        return;
    };

    let direction = (cursor_world_position - predicted_player.0).normalize_or_zero();

    if direction == Vec2::ZERO {
        return;
    }

    input_entity.0 = direction;
}

fn draw_local_aimstick(
    mut gizmos: Gizmos,
    players: Query<(&PlayerPosition, &SmoothedAimDirection), With<Predicted>>,
) {
    const AIM_STICK_LENGTH: f32 = 32.0;

    for (position, smoothed_aim) in &players {
        let direction = smoothed_aim.0.normalize_or_zero();

        if direction == Vec2::ZERO {
            continue;
        }

        let start = position.0;
        let end = start + direction * AIM_STICK_LENGTH;

        gizmos.line_2d(start, end, Color::srgb(1.0, 0.85, 0.2));
        gizmos.circle_2d(
            Isometry2d::from_translation(end),
            5.0,
            Color::srgb(1.0, 0.85, 0.2),
        );
    }
}

fn smooth_local_aim_visual(
    time: Res<Time>,
    mut players: Query<(&PlayerAimDirection, &mut SmoothedAimDirection), With<Predicted>>,
) {
    const AIM_VISUAL_SMOOTH_RATE: f32 = 35.0;

    let t = 1.0 - (-AIM_VISUAL_SMOOTH_RATE * time.delta_secs()).exp();

    for (target, mut smoothed) in &mut players {
        let target_direction = target.0.normalize_or_zero();

        if target_direction == Vec2::ZERO {
            continue;
        }

        smoothed.0 = smoothed.0.lerp(target_direction, t).normalize_or_zero();
    }
}
