//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.plugin.BaseLogicPlugin`.
//!
//! Java's `BaseLogicPlugin` is an abstract class implementing `LogicPlugin`: it
//! provides a concrete `getType()` returning `Type.BASE` and declares one
//! abstract method, `playerCanNotMove(PlayerState)`. Rust traits don't share a
//! default-method body across an unrelated supertrait the way a Java abstract
//! class shares `getType()` with its subclasses, so each concrete
//! implementation (`bb2025::BaseLogicPlugin`, `mixed::BaseLogicPlugin`) repeats
//! the small `LogicPlugin::get_type() -> LogicPluginType::BASE` body directly.

use ffb_model::model::player_state::PlayerState;

use super::logic_plugin::LogicPlugin;

/// 1:1 translation of the `BaseLogicPlugin` abstract class.
pub trait BaseLogicPlugin: LogicPlugin {
    fn player_can_not_move(&self, player_state: PlayerState) -> bool;
}
