//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.KickoffLogicModule` (102 lines).
//!
//! Java's `KickoffLogicModule` lets the kicking coach place the ball anywhere in the receiving
//! half before ending the turn to actually kick off. `fieldInteraction`/`playerInteraction`/
//! `fieldPeek`/`endTurn` need `FantasyFootballClient` access, so — matching the established
//! `MoveLogicModule` convention — the interaction methods are translated as inherent methods
//! taking `client` explicitly rather than trait overrides (see `logic_module.rs`'s module doc
//! on the narrower trait-default signature). `endTurn` itself keeps the `LogicModule` trait
//! signature since that default method already takes `client`.

use std::collections::HashSet;

use ffb_model::enums::ClientStateId;
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds};

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::LogicModule;

/// 1:1 translation of the `KickoffLogicModule` class.
#[derive(Debug, Default)]
pub struct KickoffLogicModule {
    /// java: `fKicked`.
    kicked: bool,
}

impl KickoffLogicModule {
    /// java: `public KickoffLogicModule(FantasyFootballClient pClient)`.
    pub fn new() -> Self {
        Self { kicked: false }
    }

    /// java: `public InteractionResult fieldInteraction(FieldCoordinate pCoordinate)` — see
    /// module doc regarding the trait-default signature.
    pub fn field_interaction(
        &self,
        client: &mut FantasyFootballClient,
        coordinate: FieldCoordinate,
    ) -> InteractionResult {
        if !self.kicked {
            self.place_ball(client, Some(coordinate));
            InteractionResult::handled()
        } else {
            InteractionResult::ignore()
        }
    }

    /// java: `public InteractionResult playerInteraction(Player<?> pPlayer)` — see module doc
    /// regarding the trait-default signature.
    pub fn player_interaction(
        &self,
        client: &mut FantasyFootballClient,
        player: &Player,
    ) -> InteractionResult {
        if !self.kicked {
            let player_coordinate = client.game().and_then(|game| game.field_model.player_coordinate(&player.id));
            self.place_ball(client, player_coordinate);
            InteractionResult::handled()
        } else {
            InteractionResult::ignore()
        }
    }

    /// java: `private void placeBall(FieldCoordinate pCoordinate)`.
    fn place_ball(&self, client: &mut FantasyFootballClient, coordinate: Option<FieldCoordinate>) {
        if let Some(coordinate) = coordinate {
            if FieldCoordinateBounds::HALF_AWAY.is_in_bounds(coordinate) {
                if let Some(game) = client.game_mut() {
                    game.field_model.ball_moving = true;
                    game.field_model.ball_coordinate = Some(coordinate);
                }
            }
        }
    }

    /// java: `public InteractionResult fieldPeek(FieldCoordinate pCoordinate)` — see module doc
    /// regarding the trait-default signature.
    pub fn field_peek(&self, coordinate: Option<FieldCoordinate>) -> InteractionResult {
        match coordinate {
            Some(coordinate) if !self.kicked && FieldCoordinateBounds::HALF_AWAY.is_in_bounds(coordinate) => {
                InteractionResult::perform()
            }
            _ => InteractionResult::reset(),
        }
    }

    /// java: `public boolean turnIsEnding()`.
    pub fn turn_is_ending(&self, client: &FantasyFootballClient) -> bool {
        match client.game().and_then(|game| game.field_model.ball_coordinate) {
            Some(ball_coordinate) => FieldCoordinateBounds::HALF_AWAY.is_in_bounds(ball_coordinate),
            None => false,
        }
    }
}

