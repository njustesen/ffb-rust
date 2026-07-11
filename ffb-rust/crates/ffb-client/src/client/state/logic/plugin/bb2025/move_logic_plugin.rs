//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.plugin.bb2025.MoveLogicPlugin`.
//! java: `@RulesCollection(RulesCollection.Rules.BB2025)`

use std::collections::HashSet;

use ffb_model::enums::PlayerAction;
use ffb_model::model::acting_player::ActingPlayer;

use crate::client::net::client_communication::ClientCommunication;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::ActionContext;
use crate::client::state::logic::plugin::logic_plugin::{LogicPlugin, LogicPluginType};
use crate::client::state::logic::plugin::move_logic_plugin::MoveLogicPlugin as MoveLogicPluginTrait;

/// 1:1 translation of `bb2025.MoveLogicPlugin`.
pub struct MoveLogicPlugin;

impl LogicPlugin for MoveLogicPlugin {
    fn get_type(&self) -> LogicPluginType {
        LogicPluginType::MOVE
    }
}

impl MoveLogicPluginTrait for MoveLogicPlugin {
    fn available_actions(&self) -> HashSet<ClientAction> {
        HashSet::from([ClientAction::INCORPOREAL])
    }

    fn perform_available_action(
        &self,
        action: ClientAction,
        acting_player: &ActingPlayer,
        logic_module: &mut (),
        communication: &mut ClientCommunication,
    ) {
        match action {
            ClientAction::INCORPOREAL => {
                // DOCUMENTED GAP: java calls `logicModule.isIncorporealAvailable(actingPlayer)`,
                // then resolves `actingPlayer.getPlayer().getSkillWithProperty(canAvoidDodging)`
                // and `.hasActiveEnhancement(skill)` before `communication.sendUseSkill(...)`.
                // `MoveLogicModule` isn't wired into the crate yet (see module doc comment) and
                // `ActingPlayer` here only stores the player id, not the resolved `Player`
                // object or skill-lookup helpers Java relies on, so none of this is
                // translatable today.
                let _ = (acting_player, logic_module, communication);
            }
            ClientAction::MOVE => {
                // java: `if (PlayerAction.PUNT == actingPlayer.getPlayerAction())`
                if acting_player.player_action == Some(PlayerAction::Punt) {
                    // DOCUMENTED GAP: java calls `communication.sendActingPlayer(
                    // actingPlayer.getPlayer(), PlayerAction.PUNT_MOVE, actingPlayer.isJumping())`;
                    // `ActingPlayer` only stores the player id, not the resolved `Player` object
                    // `sendActingPlayer` needs, so the send itself can't be translated yet.
                    let _ = communication;
                }
            }
            _ => {}
        }
    }

    fn action_context(
        &self,
        acting_player: &ActingPlayer,
        action_context: ActionContext,
        logic_module: &(),
    ) -> ActionContext {
        // DOCUMENTED GAP: java adds `ClientAction.INCORPOREAL` (and, conditionally,
        // `Influences.INCORPOREAL_ACTIVE`) when `logicModule.isIncorporealAvailable(actingPlayer)`
        // — not translatable until `MoveLogicModule` lands (see module doc comment).
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
    fn available_actions_is_incorporeal_only() {
        let plugin = MoveLogicPlugin;
        assert_eq!(
            plugin.available_actions(),
            HashSet::from([ClientAction::INCORPOREAL])
        );
    }

    #[test]
    fn perform_available_action_does_not_panic_for_move_and_incorporeal() {
        let plugin = MoveLogicPlugin;
        let mut acting_player = ActingPlayer::new();
        acting_player.player_action = Some(PlayerAction::Punt);
        let mut logic_module = ();
        let mut communication = ClientCommunication::new();

        plugin.perform_available_action(
            ClientAction::MOVE,
            &acting_player,
            &mut logic_module,
            &mut communication,
        );
        plugin.perform_available_action(
            ClientAction::INCORPOREAL,
            &acting_player,
            &mut logic_module,
            &mut communication,
        );
    }
}
