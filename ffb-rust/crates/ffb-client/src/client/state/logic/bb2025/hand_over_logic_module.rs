//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.bb2025.HandOverLogicModule` (156 lines).
//!
//! Java's `HandOverLogicModule extends MoveLogicModule`, overriding `getId`/`playerInteraction`/
//! `playerPeek`/`fieldPeek`/`actionContext` and adding `canPlayerGetHandOver`/`handOver`/
//! `ballInHand`. Per the `MoveLogicModule` convention (see that module's own doc comment), the
//! inherited `playerInteraction` needs `&mut FantasyFootballClient`, so this is translated as a
//! struct composing `MoveLogicModule` and delegating via its inherent (non-trait) method.

use ffb_model::enums::{ClientStateId, PlayerAction};
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::field_model::FieldModel;
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

/// java: `public boolean canPlayerGetHandOver(Player<?> pCatcher)`.
pub fn can_player_get_hand_over(game: &Game, catcher: Option<&Player>) -> bool {
    let catcher = match catcher {
        Some(c) => c,
        None => return false,
    };
    let acting_player = &game.acting_player;
    let attacker_id = match acting_player.player_id.as_deref() {
        Some(id) => id,
        None => return false,
    };
    let field_model: &FieldModel = &game.field_model;
    let thrower_coordinate = field_model.player_coordinate(attacker_id);
    let catcher_coordinate = field_model.player_coordinate(&catcher.id);
    let catcher_state = field_model.player_state(&catcher.id);

    let adjacent = match (thrower_coordinate, catcher_coordinate) {
        (Some(t), Some(c)) => t.is_adjacent(c),
        _ => false,
    };

    let attacker_race = game.player(attacker_id).and_then(|p| p.race.clone());
    let same_race_or_no_animosity = !acting_player.suffering_animosity || attacker_race == catcher.race;

    adjacent
        && catcher_state.is_some()
        && same_race_or_no_animosity
        && catcher_state.map(|s| s.has_tacklezones()).unwrap_or(false)
        && (game.team_home.has_player(&catcher.id) || acting_player.player_action == Some(PlayerAction::HandOver))
}

/// java: `public boolean ballInHand()`.
pub fn ball_in_hand(game: &Game) -> bool {
    match game.acting_player.player_id.as_deref() {
        Some(id) => UtilPlayer::has_ball(game, id),
        None => false,
    }
}

/// 1:1 translation of the `HandOverLogicModule` class.
#[derive(Debug, Default)]
pub struct HandOverLogicModule {
    move_logic: MoveLogicModule,
}

impl HandOverLogicModule {
    /// java: `public HandOverLogicModule(FantasyFootballClient pClient)`.
    pub fn new() -> Self {
        Self { move_logic: MoveLogicModule::new() }
    }

    /// java: `public InteractionResult playerInteraction(Player<?> player)`.
    pub fn player_interaction(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        let acting_player_id = match client.game() {
            Some(game) => game.acting_player.player_id.clone(),
            None => return InteractionResult::ignore(),
        };
        if acting_player_id.as_deref() == Some(player.id.as_str()) {
            self.move_logic.player_interaction(client, player)
        } else {
            self.hand_over(client, player)
        }
    }

    /// java: `private InteractionResult handOver(Player<?> pCatcher)`.
    fn hand_over(&self, client: &mut FantasyFootballClient, catcher: &Player) -> InteractionResult {
        let (has_ball, can_get_hand_over, acting_player_id) = match client.game() {
            Some(game) => (
                ball_in_hand(game),
                can_player_get_hand_over(game, Some(catcher)),
                game.acting_player.player_id.clone(),
            ),
            None => return InteractionResult::ignore(),
        };
        if has_ball && can_get_hand_over {
            if let Some(id) = acting_player_id {
                client.communication_mut().send_hand_over(id, Some(catcher));
            }
            return InteractionResult::handled();
        }
        InteractionResult::ignore()
    }

    /// java: `public InteractionResult playerPeek(Player<?> pPlayer)`.
    pub fn player_peek(&self, client: &FantasyFootballClient, player: &Player) -> InteractionResult {
        let can_get_hand_over = match client.game() {
            Some(game) => can_player_get_hand_over(game, Some(player)),
            None => false,
        };
        if can_get_hand_over {
            InteractionResult::perform()
        } else {
            InteractionResult::ignore()
        }
    }

