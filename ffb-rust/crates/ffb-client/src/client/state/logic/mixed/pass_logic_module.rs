//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.mixed.PassLogicModule` (226 lines).
//!
//! Java's `PassLogicModule extends MoveLogicModule`. Per the established composition convention
//! (see `blitz_logic_module.rs`), this struct holds a `MoveLogicModule` value for delegating
//! unmodified inherited behavior and calling `super.xxx(...)`.
//!
//! Documented gaps:
//! - `isBoundingLeapAvailable` returns `Optional<Skill>` in Java; the composed
//!   `logic_module::is_bounding_leap_available` returns `Option<SkillId>` instead (no `Skill`
//!   value type threaded through, matching that function's own documented gap).
//! - `ActingPlayer.getRace()` has no Rust-model equivalent (no `race` field on `ActingPlayer`);
//!   resolved via the acting player's underlying `Player.race` instead (same value Java's
//!   `ActingPlayer.getRace()` delegates to).

use std::collections::HashSet;

use ffb_model::enums::{ClientStateId, PlayerAction};
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::util_player::UtilPlayer;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::influences::Influences;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::{self, LogicModule};
use crate::client::state::logic::move_logic_module::MoveLogicModule;

/// 1:1 translation of the `PassLogicModule` class.
#[derive(Debug, Default)]
pub struct PassLogicModule {
    move_logic: MoveLogicModule,
}

impl PassLogicModule {
    /// java: `public PassLogicModule(FantasyFootballClient pClient)`.
    pub fn new() -> Self {
        Self { move_logic: MoveLogicModule::new() }
    }

    /// java: `public InteractionResult playerInteraction(Player<?> player)`.
    pub fn player_interaction(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        let acting_player = match client.game() {
            Some(game) => game.acting_player.clone(),
            None => return InteractionResult::ignore(),
        };

        if acting_player.player_id.as_deref() == Some(player.id.as_str()) {
            return self.move_logic.player_interaction(client, player);
        }

        let should_send = match client.game() {
            Some(game) => {
                let has_ball = acting_player.player_id.as_deref().map(|id| UtilPlayer::has_ball(game, id)).unwrap_or(false);
                !acting_player.has_passed
                    && (acting_player.player_action == Some(PlayerAction::HailMaryPass)
                        || (has_ball
                            && (acting_player.player_action == Some(PlayerAction::Pass)
                                || self.can_player_get_pass(game, &acting_player, Some(player)))))
            }
            None => false,
        };

        if should_send {
            let pass_coord = match client.game() {
                Some(game) => game.field_model.player_coordinate(&player.id),
                None => None,
            };
            if let (Some(coord), Some(id)) = (pass_coord, acting_player.player_id.clone()) {
                if let Some(game) = client.game_mut() {
                    game.pass_coordinate = Some(coord);
                }
                client.communication_mut().send_pass(id, coord);
                if let Some(game) = client.game_mut() {
                    game.field_model.range_ruler = None;
                }
                return InteractionResult::handled();
            }
        }

        InteractionResult::ignore()
    }

    /// java: `public InteractionResult fieldInteraction(FieldCoordinate pCoordinate)`.
    pub fn field_interaction(&self, client: &mut FantasyFootballClient, coordinate: FieldCoordinate) -> InteractionResult {
        let acting_player = match client.game() {
            Some(game) => game.acting_player.clone(),
            None => return InteractionResult::ignore(),
        };

        if acting_player.player_action == Some(PlayerAction::PassMove) {
            return InteractionResult::delegate(<MoveLogicModule as LogicModule>::get_id(&self.move_logic));
        }

        let has_ball_or_hmp = match client.game() {
            Some(game) => {
                acting_player.player_action == Some(PlayerAction::HailMaryPass)
                    || acting_player.player_id.as_deref().map(|id| UtilPlayer::has_ball(game, id)).unwrap_or(false)
            }
            None => false,
        };

        if has_ball_or_hmp {
            if let Some(id) = acting_player.player_id.clone() {
                if let Some(game) = client.game_mut() {
                    game.pass_coordinate = Some(coordinate);
                }
                client.communication_mut().send_pass(id, coordinate);
                if let Some(game) = client.game_mut() {
                    game.field_model.range_ruler = None;
                }
                return InteractionResult::handled();
            }
        }

        InteractionResult::ignore()
    }

    /// java: `public InteractionResult playerPeek(Player<?> pPlayer)`.
    pub fn player_peek(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        client.client_data_mut().set_selected_player(Some(player.id.clone()));

        let acting_player = match client.game() {
            Some(game) => game.acting_player.clone(),
            None => return InteractionResult::ignore(),
        };

        let not_hmp_and_has_ball = match client.game() {
            Some(game) => {
                acting_player.player_action != Some(PlayerAction::HailMaryPass)
                    && acting_player.player_id.as_deref().map(|id| UtilPlayer::has_ball(game, id)).unwrap_or(false)
            }
            None => false,
        };

        if not_hmp_and_has_ball {
            let catcher_coordinate = client.game().and_then(|g| g.field_model.player_coordinate(&player.id));
            let eligible = match client.game() {
                Some(game) => {
                    acting_player.player_action == Some(PlayerAction::Pass)
                        || self.can_player_get_pass(game, &acting_player, Some(player))
                }
                None => false,
            };
            if eligible {
                let mut result = InteractionResult::preview_throw();
                if let Some(coord) = catcher_coordinate {
                    result = result.with_coordinate(coord);
                }
                return result;
            }
        } else {
            if let Some(game) = client.game_mut() {
                game.field_model.range_ruler = None;
            }
            return if self.action_is_hmp(client) {
                InteractionResult::perform()
            } else {
                InteractionResult::reset()
            };
        }

        InteractionResult::ignore()
    }

