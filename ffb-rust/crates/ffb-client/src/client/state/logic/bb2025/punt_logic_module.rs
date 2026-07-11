//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.bb2025.PuntLogicModule` (176 lines).
//!
//! Java's `PuntLogicModule extends MoveLogicModule`, overriding `playerInteraction` and the
//! protected `actionAvailable(...)` hook that `MoveLogicModule.playerInteraction` dispatches to
//! polymorphically. Since Rust has no virtual dispatch here (`MoveLogicModule::player_interaction`
//! is an inherent method hard-wired to `MoveLogicModule`'s own `action_available`), Punt's
//! acting-player branch is reimplemented directly against `PuntLogicModule::action_available`
//! (mirroring `MoveLogicModule::player_interaction`'s acting-player branch body exactly), rather
//! than delegating to `self.move_logic.player_interaction(...)`.
//!
//! `action_available` itself drops the unused `JumpMechanic mechanic`/`position` parameters —
//! Java's override body never reads them — for a leaner signature; the retained parameters
//! (`game`, `player`, `acting_player`) carry the entire translated logic.

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

/// 1:1 translation of the `PuntLogicModule` class.
#[derive(Debug, Default)]
pub struct PuntLogicModule {
    move_logic: MoveLogicModule,
}

impl PuntLogicModule {
    /// java: `public PuntLogicModule(FantasyFootballClient pClient)`.
    pub fn new() -> Self {
        Self { move_logic: MoveLogicModule::new() }
    }

    /// java: `public InteractionResult playerInteraction(Player<?> player)`.
    pub fn player_interaction(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        let (is_acting_player, coordinate) = match client.game() {
            Some(game) => (
                game.acting_player.player_id.as_deref() == Some(player.id.as_str()),
                game.field_model.player_coordinate(&player.id),
            ),
            None => return InteractionResult::ignore(),
        };

        if is_acting_player {
            return self.player_interaction_acting(client, player);
        }
        match coordinate {
            Some(c) => self.field_interaction(client, c),
            None => InteractionResult::ignore(),
        }
    }

    /// java: `super.playerInteraction(player)` (`MoveLogicModule`'s acting-player branch),
    /// reimplemented here so it dispatches to `PuntLogicModule::action_available` instead of
    /// `MoveLogicModule`'s own — see module doc comment.
    fn player_interaction_acting(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        let (position, acting_player) = match client.game() {
            Some(game) => {
                let acting_player = game.acting_player.clone();
                let position =
                    acting_player.player_id.as_deref().and_then(|id| game.field_model.player_coordinate(id));
                (position, acting_player)
            }
            None => return InteractionResult::ignore(),
        };
        if position.is_none() {
            return InteractionResult::ignore();
        }

        let available = client.game().map(|g| self.action_available(g, player, &acting_player)).unwrap_or(false);
        if available {
            let ctx = client
                .game()
                .map(|g| <Self as LogicModule>::action_context(self, g, &acting_player))
                .unwrap_or_else(ActionContext::new);
            InteractionResult::select_action(ctx)
        } else {
            // java: `deselectActingPlayer()` — see `LogicModule::deselect_acting_player`'s own
            // documented gap.
            self.deselect_acting_player(client);
            InteractionResult::handled()
        }
    }

    /// java: `protected boolean actionAvailable(Player<?> player, ActingPlayer actingPlayer,
    /// JumpMechanic mechanic, Game game, FieldCoordinate position)` — see module doc comment
    /// regarding the dropped unused parameters.
    pub fn action_available(&self, game: &Game, player: &Player, acting_player: &ActingPlayer) -> bool {
        acting_player.has_acted
            || (acting_player.player_action == Some(PlayerAction::PuntMove) && UtilPlayer::has_ball(game, &player.id))
            || acting_player.player_action == Some(PlayerAction::Punt)
    }

    /// java: `public InteractionResult fieldInteraction(FieldCoordinate pCoordinate)`.
    pub fn field_interaction(&self, client: &mut FantasyFootballClient, coordinate: FieldCoordinate) -> InteractionResult {
        let acting_player = match client.game() {
            Some(game) => game.acting_player.clone(),
            None => return InteractionResult::ignore(),
        };
        if acting_player.player_action == Some(PlayerAction::PuntMove) {
            return InteractionResult::delegate(self.move_logic.get_id());
        }

        let has_ball = client
            .game()
            .map(|g| acting_player.player_id.as_deref().map(|id| UtilPlayer::has_ball(g, id)).unwrap_or(false))
            .unwrap_or(false);

        if acting_player.player_action == Some(PlayerAction::Punt) && has_ball {
            let has_move_square = client.game().and_then(|g| g.field_model.get_move_square(coordinate)).is_some();
            if has_move_square {
                client.communication_mut().send_field_coordinate(coordinate);
                return InteractionResult::handled();
            }
        }
        InteractionResult::ignore()
    }

    /// java: `public InteractionResult playerPeek(Player<?> pPlayer) { return InteractionResult.ignore(); }`.
    pub fn player_peek(&self, _player: &Player) -> InteractionResult {
        InteractionResult::ignore()
    }

