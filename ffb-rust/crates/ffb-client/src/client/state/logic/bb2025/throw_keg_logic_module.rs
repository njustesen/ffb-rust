//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.bb2025.ThrowKegLogicModule` (204 lines).
//!
//! Java's `ThrowKegLogicModule extends LogicModule` directly (not `MoveLogicModule`), so this
//! struct holds no composed sub-module — it implements `LogicModule` on its own.
//!
//! Documented gap:
//! - `FieldModel.findAdjacentCoordinates(coord, FieldCoordinateBounds.FIELD, distance, false)` —
//!   no shared public helper for arbitrary distances exists on the Rust `FieldModel` (only
//!   `adjacent_on_pitch` for distance 1); mirrors the private duplicate already used in
//!   `move_logic_module.rs`'s own `adjacent_field_coordinates` helper and
//!   `path_finder_with_multi_jump.rs`'s `find_adjacent_coordinates`.

use std::collections::HashSet;

use ffb_model::enums::ClientStateId;
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::property::NamedProperties;
use ffb_model::model::skill::skill::Skill;
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds, MoveSquare};

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::LogicModule;

/// java: `player.getSkillWithProperty(property)` — see `block_logic_extension.rs`'s doc gap.
fn skill_placeholder(id: ffb_model::enums::SkillId) -> Skill {
    Skill::new(id.class_name(), ffb_model::enums::SkillCategory::General)
}

/// java: `findAdjacentCoordinates(coord, FieldCoordinateBounds.FIELD, distance, false)` — see
/// module doc gap.
fn find_adjacent_coordinates(coord: FieldCoordinate, distance: i32) -> Vec<FieldCoordinate> {
    let mut result = Vec::new();
    for dx in -distance..=distance {
        for dy in -distance..=distance {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nc = FieldCoordinate::new(coord.x + dx, coord.y + dy);
            if FieldCoordinateBounds::FIELD.is_in_bounds(nc) {
                result.push(nc);
            }
        }
    }
    result
}

/// 1:1 translation of the `ThrowKegLogicModule` class.
#[derive(Debug, Default)]
pub struct ThrowKegLogicModule;

impl ThrowKegLogicModule {
    /// java: `public ThrowKegLogicModule(FantasyFootballClient pClient)`.
    pub fn new() -> Self {
        Self
    }

    /// java: `public InteractionResult playerInteraction(Player<?> player)`.
    pub fn player_interaction(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        let acting_player = match client.game() {
            Some(game) => game.acting_player.clone(),
            None => return InteractionResult::ignore(),
        };
        if acting_player.player_id.as_deref() == Some(player.id.as_str()) {
            let ctx = client
                .game()
                .map(|g| self.action_context(g, &acting_player))
                .unwrap_or_else(ActionContext::new);
            return InteractionResult::select_action(ctx);
        }
        let is_valid = client.game().map(|g| self.is_valid_target(g, player)).unwrap_or(false);
        if is_valid {
            client.communication_mut().send_throw_keg(player);
            return InteractionResult::handled();
        }
        InteractionResult::ignore()
    }

    /// java: `public InteractionResult playerPeek(Player<?> player)`.
    pub fn player_peek(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        client.client_data_mut().set_selected_player(Some(player.id.clone()));
        let is_valid = client.game().map(|g| self.is_valid_target(g, player)).unwrap_or(false);
        if is_valid {
            InteractionResult::perform()
        } else {
            InteractionResult::invalid()
        }
    }

    /// java: `private boolean isValidTarget(Player<?> player, Game game)`.
    fn is_valid_target(&self, game: &Game, player: &Player) -> bool {
        let acting_player = &game.acting_player;
        let acting_player_coordinate =
            match acting_player.player_id.as_deref().and_then(|id| game.field_model.player_coordinate(id)) {
                Some(c) => c,
                None => return false,
            };
        let player_coordinate = match game.field_model.player_coordinate(&player.id) {
            Some(c) => c,
            None => return false,
        };
        let distance = player_coordinate.distance_in_steps(acting_player_coordinate);
        let player_state = match game.field_model.player_state(&player.id) {
            Some(s) => s,
            None => return false,
        };

        distance <= 3
            && player_state.base() == ffb_model::enums::PS_STANDING
            && !game.active_team().has_player(&player.id)
    }

    /// java: `public boolean isEndPlayerActionAvailable()`.
    pub fn is_end_player_action_available(&self, game: &Game) -> bool {
        !game.acting_player.has_acted
    }
}

