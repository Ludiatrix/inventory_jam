pub mod protocol;

#[cfg(feature = "client")]
pub mod client;

#[cfg(feature = "server")]
pub mod server;

pub mod enemy;
pub mod fragment;
pub mod projectile;
pub mod weapon;

#[cfg(feature = "gui")]
pub mod renderer;

pub mod shared;
