//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.bb2025.GazeLogicModule` (61 lines).
//!
//! Java's `GazeLogicModule extends MoveLogicModule`, overriding `getId`/`playerInteraction`/
//! `playerPeek` and adding one helper (`canBeGazed`). Per the `MoveLogicModule` convention
//! (see that module's own doc comment), the inherited `playerInteraction` needs `&mut
//! FantasyFootballClient`, so this is translated as a struct composing `MoveLogicModule` and
//! delegating via its inherent (non-trait) method.

use ffb_model::enums::ClientStateId;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::util::util_player::UtilPlayer;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::LogicModule;
use crate::client::state::logic::move_logic_module::MoveLogicModule;

/// java: `protected boolean canBeGazed(Player<?> pVictim)`.
pub fn can_be_gazed(game: &Game, victim: Option<&Player>) -> bool {
    let victim = match victim {
        Some(v) => v,
        None => return false,
    };
    let acting_player = &game.acting_player;
    let actor_id = match acting_player.player_id.as_deref() {
        Some(id) => id,
        None => return false,
    };
    let actor_coordinate = game.field_model.player_coordinate(actor_id);
    let victim_coordinate = game.field_model.player_coordinate(&victim.id);
    let actor_team = game.active_team();
    let victim_is_own_team = actor_team.has_player(&victim.id);

    UtilPlayer::can_gaze(game, actor_id)
        && victim_coordinate.is_some()
        && match (victim_coordinate, actor_coordinate) {
            (Some(v), Some(a)) => v.is_adjacent(a),
            _ => false,
        }
        && !victim_is_own_team
        && game
            .field_model
            .player_state(&victim.id)
            .map(|s| s.has_tacklezones())
            .unwrap_or(false)
}

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
            return self.move_logic.player_interaction(client, player);
        }

        let can_gaze = match client.game() {
            Some(game) => can_be_gazed(game, Some(player)),
            None => false,
        };
        if can_gaze {
            if let Some(id) = acting_player_id {
                client.communication_mut().send_gaze(id, Some(player));
            }
            return InteractionResult::handled();
        }
        InteractionResult::ignore()
    }

    /// java: `public InteractionResult playerPeek(Player<?> pPlayer)`.
    pub fn player_peek(&self, client: &FantasyFootballClient, player: &Player) -> InteractionResult {
        let can_gaze = match client.game() {
            Some(game) => can_be_gazed(game, Some(player)),
            None => false,
        };
        if can_gaze {
            InteractionResult::perform()
        } else {
            InteractionResult::ignore()
        }
    }
}

impl LogicModule for GazeLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::Gaze
    }

    /// java: `public Set<ClientAction> availableActions()` — inherited unchanged from
    /// `MoveLogicModule` (not overridden in Java).
    fn available_actions(&self) -> std::collections::HashSet<ClientAction> {
        self.move_logic.available_actions()
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)` — inherited
    /// unchanged from `MoveLogicModule` (not overridden in Java).
    fn action_context(&self, game: &Game, acting_player: &ffb_model::model::acting_player::ActingPlayer) -> ActionContext {
        self.move_logic.action_context(game, acting_player)
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)` —
    /// inherited unchanged from `MoveLogicModule` (not overridden in Java).
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
    fn get_id_is_gaze() {
        assert_eq!(GazeLogicModule::new().get_id(), ClientStateId::Gaze);
    }

    #[test]
    fn can_be_gazed_false_without_victim() {
        let game = make_game();
        assert!(!can_be_gazed(&game, None));
    }

    #[test]
    fn can_be_gazed_false_without_adjacency() {
        let mut game = make_game();
        add_player(&mut game, true, "attacker", FieldCoordinate::new(1, 1));
        add_player(&mut game, false, "victim", FieldCoordinate::new(10, 10));
        game.acting_player.player_id = Some("attacker".to_string());
        let victim = game.player("victim").unwrap().clone();
        assert!(!can_be_gazed(&game, Some(&victim)));
    }

    #[test]
    fn can_be_gazed_false_for_own_team_member() {
        let mut game = make_game();
        add_player(&mut game, true, "attacker", FieldCoordinate::new(1, 1));
        add_player(&mut game, true, "friend", FieldCoordinate::new(2, 1));
        game.acting_player.player_id = Some("attacker".to_string());
        let friend = game.player("friend").unwrap().clone();
        assert!(!can_be_gazed(&game, Some(&friend)));
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
    fn player_peek_ignores_without_game() {
        let client = make_client();
        let module = GazeLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_peek(&client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn available_actions_delegates_to_move_logic() {
        let module = GazeLogicModule::new();
        assert_eq!(module.available_actions(), MoveLogicModule::new().available_actions());
    }
}
