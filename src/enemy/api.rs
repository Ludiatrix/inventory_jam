use bevy::prelude::*;

#[derive(Message, Clone, Copy, Debug)]
pub(crate) struct SpawnEnemy {
    pub position: Vec2,
}

impl SpawnEnemy {
    pub const fn new(position: Vec2) -> Self {
        Self {
            position,
        }
    }
}