use bevy::app::{App, Plugin};
use bevy::math::Vec2;
use bevy::prelude::Component;

pub struct PlayerProtocolPlugin;

impl Plugin for PlayerProtocolPlugin {
    fn build(&self, _app: &mut App) {}
}

#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct CachedCursorAim(pub Vec2);

impl Default for CachedCursorAim {
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
