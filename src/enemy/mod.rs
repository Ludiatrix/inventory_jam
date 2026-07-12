#[cfg(feature = "client")]
use bevy::app::{App, Plugin};

#[cfg(feature = "client")]
pub(crate) mod client;

pub mod protocol;

#[cfg(feature = "gui")]
pub(crate) mod render;

#[cfg(feature = "server")]
pub(crate) mod server;

pub mod shared;

/// Installs only client-side fragment presentation behavior.
#[cfg(feature = "client")]
pub struct EnemyClientPlugin;

#[cfg(feature = "client")]
impl Plugin for EnemyClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(client::initialize_enemy);
        // app.add_systems(FixedUpdate, client::simulate_client_fragments);
    }
}