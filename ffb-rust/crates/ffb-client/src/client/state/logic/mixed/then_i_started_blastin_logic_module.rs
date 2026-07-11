//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.mixed.ThenIStartedBlastinLogicModule`
//! (131 lines).
//!
//! Java's `ThenIStartedBlastinLogicModule extends LogicModule` directly.
//!
//! Documented gaps:
//! - `FieldModel.findAdjacentCoordinates(FieldCoordinate, FieldCoordinateBounds, int steps,
//!   boolean withStart)` (the general form, not the `distance == 1` special case already used by
//!   `logic_module.rs`'s `adjacent_field_coordinates`) has no Rust `FieldModel` equivalent;
//!   reimplemented here as a local free function matching the Java body exactly.
//! - `Game.playingTeamHasActingPLayer()` / `Game.getDefender()` have no equivalents on the Rust
//!   `Game`; reimplemented here as local free functions from the pieces that do exist
//!   (`Team::has_player`, `game.defender_id` + `game.player(id)`).

use std::collections::HashSet;

use ffb_model::enums::{ClientStateId, PS_STANDING};
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds, MoveSquare};

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::influences::Influences;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::LogicModule;

/// java: `FieldModel.findAdjacentCoordinates(FieldCoordinate, FieldCoordinateBounds, int, boolean)`
/// — see module doc gap.
fn find_adjacent_coordinates(coordinate: FieldCoordinate, steps: i32, with_start: bool) -> Vec<FieldCoordinate> {
    let mut result = Vec::new();
    for y in -steps..=steps {
        for x in -steps..=steps {
            if x != 0 || y != 0 || with_start {
                let adjacent = FieldCoordinate::new(coordinate.x + x, coordinate.y + y);
                if FieldCoordinateBounds::FIELD.is_in_bounds(adjacent) {
                    result.push(adjacent);
                }
            }
        }
    }
    result
}

/// java: `Game.playingTeamHasActingPLayer()` — see module doc gap.
fn playing_team_has_acting_player(game: &Game) -> bool {
    match game.acting_player.player_id.as_deref() {
        Some(id) => game.active_team().has_player(id),
        None => false,
    }
}

/// java: `Game.getDefender()` — see module doc gap.
fn defender<'a>(game: &'a Game) -> Option<&'a Player> {
    game.defender_id.as_deref().and_then(|id| game.player(id))
}

/// 1:1 translation of the `ThenIStartedBlastinLogicModule` class.
#[derive(Debug, Default)]
pub struct ThenIStartedBlastinLogicModule;

impl ThenIStartedBlastinLogicModule {
    /// java: `public ThenIStartedBlastinLogicModule(FantasyFootballClient pClient)`.
    pub fn new() -> Self {
        Self
    }

    /// java: `public void setUp()`.
    pub fn set_up(&mut self, client: &mut FantasyFootballClient) {
        let source_id = match client.game() {
            Some(game) => {
                if playing_team_has_acting_player(game) {
                    game.acting_player.player_id.clone()
                } else {
                    game.defender_id.clone()
                }
            }
            None => return,
        };
        let source_id = match source_id {
            Some(id) => id,
            None => return,
        };
        let coord = match client.game() {
            Some(game) => game.field_model.player_coordinate(&source_id),
            None => None,
        };
        if let (Some(coord), Some(game)) = (coord, client.game_mut()) {
            for adjacent in find_adjacent_coordinates(coord, 3, false) {
                game.field_model.add_move_square(MoveSquare::new(adjacent, 0, 0));
            }
        }
    }

    /// java: `public void teardown()`.
    pub fn teardown(&mut self, client: &mut FantasyFootballClient) {
        if let Some(game) = client.game_mut() {
            game.field_model.clear_move_squares();
        }
    }

    /// java: `public InteractionResult playerInteraction(Player<?> player)`.
    pub fn player_interaction(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        let acting_player = match client.game() {
            Some(game) => game.acting_player.clone(),
            None => return InteractionResult::ignore(),
        };

        if acting_player.player_id.as_deref() == Some(player.id.as_str()) {
            let is_playing_team = client.game().map(playing_team_has_acting_player).unwrap_or(false);
            if is_playing_team {
                let ctx = match client.game() {
                    Some(game) => self.action_context(game, &acting_player),
                    None => ActionContext::new(),
                };
                return InteractionResult::select_action(ctx);
            }
        } else {
            let valid_target = client.game().map(|game| self.is_valid_target(game, player)).unwrap_or(false);
            if valid_target {
                client.communication_mut().send_target_selected(player.id.clone());
                return InteractionResult::handled();
            }
        }
        InteractionResult::ignore()
    }

    /// java: `public InteractionResult playerPeek(Player<?> player)`.
    pub fn player_peek(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        client.client_data_mut().set_selected_player(Some(player.id.clone()));
        let valid_target = client.game().map(|game| self.is_valid_target(game, player)).unwrap_or(false);
        if valid_target {
            InteractionResult::perform()
        } else {
            InteractionResult::invalid()
        }
    }

    /// java: `private boolean isValidTarget(Player<?> player, Game game)`.
    fn is_valid_target(&self, game: &Game, player: &Player) -> bool {
        let source_coordinate = if playing_team_has_acting_player(game) {
            game.acting_player.player_id.as_deref().and_then(|id| game.field_model.player_coordinate(id))
        } else {
            defender(game).and_then(|d| game.field_model.player_coordinate(&d.id))
        };
        let target_coordinate = match game.field_model.player_coordinate(&player.id) {
            Some(c) => c,
            None => return false,
        };
        let source_coordinate = match source_coordinate {
            Some(c) => c,
            None => return false,
        };
        let distance = target_coordinate.distance_in_steps(source_coordinate);

        let player_state = match game.field_model.player_state(&player.id) {
            Some(s) => s,
            None => return false,
        };

        let on_active_team = game.active_team().has_player(&player.id);

        distance <= 3
            && player_state.base() == PS_STANDING
            && (!on_active_team || !playing_team_has_acting_player(game))
    }

