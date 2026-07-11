//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.mixed.GazeLogicModule` (73 lines).
//!
//! Java's `GazeLogicModule extends MoveLogicModule`, composing a held `MoveLogicModule`
//! instance and delegating to it for inherited behavior, matching the established
//! `BlitzLogicModule`/`FoulLogicModule` convention.
//!
//! Documented gap:
//! - Java doesn't override `endTurn()`, so it inherits `MoveLogicModule.endTurn()`; the trait
//!   impl below delegates to the held `move_logic`'s own `end_turn` via fully-qualified syntax
//!   to reproduce that inheritance (same pattern as `foul_logic_module.rs`).

use ffb_model::enums::ClientStateId;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::util_player::UtilPlayer;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::LogicModule;
use crate::client::state::logic::move_logic_module::MoveLogicModule;

/// 1:1 translation of the `GazeLogicModule` class.
#[derive(Debug, Default)]
pub struct GazeLogicModule {
    move_logic: MoveLogicModule,
}

impl GazeLogicModule {
    /// java: `public GazeLogicModule(FantasyFootballClient pClient)`.
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
        } else if can_be_gazed(client, Some(player)) {
            let acting_player_id = match client.game() {
                Some(game) => game.acting_player.player_id.clone(),
                None => return InteractionResult::ignore(),
            };
            if let Some(id) = acting_player_id {
                client.communication_mut().send_gaze(id, Some(player));
            }
            InteractionResult::handled()
        } else {
            InteractionResult::ignore()
        }
    }

    /// java: `public InteractionResult playerPeek(Player<?> pPlayer)`.
    pub fn player_peek(&self, client: &FantasyFootballClient, player: &Player) -> InteractionResult {
        if can_be_gazed(client, Some(player)) {
            InteractionResult::perform()
        } else {
            InteractionResult::invalid()
        }
    }

    /// java: `public boolean playerActivationUsed()`.
    pub fn player_activation_used(&self, client: &FantasyFootballClient) -> bool {
        let field_model = match client.game() {
            Some(game) => &game.field_model,
            None => return <Self as LogicModule>::player_activation_used(self, client),
        };
        match &field_model.target_selection_state {
            Some(state) => state.is_committed(),
            None => <Self as LogicModule>::player_activation_used(self, client),
        }
    }
}

/// java: `private boolean canBeGazed(Player<?> pVictim)`.
///
/// Added a check to see if the player had tacklezones so no prone players could be gazed or
/// already-gazed players.
fn can_be_gazed(client: &FantasyFootballClient, victim: Option<&Player>) -> bool {
    let victim = match victim {
        Some(v) => v,
        None => return false,
    };
    let game = match client.game() {
        Some(g) => g,
        None => return false,
    };
    let acting_player = &game.acting_player;
    let actor_id = match acting_player.player_id.as_deref() {
        Some(id) => id,
        None => return false,
    };
    let actor_coordinate = game.field_model.player_coordinate(actor_id);
    let victim_coordinate = game.field_model.player_coordinate(&victim.id);
    let actor_team = if game.team_home.has_player(actor_id) { &game.team_home } else { &game.team_away };
    let victim_team = if game.team_home.has_player(&victim.id) { &game.team_home } else { &game.team_away };
    let has_tacklezones =
        game.field_model.player_state(&victim.id).map(|s| s.has_tacklezones()).unwrap_or(false);

    UtilPlayer::can_gaze(game, actor_id)
        && victim_coordinate.is_some()
        && match (victim_coordinate, actor_coordinate) {
            (Some(v), Some(a)) => v.is_adjacent(a),
            _ => false,
        }
        && !std::ptr::eq(actor_team, victim_team)
        && has_tacklezones
}

impl LogicModule for GazeLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::Gaze
    }

    /// java: `public Set<ClientAction> availableActions()` — not overridden in Java, inherited
    /// unchanged from `MoveLogicModule`.
    fn available_actions(&self) -> std::collections::HashSet<ClientAction> {
        self.move_logic.available_actions()
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)` — not
    /// overridden in Java, inherited unchanged from `MoveLogicModule`.
    fn action_context(&self, game: &Game, acting_player: &ffb_model::model::acting_player::ActingPlayer) -> ActionContext {
        self.move_logic.action_context(game, acting_player)
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)` —
    /// not overridden in Java, inherited unchanged from `MoveLogicModule`.
    fn perform_available_action(&mut self, client: &mut FantasyFootballClient, player: &Player, action: ClientAction) {
        self.move_logic.perform_available_action(client, player, action);
    }

    /// java: not overridden — inherited from `MoveLogicModule.endTurn()`; see module doc gap.
    fn end_turn(&mut self, client: &mut FantasyFootballClient) {
        <MoveLogicModule as LogicModule>::end_turn(&mut self.move_logic, client);
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
    fn get_id_is_gaze() {
        assert_eq!(GazeLogicModule::new().get_id(), ClientStateId::Gaze);
    }

    #[test]
    fn available_actions_delegates_to_move_logic() {
        let module = GazeLogicModule::new();
        let actions = module.available_actions();
        assert!(actions.contains(&ClientAction::MOVE));
    }

    #[test]
    fn can_be_gazed_false_without_victim() {
        let client = make_client();
        assert!(!can_be_gazed(&client, None));
    }

    #[test]
    fn can_be_gazed_false_without_adjacency() {
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, true, "actor", FieldCoordinate::new(1, 1));
        add_player(&mut game, false, "victim", FieldCoordinate::new(10, 10));
        game.acting_player.player_id = Some("actor".to_string());
        client.set_game(game);
        let victim = client.game().unwrap().player("victim").unwrap().clone();
        assert!(!can_be_gazed(&client, Some(&victim)));
    }

    #[test]
    fn player_activation_used_falls_back_without_target_selection_state() {
        let mut client = make_client();
        client.set_game(make_game());
        let module = GazeLogicModule::new();
        assert!(!module.player_activation_used(&client));
    }

    #[test]
    fn player_peek_invalid_when_not_gazeable() {
        let mut client = make_client();
        client.set_game(make_game());
        let module = GazeLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_peek(&client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Invalid
        );
    }

    #[test]
    fn player_interaction_ignores_without_game() {
        let mut client = make_client();
        let module = GazeLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_interaction(&mut client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn perform_available_action_delegates_without_panicking() {
        let mut module = GazeLogicModule::new();
        let mut client = make_client();
        client.set_game(make_game());
        let mut player = Player::default();
        player.id = "p1".to_string();
        module.perform_available_action(&mut client, &player, ClientAction::END_MOVE);
    }

    #[test]
    fn end_turn_no_op_without_game() {
        let mut module = GazeLogicModule::new();
        let mut client = make_client();
        module.end_turn(&mut client);
    }
}
