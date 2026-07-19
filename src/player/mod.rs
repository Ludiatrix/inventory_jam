#[cfg(feature = "client")]
mod client;
pub mod protocol;
#[cfg(feature = "gui")]
mod render;
#[cfg(feature = "server")]
mod server;
pub mod shared;

use bevy::app::{App, Plugin};

pub use protocol::{PlayerAimDirection, PlayerColor, PlayerId, PlayerPosition, PlayerUsername};

#[cfg(feature = "server")]
pub use server::PlayerSpawnMode;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(protocol::PlayerProtocolPlugin);

        #[cfg(feature = "client")]
        app.add_plugins(client::PlayerClientPlugin);

        #[cfg(feature = "server")]
        app.add_plugins(server::PlayerServerPlugin);

        #[cfg(feature = "gui")]
        app.add_plugins(render::PlayerRenderPlugin);
    }
}
