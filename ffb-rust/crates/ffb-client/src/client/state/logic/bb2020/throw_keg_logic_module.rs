//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.bb2020.ThrowKegLogicModule`
//! (204 lines).
//!
//! Java's `ThrowKegLogicModule extends LogicModule`, overriding `getId`, `setUp`, `teardown`,
//! `playerInteraction`, `playerPeek`, `availableActions`, `actionContext`,
//! `performAvailableAction`, and adding a private `isValidTarget`/public `isEndPlayerActionAvailable`
//! helper.
//!
//! Documented gap: `FieldModel.findAdjacentCoordinates(FieldCoordinate, FieldCoordinateBounds.FIELD,
//! distance, false)` has no shared public helper on the Rust `FieldModel` for `distance != 1`
//! (only `adjacent_on_pitch`, distance == 1); mirrored here as a local
//! `adjacent_field_coordinates` helper, matching the private duplicate already used in
//! `ffb-model/src/util/util_player.rs`.

use ffb_model::enums::ClientStateId;
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::property::NamedProperties;
use ffb_model::types::{FieldCoordinate, MoveSquare};
use ffb_model::util::util_cards::UtilCards;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::{self, LogicModule};

/// java: `player.getSkillWithProperty(property)` — see `block_logic_extension.rs`'s doc gap.
fn skill_placeholder(id: ffb_model::enums::SkillId) -> ffb_model::model::skill::skill::Skill {
    ffb_model::model::skill::skill::Skill::new(id.class_name(), ffb_model::enums::SkillCategory::General)
}

