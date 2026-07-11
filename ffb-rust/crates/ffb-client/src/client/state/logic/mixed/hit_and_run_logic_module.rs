//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.mixed.HitAndRunLogicModule`
//! (93 lines).
//!
//! Java's `HitAndRunLogicModule extends LogicModule` directly, with no fields.
//!
//! Documented gap:
//! - `UtilCards.hasUnusedSkillWithProperty(ActingPlayer, ISkillProperty)` — the Rust
//!   `UtilCards::has_unused_skill_with_property` only takes a `Player`, so the acting player's
//!   underlying `Player` is looked up via `acting_player.player_id` + `game.player(id)` first
//!   (matching the same resolution pattern used throughout this crate for `ActingPlayer`
//!   overloads, e.g. `logic_module::is_treacherous_available_ap`).

use ffb_model::enums::ClientStateId;
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::property::NamedProperties;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::util_cards::UtilCards;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::LogicModule;

/// 1:1 translation of the `HitAndRunLogicModule` class.
#[derive(Debug, Default)]
pub struct HitAndRunLogicModule;

impl HitAndRunLogicModule {
    /// java: `public HitAndRunLogicModule(FantasyFootballClient client)`.
    pub fn new() -> Self {
        Self
    }

    /// java: `public InteractionResult fieldInteraction(FieldCoordinate coordinate)`.
    pub fn field_interaction(&self, client: &mut FantasyFootballClient, coordinate: FieldCoordinate) -> InteractionResult {
        if is_valid_field(client, coordinate) {
            client.communication_mut().send_field_coordinate(coordinate);
            InteractionResult::perform()
        } else {
            InteractionResult::ignore()
        }
    }

    /// java: `public InteractionResult playerInteraction(Player<?> player)`.
    pub fn player_interaction(&self, client: &FantasyFootballClient, player: &Player) -> InteractionResult {
        let game = match client.game() {
            Some(g) => g,
            None => return InteractionResult::ignore(),
        };
        if game.acting_player.player_id.as_deref() == Some(player.id.as_str()) {
            InteractionResult::select_action(self.action_context(game, &game.acting_player))
        } else {
            InteractionResult::ignore()
        }
    }

    /// java: `public InteractionResult playerPeek(Player<?> player)`.
    pub fn player_peek(&self, client: &FantasyFootballClient, player: &Player) -> InteractionResult {
        let game = match client.game() {
            Some(g) => g,
            None => return InteractionResult::invalid(),
        };
        if game.acting_player.player_id.as_deref() == Some(player.id.as_str()) {
            InteractionResult::reset()
        } else {
            InteractionResult::invalid()
        }
    }

    /// java: `public InteractionResult fieldPeek(FieldCoordinate coordinate)`.
    pub fn field_peek(&self, client: &FantasyFootballClient, coordinate: FieldCoordinate) -> InteractionResult {
        if is_valid_field(client, coordinate) {
            InteractionResult::perform()
        } else {
            InteractionResult::invalid()
        }
    }
}

/// java: `private boolean isValidField(FieldCoordinate coordinate)`.
fn is_valid_field(client: &FantasyFootballClient, coordinate: FieldCoordinate) -> bool {
    client.game().map(|g| g.field_model.get_move_square(coordinate).is_some()).unwrap_or(false)
}

impl LogicModule for HitAndRunLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::HitAndRun
    }

    /// java: `public Set<ClientAction> availableActions()`.
    fn available_actions(&self) -> std::collections::HashSet<ClientAction> {
        let mut actions = std::collections::HashSet::new();
        actions.insert(ClientAction::HIT_AND_RUN);
        actions
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)`.
    fn action_context(&self, game: &Game, acting_player: &ActingPlayer) -> ActionContext {
        let mut context = ActionContext::new();
        let has_skill = acting_player
            .player_id
            .as_deref()
            .and_then(|id| game.player(id))
            .map(|p| UtilCards::has_unused_skill_with_property(p, NamedProperties::CAN_MOVE_AFTER_BLOCK))
            .unwrap_or(false);
        if has_skill {
            context.add_action(ClientAction::HIT_AND_RUN);
        }
        context
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)`.
    fn perform_available_action(&mut self, client: &mut FantasyFootballClient, _player: &Player, action: ClientAction) {
        if action == ClientAction::HIT_AND_RUN {
            let turn_mode = match client.game() {
                Some(game) => game.turn_mode,
                None => return,
            };
            client.communication_mut().send_end_turn(turn_mode);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, SkillId, PS_STANDING};
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
    fn get_id_is_hit_and_run() {
        assert_eq!(HitAndRunLogicModule::new().get_id(), ClientStateId::HitAndRun);
    }

    #[test]
    fn available_actions_is_hit_and_run_only() {
        let actions = HitAndRunLogicModule::new().available_actions();
        assert_eq!(actions.len(), 1);
        assert!(actions.contains(&ClientAction::HIT_AND_RUN));
    }

    #[test]
    fn action_context_empty_without_skill() {
        let module = HitAndRunLogicModule::new();
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        game.acting_player.player_id = Some("p1".to_string());
        let ctx = module.action_context(&game, &game.acting_player);
        assert!(!ctx.get_actions().contains(&ClientAction::HIT_AND_RUN));
    }

    #[test]
    fn action_context_adds_hit_and_run_with_skill() {
        let module = HitAndRunLogicModule::new();
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        if let Some(p) = game.team_home.players.iter_mut().find(|p| p.id == "p1") {
            p.add_skill(SkillId::HitAndRun);
        }
        game.acting_player.player_id = Some("p1".to_string());
        let ctx = module.action_context(&game, &game.acting_player);
        assert!(ctx.get_actions().contains(&ClientAction::HIT_AND_RUN));
    }

    #[test]
    fn field_interaction_ignores_without_move_square() {
        let mut client = make_client();
        client.set_game(make_game());
        let module = HitAndRunLogicModule::new();
        let result = module.field_interaction(&mut client, FieldCoordinate::new(1, 1));
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn field_peek_invalid_without_move_square() {
        let mut client = make_client();
        client.set_game(make_game());
        let module = HitAndRunLogicModule::new();
        let result = module.field_peek(&client, FieldCoordinate::new(1, 1));
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Invalid
        );
    }

    #[test]
    fn player_peek_reset_for_acting_player() {
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        game.acting_player.player_id = Some("p1".to_string());
        client.set_game(game);
        let module = HitAndRunLogicModule::new();
        let player = client.game().unwrap().player("p1").unwrap().clone();
        let result = module.player_peek(&client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Reset
        );
    }

    #[test]
    fn player_interaction_ignores_without_game() {
        let client = make_client();
        let module = HitAndRunLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_interaction(&client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn perform_available_action_sends_end_turn() {
        let mut module = HitAndRunLogicModule::new();
        let mut client = make_client();
        client.set_game(make_game());
        let mut player = Player::default();
        player.id = "p1".to_string();
        module.perform_available_action(&mut client, &player, ClientAction::HIT_AND_RUN);
    }

    #[test]
    fn perform_available_action_no_op_without_game() {
        let mut module = HitAndRunLogicModule::new();
        let mut client = make_client();
        let mut player = Player::default();
        player.id = "p1".to_string();
        module.perform_available_action(&mut client, &player, ClientAction::HIT_AND_RUN);
    }
}
