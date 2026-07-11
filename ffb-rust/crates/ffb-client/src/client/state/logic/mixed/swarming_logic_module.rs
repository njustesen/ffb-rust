//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.mixed.SwarmingLogicModule` (35 lines).
//!
//! Java's `SwarmingLogicModule extends SetupLogicModule` (already translated at
//! `crate::client::state::logic::setup_logic_module::SetupLogicModule`). Per the established
//! composition convention, this struct holds a `SetupLogicModule` value for delegating every
//! inherited/unmodified `LogicModule` behavior (Java doesn't override any `LogicModule` method
//! here besides `getId()`).

use std::collections::HashSet;

use ffb_model::enums::ClientStateId;
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::property::NamedProperties;
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds};

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::logic_module::LogicModule;
use crate::client::state::logic::setup_logic_module::SetupLogicModule;

/// 1:1 translation of the `SwarmingLogicModule` class.
#[derive(Debug, Default)]
pub struct SwarmingLogicModule {
    setup: SetupLogicModule,
}

impl SwarmingLogicModule {
    /// java: `public SwarmingLogicModule(FantasyFootballClient pClient)`.
    pub fn new() -> Self {
        Self { setup: SetupLogicModule::new() }
    }

    /// java: `public boolean squareHasSwarmingPlayer(FieldCoordinate pCoordinate)`.
    pub fn square_has_swarming_player(&self, client: &FantasyFootballClient, coordinate: FieldCoordinate) -> bool {
        self.get_player(client, coordinate)
            .map(|p| p.has_skill_property(NamedProperties::CAN_SNEAK_EXTRA_PLAYERS_ONTO_PITCH))
            .unwrap_or(false)
    }

    /// java: `public boolean squareIsValidForSwarming(FieldCoordinate pCoordinate)`.
    pub fn square_is_valid_for_swarming(
        &self,
        client: &FantasyFootballClient,
        coordinate: Option<FieldCoordinate>,
    ) -> bool {
        let coordinate = match coordinate {
            Some(c) => c,
            None => return false,
        };
        let in_half_no_los_no_wide = FieldCoordinateBounds::HALF_HOME.is_in_bounds(coordinate)
            && !FieldCoordinateBounds::LOS_HOME.is_in_bounds(coordinate)
            && !FieldCoordinateBounds::LOWER_WIDE_ZONE_HOME.is_in_bounds(coordinate)
            && !FieldCoordinateBounds::UPPER_WIDE_ZONE_HOME.is_in_bounds(coordinate);
        (in_half_no_los_no_wide || coordinate.is_box_coordinate())
            && self.get_player(client, coordinate).is_none()
    }
}

impl LogicModule for SwarmingLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::Swarming
    }

    /// java: (not overridden) inherited unchanged from `SetupLogicModule`.
    fn available_actions(&self) -> HashSet<ClientAction> {
        LogicModule::available_actions(&self.setup)
    }

    /// java: (not overridden) inherited unchanged from `SetupLogicModule` — always panics.
    fn action_context(&self, game: &Game, acting_player: &ActingPlayer) -> ActionContext {
        LogicModule::action_context(&self.setup, game, acting_player)
    }

    /// java: (not overridden) inherited unchanged from `SetupLogicModule`.
    fn perform_available_action(&mut self, client: &mut FantasyFootballClient, player: &Player, action: ClientAction) {
        LogicModule::perform_available_action(&mut self.setup, client, player, action)
    }

    /// java: (not overridden) inherited unchanged from `SetupLogicModule`.
    fn set_up(&mut self, client: &mut FantasyFootballClient) {
        LogicModule::set_up(&mut self.setup, client)
    }

    /// java: (not overridden) inherited unchanged from `SetupLogicModule`.
    fn end_turn(&mut self, client: &mut FantasyFootballClient) {
        LogicModule::end_turn(&mut self.setup, client)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, PS_STANDING};
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
        let mut player = Player::default();
        player.id = id.to_string();
        if home {
            game.team_home.players.push(player);
        } else {
            game.team_away.players.push(player);
        }
        game.field_model.set_player_coordinate(id, coord);
        game.field_model.set_player_state(id, PlayerState::new(PS_STANDING).change_active(true));
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
    fn square_has_swarming_player_false_without_player() {
        let client = make_client();
        let module = SwarmingLogicModule::new();
        assert!(!module.square_has_swarming_player(&client, FieldCoordinate::new(1, 1)));
    }

    #[test]
    fn square_is_valid_for_swarming_false_for_none() {
        let client = make_client();
        let module = SwarmingLogicModule::new();
        assert!(!module.square_is_valid_for_swarming(&client, None));
    }

    #[test]
    fn square_is_valid_for_swarming_true_in_half_home_when_empty() {
        let mut client = make_client();
        client.set_game(make_game());
        let module = SwarmingLogicModule::new();
        // (0, 7) is in HALF_HOME but outside LOS_HOME/UPPER_WIDE_ZONE_HOME/LOWER_WIDE_ZONE_HOME.
        assert!(module.square_is_valid_for_swarming(&client, Some(FieldCoordinate::new(0, 7))));
    }

    #[test]
    fn square_is_valid_for_swarming_false_when_occupied() {
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, true, "h1", FieldCoordinate::new(0, 0));
        client.set_game(game);
        let module = SwarmingLogicModule::new();
        assert!(!module.square_is_valid_for_swarming(&client, Some(FieldCoordinate::new(0, 0))));
    }

    #[test]
    #[should_panic]
    fn action_context_panics_inherited_from_setup() {
        let module = SwarmingLogicModule::new();
        let game = make_game();
        let ap = ActingPlayer::new();
        module.action_context(&game, &ap);
    }
}
