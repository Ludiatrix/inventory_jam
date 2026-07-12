use bevy::prelude::*;
use lightyear::interpolation::Interpolated;

use crate::enemy::protocol::{EnemyPosition};

pub(crate) fn initialize_enemy(
    trigger: On<Add, Interpolated>,
    mut commands: Commands,
    enemies: Query<&EnemyPosition>,
) {
    let entity = trigger.entity;
    
    info!(
        "Incoming enemy entity {:?} ",
        entity
    );

    let Ok(enemy) = enemies.get(entity) else {
        return;
    };
    
    info!(
        "Incoming enemy pos {:?} ",
        enemy
    );
    
    commands.entity(entity).insert(EnemyPosition(enemy.0));
}