    /// java: `public InteractionResult fieldPeek(FieldCoordinate pCoordinate)`.
    pub fn field_peek(&self, _coordinate: FieldCoordinate) -> InteractionResult {
        InteractionResult::delegate(ClientStateId::Move)
    }
}

impl LogicModule for HandOverLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::HandOver
    }

    /// java: `public Set<ClientAction> availableActions()` — inherited unchanged from
    /// `MoveLogicModule` (not overridden in Java).
    fn available_actions(&self) -> std::collections::HashSet<ClientAction> {
        self.move_logic.available_actions()
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)`.
    fn action_context(&self, game: &Game, acting_player: &ActingPlayer) -> ActionContext {
        let mut action_context = ActionContext::new();

        if ball_in_hand(game) {
            action_context.add_action(ClientAction::HAND_OVER);
            if acting_player.player_action == Some(PlayerAction::HandOver) {
                action_context.add_influence(Influences::HANDS_OVER_TO_ANYONE);
            }
        }

        if logic_module::is_jump_available_as_next_move(game, acting_player, true) {
            action_context.add_action(ClientAction::JUMP);
            if acting_player.jumping {
                action_context.add_influence(Influences::IS_JUMPING);
            } else if logic_module::is_bounding_leap_available(game, acting_player).is_some() {
                action_context.add_action(ClientAction::BOUNDING_LEAP);
            }
        }

        if logic_module::is_treacherous_available_ap(game, acting_player) {
            action_context.add_action(ClientAction::TREACHEROUS);
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
            // java: `player.hasActiveEnhancement(NamedProperties.canAvoidDodging)` — no
            // enhancement-tracking exists on the Rust `Player` (documented gap already
            // established in `logic_module.rs`'s `is_incorporeal_available_ap`); conservatively
            // `false`.
            let has_active_enhancement = false;
            if has_active_enhancement {
                action_context.add_influence(Influences::INCORPOREAL_ACTIVE);
            }
        }

        action_context
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)` —
    /// not overridden in Java, inherited unchanged from `MoveLogicModule`.
    fn perform_available_action(&mut self, client: &mut FantasyFootballClient, player: &Player, action: ClientAction) {
        self.move_logic.perform_available_action(client, player, action);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, PS_STANDING};
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
    fn get_id_is_hand_over() {
        assert_eq!(HandOverLogicModule::new().get_id(), ClientStateId::HandOver);
    }

    #[test]
    fn ball_in_hand_false_without_acting_player() {
        let game = make_game();
        assert!(!ball_in_hand(&game));
    }

    #[test]
    fn can_player_get_hand_over_false_without_catcher() {
        let game = make_game();
        assert!(!can_player_get_hand_over(&game, None));
    }

    #[test]
    fn can_player_get_hand_over_false_without_adjacency() {
        let mut game = make_game();
        add_player(&mut game, true, "thrower", FieldCoordinate::new(1, 1));
        add_player(&mut game, true, "catcher", FieldCoordinate::new(10, 10));
        game.acting_player.player_id = Some("thrower".to_string());
        let catcher = game.player("catcher").unwrap().clone();
        assert!(!can_player_get_hand_over(&game, Some(&catcher)));
    }

    #[test]
    fn can_player_get_hand_over_true_when_adjacent_home_team() {
        let mut game = make_game();
        add_player(&mut game, true, "thrower", FieldCoordinate::new(1, 1));
        add_player(&mut game, true, "catcher", FieldCoordinate::new(2, 1));
        game.acting_player.player_id = Some("thrower".to_string());
        let catcher = game.player("catcher").unwrap().clone();
        assert!(can_player_get_hand_over(&game, Some(&catcher)));
    }

    #[test]
    fn field_peek_delegates_to_move() {
        let module = HandOverLogicModule::new();
        let result = module.field_peek(FieldCoordinate::new(1, 1));
        assert_eq!(result.get_kind(), crate::client::state::logic::interaction::interaction_result::Kind::Delegate);
        assert_eq!(result.get_delegate(), Some(ClientStateId::Move));
    }

    #[test]
    fn action_context_empty_without_any_availability() {
        let module = HandOverLogicModule::new();
        let game = make_game();
        let ap = ActingPlayer::new();
        let ctx = module.action_context(&game, &ap);
        assert_eq!(ctx.get_actions(), &vec![ClientAction::END_MOVE]);
    }

    #[test]
    fn player_interaction_ignores_without_game() {
        let mut client = make_client();
        let module = HandOverLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_interaction(&mut client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn player_peek_ignores_without_game() {
        let client = make_client();
        let module = HandOverLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_peek(&client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }
}
