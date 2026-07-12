//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.bb2025.SwarmingLogicModule` (32 lines).
//!
//! Java's `SwarmingLogicModule extends SetupLogicModule`, adding two predicate methods and no
//! other overrides. Per the established batch convention, this struct composes a
//! `SetupLogicModule` value and implements `LogicModule` by delegating every method to it,
//! matching the "thin subclass" precedent already used elsewhere in this batch.
//!
//! `squareHasSwarmingPlayer`'s `player.getPosition().getKeywords().contains(Keyword.LINEMAN)`
//! uses the precomputed `Player::is_lineman` flag (mirrors `Player::is_big_guy`; see
//! `ffb-model/src/model/player.rs`), set from the roster position's keyword list at load time.

use ffb_model::enums::ClientStateId;
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds};

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::logic_module::LogicModule;
use crate::client::state::logic::setup_logic_module::SetupLogicModule;

/// 1:1 translation of the `SwarmingLogicModule` class.
#[derive(Debug, Default)]
pub struct SwarmingLogicModule {
    setup_logic: SetupLogicModule,
}

impl SwarmingLogicModule {
    /// java: `public SwarmingLogicModule(FantasyFootballClient pClient)`.
    pub fn new() -> Self {
        Self { setup_logic: SetupLogicModule::new() }
    }

    /// java: `public boolean squareHasSwarmingPlayer(FieldCoordinate pCoordinate)`.
    pub fn square_has_swarming_player(&self, client: &FantasyFootballClient, coordinate: FieldCoordinate) -> bool {
        match self.get_player(client, coordinate) {
            // java: `player.getPosition().getKeywords().contains(Keyword.LINEMAN)`.
            Some(player) => player.is_lineman,
            None => false,
        }
    }

    /// java: `public boolean squareIsValidForSwarming(FieldCoordinate pCoordinate)`.
    ///
    /// Java's `pCoordinate != null` guard is dropped: the Rust `FieldCoordinate` parameter is
    /// never null (not an `Option`), so the check is trivially always true here.
    pub fn square_is_valid_for_swarming(&self, client: &FantasyFootballClient, coordinate: FieldCoordinate) -> bool {
        (FieldCoordinateBounds::HALF_HOME.is_in_bounds(coordinate) || coordinate.is_box_coordinate())
            && self.get_player(client, coordinate).is_none()
    }
}

impl LogicModule for SwarmingLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::Swarming
    }

    /// java: `public void setUp()` — not overridden in `SwarmingLogicModule.java`; inherited
    /// unchanged from `SetupLogicModule`.
    fn set_up(&mut self, client: &mut FantasyFootballClient) {
        self.setup_logic.set_up(client);
    }

    /// java: `public Set<ClientAction> availableActions()` — not overridden; inherited from
    /// `SetupLogicModule`.
    fn available_actions(&self) -> std::collections::HashSet<ClientAction> {
        self.setup_logic.available_actions()
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)` —
    /// not overridden; inherited from `SetupLogicModule`.
    fn perform_available_action(&mut self, client: &mut FantasyFootballClient, player: &Player, action: ClientAction) {
        self.setup_logic.perform_available_action(client, player, action);
    }

    /// java: `public void endTurn()` — not overridden; inherited from `SetupLogicModule`.
    fn end_turn(&mut self, client: &mut FantasyFootballClient) {
        self.setup_logic.end_turn(client);
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)` — not overridden;
    /// inherited from `SetupLogicModule` (always panics there).
    fn action_context(&self, game: &Game, acting_player: &ActingPlayer) -> ActionContext {
        self.setup_logic.action_context(game, acting_player)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::player_state::PlayerState;
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

    fn add_player(game: &mut Game, home: bool, id: &str, coord: FieldCoordinate) {
        add_player_with_lineman(game, home, id, coord, false);
    }

    fn add_player_with_lineman(game: &mut Game, home: bool, id: &str, coord: FieldCoordinate, is_lineman: bool) {
        let mut player = Player::default();
        player.id = id.to_string();
        player.is_lineman = is_lineman;
        if home {
            game.team_home.players.push(player);
        } else {
            game.team_away.players.push(player);
        }
        game.field_model.set_player_coordinate(id, coord);
        game.field_model
            .set_player_state(id, PlayerState::new(ffb_model::enums::PS_STANDING).change_active(true));
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
    fn get_id_is_swarming() {
        assert_eq!(SwarmingLogicModule::new().get_id(), ClientStateId::Swarming);
    }

    #[test]
    fn available_actions_is_empty() {
        assert!(SwarmingLogicModule::new().available_actions().is_empty());
    }

    #[test]
    #[should_panic(expected = "actionContext for acting player is not supported in setup context")]
    fn action_context_panics_like_setup_logic_module() {
        let module = SwarmingLogicModule::new();
        let game = make_game();
        let ap = ActingPlayer::new();
        module.action_context(&game, &ap);
    }

    #[test]
    fn square_has_swarming_player_false_without_game() {
        let client = make_client();
        let module = SwarmingLogicModule::new();
        assert!(!module.square_has_swarming_player(&client, FieldCoordinate::new(1, 1)));
    }

    #[test]
    fn square_has_swarming_player_false_for_non_lineman_player() {
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(3, 3));
        client.set_game(game);
        let module = SwarmingLogicModule::new();
        assert!(!module.square_has_swarming_player(&client, FieldCoordinate::new(3, 3)));
    }

    #[test]
    fn square_has_swarming_player_true_for_lineman_player() {
        let mut client = make_client();
        let mut game = make_game();
        add_player_with_lineman(&mut game, true, "p1", FieldCoordinate::new(3, 3), true);
        client.set_game(game);
        let module = SwarmingLogicModule::new();
        assert!(module.square_has_swarming_player(&client, FieldCoordinate::new(3, 3)));
    }

    #[test]
    fn square_is_valid_for_swarming_true_for_empty_half_home_square() {
        let mut client = make_client();
        client.set_game(make_game());
        let module = SwarmingLogicModule::new();
        assert!(module.square_is_valid_for_swarming(&client, FieldCoordinate::new(1, 1)));
    }

    #[test]
    fn square_is_valid_for_swarming_false_when_occupied() {
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(2, 2));
        client.set_game(game);
        let module = SwarmingLogicModule::new();
        assert!(!module.square_is_valid_for_swarming(&client, FieldCoordinate::new(2, 2)));
    }

    #[test]
    fn square_is_valid_for_swarming_false_outside_half_home_and_not_box() {
        let mut client = make_client();
        client.set_game(make_game());
        let module = SwarmingLogicModule::new();
        assert!(!module.square_is_valid_for_swarming(&client, FieldCoordinate::new(20, 5)));
    }
}
