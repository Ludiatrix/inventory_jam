use bevy::prelude::*;

#[derive(Message, Clone, Copy, Debug)]
pub struct AddGlobalAristeia(pub u32);

#[derive(Message, Clone, Copy, Debug, Default)]
pub struct GrandChampionDefeated;
