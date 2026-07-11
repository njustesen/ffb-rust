//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.QuickSnapLogicModule` (29 lines).
//!
//! Extends `SetupLogicModule` (delegated to via composition — see `setup_logic_module.rs`),
//! overriding `useTurnMode()` to `true` and adding two pure square-adjacency helpers.

use std::collections::HashSet;

use ffb_model::enums::ClientStateId;
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::types::FieldCoordinate;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::logic_module::LogicModule;
use crate::client::state::logic::setup_logic_module::SetupLogicModule;

/// 1:1 translation of the `QuickSnapLogicModule` class.
#[derive(Debug, Default)]
pub struct QuickSnapLogicModule {
    setup: SetupLogicModule,
}

impl QuickSnapLogicModule {
    /// java: `public QuickSnapLogicModule(FantasyFootballClient pClient)`.
    pub fn new() -> Self {
        Self {
            setup: SetupLogicModule::new(),
        }
    }

    /// java: `public boolean squareIsOnPitch(FieldCoordinate pCoordinate)`.
    pub fn square_is_on_pitch(&self, coordinate: Option<FieldCoordinate>) -> bool {
        match coordinate {
            Some(c) => !c.is_box_coordinate(),
            None => false,
        }
    }

    /// java: `public boolean squaresAreSameOrAdjacent(FieldCoordinate start, FieldCoordinate end)`.
    pub fn squares_are_same_or_adjacent(&self, start: Option<FieldCoordinate>, end: Option<FieldCoordinate>) -> bool {
        match (start, end) {
            (Some(s), Some(e)) => s == e || s.is_adjacent(e),
            _ => false,
        }
    }

    /// java: `protected boolean useTurnMode()`.
    pub fn use_turn_mode(&self) -> bool {
        true
    }
}

impl LogicModule for QuickSnapLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::QuickSnap
    }

    /// java: `public void setUp()` — delegates to `SetupLogicModule`.
    fn set_up(&mut self, client: &mut FantasyFootballClient) {
        self.setup.set_up(client);
    }

    /// java: `public Set<ClientAction> availableActions()` — delegates to `SetupLogicModule`.
    fn available_actions(&self) -> HashSet<ClientAction> {
        self.setup.available_actions()
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action) {}` —
    /// delegates to `SetupLogicModule`.
    fn perform_available_action(&mut self, client: &mut FantasyFootballClient, player: &Player, action: ClientAction) {
        self.setup.perform_available_action(client, player, action);
    }

    /// java: `public void endTurn()` — delegates to `SetupLogicModule`; note that unlike the
    /// base class, `useTurnMode()` returns `true` here, but `SetupLogicModule::end_turn` is a
    /// documented no-op gap in the base class regardless of `useTurnMode()`'s value (see
    /// `setup_logic_module.rs` module doc).
    fn end_turn(&mut self, client: &mut FantasyFootballClient) {
        self.setup.end_turn(client);
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)` — delegates to
    /// `SetupLogicModule`, which always panics.
    fn action_context(&self, game: &Game, acting_player: &ActingPlayer) -> ActionContext {
        self.setup.action_context(game, acting_player)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_id_is_quick_snap() {
        assert_eq!(QuickSnapLogicModule::new().get_id(), ClientStateId::QuickSnap);
    }

    #[test]
    fn use_turn_mode_is_true() {
        assert!(QuickSnapLogicModule::new().use_turn_mode());
    }

    #[test]
    fn square_is_on_pitch_false_for_box_coordinate() {
        let module = QuickSnapLogicModule::new();
        assert!(!module.square_is_on_pitch(None));
        assert!(module.square_is_on_pitch(Some(FieldCoordinate::new(5, 5))));
        assert!(!module.square_is_on_pitch(Some(FieldCoordinate::new(
            ffb_model::types::RSV_HOME_X,
            3
        ))));
    }

    #[test]
    fn squares_are_same_or_adjacent_checks_equality_and_adjacency() {
        let module = QuickSnapLogicModule::new();
        let a = FieldCoordinate::new(5, 5);
        let b = FieldCoordinate::new(6, 5);
        let far = FieldCoordinate::new(10, 10);
        assert!(module.squares_are_same_or_adjacent(Some(a), Some(a)));
        assert!(module.squares_are_same_or_adjacent(Some(a), Some(b)));
        assert!(!module.squares_are_same_or_adjacent(Some(a), Some(far)));
        assert!(!module.squares_are_same_or_adjacent(None, Some(a)));
    }

    #[test]
    fn available_actions_is_empty() {
        assert!(QuickSnapLogicModule::new().available_actions().is_empty());
    }
}
