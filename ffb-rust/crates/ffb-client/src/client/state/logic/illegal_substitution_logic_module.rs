//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.IllegalSubstitutionLogicModule` (47 lines).
//!
//! Extends `SetupLogicModule` (delegated to via composition — see
//! `setup_logic_module.rs`), adding tracking of which home-team players were on the pitch at
//! the start of setup (`fFieldPlayers`), used to detect illegally substituted players.

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

/// 1:1 translation of the `IllegalSubstitutionLogicModule` class.
#[derive(Debug, Default)]
pub struct IllegalSubstitutionLogicModule {
    setup: SetupLogicModule,
    /// java: `private Set<Player<?>> fFieldPlayers;` — stores player ids in place of `Player`
    /// references (matches the id-keyed convention used throughout the Rust model).
    field_players: HashSet<String>,
}

impl IllegalSubstitutionLogicModule {
    /// java: `public IllegalSubstitutionLogicModule(FantasyFootballClient pClient)`.
    pub fn new() -> Self {
        Self {
            setup: SetupLogicModule::new(),
            field_players: HashSet::new(),
        }
    }

    /// java: `public boolean squareContainsSubstitute(FieldCoordinate coordinate)`.
    pub fn square_contains_substitute(&self, client: &FantasyFootballClient, coordinate: FieldCoordinate) -> bool {
        let player = match LogicModule::get_player(self, client, coordinate) {
            Some(p) => p,
            None => return false,
        };
        match client.game() {
            Some(game) => game.team_home.has_player(&player.id) && self.is_substitute(player),
            None => false,
        }
    }

    /// java: `public boolean isSubstitute(Player<?> draggedPlayer)`.
    pub fn is_substitute(&self, dragged_player: &Player) -> bool {
        !self.field_players.contains(&dragged_player.id)
    }
}

impl LogicModule for IllegalSubstitutionLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::IllegalSubstitution
    }

    /// java: `public void setUp() { super.setUp(); ... }`.
    fn set_up(&mut self, client: &mut FantasyFootballClient) {
        self.setup.set_up(client);
        self.field_players.clear();
        if let Some(game) = client.game() {
            for player in &game.team_home.players {
                let on_pitch = game
                    .field_model
                    .player_coordinate(&player.id)
                    .map(|coord| !coord.is_box_coordinate())
                    .unwrap_or(false);
                if on_pitch {
                    self.field_players.insert(player.id.clone());
                }
            }
        }
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

    /// java: `public void endTurn()` — delegates to `SetupLogicModule`.
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

    fn add_home_player(game: &mut Game, id: &str, coord: FieldCoordinate) {
        let mut player = Player::default();
        player.id = id.to_string();
        game.team_home.players.push(player);
        game.field_model.set_player_coordinate(id, coord);
    }

    #[test]
    fn get_id_is_illegal_substitution() {
        assert_eq!(
            IllegalSubstitutionLogicModule::new().get_id(),
            ClientStateId::IllegalSubstitution
        );
    }

    #[test]
    fn set_up_records_players_on_pitch_only() {
        let params = crate::client::client_parameters::ClientParameters::create_valid_params(&[
            "-spectator".into(),
            "-coach".into(),
            "bob".into(),
        ])
        .unwrap();
        let mut client = FantasyFootballClient::new(params);
        let mut game = make_game();
        add_home_player(&mut game, "on_pitch", FieldCoordinate::new(5, 5));
        add_home_player(
            &mut game,
            "in_box",
            FieldCoordinate::new(ffb_model::types::RSV_HOME_X, 3),
        );
        client.set_game(game);

        let mut module = IllegalSubstitutionLogicModule::new();
        module.set_up(&mut client);

        assert!(module.field_players.contains("on_pitch"));
        assert!(!module.field_players.contains("in_box"));
    }

    #[test]
    fn is_substitute_true_only_when_not_in_field_players() {
        let mut module = IllegalSubstitutionLogicModule::new();
        module.field_players.insert("veteran".to_string());
        let mut veteran = Player::default();
        veteran.id = "veteran".to_string();
        let mut substitute = Player::default();
        substitute.id = "substitute".to_string();

        assert!(!module.is_substitute(&veteran));
        assert!(module.is_substitute(&substitute));
    }

    #[test]
    fn available_actions_delegates_to_setup_and_is_empty() {
        assert!(IllegalSubstitutionLogicModule::new().available_actions().is_empty());
    }

    #[test]
    #[should_panic(expected = "actionContext for acting player is not supported in setup context")]
    fn action_context_panics_via_delegation() {
        let module = IllegalSubstitutionLogicModule::new();
        let game = make_game();
        let ap = ActingPlayer::new();
        module.action_context(&game, &ap);
    }
}
