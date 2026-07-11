//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.plugin`.

pub mod base_logic_plugin;
pub mod bb2025;
pub mod block_logic_extension_plugin;
pub mod logic_plugin;
pub mod mixed;
pub mod move_logic_plugin;

pub use base_logic_plugin::BaseLogicPlugin;
pub use block_logic_extension_plugin::BlockLogicExtensionPlugin;
pub use logic_plugin::{LogicPlugin, LogicPluginType};
pub use move_logic_plugin::MoveLogicPlugin;
