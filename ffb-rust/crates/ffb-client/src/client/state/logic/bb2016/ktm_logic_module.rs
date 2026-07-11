//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.bb2016.KtmLogicModule` (115 lines).
//!
//! Java's `KtmLogicModule extends MoveLogicModule`, overriding `getId`, `playerInteraction`,
//! `fieldInteraction`, `playerPeek`, `availableActions`, `performAvailableAction`, and adding
//! `canBeKicked`/`ktmActionContext` helpers. Non-overridden methods (`actionContext`, `endTurn`)
//! are inherited from a held `MoveLogicModule` instance, per the established batch convention
//! (see `throw_team_mate_logic_module.rs`).
//!
//! Documented gaps:
//! - `Player.getTeam()` — the Rust `Player` has no back-reference to its owning `Team`; looked
//!   up via `Game::player_team_id`, matching `logic_module.rs`'s own `player_own_team` helper.

use ffb_model::enums::{ClientStateId, PlayerAction};
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::property::NamedProperties;
use ffb_model::types::FieldCoordinate;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::LogicModule;
use crate::client::state::logic::move_logic_module::MoveLogicModule;

/// 1:1 translation of the `KtmLogicModule` class.
#[derive(Debug, Default)]
pub struct KtmLogicModule {
    move_logic: MoveLogicModule,
}

impl KtmLogicModule {
    /// java: `public KtmLogicModule(FantasyFootballClient client)`.
    pub fn new() -> Self {
        Self { move_logic: MoveLogicModule::new() }
    }

    /// java: `public boolean canBeKicked(Player<?> player)`.
    pub fn can_be_kicked(&self, client: &FantasyFootballClient, player: &Player) -> bool {
        let game = match client.game() {
            Some(g) => g,
            None => return false,
        };
        let acting_player = &game.acting_player;
        let attacker = match acting_player.player_id.as_deref().and_then(|id| game.player(id)) {
            Some(p) => p,
            None => return false,
        };
        let catcher_state = match game.field_model.player_state(&player.id) {
            Some(s) => s,
            None => return false,
        };
        let thrower_coordinate = game.field_model.player_coordinate(&attacker.id);
        let catcher_coordinate = game.field_model.player_coordinate(&player.id);
        let adjacent = match (catcher_coordinate, thrower_coordinate) {
            (Some(c), Some(t)) => c.is_adjacent(t),
            _ => false,
        };
        // java: added a check so you could not throw the opponents players, maybe this should
        // be in the server-check?
        attacker.has_skill_property(NamedProperties::CAN_KICK_TEAM_MATES)
            && player.has_skill_property(NamedProperties::CAN_BE_KICKED)
            && catcher_state.has_tacklezones()
            && adjacent
            && (game.player_team_id(&attacker.id) == game.player_team_id(&player.id))
    }

    /// java: `public InteractionResult playerInteraction(Player<?> player)`.
    pub fn player_interaction(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        let (is_acting_player, has_defender) = match client.game() {
            Some(game) => (
                game.acting_player.player_id.as_deref() == Some(player.id.as_str()),
                game.defender_id.is_some(),
            ),
            None => return InteractionResult::ignore(),
        };
        if is_acting_player {
            return self.move_logic.player_interaction(client, player);
        }
        if !has_defender && self.can_be_kicked(client, player) {
            return InteractionResult::select_action(self.ktm_action_context());
        }
        InteractionResult::ignore()
    }

    /// java: `public InteractionResult fieldInteraction(FieldCoordinate ignoredCoordinate)`.
    pub fn field_interaction(&self, client: &FantasyFootballClient, _coordinate: FieldCoordinate) -> InteractionResult {
        let is_ktm_move = client
            .game()
            .map(|g| g.acting_player.player_action == Some(PlayerAction::KickTeamMateMove))
            .unwrap_or(false);
        if is_ktm_move {
            InteractionResult::perform()
        } else {
            InteractionResult::ignore()
        }
    }

    /// java: `public InteractionResult playerPeek(Player<?> player)`.
    pub fn player_peek(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        client.client_data_mut().set_selected_player(Some(player.id.clone()));
        let (has_defender, has_pass_coordinate) = match client.game() {
            Some(game) => (game.defender_id.is_some(), game.pass_coordinate.is_some()),
            None => return InteractionResult::ignore(),
        };
        if !has_defender && !has_pass_coordinate {
            return if self.can_be_kicked(client, player) {
                InteractionResult::perform()
            } else {
                InteractionResult::reset()
            };
        }
        InteractionResult::ignore()
    }