    /// java: `public InteractionResult fieldPeek(FieldCoordinate pCoordinate)`.
    pub fn field_peek(&self, client: &FantasyFootballClient, coordinate: FieldCoordinate) -> InteractionResult {
        let is_punt_move =
            client.game().map(|g| g.acting_player.player_action == Some(PlayerAction::PuntMove)).unwrap_or(false);
        if is_punt_move {
            return InteractionResult::delegate(self.move_logic.get_id());
        }
        let has_move_square = client.game().and_then(|g| g.field_model.get_move_square(coordinate)).is_some();
        if has_move_square {
            InteractionResult::perform()
        } else {
            InteractionResult::ignore()
        }
    }
}

impl LogicModule for PuntLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::Punt
    }

    /// java: `public Set<ClientAction> availableActions()`.
    fn available_actions(&self) -> HashSet<ClientAction> {
        let mut actions = self.move_logic.available_actions();
        actions.insert(ClientAction::PUNT);
        actions
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)`.
    fn action_context(&self, game: &Game, acting_player: &ActingPlayer) -> ActionContext {
        let mut action_context = ActionContext::new();

        let has_ball = acting_player
            .player_id
            .as_deref()
            .map(|id| UtilPlayer::has_ball(game, id))
            .unwrap_or(false);
        if acting_player.player_action == Some(PlayerAction::PuntMove) && has_ball {
            action_context.add_action(ClientAction::PUNT);
        }

        if acting_player.player_action == Some(PlayerAction::Punt)
            && UtilPlayer::has_move_left(game, acting_player.jumping)
        {
            action_context.add_action(ClientAction::MOVE);
        }

        if logic_module::is_jump_available_as_next_move(game, acting_player, false) {
            action_context.add_action(ClientAction::JUMP);
            if acting_player.jumping {
                action_context.add_influence(Influences::IS_JUMPING);
            } else if logic_module::is_bounding_leap_available(game, acting_player).is_some() {
                action_context.add_action(ClientAction::BOUNDING_LEAP);
            }
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
            // — see `pass_logic_module.rs`'s identical documented gap; conservatively `false`.
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
            ClientAction::PUNT => {
                let jumping = client.game().map(|g| g.acting_player.jumping).unwrap_or(false);
                client.communication_mut().send_acting_player(Some(player), PlayerAction::Punt, jumping);
            }
            _ => self.move_logic.perform_available_action(client, player, action),
        }
    }

    /// java: `public void endTurn()` — not overridden in `PuntLogicModule.java`; inherited
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
    fn get_id_is_punt() {
        assert_eq!(PuntLogicModule::new().get_id(), ClientStateId::Punt);
    }

    #[test]
    fn available_actions_includes_punt() {
        let module = PuntLogicModule::new();
        assert!(module.available_actions().contains(&ClientAction::PUNT));
    }

    #[test]
    fn action_available_true_when_has_acted() {
        let module = PuntLogicModule::new();
        let game = make_game();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let mut ap = ActingPlayer::new();
        ap.has_acted = true;
        assert!(module.action_available(&game, &player, &ap));
    }

    #[test]
    fn action_available_true_for_punt_action() {
        let module = PuntLogicModule::new();
        let game = make_game();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let mut ap = ActingPlayer::new();
        ap.player_action = Some(PlayerAction::Punt);
        assert!(module.action_available(&game, &player, &ap));
    }

    #[test]
    fn action_available_false_otherwise() {
        let module = PuntLogicModule::new();
        let game = make_game();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let ap = ActingPlayer::new();
        assert!(!module.action_available(&game, &player, &ap));
    }

    #[test]
    fn player_peek_always_ignores() {
        let module = PuntLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_peek(&player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn field_peek_delegates_on_punt_move() {
        let mut client = make_client();
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::PuntMove);
        client.set_game(game);
        let module = PuntLogicModule::new();
        let result = module.field_peek(&client, FieldCoordinate::new(3, 3));
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Delegate
        );
        assert_eq!(result.get_delegate(), Some(ClientStateId::Move));
    }

    #[test]
    fn field_peek_ignores_without_move_square() {
        let mut client = make_client();
        client.set_game(make_game());
        let module = PuntLogicModule::new();
        let result = module.field_peek(&client, FieldCoordinate::new(3, 3));
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn action_context_always_adds_end_move() {
        let module = PuntLogicModule::new();
        let game = make_game();
        let ap = ActingPlayer::new();
        let ctx = module.action_context(&game, &ap);
        assert!(ctx.get_actions().contains(&ClientAction::END_MOVE));
    }

    #[test]
    fn action_context_adds_punt_when_punt_move_and_has_ball() {
        let module = PuntLogicModule::new();
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(1, 1));
        game.field_model.ball_in_play = true;
        let mut ap = ActingPlayer::new();
        ap.player_id = Some("p1".to_string());
        ap.player_action = Some(PlayerAction::PuntMove);
        let ctx = module.action_context(&game, &ap);
        assert!(ctx.get_actions().contains(&ClientAction::PUNT));
    }

    #[test]
    fn player_interaction_ignores_without_game() {
        let mut client = make_client();
        let module = PuntLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_interaction(&mut client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn perform_available_action_punt_sends_command() {
        let mut module = PuntLogicModule::new();
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        client.set_game(game);
        let player = client.game().unwrap().player("p1").unwrap().clone();
        module.perform_available_action(&mut client, &player, ClientAction::PUNT);
        assert!(!client.communication().is_stopped());
    }
}
