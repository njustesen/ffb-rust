//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.plugin.mixed.BaseLogicPlugin`.
//! java: `@RulesCollection(RulesCollection.Rules.BB2020)` `@RulesCollection(RulesCollection.Rules.BB2016)`

use ffb_model::model::player_state::PlayerState;

use crate::client::state::logic::plugin::base_logic_plugin::BaseLogicPlugin as BaseLogicPluginTrait;
use crate::client::state::logic::plugin::logic_plugin::{LogicPlugin, LogicPluginType};

/// 1:1 translation of `mixed.BaseLogicPlugin`.
pub struct BaseLogicPlugin;

impl LogicPlugin for BaseLogicPlugin {
    fn get_type(&self) -> LogicPluginType {
        LogicPluginType::BASE
    }
}

impl BaseLogicPluginTrait for BaseLogicPlugin {
    fn player_can_not_move(&self, player_state: PlayerState) -> bool {
        player_state.is_rooted()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::PS_STANDING;

    #[test]
    fn get_type_and_name_are_base() {
        let plugin = BaseLogicPlugin;
        assert_eq!(plugin.get_type(), LogicPluginType::BASE);
        assert_eq!(plugin.get_name(), "BASE");
    }

    #[test]
    fn player_can_not_move_when_rooted() {
        let plugin = BaseLogicPlugin;
        let rooted = PlayerState::new(PS_STANDING).change_rooted(true);
        assert!(plugin.player_can_not_move(rooted));
    }

    #[test]
    fn player_can_move_when_chomped_but_not_rooted() {
        // mixed edition checks `isRooted()` only (unlike bb2025's `isPinned()`), so a chomped
        // (but not rooted) player state should NOT report `player_can_not_move`.
        let plugin = BaseLogicPlugin;
        let chomped = PlayerState::new(PS_STANDING).change_chomped(true);
        assert!(!plugin.player_can_not_move(chomped));
    }
}