    /// java: `private ActionContext ktmActionContext()`.
    fn ktm_action_context(&self) -> ActionContext {
        let mut action_context = ActionContext::new();
        action_context.add_action(ClientAction::PASS_SHORT);
        action_context.add_action(ClientAction::PASS_LONG);
        action_context
    }
}

impl LogicModule for KtmLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::KickTeamMate
    }

    /// java: `public Set<ClientAction> availableActions()`.
    fn available_actions(&self) -> std::collections::HashSet<ClientAction> {
        let mut actions = std::collections::HashSet::new();
        actions.insert(ClientAction::PASS_SHORT);
        actions.insert(ClientAction::PASS_LONG);
        actions.insert(ClientAction::END_MOVE);
        actions
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)` — not
    /// overridden; inherited from `MoveLogicModule`.
    fn action_context(&self, game: &Game, acting_player: &ActingPlayer) -> ActionContext {
        self.move_logic.action_context(game, acting_player)
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)`.
    fn perform_available_action(&mut self, client: &mut FantasyFootballClient, player: &Player, action: ClientAction) {
        let acting_player_id = match client.game() {
            Some(game) => game.acting_player.player_id.clone(),
            None => return,
        };
        let acting_player_id = match acting_player_id {
            Some(id) => id,
            None => return,
        };
        match action {
            ClientAction::PASS_SHORT => {
                client.communication_mut().send_kick_team_mate(acting_player_id, player.id.clone(), 1);
            }
            ClientAction::PASS_LONG => {
                client.communication_mut().send_kick_team_mate(acting_player_id, player.id.clone(), 2);
            }
            _ => {
                self.move_logic.perform_available_action(client, player, action);
            }
        }
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
        Game::new(make_team("home"), make_team("away"), Rules::Bb2016)
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
    fn get_id_is_kick_team_mate() {
        assert_eq!(KtmLogicModule::new().get_id(), ClientStateId::KickTeamMate);
    }

    #[test]
    fn available_actions_matches_java() {
        let actions = KtmLogicModule::new().available_actions();
        assert!(actions.contains(&ClientAction::PASS_SHORT));
        assert!(actions.contains(&ClientAction::PASS_LONG));
        assert!(actions.contains(&ClientAction::END_MOVE));
        assert_eq!(actions.len(), 3);
    }

    #[test]
    fn can_be_kicked_false_without_game() {
        let module = KtmLogicModule::new();
        let client = make_client();
        let mut player = Player::default();
        player.id = "p1".to_string();
        assert!(!module.can_be_kicked(&client, &player));
    }

    #[test]
    fn can_be_kicked_false_without_required_skills() {
        let module = KtmLogicModule::new();
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, true, "thrower", FieldCoordinate::new(1, 1));
        add_player(&mut game, true, "catcher", FieldCoordinate::new(1, 2));
        game.acting_player.player_id = Some("thrower".to_string());
        client.set_game(game);
        let player = client.game().unwrap().player("catcher").unwrap().clone();
        assert!(!module.can_be_kicked(&client, &player));
    }

    #[test]
    fn field_interaction_performs_when_ktm_move() {
        let module = KtmLogicModule::new();
        let mut client = make_client();
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::KickTeamMateMove);
        client.set_game(game);
        let result = module.field_interaction(&client, FieldCoordinate::new(3, 3));
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Perform
        );
    }

    #[test]
    fn field_interaction_ignores_without_ktm_move() {
        let module = KtmLogicModule::new();
        let mut client = make_client();
        client.set_game(make_game());
        let result = module.field_interaction(&client, FieldCoordinate::new(3, 3));
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn player_peek_sets_selected_player_and_resets_when_not_kickable() {
        let module = KtmLogicModule::new();
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(2, 2));
        client.set_game(game);
        let player = client.game().unwrap().player("p1").unwrap().clone();
        let result = module.player_peek(&mut client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Reset
        );
        assert_eq!(client.client_data().selected_player(), Some(&"p1".to_string()));
    }

    #[test]
    fn player_interaction_ignores_without_game() {
        let mut client = make_client();
        let module = KtmLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_interaction(&mut client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn perform_available_action_no_op_without_game() {
        let mut module = KtmLogicModule::new();
        let mut client = make_client();
        let mut player = Player::default();
        player.id = "p1".to_string();
        module.perform_available_action(&mut client, &player, ClientAction::PASS_SHORT);
    }
}
