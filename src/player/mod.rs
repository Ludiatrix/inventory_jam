#[cfg(feature = "client")]
mod client;
mod protocol;
#[cfg(feature = "gui")]
mod render;
#[cfg(feature = "server")]
mod server;

#[cfg(feature = "client")]
pub use client::PlayerClientPlugin;
pub use protocol::PlayerProtocolPlugin;
#[cfg(feature = "gui")]
pub use render::PlayerRenderPlugin;
#[cfg(feature = "server")]
pub use server::PlayerServerPlugin;
