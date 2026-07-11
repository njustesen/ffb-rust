//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.ThrowTeamMateLogicModule` (102 lines).
//!
//! Java's `ThrowTeamMateLogicModule extends MoveLogicModule`, overriding `playerInteraction`,
//! `fieldInteraction`, `fieldPeek`, and `playerPeek`. Non-overridden methods (`getId`
//! delegated to via `super.getId()` for the `fieldInteraction` delegate result,
//! `availableActions`/`actionContext`/`performAvailableAction`/`endTurn`) are inherited from a
//! held `MoveLogicModule` instance, per the established batch convention.

use ffb_engine::mechanic::ttm_mechanic_for;
use ffb_model::enums::{ClientStateId, PlayerAction};
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::types::FieldCoordinate;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::LogicModule;
use crate::client::state::logic::move_logic_module::MoveLogicModule;

/// 1:1 translation of the `ThrowTeamMateLogicModule` class.
#[derive(Debug, Default)]
pub struct ThrowTeamMateLogicModule {
    move_logic: MoveLogicModule,
}

impl ThrowTeamMateLogicModule {
    /// java: `public ThrowTeamMateLogicModule(FantasyFootballClient pClient)`.
    pub fn new() -> Self {
        Self { move_logic: MoveLogicModule::new() }
    }

    /// java: `public InteractionResult playerInteraction(Player<?> player)`.
    pub fn player_interaction(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        let (is_acting_player, has_defender, acting_player_id) = match client.game() {
            Some(game) => (
                game.acting_player.player_id.as_deref() == Some(player.id.as_str()),
                game.defender_id.is_some(),
                game.acting_player.player_id.clone(),
            ),
            None => return InteractionResult::ignore(),
        };

        if is_acting_player {
            return self.move_logic.player_interaction(client, player);
        }

        if !has_defender && self.can_be_thrown(client, player) {
            if let Some(id) = acting_player_id {
                client.communication_mut().send_throw_team_mate_by_id(id, player.id.clone());
            }
            return InteractionResult::perform();
        }
        if has_defender {
            let coordinate = client.game().and_then(|g| g.field_model.player_coordinate(&player.id));
            if let Some(game) = client.game_mut() {
                game.field_model.range_ruler = None;
            }
            if let (Some(id), Some(coordinate)) = (acting_player_id, coordinate) {
                client.communication_mut().send_throw_team_mate(id, coordinate);
            }
            return InteractionResult::handled();
        }
        InteractionResult::ignore()
    }

    /// java: `public InteractionResult fieldInteraction(FieldCoordinate pCoordinate)`.
    pub fn field_interaction(&self, client: &mut FantasyFootballClient, coordinate: FieldCoordinate) -> InteractionResult {
        let (is_move, acting_player_id) = match client.game() {
            Some(game) => (
                game.acting_player.player_action == Some(PlayerAction::ThrowTeamMateMove),
                game.acting_player.player_id.clone(),
            ),
            None => return InteractionResult::ignore(),
        };
        if is_move {
            return InteractionResult::delegate(self.move_logic.get_id());
        }
        if let Some(game) = client.game_mut() {
            game.field_model.range_ruler = None;
        }
        if let Some(id) = acting_player_id {
            client.communication_mut().send_throw_team_mate(id, coordinate);
        }
        InteractionResult::handled()
    }

    /// java: `public InteractionResult fieldPeek(FieldCoordinate pCoordinate)`.
    pub fn field_peek(&self, client: &FantasyFootballClient, coordinate: FieldCoordinate) -> InteractionResult {
        let show_preview = match client.game() {
            Some(game) => game.defender_id.is_some() && game.pass_coordinate.is_none(),
            None => false,
        };
        if show_preview {
            return InteractionResult::preview_throw();
        }
        self.move_logic.field_peek(client, coordinate)
    }

