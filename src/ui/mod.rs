#[cfg(feature = "gui")]
mod help_text;
#[cfg(feature = "gui")]
mod menu;

use bevy::app::{App, Plugin};
use help_text::HelpTextPlugin;
use menu::MenuPlugin;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "gui")]
        app.add_plugins(MenuPlugin);
        #[cfg(feature = "gui")]
        app.add_plugins(HelpTextPlugin);
    }
}
