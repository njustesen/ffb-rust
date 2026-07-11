//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.bb2025.PassLogicModule` (230 lines).
//!
//! Java's `PassLogicModule extends MoveLogicModule` (no plugin/mixin fields of its own). Per the
//! established batch convention (see `move_logic_module.rs`/`blitz_logic_module.rs`), this struct
//! composes a `MoveLogicModule` value and delegates to it for `super.playerInteraction(player)`/
//! `super.getId()`/the default `performAvailableAction` branch/`availableActions()`.
//!
//! Documented gaps:
//! - `actingPlayer.getPlayer().hasActiveEnhancement(NamedProperties.canAvoidDodging)` — no
//!   enhancement-tracking exists on the Rust `Player` (same gap as `logic_module.rs`'s
//!   `is_incorporeal_available_ap`); conservatively `false`.

use std::collections::HashSet;

use ffb_model::enums::{ClientStateId, PlayerAction};
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::property::NamedProperties;
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

        let condition = match client.game() {
            Some(game) => {
                let has_ball = acting_player
                    .player_id
                    .as_deref()
                    .map(|id| UtilPlayer::has_ball(game, id))
                    .unwrap_or(false);
                !acting_player.has_passed
                    && (acting_player.player_action == Some(PlayerAction::HailMaryPass)
                        || (has_ball
                            && (acting_player.player_action == Some(PlayerAction::Pass)
                                || self.can_player_get_pass(game, &acting_player, player))))
            }
            None => false,
        };

        if condition {
            let target_coordinate = client.game().and_then(|g| g.field_model.player_coordinate(&player.id));
            if let Some(coord) = target_coordinate {
                if let Some(game) = client.game_mut() {
                    game.pass_coordinate = Some(coord);
                }
                if let Some(id) = acting_player.player_id.clone() {
                    let pass_coord = client.game().and_then(|g| g.pass_coordinate);
                    if let Some(pc) = pass_coord {
                        client.communication_mut().send_pass(id, pc);
                    }
                }
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
            return InteractionResult::delegate(self.move_logic.get_id());
        }

        let has_ball = client
            .game()
            .map(|g| acting_player.player_id.as_deref().map(|id| UtilPlayer::has_ball(g, id)).unwrap_or(false))
            .unwrap_or(false);

        if acting_player.player_action == Some(PlayerAction::HailMaryPass) || has_ball {
            if let Some(game) = client.game_mut() {
                game.pass_coordinate = Some(coordinate);
            }
            if let Some(id) = acting_player.player_id.clone() {
                let pass_coord = client.game().and_then(|g| g.pass_coordinate);
                if let Some(pc) = pass_coord {
                    client.communication_mut().send_pass(id, pc);
                }
            }
            if let Some(game) = client.game_mut() {
                game.field_model.range_ruler = None;
            }
            return InteractionResult::handled();
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

        let has_ball = client
            .game()
            .map(|g| acting_player.player_id.as_deref().map(|id| UtilPlayer::has_ball(g, id)).unwrap_or(false))
            .unwrap_or(false);

        if acting_player.player_action != Some(PlayerAction::HailMaryPass) && has_ball {
            let catcher_coordinate = client.game().and_then(|g| g.field_model.player_coordinate(&player.id));
            let can_get_pass = client
                .game()
                .map(|g| self.can_player_get_pass(g, &acting_player, player))
                .unwrap_or(false);
            if acting_player.player_action == Some(PlayerAction::Pass) || can_get_pass {
                let mut result = InteractionResult::preview_throw();
                if let Some(c) = catcher_coordinate {
                    result = result.with_coordinate(c);
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
        let is_pass_move =
            client.game().map(|g| g.acting_player.player_action == Some(PlayerAction::PassMove)).unwrap_or(false);
        if is_pass_move {
            if let Some(game) = client.game_mut() {
                game.field_model.range_ruler = None;
            }
            return InteractionResult::delegate(self.move_logic.get_id());
        }
        InteractionResult::preview_throw().with_coordinate(coordinate)
    }

    /// java: `public boolean canPlayerGetPass(Player<?> pCatcher)`.
    pub fn can_player_get_pass(&self, game: &Game, acting_player: &ActingPlayer, catcher: &Player) -> bool {
        let attacker = match acting_player.player_id.as_deref().and_then(|id| game.player(id)) {
            Some(p) => p,
            None => return false,
        };
        let catcher_state = match game.field_model.player_state(&catcher.id) {
            Some(s) => s,
            None => return false,
        };
        catcher_state.has_tacklezones()
            && game.team_home.has_player(&catcher.id)
            && (!acting_player.suffering_animosity || attacker.race.as_deref() == catcher.race.as_deref())
    }

    /// java: `public boolean actionIsHmp()`.
    pub fn action_is_hmp(&self, client: &FantasyFootballClient) -> bool {
        client.game().map(|g| g.acting_player.player_action == Some(PlayerAction::HailMaryPass)).unwrap_or(false)
    }

    /// java: `public boolean performsRangeGridAction(ActingPlayer actingPlayer, Game game)`.
    pub fn performs_range_grid_action(&self, acting_player: &ActingPlayer, _game: &Game) -> bool {
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
        let mut actions = self.move_logic.available_actions();
        actions.insert(ClientAction::HAIL_MARY_PASS);
        actions
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)`.
    fn action_context(&self, game: &Game, acting_player: &ActingPlayer) -> ActionContext {
        let mut action_context = ActionContext::new();

        if logic_module::is_pass_any_square_available(acting_player, game) && !acting_player.has_passed {
            action_context.add_action(ClientAction::PASS);
        }

        let has_ball = acting_player
            .player_id
            .as_deref()
            .map(|id| UtilPlayer::has_ball(game, id))
            .unwrap_or(false);
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
        if logic_module::is_incorporeal_available_ap(game, acting_player) {
            action_context.add_action(ClientAction::INCORPOREAL);
            // java: `actingPlayer.getPlayer().hasActiveEnhancement(NamedProperties.canAvoidDodging)`
            // — see module doc gap; conservatively `false`, so `INCORPOREAL_ACTIVE` never fires.
            let has_active_enhancement = false;
            if has_active_enhancement {
                action_context.add_influence(Influences::INCORPOREAL_ACTIVE);
            }
        }
        action_context
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)`.
    fn perform_available_action(&mut self, client: &mut FantasyFootballClient, player: &Player, action: ClientAction) {
        match action {
            ClientAction::HAIL_MARY_PASS => {
                let available = client.game().map(logic_module::is_hail_mary_pass_action_available).unwrap_or(false);
                if available {
                    let (is_hmp, jumping) = client
                        .game()
                        .map(|g| (g.acting_player.player_action == Some(PlayerAction::HailMaryPass), g.acting_player.jumping))
                        .unwrap_or((false, false));
                    if is_hmp {
                        client.communication_mut().send_acting_player(Some(player), PlayerAction::Pass, jumping);
                    } else {
                        client.communication_mut().send_acting_player(Some(player), PlayerAction::HailMaryPass, jumping);
                        if let Some(game) = client.game_mut() {
                            game.field_model.range_ruler = None;
                        }
                    }
                }
            }
            _ => self.move_logic.perform_available_action(client, player, action),
        }
    }

    /// java: `public void endTurn()` — not overridden in `PassLogicModule.java`; inherited
    /// unchanged from `MoveLogicModule`.
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
    fn get_id_is_pass() {
        assert_eq!(PassLogicModule::new().get_id(), ClientStateId::Pass);
    }

    #[test]
    fn available_actions_includes_hail_mary_pass_and_move_actions() {
        let module = PassLogicModule::new();
        let actions = module.available_actions();
        assert!(actions.contains(&ClientAction::HAIL_MARY_PASS));
        assert!(actions.contains(&ClientAction::MOVE));
    }

    #[test]
    fn action_context_empty_without_any_special_availability() {
        let module = PassLogicModule::new();
        let game = make_game();
        let ap = ActingPlayer::new();
        let ctx = module.action_context(&game, &ap);
        assert!(!ctx.get_actions().contains(&ClientAction::PASS));
    }

    #[test]
    fn action_context_always_adds_end_move() {
        let module = PassLogicModule::new();
        let game = make_game();
        let ap = ActingPlayer::new();
        let ctx = module.action_context(&game, &ap);
        assert!(ctx.get_actions().contains(&ClientAction::END_MOVE));
    }

    #[test]
    fn action_is_hmp_false_without_game() {
        let module = PassLogicModule::new();
        let client = make_client();
        assert!(!module.action_is_hmp(&client));
    }

    #[test]
    fn action_is_hmp_true_when_action_matches() {
        let module = PassLogicModule::new();
        let mut client = make_client();
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::HailMaryPass);
        client.set_game(game);
        assert!(module.action_is_hmp(&client));
    }

    #[test]
    fn can_player_get_pass_false_without_acting_player() {
        let module = PassLogicModule::new();
        let mut game = make_game();
        add_player(&mut game, true, "c1", FieldCoordinate::new(2, 2));
        let ap = ActingPlayer::new();
        let catcher = game.player("c1").unwrap().clone();
        assert!(!module.can_player_get_pass(&game, &ap, &catcher));
    }

    #[test]
    fn can_player_get_pass_requires_home_team_and_tacklezones() {
        let module = PassLogicModule::new();
        let mut game = make_game();
        add_player(&mut game, true, "attacker", FieldCoordinate::new(1, 1));
        add_player(&mut game, true, "c1", FieldCoordinate::new(2, 2));
        let mut ap = ActingPlayer::new();
        ap.player_id = Some("attacker".to_string());
        let catcher = game.player("c1").unwrap().clone();
        assert!(module.can_player_get_pass(&game, &ap, &catcher));
    }

    #[test]
    fn performs_range_grid_action_true_when_not_passed() {
        let module = PassLogicModule::new();
        let game = make_game();
        let ap = ActingPlayer::new();
        assert!(module.performs_range_grid_action(&ap, &game));
    }

    #[test]
    fn performs_range_grid_action_false_when_passed() {
        let module = PassLogicModule::new();
        let game = make_game();
        let mut ap = ActingPlayer::new();
        ap.has_passed = true;
        assert!(!module.performs_range_grid_action(&ap, &game));
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
    fn field_interaction_delegates_on_pass_move() {
        let mut client = make_client();
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::PassMove);
        client.set_game(game);
        let module = PassLogicModule::new();
        let result = module.field_interaction(&mut client, FieldCoordinate::new(3, 3));
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Delegate
        );
        assert_eq!(result.get_delegate(), Some(ClientStateId::Move));
    }

    #[test]
    fn field_peek_previews_throw_by_default() {
        let mut client = make_client();
        client.set_game(make_game());
        let module = PassLogicModule::new();
        let result = module.field_peek(&mut client, FieldCoordinate::new(3, 3));
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::PreviewThrow
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
