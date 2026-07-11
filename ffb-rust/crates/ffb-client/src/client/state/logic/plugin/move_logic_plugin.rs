//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.plugin.MoveLogicPlugin`.
//!
//! Java's abstract class implements `LogicPlugin` (no `getType()` override —
//! each concrete subclass provides its own) and declares three abstract
//! methods taking the client's `MoveLogicModule` logic module.
//!
//! DOCUMENTED GAP: as with `BlockLogicExtensionPlugin`, `MoveLogicModule`
//! (`crate::client::state::logic::MoveLogicModule`) is still an unwired
//! placeholder pending a later batch, so this trait is generic over the
//! logic-module type (`LM`, defaulting to `()`); concrete plugin impls below
//! use the default and document per call site which Java calls can't yet be
//! translated.

use std::collections::HashSet;

use ffb_model::model::acting_player::ActingPlayer;

use crate::client::net::client_communication::ClientCommunication;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::ActionContext;

use super::logic_plugin::LogicPlugin;

/// 1:1 translation of the `MoveLogicPlugin` abstract class.
pub trait MoveLogicPlugin<LM = ()>: LogicPlugin {
    fn available_actions(&self) -> HashSet<ClientAction>;

    fn perform_available_action(
        &self,
        action: ClientAction,
        acting_player: &ActingPlayer,
        logic_module: &mut LM,
        communication: &mut ClientCommunication,
    );

    fn action_context(
        &self,
        acting_player: &ActingPlayer,
        action_context: ActionContext,
        logic_module: &LM,
    ) -> ActionContext;
}
