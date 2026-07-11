//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.plugin.bb2025.BaseLogicPlugin`.
//! java: `@RulesCollection(RulesCollection.Rules.BB2025)`

use ffb_model::model::player_state::PlayerState;

use crate::client::state::logic::plugin::base_logic_plugin::BaseLogicPlugin as BaseLogicPluginTrait;
use crate::client::state::logic::plugin::logic_plugin::{LogicPlugin, LogicPluginType};

/// 1:1 translation of `bb2025.BaseLogicPlugin`.
pub struct BaseLogicPlugin;

impl LogicPlugin for BaseLogicPlugin {
    fn get_type(&self) -> LogicPluginType {
        LogicPluginType::BASE
    }
}

impl BaseLogicPluginTrait for BaseLogicPlugin {
    fn player_can_not_move(&self, player_state: PlayerState) -> bool {
        player_state.is_pinned()
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
    fn player_can_move_when_not_pinned() {
        let plugin = BaseLogicPlugin;
        let standing = PlayerState::new(PS_STANDING);
        assert!(!plugin.player_can_not_move(standing));
    }
}
