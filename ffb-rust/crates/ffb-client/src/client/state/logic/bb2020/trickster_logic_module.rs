//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.bb2020.TricksterLogicModule`
//! (76 lines).
//!
//! Java's `TricksterLogicModule extends LogicModule`, overriding `getId`, `fieldInteraction`,
//! `playerInteraction`, `fieldPeek`, `performAvailableAction`, `availableActions`,
//! `actionContext`.

use ffb_model::enums::{ClientStateId, TurnMode};
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::types::FieldCoordinate;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::LogicModule;

/// 1:1 translation of the `TricksterLogicModule` class.
#[derive(Debug, Default)]
pub struct TricksterLogicModule;

impl TricksterLogicModule {
    /// java: `public TricksterLogicModule(FantasyFootballClient pClient)`.
    pub fn new() -> Self {
        Self
    }

    /// java: `public InteractionResult fieldInteraction(FieldCoordinate pCoordinate)`.
    pub fn field_interaction(&self, client: &mut FantasyFootballClient, coordinate: FieldCoordinate) -> InteractionResult {
        let has_move_square = client.game().and_then(|g| g.field_model.get_move_square(coordinate)).is_some();
        if has_move_square {
            client.communication_mut().send_field_coordinate(coordinate);
            InteractionResult::handled()
        } else {
            InteractionResult::ignore()
        }
    }

    /// java: `public InteractionResult playerInteraction(Player<?> player)`.
    pub fn player_interaction(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        let (is_defender, acting_player) = match client.game() {
            Some(game) => (game.defender_id.as_deref() == Some(player.id.as_str()), game.acting_player.clone()),
            None => return InteractionResult::ignore(),
        };
        if is_defender {
            let ctx = match client.game() {
                Some(game) => self.action_context(game, &acting_player),
                None => ActionContext::new(),
            };
            return InteractionResult::select_action(ctx);
        }
        InteractionResult::ignore()
    }

    /// java: `public InteractionResult fieldPeek(FieldCoordinate pCoordinate)`.
    pub fn field_peek(&self, client: &FantasyFootballClient, coordinate: FieldCoordinate) -> InteractionResult {
        let has_move_square = client.game().and_then(|g| g.field_model.get_move_square(coordinate)).is_some();
        if has_move_square {
            InteractionResult::perform()
        } else {
            InteractionResult::invalid()
        }
    }
}

impl LogicModule for TricksterLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::Trickster
    }

    /// java: `public Set<ClientAction> availableActions()`.
    fn available_actions(&self) -> std::collections::HashSet<ClientAction> {
        std::iter::once(ClientAction::END_MOVE).collect()
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)`.
    fn action_context(&self, _game: &Game, _acting_player: &ActingPlayer) -> ActionContext {
        let mut action_context = ActionContext::new();
        action_context.add_action(ClientAction::END_MOVE);
        action_context
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)`.
    fn perform_available_action(&mut self, client: &mut FantasyFootballClient, _player: &Player, action: ClientAction) {
        if action == ClientAction::END_MOVE {
            client.communication_mut().send_end_turn(TurnMode::Trickster);
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
    fn get_id_is_trickster() {
        assert_eq!(TricksterLogicModule::new().get_id(), ClientStateId::Trickster);
    }

    #[test]
    fn available_actions_is_end_move_only() {
        let actions = TricksterLogicModule::new().available_actions();
        assert_eq!(actions.len(), 1);
        assert!(actions.contains(&ClientAction::END_MOVE));
    }

    #[test]
    fn action_context_always_has_end_move() {
        let module = TricksterLogicModule::new();
        let game = make_game();
        let ap = ActingPlayer::new();
        let ctx = module.action_context(&game, &ap);
        assert_eq!(ctx.get_actions(), &vec![ClientAction::END_MOVE]);
    }

    #[test]
    fn field_interaction_ignores_without_move_square() {
        let module = TricksterLogicModule::new();
        let mut client = make_client();
        client.set_game(make_game());
        let result = module.field_interaction(&mut client, FieldCoordinate::new(1, 1));
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn field_peek_invalid_without_move_square() {
        let module = TricksterLogicModule::new();
        let mut client = make_client();
        client.set_game(make_game());
        let result = module.field_peek(&client, FieldCoordinate::new(1, 1));
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Invalid
        );
    }

    #[test]
    fn player_interaction_selects_action_for_defender() {
        let module = TricksterLogicModule::new();
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, false, "d1", FieldCoordinate::new(3, 3));
        game.defender_id = Some("d1".to_string());
        client.set_game(game);
        let player = client.game().unwrap().player("d1").unwrap().clone();
        let result = module.player_interaction(&mut client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::SelectAction
        );
    }

    #[test]
    fn player_interaction_ignores_non_defender() {
        let module = TricksterLogicModule::new();
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, false, "d1", FieldCoordinate::new(3, 3));
        client.set_game(game);
        let player = client.game().unwrap().player("d1").unwrap().clone();
        let result = module.player_interaction(&mut client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn perform_available_action_sends_end_turn_for_end_move() {
        let mut module = TricksterLogicModule::new();
        let mut client = make_client();
        let mut player = Player::default();
        player.id = "p1".to_string();
        module.perform_available_action(&mut client, &player, ClientAction::END_MOVE);
        assert!(!client.communication().is_stopped());
    }

    #[test]
    fn perform_available_action_no_op_for_other_actions() {
        let mut module = TricksterLogicModule::new();
        let mut client = make_client();
        let mut player = Player::default();
        player.id = "p1".to_string();
        module.perform_available_action(&mut client, &player, ClientAction::MOVE);
    }
}