impl LogicModule for ThrowKegLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::ThrowKeg
    }

    /// java: `public void setUp()`.
    fn set_up(&mut self, client: &mut FantasyFootballClient) {
        let (coord, squares) = match client.game() {
            Some(game) => {
                let coord = game
                    .acting_player
                    .player_id
                    .as_deref()
                    .and_then(|id| game.player(id))
                    .and_then(|p| game.field_model.player_coordinate(&p.id));
                let squares: Vec<MoveSquare> = match coord {
                    Some(c) => find_adjacent_coordinates(c, 3)
                        .into_iter()
                        .map(|fc| MoveSquare::new(fc, 0, 0))
                        .collect(),
                    None => Vec::new(),
                };
                (coord, squares)
            }
            None => (None, Vec::new()),
        };
        let _ = coord;
        if let Some(game) = client.game_mut() {
            for square in squares {
                game.field_model.add_move_square(square);
            }
        }
    }

    /// java: `public void teardown()`.
    fn teardown(&mut self, client: &mut FantasyFootballClient) {
        if let Some(game) = client.game_mut() {
            game.field_model.clear_move_squares();
        }
    }

    /// java: `public Set<ClientAction> availableActions()`.
    fn available_actions(&self) -> HashSet<ClientAction> {
        let mut actions = HashSet::new();
        actions.insert(ClientAction::END_MOVE);
        actions.insert(ClientAction::TREACHEROUS);
        actions.insert(ClientAction::WISDOM);
        actions.insert(ClientAction::RAIDING_PARTY);
        actions.insert(ClientAction::LOOK_INTO_MY_EYES);
        actions.insert(ClientAction::BALEFUL_HEX);
        actions.insert(ClientAction::BLACK_INK);
        actions.insert(ClientAction::CATCH_OF_THE_DAY);
        actions.insert(ClientAction::AUTO_GAZE_ZOAT);
        actions
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)`.
    fn action_context(&self, game: &Game, acting_player: &ActingPlayer) -> ActionContext {
        let mut action_context = ActionContext::new();
        if self.is_end_player_action_available(game) {
            action_context.add_action(ClientAction::END_MOVE);
        }

        let player = acting_player.player_id.as_deref().and_then(|id| game.player(id));
        if let Some(player) = player {
            use crate::client::state::logic::logic_module::{
                is_baleful_hex_available, is_black_ink_available, is_catch_of_the_day_available,
                is_look_into_my_eyes_available, is_raiding_party_available, is_treacherous_available,
                is_wisdom_available, is_zoat_gaze_available,
            };
            if is_treacherous_available(game, player) {
                action_context.add_action(ClientAction::TREACHEROUS);
            }
            if is_wisdom_available(game, player) {
                action_context.add_action(ClientAction::WISDOM);
            }
            if is_raiding_party_available(game, player) {
                action_context.add_action(ClientAction::RAIDING_PARTY);
            }
            if is_look_into_my_eyes_available(game, player) {
                action_context.add_action(ClientAction::LOOK_INTO_MY_EYES);
            }
            if is_baleful_hex_available(game, player) {
                action_context.add_action(ClientAction::BALEFUL_HEX);
            }
            if is_black_ink_available(game, player) {
                action_context.add_action(ClientAction::BLACK_INK);
            }
            if is_catch_of_the_day_available(game, player) {
                action_context.add_action(ClientAction::CATCH_OF_THE_DAY);
            }
            if is_zoat_gaze_available(game, player) {
                action_context.add_action(ClientAction::AUTO_GAZE_ZOAT);
            }
        }
        action_context
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)`.
    fn perform_available_action(&mut self, client: &mut FantasyFootballClient, player: &Player, action: ClientAction) {
        let game = match client.game() {
            Some(g) => g,
            None => return,
        };
        match action {
            ClientAction::END_MOVE => {
                if self.is_end_player_action_available(game) {
                    client.communication_mut().send_acting_player(None, ffb_model::enums::PlayerAction::Move, false);
                }
            }
            ClientAction::TREACHEROUS => {
                if crate::client::state::logic::logic_module::is_treacherous_available(game, player) {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_STAB_TEAM_MATE_FOR_BALL) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::WISDOM => {
                if crate::client::state::logic::logic_module::is_wisdom_available(game, player) {
                    client.communication_mut().send_use_wisdom();
                }
            }
            ClientAction::RAIDING_PARTY => {
                if crate::client::state::logic::logic_module::is_raiding_party_available(game, player) {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_MOVE_OPEN_TEAM_MATE) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::LOOK_INTO_MY_EYES => {
                if crate::client::state::logic::logic_module::is_look_into_my_eyes_available(game, player) {
                    if let Some(skill_id) = ffb_model::util::util_cards::UtilCards::get_unused_skill_with_property(
                        player,
                        NamedProperties::CAN_STEAL_BALL_FROM_OPPONENT,
                    ) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::BALEFUL_HEX => {
                if crate::client::state::logic::logic_module::is_baleful_hex_available(game, player) {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_MAKE_OPPONENT_MISS_TURN) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::BLACK_INK => {
                if crate::client::state::logic::logic_module::is_black_ink_available(game, player) {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_GAZE_AUTOMATICALLY) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::CATCH_OF_THE_DAY => {
                if crate::client::state::logic::logic_module::is_catch_of_the_day_available(game, player) {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_GET_BALL_ON_GROUND) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::AUTO_GAZE_ZOAT => {
                if crate::client::state::logic::logic_module::is_zoat_gaze_available(game, player) {
                    if let Some(skill_id) = player
                        .skill_id_with_property(NamedProperties::CAN_GAZE_AUTOMATICALLY_THREE_SQUARES_AWAY)
                    {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            _ => {}
        }
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
    fn get_id_is_throw_keg() {
        assert_eq!(ThrowKegLogicModule::new().get_id(), ClientStateId::ThrowKeg);
    }

    #[test]
    fn available_actions_contains_end_move_and_wisdom() {
        let actions = ThrowKegLogicModule::new().available_actions();
        assert!(actions.contains(&ClientAction::END_MOVE));
        assert!(actions.contains(&ClientAction::WISDOM));
        assert_eq!(actions.len(), 9);
    }

    #[test]
    fn is_end_player_action_available_true_when_not_acted() {
        let module = ThrowKegLogicModule::new();
        let game = make_game();
        assert!(module.is_end_player_action_available(&game));
    }

    #[test]
    fn is_end_player_action_available_false_when_acted() {
        let module = ThrowKegLogicModule::new();
        let mut game = make_game();
        game.acting_player.has_acted = true;
        assert!(!module.is_end_player_action_available(&game));
    }

    #[test]
    fn is_valid_target_false_without_acting_player_coordinate() {
        let module = ThrowKegLogicModule::new();
        let mut game = make_game();
        add_player(&mut game, false, "t1", FieldCoordinate::new(1, 1));
        let target = game.player("t1").unwrap().clone();
        assert!(!module.is_valid_target(&game, &target));
    }

    #[test]
    fn is_valid_target_true_for_standing_opponent_within_three_squares() {
        let module = ThrowKegLogicModule::new();
        let mut game = make_game();
        add_player(&mut game, true, "attacker", FieldCoordinate::new(5, 5));
        add_player(&mut game, false, "t1", FieldCoordinate::new(6, 6));
        game.acting_player.player_id = Some("attacker".to_string());
        let target = game.player("t1").unwrap().clone();
        assert!(module.is_valid_target(&game, &target));
    }

    #[test]
    fn is_valid_target_false_for_own_team() {
        let module = ThrowKegLogicModule::new();
        let mut game = make_game();
        add_player(&mut game, true, "attacker", FieldCoordinate::new(5, 5));
        add_player(&mut game, true, "t1", FieldCoordinate::new(6, 6));
        game.acting_player.player_id = Some("attacker".to_string());
        let target = game.player("t1").unwrap().clone();
        assert!(!module.is_valid_target(&game, &target));
    }

    #[test]
    fn action_context_adds_end_move_when_available() {
        let module = ThrowKegLogicModule::new();
        let game = make_game();
        let ap = ActingPlayer::new();
        let ctx = module.action_context(&game, &ap);
        assert!(ctx.get_actions().contains(&ClientAction::END_MOVE));
    }

    #[test]
    fn set_up_and_teardown_do_not_panic_without_game() {
        let mut module = ThrowKegLogicModule::new();
        let mut client = make_client();
        module.set_up(&mut client);
        module.teardown(&mut client);
    }

    #[test]
    fn set_up_adds_move_squares_around_acting_player() {
        let mut module = ThrowKegLogicModule::new();
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, true, "attacker", FieldCoordinate::new(5, 5));
        game.acting_player.player_id = Some("attacker".to_string());
        client.set_game(game);
        module.set_up(&mut client);
        let count = client.game().unwrap().field_model.move_squares.len();
        assert!(count > 0);
    }

    #[test]
    fn player_interaction_ignores_without_game() {
        let mut client = make_client();
        let module = ThrowKegLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_interaction(&mut client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }
}
