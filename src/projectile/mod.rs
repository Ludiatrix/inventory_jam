#[cfg(feature = "client")]
mod client;
mod protocol;
#[cfg(feature = "gui")]
mod render;
#[cfg(feature = "server")]
mod server;
pub mod shared;

pub use protocol::PlayerProjectile;

#[cfg(feature = "client")]
pub use client::ProjectileClientPlugin;
pub use protocol::ProjectileProtocolPlugin;
#[cfg(feature = "gui")]
pub use render::ProjectileRenderPlugin;
#[cfg(feature = "server")]
pub use server::ProjectileServerPlugin;