    /// java: `public InteractionResult fieldPeek(FieldCoordinate pCoordinate)`.
    pub fn field_peek(&self, client: &mut FantasyFootballClient, coordinate: FieldCoordinate) -> InteractionResult {
        if self.action_is_hmp(client) {
            if let Some(game) = client.game_mut() {
                game.field_model.range_ruler = None;
            }
            return InteractionResult::perform();
        }
        let is_pass_move = client.game().map(|g| g.acting_player.player_action == Some(PlayerAction::PassMove)).unwrap_or(false);
        if is_pass_move {
            if let Some(game) = client.game_mut() {
                game.field_model.range_ruler = None;
            }
            return InteractionResult::delegate(<MoveLogicModule as LogicModule>::get_id(&self.move_logic));
        }
        InteractionResult::preview_throw().with_coordinate(coordinate)
    }

    /// java: `public boolean canPlayerGetPass(Player<?> pCatcher)`.
    pub fn can_player_get_pass(&self, game: &Game, acting_player: &ActingPlayer, catcher: Option<&Player>) -> bool {
        let catcher = match catcher {
            Some(c) => c,
            None => return false,
        };
        if acting_player.player_id.is_none() {
            return false;
        }
        let catcher_state = match game.field_model.player_state(&catcher.id) {
            Some(s) => s,
            None => return false,
        };
        let acting_race = acting_player
            .player_id
            .as_deref()
            .and_then(|id| game.player(id))
            .and_then(|p| p.race.as_deref());
        catcher_state.has_tacklezones()
            && game.team_home.has_player(&catcher.id)
            && (!acting_player.suffering_animosity || acting_race == catcher.race.as_deref())
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)`.
    fn perform_available_action_impl(
        &mut self,
        client: &mut FantasyFootballClient,
        player: &Player,
        action: ClientAction,
    ) {
        match action {
            ClientAction::HAIL_MARY_PASS => {
                let available = client.game().map(logic_module::is_hail_mary_pass_action_available).unwrap_or(false);
                if available {
                    let jumping = client.game().map(|g| g.acting_player.jumping).unwrap_or(false);
                    if self.action_is_hmp(client) {
                        client.communication_mut().send_acting_player(Some(player), PlayerAction::Pass, jumping);
                    } else {
                        client.communication_mut().send_acting_player(Some(player), PlayerAction::HailMaryPass, jumping);
                        if let Some(game) = client.game_mut() {
                            game.field_model.range_ruler = None;
                        }
                    }
                }
            }
            _ => {
                <MoveLogicModule as LogicModule>::perform_available_action(&mut self.move_logic, client, player, action);
            }
        }
    }

    /// java: `public boolean actionIsHmp()`.
    pub fn action_is_hmp(&self, client: &FantasyFootballClient) -> bool {
        client.game().map(|g| g.acting_player.player_action == Some(PlayerAction::HailMaryPass)).unwrap_or(false)
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)`.
    fn action_context_impl(&self, game: &Game, acting_player: &ActingPlayer) -> ActionContext {
        let mut action_context = ActionContext::new();

        if logic_module::is_pass_any_square_available(acting_player, game) && !acting_player.has_passed {
            action_context.add_action(ClientAction::PASS);
        }

        let has_ball = acting_player.player_id.as_deref().map(|id| UtilPlayer::has_ball(game, id)).unwrap_or(false);
        if logic_module::is_hail_mary_pass_action_available(game) && has_ball && !acting_player.has_passed {
            if acting_player.player_action == Some(PlayerAction::HailMaryPass) {
                action_context.add_influence(Influences::IS_THROWING_HAIL_MARY);
            }
            action_context.add_action(ClientAction::HAIL_MARY_PASS);
        }

        if logic_module::is_jump_available_as_next_move(game, acting_player, false) {
            action_context.add_action(ClientAction::JUMP);
            if acting_player.jumping {
                action_context.add_influence(Influences::IS_JUMPING);
            } else if logic_module::is_bounding_leap_available(game, acting_player).is_some() {
                action_context.add_action(ClientAction::BOUNDING_LEAP);
            }
        }

        if !acting_player.has_passed
            && !acting_player.suffering_animosity
            && (acting_player.player_action == Some(PlayerAction::Pass)
                || acting_player.player_action == Some(PlayerAction::HailMaryPass))
        {
            action_context.add_action(ClientAction::MOVE);
        }

        if logic_module::is_wisdom_available_ap(game, acting_player) {
            action_context.add_action(ClientAction::WISDOM);
        }
        if logic_module::is_raiding_party_available_ap(game, acting_player) {
            action_context.add_action(ClientAction::RAIDING_PARTY);
        }
        if logic_module::is_baleful_hex_available_ap(game, acting_player) {
            action_context.add_action(ClientAction::BALEFUL_HEX);
        }
        if logic_module::is_black_ink_available_ap(game, acting_player) {
            action_context.add_action(ClientAction::BLACK_INK);
        }
        if logic_module::is_catch_of_the_day_available_ap(game, acting_player) {
            action_context.add_action(ClientAction::CATCH_OF_THE_DAY);
        }
        if logic_module::is_then_i_started_blastin_available_ap(game, acting_player) {
            action_context.add_action(ClientAction::THEN_I_STARTED_BLASTIN);
        }
        if logic_module::is_fumblerooskie_available(game) {
            action_context.add_action(ClientAction::FUMBLEROOSKIE);
        }
        if logic_module::is_zoat_gaze_available_ap(game, acting_player) {
            action_context.add_action(ClientAction::AUTO_GAZE_ZOAT);
        }
        action_context.add_action(ClientAction::END_MOVE);
        if acting_player.has_acted {
            action_context.add_influence(Influences::HAS_ACTED);
        }
        action_context
    }

