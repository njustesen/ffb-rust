//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.bb2020.GazeMoveLogicModule` (47 lines).
//!
//! Java's `GazeMoveLogicModule extends MoveLogicModule`, overriding `getId`, `playerPeek`, and
//! `playerInteraction`. Non-overridden methods (`availableActions`, `actionContext`,
//! `performAvailableAction`, `endTurn`) are inherited from a held `MoveLogicModule` instance,
//! per the established batch convention (see `throw_team_mate_logic_module.rs`).

use ffb_model::enums::ClientStateId;
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::util::util_player::UtilPlayer;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::LogicModule;
use crate::client::state::logic::move_logic_module::MoveLogicModule;

/// 1:1 translation of the `GazeMoveLogicModule` class.
#[derive(Debug, Default)]
pub struct GazeMoveLogicModule {
    move_logic: MoveLogicModule,
}

impl GazeMoveLogicModule {
    /// java: `public GazeMoveLogicModule(FantasyFootballClient client)`.
    pub fn new() -> Self {
        Self { move_logic: MoveLogicModule::new() }
    }

    /// java: `public InteractionResult playerPeek(Player<?> player)`.
    pub fn player_peek(&self, client: &FantasyFootballClient, player: &Player) -> InteractionResult {
        let is_adjacent_gaze_target =
            client.game().map(|game| UtilPlayer::has_adjacent_gaze_target(game, &player.id)).unwrap_or(false);
        if is_adjacent_gaze_target {
            InteractionResult::perform()
        } else {
            InteractionResult::ignore()
        }
    }

    /// java: `public InteractionResult playerInteraction(Player<?> player)`.
    pub fn player_interaction(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        let (is_adjacent_gaze_target, acting_player_id) = match client.game() {
            Some(game) => (UtilPlayer::has_adjacent_gaze_target(game, &player.id), game.acting_player.player_id.clone()),
            None => return self.move_logic.player_interaction(client, player),
        };

        if is_adjacent_gaze_target {
            if let Some(id) = acting_player_id {
                client.communication_mut().send_gaze(id, Some(player));
            }
            return InteractionResult::handled();
        }

        self.move_logic.player_interaction(client, player)
    }
}

impl LogicModule for GazeMoveLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::GazeMove
    }

    /// java: `public Set<ClientAction> availableActions()` — not overridden; inherited from
    /// `MoveLogicModule`.
    fn available_actions(&self) -> std::collections::HashSet<ClientAction> {
        self.move_logic.available_actions()
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)` — not
    /// overridden; inherited from `MoveLogicModule`.
    fn action_context(&self, game: &Game, acting_player: &ActingPlayer) -> ActionContext {
        self.move_logic.action_context(game, acting_player)
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)` —
    /// not overridden; inherited from `MoveLogicModule`.
    fn perform_available_action(&mut self, client: &mut FantasyFootballClient, player: &Player, action: ClientAction) {
        self.move_logic.perform_available_action(client, player, action);
    }

    /// java: `public void endTurn()` — not overridden; inherited from `MoveLogicModule`.
    fn end_turn(&mut self, client: &mut FantasyFootballClient) {
        self.move_logic.end_turn(client);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::player_state::PlayerState;
    use ffb_model::model::team::Team;
    use ffb_model::types::FieldCoordinate;

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
        Game::new(make_team("home"), make_team("away"), Rules::Bb2020)
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
    fn get_id_is_gaze_move() {
        assert_eq!(GazeMoveLogicModule::new().get_id(), ClientStateId::GazeMove);
    }

    #[test]
    fn available_actions_matches_move_logic_module() {
        let module = GazeMoveLogicModule::new();
        assert_eq!(module.available_actions().len(), MoveLogicModule::new().available_actions().len());
    }

    #[test]
    fn player_peek_ignores_without_game() {
        let module = GazeMoveLogicModule::new();
        let client = make_client();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_peek(&client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn player_peek_ignores_without_adjacent_gaze_target() {
        let module = GazeMoveLogicModule::new();
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(10, 10));
        client.set_game(game);
        let player = client.game().unwrap().player("p1").unwrap().clone();
        let result = module.player_peek(&client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn player_interaction_falls_through_to_move_logic_without_gaze_target() {
        let module = GazeMoveLogicModule::new();
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(10, 10));
        client.set_game(game);
        let player = client.game().unwrap().player("p1").unwrap().clone();
        let result = module.player_interaction(&mut client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }
}
