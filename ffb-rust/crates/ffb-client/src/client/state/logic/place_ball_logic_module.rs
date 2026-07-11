//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.PlaceBallLogicModule`.
//!
//! Handles the `PLACE_BALL` client state: the only interaction accepted is a field click on a
//! square that is a valid "move square" (i.e. a legal kick-off/touchback ball placement target),
//! which sends the chosen `FieldCoordinate` to the server. No `ClientAction`s are ever available
//! and `action_context` is unreachable for this module (mirrors Java's
//! `UnsupportedOperationException`).

use std::collections::HashSet;

use ffb_model::enums::ClientStateId;
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::types::FieldCoordinate;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::LogicModule;

/// Java: `PlaceBallLogicModule`.
#[derive(Debug, Default)]
pub struct PlaceBallLogicModule;

impl PlaceBallLogicModule {
    /// Java: `public PlaceBallLogicModule(FantasyFootballClient pClient)`. The `client` field is
    /// no longer stored on the struct (see `logic_module.rs` module doc); construction is
    /// trivial.
    pub fn new() -> Self {
        Self
    }

    /// java: `public InteractionResult fieldInteraction(FieldCoordinate pCoordinate)`
    pub fn field_interaction(
        &self,
        client: &mut FantasyFootballClient,
        coordinate: FieldCoordinate,
    ) -> InteractionResult {
        let has_move_square = client
            .game()
            .map(|game| game.field_model.get_move_square(coordinate).is_some())
            .unwrap_or(false);
        if has_move_square {
            client.communication_mut().send_field_coordinate(coordinate);
            return InteractionResult::handled();
        }
        InteractionResult::ignore()
    }

    /// java: `public InteractionResult fieldPeek(FieldCoordinate pCoordinate)`
    pub fn field_peek(&self, client: &FantasyFootballClient, coordinate: FieldCoordinate) -> InteractionResult {
        let has_move_square = client
            .game()
            .map(|game| game.field_model.get_move_square(coordinate).is_some())
            .unwrap_or(false);
        if has_move_square {
            InteractionResult::perform()
        } else {
            InteractionResult::reset()
        }
    }
}

impl LogicModule for PlaceBallLogicModule {
    /// java: `public ClientStateId getId()`
    fn get_id(&self) -> ClientStateId {
        ClientStateId::PlaceBall
    }

    /// java: `public Set<ClientAction> availableActions()`
    fn available_actions(&self) -> HashSet<ClientAction> {
        HashSet::new()
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)` — always throws
    /// `UnsupportedOperationException` in Java; there is no safe Rust return value, so this
    /// panics to match (documented, intentional, per the batch plan for these five modules).
    fn action_context(&self, _game: &Game, _acting_player: &ActingPlayer) -> ActionContext {
        panic!("actionContext for acting player is not supported in place ball context");
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)` —
    /// empty body in Java.
    fn perform_available_action(
        &mut self,
        _client: &mut FantasyFootballClient,
        _player: &Player,
        _action: ClientAction,
    ) {
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::client_parameters::ClientParameters;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;
    use ffb_model::types::MoveSquare;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.to_string(),
            name: id.to_string(),
            race: "human".into(),
            roster_id: "human".into(),
            coach: "coach".into(),
            rerolls: 0,
            apothecaries: 0,
            bribes: 0,
            master_chefs: 0,
            prayers_to_nuffle: 0,
            bloodweiser_kegs: 0,
            riotous_rookies: 0,
            cheerleaders: 0,
            assistant_coaches: 0,
            fan_factor: 0,
            dedicated_fans: 0,
            team_value: 0,
            treasury: 0,
            special_rules: Vec::new(),
            players: Vec::new(),
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_client() -> FantasyFootballClient {
        let params =
            ClientParameters::create_valid_params(&["-spectator".into(), "-coach".into(), "bob".into()]).unwrap();
        let mut client = FantasyFootballClient::new(params);
        client.set_game(Game::new(make_team("home"), make_team("away"), Rules::Bb2025));
        client
    }

    #[test]
    fn get_id_is_place_ball() {
        let module = PlaceBallLogicModule::new();
        assert_eq!(module.get_id(), ClientStateId::PlaceBall);
    }

    #[test]
    fn available_actions_is_always_empty() {
        let module = PlaceBallLogicModule::new();
        assert!(module.available_actions().is_empty());
    }

    #[test]
    fn field_interaction_ignores_non_move_square() {
        let module = PlaceBallLogicModule::new();
        let mut client = make_client();
        let result = module.field_interaction(&mut client, FieldCoordinate::new(1, 1));
        assert_eq!(result.get_kind(), crate::client::state::logic::interaction::interaction_result::Kind::Ignore);
    }

    #[test]
    fn field_interaction_handles_move_square_and_sends_coordinate() {
        let module = PlaceBallLogicModule::new();
        let mut client = make_client();
        let coord = FieldCoordinate::new(3, 4);
        client.game_mut().unwrap().field_model.add_move_square(MoveSquare::new(coord, 1, 0));
        let result = module.field_interaction(&mut client, coord);
        assert_eq!(result.get_kind(), crate::client::state::logic::interaction::interaction_result::Kind::Handled);
    }

    #[test]
    fn field_peek_returns_perform_for_move_square_and_reset_otherwise() {
        let module = PlaceBallLogicModule::new();
        let mut client = make_client();
        let coord = FieldCoordinate::new(5, 5);
        assert_eq!(
            module.field_peek(&client, coord).get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Reset
        );
        client.game_mut().unwrap().field_model.add_move_square(MoveSquare::new(coord, 1, 0));
        assert_eq!(
            module.field_peek(&client, coord).get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Perform
        );
    }

    #[test]
    #[should_panic(expected = "actionContext for acting player is not supported in place ball context")]
    fn action_context_panics() {
        let module = PlaceBallLogicModule::new();
        let client = make_client();
        let game = client.game().unwrap();
        let ap = ActingPlayer::new();
        module.action_context(game, &ap);
    }
}
