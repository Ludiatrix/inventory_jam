mod protocol;
#[cfg(feature = "gui")]
mod render;
#[cfg(feature = "server")]
mod server;
pub mod shared;

pub use protocol::{EnemyHealth, EnemyPosition};

pub use protocol::EnemyProtocolPlugin;
#[cfg(feature = "gui")]
pub use render::EnemyRenderPlugin;
#[cfg(feature = "server")]
pub use server::EnemyServerPlugin;
