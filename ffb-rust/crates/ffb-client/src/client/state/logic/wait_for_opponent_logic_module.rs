//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.WaitForOpponentLogicModule` (61 lines).
//!
//! Extends `LogicModule` directly (no plugin dependency). No actions are available and
//! `actionContext` always throws (there is no acting player while waiting).
//!
//! Documented gap:
//! - `getPlayer(FieldCoordinate coordinate)`: Java calls `getFieldModel().getPlayers(coordinate)`
//!   (multi-occupancy — a square can list several players) and, when there's more than one,
//!   prefers a home-team player, else the last player in the list. The Rust `FieldModel` is
//!   1:1 coordinate->player (`player_at`), so there is never more than a single candidate; the
//!   override degenerates to the single-player lookup, which matches the "exactly one player"
//!   and "no players" branches faithfully and makes the "prefer home team" tie-break branch
//!   unreachable (same documented-gap pattern as `is_raiding_party_available` in
//!   `logic_module.rs`).
//!
//! @author Kalimar

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

/// 1:1 translation of the `WaitForOpponentLogicModule` class.
#[derive(Debug, Default)]
pub struct WaitForOpponentLogicModule;

impl WaitForOpponentLogicModule {
    /// java: `public WaitForOpponentLogicModule(FantasyFootballClient pClient)`.
    pub fn new() -> Self {
        Self
    }

    /// java: `public void illegalProcedure()`.
    pub fn illegal_procedure(&self, client: &mut FantasyFootballClient) {
        client.communication_mut().send_illegal_procedure();
    }
}

impl LogicModule for WaitForOpponentLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::WaitForOpponent
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

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)` — always throws
    /// `UnsupportedOperationException` in Java; faithfully translated as a panic.
    fn action_context(&self, _game: &Game, _acting_player: &ActingPlayer) -> ActionContext {
        panic!("actionContext for acting player is not supported in waiting context")
    }

    /// java: `public Optional<Player<?>> getPlayer(FieldCoordinate coordinate)` — see module
    /// doc gap regarding `FieldModel` multi-occupancy.
    fn get_player<'a>(&self, client: &'a FantasyFootballClient, coordinate: FieldCoordinate) -> Option<&'a Player> {
        let game = client.game()?;
        let id = game.field_model.player_at(coordinate)?;
        game.player(id)
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

    #[test]
    fn get_id_is_wait_for_opponent() {
        assert_eq!(
            WaitForOpponentLogicModule::new().get_id(),
            ClientStateId::WaitForOpponent
        );
    }

    #[test]
    fn available_actions_is_empty() {
        assert!(WaitForOpponentLogicModule::new().available_actions().is_empty());
    }

    #[test]
    #[should_panic(expected = "actionContext for acting player is not supported in waiting context")]
    fn action_context_panics() {
        let module = WaitForOpponentLogicModule::new();
        let game = make_game();
        let ap = ActingPlayer::new();
        module.action_context(&game, &ap);
    }

    #[test]
    fn illegal_procedure_sends_command_without_panicking() {
        let params = crate::client::client_parameters::ClientParameters::create_valid_params(&[
            "-spectator".into(),
            "-coach".into(),
            "bob".into(),
        ])
        .unwrap();
        let mut client = FantasyFootballClient::new(params);
        let module = WaitForOpponentLogicModule::new();
        module.illegal_procedure(&mut client);
    }

    #[test]
    fn get_player_returns_none_without_game() {
        let params = crate::client::client_parameters::ClientParameters::create_valid_params(&[
            "-spectator".into(),
            "-coach".into(),
            "bob".into(),
        ])
        .unwrap();
        let client = FantasyFootballClient::new(params);
        let module = WaitForOpponentLogicModule::new();
        assert!(module
            .get_player(&client, FieldCoordinate::new(1, 1))
            .is_none());
    }

    #[test]
    fn get_player_returns_single_occupant() {
        let params = crate::client::client_parameters::ClientParameters::create_valid_params(&[
            "-spectator".into(),
            "-coach".into(),
            "bob".into(),
        ])
        .unwrap();
        let mut client = FantasyFootballClient::new(params);
        let mut game = make_game();
        let mut player = Player::default();
        player.id = "p1".to_string();
        game.team_home.players.push(player);
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(3, 3));
        client.set_game(game);

        let module = WaitForOpponentLogicModule::new();
        let found = module.get_player(&client, FieldCoordinate::new(3, 3));
        assert_eq!(found.map(|p| p.id.as_str()), Some("p1"));
    }
}
