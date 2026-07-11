//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.plugin.mixed.MoveLogicPlugin`.
//! java: `@RulesCollection(RulesCollection.Rules.BB2020)` `@RulesCollection(RulesCollection.Rules.BB2016)`

use std::collections::HashSet;

use ffb_model::model::acting_player::ActingPlayer;

use crate::client::net::client_communication::ClientCommunication;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::ActionContext;
use crate::client::state::logic::plugin::logic_plugin::{LogicPlugin, LogicPluginType};
use crate::client::state::logic::plugin::move_logic_plugin::MoveLogicPlugin as MoveLogicPluginTrait;

/// 1:1 translation of `mixed.MoveLogicPlugin`.
pub struct MoveLogicPlugin;

impl LogicPlugin for MoveLogicPlugin {
    fn get_type(&self) -> LogicPluginType {
        LogicPluginType::MOVE
    }
}

impl MoveLogicPluginTrait for MoveLogicPlugin {
    fn available_actions(&self) -> HashSet<ClientAction> {
        HashSet::from([ClientAction::THEN_I_STARTED_BLASTIN])
    }

    fn perform_available_action(
        &self,
        action: ClientAction,
        acting_player: &ActingPlayer,
        logic_module: &mut (),
        communication: &mut ClientCommunication,
    ) {
        // DOCUMENTED GAP: java calls `logicModule.isThenIStartedBlastinAvailable(actingPlayer)`
        // then resolves `actingPlayer.getPlayer().getSkillWithProperty(canBlastRemotePlayer)`
        // before `communication.sendUseSkill(...)`. `MoveLogicModule` isn't wired into the
        // crate yet (see `move_logic_plugin.rs`'s module doc), and `ActingPlayer` here only
        // stores the player id, not the resolved `Player`/skill Java relies on, so none of
        // this is translatable today.
        let _ = (acting_player, logic_module, communication);
        if let ClientAction::THEN_I_STARTED_BLASTIN = action {
            // no-op until MoveLogicModule lands.
        }
    }

    fn action_context(
        &self,
        acting_player: &ActingPlayer,
        action_context: ActionContext,
        logic_module: &(),
    ) -> ActionContext {
        // DOCUMENTED GAP: java adds `ClientAction.THEN_I_STARTED_BLASTIN` when
        // `logicModule.isThenIStartedBlastinAvailable(actingPlayer)` — not translatable until
        // `MoveLogicModule` lands (see module doc comment).
        let _ = (acting_player, logic_module);
        action_context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_type_and_name_are_move() {
        let plugin = MoveLogicPlugin;
        assert_eq!(plugin.get_type(), LogicPluginType::MOVE);
        assert_eq!(plugin.get_name(), "MOVE");
    }

    #[test]
    fn available_actions_is_then_i_started_blastin_only() {
        let plugin = MoveLogicPlugin;
        assert_eq!(
            plugin.available_actions(),
            HashSet::from([ClientAction::THEN_I_STARTED_BLASTIN])
        );
    }
}