    /// java: `public boolean performsRangeGridAction(ActingPlayer actingPlayer, Game game)`.
    pub fn performs_range_grid_action(&self, acting_player: &ActingPlayer) -> bool {
        !acting_player.has_passed
    }
}

impl LogicModule for PassLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::Pass
    }

    /// java: `public Set<ClientAction> availableActions()`.
    fn available_actions(&self) -> HashSet<ClientAction> {
        let mut actions = <MoveLogicModule as LogicModule>::available_actions(&self.move_logic);
        actions.insert(ClientAction::HAIL_MARY_PASS);
        actions
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)`.
    fn action_context(&self, game: &Game, acting_player: &ActingPlayer) -> ActionContext {
        self.action_context_impl(game, acting_player)
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)`.
    fn perform_available_action(&mut self, client: &mut FantasyFootballClient, player: &Player, action: ClientAction) {
        self.perform_available_action_impl(client, player, action)
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
    fn get_id_is_pass() {
        assert_eq!(PassLogicModule::new().get_id(), ClientStateId::Pass);
    }

    #[test]
    fn available_actions_contains_hail_mary_pass() {
        let actions = PassLogicModule::new().available_actions();
        assert!(actions.contains(&ClientAction::HAIL_MARY_PASS));
        assert!(actions.contains(&ClientAction::MOVE));
    }

    #[test]
    fn action_context_adds_end_move_by_default() {
        let module = PassLogicModule::new();
        let game = make_game();
        let ap = ActingPlayer::new();
        let ctx = module.action_context(&game, &ap);
        assert!(ctx.get_actions().contains(&ClientAction::END_MOVE));
    }

    #[test]
    fn performs_range_grid_action_true_without_pass() {
        let module = PassLogicModule::new();
        let ap = ActingPlayer::new();
        assert!(module.performs_range_grid_action(&ap));
    }

    #[test]
    fn performs_range_grid_action_false_after_pass() {
        let module = PassLogicModule::new();
        let mut ap = ActingPlayer::new();
        ap.has_passed = true;
        assert!(!module.performs_range_grid_action(&ap));
    }

    #[test]
    fn action_is_hmp_false_without_game() {
        let client = make_client();
        let module = PassLogicModule::new();
        assert!(!module.action_is_hmp(&client));
    }

    #[test]
    fn can_player_get_pass_false_without_catcher_state() {
        let module = PassLogicModule::new();
        let game = make_game();
        let ap = ActingPlayer::new();
        let mut catcher = Player::default();
        catcher.id = "c1".to_string();
        assert!(!module.can_player_get_pass(&game, &ap, Some(&catcher)));
    }

    #[test]
    fn can_player_get_pass_true_for_home_team_with_tacklezones() {
        let module = PassLogicModule::new();
        let mut game = make_game();
        add_player(&mut game, true, "h1", FieldCoordinate::new(1, 1));
        add_player(&mut game, true, "c1", FieldCoordinate::new(2, 2));
        let mut ap = ActingPlayer::new();
        ap.player_id = Some("h1".to_string());
        let catcher = game.player("c1").unwrap().clone();
        assert!(module.can_player_get_pass(&game, &ap, Some(&catcher)));
    }

    #[test]
    fn player_interaction_ignores_without_game() {
        let mut client = make_client();
        let module = PassLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_interaction(&mut client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn field_interaction_ignores_without_game() {
        let mut client = make_client();
        let module = PassLogicModule::new();
        let result = module.field_interaction(&mut client, FieldCoordinate::new(1, 1));
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn perform_available_action_no_op_without_game() {
        let mut module = PassLogicModule::new();
        let mut client = make_client();
        let mut player = Player::default();
        player.id = "p1".to_string();
        module.perform_available_action(&mut client, &player, ClientAction::HAIL_MARY_PASS);
    }
}
