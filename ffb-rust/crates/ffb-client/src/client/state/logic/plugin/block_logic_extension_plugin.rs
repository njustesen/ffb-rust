//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.plugin.BlockLogicExtensionPlugin`.
//!
//! Java's abstract class implements `LogicPlugin` (with a concrete `getType()`
//! returning `Type.BLOCK`) and declares four further abstract methods, each
//! taking the client's `BlockLogicExtension` logic module.
//!
//! DOCUMENTED GAP: `BlockLogicExtension` (`crate::client::state::logic::BlockLogicExtension`)
//! is still an unwired placeholder file (see `TRANSLATION_TRACKER.md`, batch
//! "client/state/logic root shared/base") — it isn't declared in
//! `logic/mod.rs` yet and exposes none of the real methods
//! (`isChompAvailable`, `block`, …) that Java's concrete plugins call. This
//! trait is generic over the logic-module type (`LM`, defaulting to `()`) so
//! it compiles today without prematurely wiring that unfinished module;
//! concrete plugin impls below use the default `()` and document, call site by
//! call site, which Java calls can't yet be translated.
//! Likewise, `getType()` (concrete in the Java abstract class) is repeated in
//! each concrete impl's `LogicPlugin` implementation rather than shared via a
//! default method here — see `base_logic_plugin.rs`'s note for why.

use std::collections::HashSet;

use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::player::Player;
use ffb_model::model::player_state::PlayerState;

use crate::client::net::client_communication::ClientCommunication;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::ActionContext;

use super::logic_plugin::LogicPlugin;

/// 1:1 translation of the `BlockLogicExtensionPlugin` abstract class.
pub trait BlockLogicExtensionPlugin<LM = ()>: LogicPlugin {
    fn available_actions(&self) -> HashSet<ClientAction>;

    fn perform_available_action(
        &self,
        action: ClientAction,
        acting_player: &ActingPlayer,
        logic_module: &mut LM,
        communication: &mut ClientCommunication,
        defender: &Player,
    );

    fn action_context(
        &self,
        acting_player: &ActingPlayer,
        action_context: ActionContext,
        logic_module: &LM,
    ) -> ActionContext;

    fn player_can_not_move(&self, player_state: PlayerState) -> bool;

    fn block_action_context(
        &self,
        acting_player: &ActingPlayer,
        multi_block: bool,
        action_context: ActionContext,
        logic_module: &LM,
    ) -> ActionContext;
}