impl LogicModule for KickoffLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::Kickoff
    }

    /// java: `public void setUp() { super.setUp(); fKicked = false; }`.
    fn set_up(&mut self, _client: &mut FantasyFootballClient) {
        self.kicked = false;
    }

    /// java: `public void endTurn()`.
    fn end_turn(&mut self, client: &mut FantasyFootballClient) {
        let ball_coordinate = client.game().and_then(|game| game.field_model.ball_coordinate);
        if let Some(ball_coordinate) = ball_coordinate {
            if FieldCoordinateBounds::HALF_AWAY.is_in_bounds(ball_coordinate) {
                self.kicked = true;
                client.communication_mut().send_kickoff(ball_coordinate);
                client.client_data_mut().set_end_turn_button_hidden(true);
            }
        }
    }

    /// java: `public Set<ClientAction> availableActions()`.
    fn available_actions(&self) -> HashSet<ClientAction> {
        HashSet::new()
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action) {}`.
    fn perform_available_action(
        &mut self,
        _client: &mut FantasyFootballClient,
        _player: &Player,
        _action: ClientAction,
    ) {
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)` — always
    /// throws `UnsupportedOperationException` in Java; faithfully translated as a panic.
    fn action_context(&self, _game: &Game, _acting_player: &ActingPlayer) -> ActionContext {
        panic!("actionContext for acting player is not supported in kick off context")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;

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

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2025)
    }

    fn make_client() -> FantasyFootballClient {
        let params = crate::client::client_parameters::ClientParameters::create_valid_params(&[
            "-spectator".into(),
            "-coach".into(),
            "bob".into(),
        ])
        .unwrap();
        FantasyFootballClient::new(params)
    }

    #[test]
    fn get_id_is_kickoff() {
        assert_eq!(KickoffLogicModule::new().get_id(), ClientStateId::Kickoff);
    }

    #[test]
    fn field_peek_performs_when_in_away_half_and_not_kicked() {
        let module = KickoffLogicModule::new();
        assert_eq!(
            module.field_peek(Some(FieldCoordinate::new(20, 5))).get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Perform
        );
    }

    #[test]
    fn field_peek_resets_outside_away_half() {
        let module = KickoffLogicModule::new();
        assert_eq!(
            module.field_peek(Some(FieldCoordinate::new(1, 5))).get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Reset
        );
    }

    #[test]
    fn field_interaction_places_ball_in_bounds() {
        let mut module = KickoffLogicModule::new();
        let mut client = make_client();
        client.set_game(make_game());
        let result = module.field_interaction(&mut client, FieldCoordinate::new(20, 5));
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Handled
        );
        assert_eq!(client.game().unwrap().field_model.ball_coordinate, Some(FieldCoordinate::new(20, 5)));
        assert!(client.game().unwrap().field_model.ball_moving);
    }

    #[test]
    fn field_interaction_ignores_once_kicked() {
        let mut module = KickoffLogicModule::new();
        module.kicked = true;
        let mut client = make_client();
        client.set_game(make_game());
        let result = module.field_interaction(&mut client, FieldCoordinate::new(20, 5));
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn end_turn_marks_kicked_when_ball_in_away_half() {
        let mut module = KickoffLogicModule::new();
        let mut client = make_client();
        let mut game = make_game();
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(20, 5));
        client.set_game(game);
        module.end_turn(&mut client);
        assert!(module.kicked);
        assert!(client.client_data().is_end_turn_button_hidden());
    }

    #[test]
    fn end_turn_does_nothing_when_ball_not_in_away_half() {
        let mut module = KickoffLogicModule::new();
        let mut client = make_client();
        let mut game = make_game();
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(1, 5));
        client.set_game(game);
        module.end_turn(&mut client);
        assert!(!module.kicked);
    }

    #[test]
    fn turn_is_ending_reflects_ball_position() {
        let module = KickoffLogicModule::new();
        let mut client = make_client();
        let mut game = make_game();
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(20, 5));
        client.set_game(game);
        assert!(module.turn_is_ending(&client));
    }

    #[test]
    #[should_panic(expected = "actionContext for acting player is not supported in kick off context")]
    fn action_context_panics() {
        let module = KickoffLogicModule::new();
        let game = make_game();
        let ap = ActingPlayer::new();
        module.action_context(&game, &ap);
    }
}