/// java: `FieldModel.findAdjacentCoordinates(FieldCoordinate, FieldCoordinateBounds.FIELD,
/// distance, false)` — see module doc gap.
fn adjacent_field_coordinates(game: &Game, coord: FieldCoordinate, distance: i32) -> Vec<FieldCoordinate> {
    if distance == 1 {
        return game.field_model.adjacent_on_pitch(coord);
    }
    let mut result = Vec::new();
    for dx in -distance..=distance {
        for dy in -distance..=distance {
            if dx == 0 && dy == 0 {
                continue;
            }
            let c = FieldCoordinate::new(coord.x + dx, coord.y + dy);
            if c.is_on_pitch() {
                result.push(c);
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
        let (acting_player, is_acting_player) = match client.game() {
            Some(game) => {
                let acting_player = game.acting_player.clone();
                let is_acting = acting_player.player_id.as_deref() == Some(player.id.as_str());
                (acting_player, is_acting)
            }
            None => return InteractionResult::ignore(),
        };
        if is_acting_player {
            let ctx = match client.game() {
                Some(game) => self.action_context(game, &acting_player),
                None => ActionContext::new(),
            };
            return InteractionResult::select_action(ctx);
        }
        let valid = client.game().map(|g| is_valid_target(g, player)).unwrap_or(false);
        if valid {
            client.communication_mut().send_throw_keg(player);
            return InteractionResult::handled();
        }
        InteractionResult::ignore()
    }

    /// java: `public InteractionResult playerPeek(Player<?> player)`.
    pub fn player_peek(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        client.client_data_mut().set_selected_player(Some(player.id.clone()));
        let valid = client.game().map(|g| is_valid_target(g, player)).unwrap_or(false);
        if valid {
            InteractionResult::perform()
        } else {
            InteractionResult::invalid()
        }
    }

    /// java: `public boolean isEndPlayerActionAvailable()`.
    pub fn is_end_player_action_available(&self, client: &FantasyFootballClient) -> bool {
        client.game().map(|g| !g.acting_player.has_acted).unwrap_or(false)
    }
}

/// java: `private boolean isValidTarget(Player<?> player, Game game)`.
fn is_valid_target(game: &Game, player: &Player) -> bool {
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
        && game.player_team_id(&player.id) != Some(game.active_team().id.as_str())
}

impl LogicModule for ThrowKegLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::ThrowKeg
    }

    /// java: `public void setUp()`.
    fn set_up(&mut self, client: &mut FantasyFootballClient) {
        let (coordinate, squares) = match client.game() {
            Some(game) => {
                let coordinate = game
                    .acting_player
                    .player_id
                    .as_deref()
                    .and_then(|id| game.player(id))
                    .and_then(|player| game.field_model.player_coordinate(&player.id));
                let squares = match coordinate {
                    Some(coord) => adjacent_field_coordinates(game, coord, 3)
                        .into_iter()
                        .map(|c| MoveSquare::new(c, 0, 0))
                        .collect(),
                    None => Vec::new(),
                };
                (coordinate, squares)
            }
            None => return,
        };
        let _ = coordinate;
        if let Some(game) = client.game_mut() {
            let squares: Vec<MoveSquare> = squares;
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
    fn available_actions(&self) -> std::collections::HashSet<ClientAction> {
        let mut actions = std::collections::HashSet::new();
        actions.insert(ClientAction::END_MOVE);
        actions.insert(ClientAction::TREACHEROUS);
        actions.insert(ClientAction::WISDOM);
        actions.insert(ClientAction::RAIDING_PARTY);
        actions.insert(ClientAction::LOOK_INTO_MY_EYES);
        actions.insert(ClientAction::BALEFUL_HEX);
        actions.insert(ClientAction::BLACK_INK);
        actions.insert(ClientAction::CATCH_OF_THE_DAY);
        actions.insert(ClientAction::THEN_I_STARTED_BLASTIN);
        actions
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)`.
    fn action_context(&self, game: &Game, acting_player: &ActingPlayer) -> ActionContext {
        let mut action_context = ActionContext::new();
        if !acting_player.has_acted {
            action_context.add_action(ClientAction::END_MOVE);
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
        if logic_module::is_look_into_my_eyes_available_ap(game, acting_player) {
            action_context.add_action(ClientAction::LOOK_INTO_MY_EYES);
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
        action_context
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)`.
    fn perform_available_action(&mut self, client: &mut FantasyFootballClient, player: &Player, action: ClientAction) {
        match action {
            ClientAction::END_MOVE => {
                if self.is_end_player_action_available(client) {
                    // java: `communication.sendActingPlayer(null, null, false);` — see
                    // `LogicModule::deselect_acting_player`'s documented gap.
                }
            }
            ClientAction::TREACHEROUS => {
                if client.game().map(|g| logic_module::is_treacherous_available(g, player)).unwrap_or(false) {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_STAB_TEAM_MATE_FOR_BALL) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::WISDOM => {
                if client.game().map(|g| logic_module::is_wisdom_available(g, player)).unwrap_or(false) {
                    client.communication_mut().send_use_wisdom();
                }
            }
            ClientAction::RAIDING_PARTY => {
                if client.game().map(|g| logic_module::is_raiding_party_available(g, player)).unwrap_or(false) {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_MOVE_OPEN_TEAM_MATE) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::LOOK_INTO_MY_EYES => {
                if client.game().map(|g| logic_module::is_look_into_my_eyes_available(g, player)).unwrap_or(false) {
                    if let Some(skill_id) = UtilCards::get_unused_skill_with_property(
                        player,
                        NamedProperties::CAN_STEAL_BALL_FROM_OPPONENT,
                    ) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::BALEFUL_HEX => {
                if client.game().map(|g| logic_module::is_baleful_hex_available(g, player)).unwrap_or(false) {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_MAKE_OPPONENT_MISS_TURN) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::BLACK_INK => {
                if client.game().map(|g| logic_module::is_black_ink_available(g, player)).unwrap_or(false) {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_GAZE_AUTOMATICALLY) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::CATCH_OF_THE_DAY => {
                if client.game().map(|g| logic_module::is_catch_of_the_day_available(g, player)).unwrap_or(false) {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_GET_BALL_ON_GROUND) {
                        client.communication_mut().send_use_skill(&skill_placeholder(skill_id), true, player.id.clone());
                    }
                }
            }
            ClientAction::THEN_I_STARTED_BLASTIN => {
                if client.game().map(|g| logic_module::is_then_i_started_blastin_available(g, player)).unwrap_or(false) {
                    if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_BLAST_REMOTE_PLAYER) {
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
    fn available_actions_matches_java() {
        let actions = ThrowKegLogicModule::new().available_actions();
        assert!(actions.contains(&ClientAction::END_MOVE));
        assert_eq!(actions.len(), 9);
    }

    #[test]
    fn is_end_player_action_available_true_without_acting() {
        let module = ThrowKegLogicModule::new();
        let mut client = make_client();
        client.set_game(make_game());
        assert!(module.is_end_player_action_available(&client));
    }

    #[test]
    fn is_valid_target_false_without_positions() {
        let game = make_game();
        let mut player = Player::default();
        player.id = "p1".to_string();
        assert!(!is_valid_target(&game, &player));
    }

    #[test]
    fn is_valid_target_true_for_close_standing_opponent() {
        let mut game = make_game();
        add_player(&mut game, true, "attacker", FieldCoordinate::new(5, 5));
        add_player(&mut game, false, "defender", FieldCoordinate::new(6, 6));
        game.acting_player.player_id = Some("attacker".to_string());
        let player = game.player("defender").unwrap().clone();
        assert!(is_valid_target(&game, &player));
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
        assert!(!client.game().unwrap().field_model.move_squares.is_empty());
    }

    #[test]
    fn teardown_clears_move_squares() {
        let mut module = ThrowKegLogicModule::new();
        let mut client = make_client();
        let mut game = make_game();
        game.field_model.add_move_square(MoveSquare::new(FieldCoordinate::new(1, 1), 0, 0));
        client.set_game(game);
        module.teardown(&mut client);
        assert!(client.game().unwrap().field_model.move_squares.is_empty());
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
