//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.plugin.bb2025.BlockLogicExtensionPlugin`.
//! java: `@RulesCollection(RulesCollection.Rules.BB2025)`

use std::collections::HashSet;

use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::player::Player;
use ffb_model::model::player_state::PlayerState;

use crate::client::net::client_communication::ClientCommunication;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::ActionContext;
use crate::client::state::logic::plugin::block_logic_extension_plugin::BlockLogicExtensionPlugin as BlockLogicExtensionPluginTrait;
use crate::client::state::logic::plugin::logic_plugin::{LogicPlugin, LogicPluginType};

/// 1:1 translation of `bb2025.BlockLogicExtensionPlugin`.
pub struct BlockLogicExtensionPlugin;

impl LogicPlugin for BlockLogicExtensionPlugin {
    fn get_type(&self) -> LogicPluginType {
        LogicPluginType::BLOCK
    }
}

impl BlockLogicExtensionPluginTrait for BlockLogicExtensionPlugin {
    fn available_actions(&self) -> HashSet<ClientAction> {
        HashSet::from([ClientAction::CHOMP])
    }

    fn perform_available_action(
        &self,
        action: ClientAction,
        acting_player: &ActingPlayer,
        logic_module: &mut (),
        communication: &mut ClientCommunication,
        defender: &Player,
    ) {
        // DOCUMENTED GAP: java calls `logicModule.isChompAvailable(actingPlayer.getPlayer(),
        // defender)` then, if available, `logicModule.block(actingPlayer.getPlayerId(),
        // defender, ..., true)`. `BlockLogicExtension` isn't wired into the crate yet (see
        // `block_logic_extension_plugin.rs`'s module doc), so neither call has a translation
        // target today.
        let _ = (acting_player, logic_module, communication, defender);
        if let ClientAction::CHOMP = action {
            // no-op until BlockLogicExtension lands.
        }
    }

    fn action_context(
        &self,
        _acting_player: &ActingPlayer,
        action_context: ActionContext,
        _logic_module: &(),
    ) -> ActionContext {
        // java: `return actionContext;`
        action_context
    }

    fn player_can_not_move(&self, player_state: PlayerState) -> bool {
        player_state.is_pinned()
    }

    fn block_action_context(
        &self,
        _acting_player: &ActingPlayer,
        _multi_block: bool,
        action_context: ActionContext,
        _logic_module: &(),
    ) -> ActionContext {
        // DOCUMENTED GAP: java adds `ClientAction.CHOMP` when
        // `logicModule.isChompAvailable(actingPlayer.getPlayer())` — not translatable until
        // `BlockLogicExtension` lands (see module doc comment).
        action_context
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::PS_STANDING;

    #[test]
    fn get_type_and_name_are_block() {
        let plugin = BlockLogicExtensionPlugin;
        assert_eq!(plugin.get_type(), LogicPluginType::BLOCK);
        assert_eq!(plugin.get_name(), "BLOCK");
    }

    #[test]
    fn available_actions_is_chomp_only() {
        let plugin = BlockLogicExtensionPlugin;
        assert_eq!(plugin.available_actions(), HashSet::from([ClientAction::CHOMP]));
    }

    #[test]
    fn player_can_not_move_when_pinned() {
        let plugin = BlockLogicExtensionPlugin;
        let chomped = PlayerState::new(PS_STANDING).change_chomped(true);
        assert!(plugin.player_can_not_move(chomped));
    }

    #[test]
    fn player_can_move_when_not_pinned() {
        let plugin = BlockLogicExtensionPlugin;
        let standing = PlayerState::new(PS_STANDING);
        assert!(!plugin.player_can_not_move(standing));
    }
}
