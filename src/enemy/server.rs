use bevy::prelude::*;
use lightyear::prelude::*;

use crate::enemy::protocol::{EnemyHealth, EnemyPosition};

/// Temporary host/server debug spawner. Press N to create a target.
pub(crate) fn spawn_enemy(input: Option<Res<ButtonInput<KeyCode>>>, mut commands: Commands) {
    if input.is_some_and(|input| input.just_pressed(KeyCode::KeyN)) {
        let entity = commands
            .spawn((
                EnemyPosition(Vec2::new(220.0, 0.0)),
                EnemyHealth::new(100),
                Replicate::to_clients(NetworkTarget::All),
                InterpolationTarget::to_clients(NetworkTarget::All),
                Name::new("Enemy"),
            ))
            .id();

        info!(?entity, "Created debug enemy");
    }
}