    /// java: `protected boolean isEndPlayerActionAvailable()` — a genuine override (see module
    /// doc comment); distinct from `logic_module::is_end_player_action_available`.
    fn is_end_player_action_available(&self, acting_player: &ActingPlayer) -> bool {
        !acting_player.has_acted
    }
}

impl LogicModule for ThenIStartedBlastinLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::ThenIStartedBlastin
    }

    /// java: `public Set<ClientAction> availableActions()`.
    fn available_actions(&self) -> HashSet<ClientAction> {
        let mut actions = HashSet::new();
        actions.insert(ClientAction::END_MOVE);
        actions
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)`.
    fn action_context(&self, _game: &Game, acting_player: &ActingPlayer) -> ActionContext {
        let mut action_context = ActionContext::new();
        if self.is_end_player_action_available(acting_player) {
            if acting_player.has_acted {
                action_context.add_influence(Influences::HAS_ACTED);
            }
            action_context.add_action(ClientAction::END_MOVE);
        }
        action_context
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)`.
    fn perform_available_action(&mut self, client: &mut FantasyFootballClient, _player: &Player, action: ClientAction) {
        if action == ClientAction::END_MOVE {
            let (available, turn_mode) = match client.game() {
                Some(game) => (self.is_end_player_action_available(&game.acting_player), Some(game.turn_mode)),
                None => (false, None),
            };
            if available {
                if let Some(turn_mode) = turn_mode {
                    client.communication_mut().send_end_turn(turn_mode);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerAction, Rules};
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
    fn get_id_is_then_i_started_blastin() {
        assert_eq!(ThenIStartedBlastinLogicModule::new().get_id(), ClientStateId::ThenIStartedBlastin);
    }

    #[test]
    fn available_actions_is_end_move_only() {
        let actions = ThenIStartedBlastinLogicModule::new().available_actions();
        assert_eq!(actions.len(), 1);
        assert!(actions.contains(&ClientAction::END_MOVE));
    }

    #[test]
    fn find_adjacent_coordinates_excludes_start_by_default() {
        let coords = find_adjacent_coordinates(FieldCoordinate::new(5, 5), 1, false);
        assert!(!coords.contains(&FieldCoordinate::new(5, 5)));
        assert!(coords.contains(&FieldCoordinate::new(6, 5)));
    }

    #[test]
    fn find_adjacent_coordinates_includes_start_when_requested() {
        let coords = find_adjacent_coordinates(FieldCoordinate::new(5, 5), 1, true);
        assert!(coords.contains(&FieldCoordinate::new(5, 5)));
    }

    #[test]
    fn action_context_adds_end_move_when_not_acted() {
        let module = ThenIStartedBlastinLogicModule::new();
        let game = make_game();
        let ap = ActingPlayer::new();
        let ctx = module.action_context(&game, &ap);
        assert!(ctx.get_actions().contains(&ClientAction::END_MOVE));
    }

    #[test]
    fn action_context_empty_when_already_acted() {
        let module = ThenIStartedBlastinLogicModule::new();
        let game = make_game();
        let mut ap = ActingPlayer::new();
        ap.has_acted = true;
        let ctx = module.action_context(&game, &ap);
        assert!(ctx.get_actions().is_empty());
    }

    #[test]
    fn is_valid_target_false_without_source_coordinate() {
        let module = ThenIStartedBlastinLogicModule::new();
        let mut game = make_game();
        add_player(&mut game, false, "a1", FieldCoordinate::new(5, 5));
        let target = game.player("a1").unwrap().clone();
        assert!(!module.is_valid_target(&game, &target));
    }

    #[test]
    fn is_valid_target_true_within_distance_and_standing() {
        let module = ThenIStartedBlastinLogicModule::new();
        let mut game = make_game();
        add_player(&mut game, true, "h1", FieldCoordinate::new(1, 1));
        add_player(&mut game, false, "a1", FieldCoordinate::new(2, 1));
        game.acting_player.player_id = Some("h1".to_string());
        game.acting_player.player_action = Some(PlayerAction::Blitz);
        let target = game.player("a1").unwrap().clone();
        assert!(module.is_valid_target(&game, &target));
    }

    #[test]
    fn player_interaction_ignores_without_game() {
        let mut client = make_client();
        let module = ThenIStartedBlastinLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_interaction(&mut client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn set_up_and_teardown_manage_move_squares() {
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, true, "h1", FieldCoordinate::new(5, 5));
        game.acting_player.player_id = Some("h1".to_string());
        client.set_game(game);
        let mut module = ThenIStartedBlastinLogicModule::new();
        module.set_up(&mut client);
        assert!(!client.game().unwrap().field_model.move_squares.is_empty());
        module.teardown(&mut client);
        assert!(client.game().unwrap().field_model.move_squares.is_empty());
    }

    #[test]
    fn perform_available_action_no_op_without_game() {
        let mut module = ThenIStartedBlastinLogicModule::new();
        let mut client = make_client();
        let mut player = Player::default();
        player.id = "p1".to_string();
        module.perform_available_action(&mut client, &player, ClientAction::END_MOVE);
    }

    #[test]
    fn defender_none_without_defender_id() {
        let game = make_game();
        assert!(defender(&game).is_none());
    }
}