    /// java: `public InteractionResult playerPeek(Player<?> pPlayer)`.
    pub fn player_peek(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        client.client_data_mut().set_selected_player(Some(player.id.clone()));
        let (has_defender, has_pass_coordinate) = match client.game() {
            Some(game) => (game.defender_id.is_some(), game.pass_coordinate.is_some()),
            None => return InteractionResult::ignore(),
        };
        if !has_defender && !has_pass_coordinate {
            return if self.can_be_thrown(client, player) {
                InteractionResult::perform()
            } else {
                InteractionResult::reset()
            };
        }
        if has_defender && !has_pass_coordinate {
            return InteractionResult::preview_throw();
        }
        InteractionResult::ignore()
    }

    /// java: `private boolean canBeThrown(Player<?> pPlayer)`.
    fn can_be_thrown(&self, client: &FantasyFootballClient, player: &Player) -> bool {
        let game = match client.game() {
            Some(g) => g,
            None => return false,
        };
        let mechanic = ttm_mechanic_for(game.rules);
        let acting_player = &game.acting_player;
        let thrower = match acting_player.player_id.as_deref().and_then(|id| game.player(id)) {
            Some(p) => p,
            None => return false,
        };
        let thrower_coordinate = game.field_model.player_coordinate(&thrower.id);
        let catcher_coordinate = game.field_model.player_coordinate(&player.id);

        mechanic.can_throw(game, thrower)
            && mechanic.can_be_thrown(game, player)
            && match (catcher_coordinate, thrower_coordinate) {
                (Some(catcher), Some(thrower)) => catcher.is_adjacent(thrower),
                _ => false,
            }
    }
}

impl LogicModule for ThrowTeamMateLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::ThrowTeamMate
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
    fn get_id_is_throw_team_mate() {
        assert_eq!(ThrowTeamMateLogicModule::new().get_id(), ClientStateId::ThrowTeamMate);
    }

    #[test]
    fn available_actions_matches_move_logic_module() {
        let module = ThrowTeamMateLogicModule::new();
        assert_eq!(module.available_actions().len(), MoveLogicModule::new().available_actions().len());
    }

    #[test]
    fn can_be_thrown_false_without_thrower() {
        let mut client = make_client();
        client.set_game(make_game());
        let module = ThrowTeamMateLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        assert!(!module.can_be_thrown(&client, &player));
    }

    #[test]
    fn player_peek_sets_selected_player_and_resets_when_not_throwable() {
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(2, 2));
        client.set_game(game);
        let module = ThrowTeamMateLogicModule::new();
        let player = client.game().unwrap().player("p1").unwrap().clone();
        let result = module.player_peek(&mut client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Reset
        );
        assert_eq!(client.client_data().selected_player(), Some(&"p1".to_string()));
    }

    #[test]
    fn player_peek_preview_throw_when_defender_present() {
        let mut client = make_client();
        let mut game = make_game();
        game.defender_id = Some("d1".to_string());
        client.set_game(game);
        let module = ThrowTeamMateLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_peek(&mut client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::PreviewThrow
        );
    }

    #[test]
    fn field_peek_preview_throw_when_defender_present_and_no_pass_coordinate() {
        let mut client = make_client();
        let mut game = make_game();
        game.defender_id = Some("d1".to_string());
        client.set_game(game);
        let module = ThrowTeamMateLogicModule::new();
        let result = module.field_peek(&client, FieldCoordinate::new(1, 1));
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::PreviewThrow
        );
    }

    #[test]
    fn field_interaction_delegates_when_throw_team_mate_move() {
        let mut client = make_client();
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::ThrowTeamMateMove);
        client.set_game(game);
        let module = ThrowTeamMateLogicModule::new();
        let result = module.field_interaction(&mut client, FieldCoordinate::new(1, 1));
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Delegate
        );
        assert_eq!(result.get_delegate(), Some(ClientStateId::Move));
    }

    #[test]
    fn player_interaction_ignores_without_game() {
        let mut client = make_client();
        let module = ThrowTeamMateLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_interaction(&mut client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }
}
