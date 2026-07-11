//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.KickoffReturnLogicModule` (100 lines).
//!
//! Java's `KickoffReturnLogicModule extends MoveLogicModule`, overriding every method it needs
//! (no inherited behavior is actually exercised besides `movePlayer`, delegated to a held
//! `MoveLogicModule` instance per the established batch convention).
//!
//! Documented gap:
//! - `protected ActionContext actionContext(ActingPlayer actingPlayer)` throws
//!   `UnsupportedOperationException` in Java (kickoff-return context is keyed by `Player`, not
//!   `ActingPlayer`); the `LogicModule::action_context` trait method mirrors this with
//!   `unimplemented!()`, and the real logic lives in the inherent
//!   `action_context_for_player(&self, client, player)` method (matching Java's overloaded
//!   `actionContext(Player<?> player)`).

use ffb_model::enums::{ClientStateId, PlayerAction};
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::types::FieldCoordinate;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::influences::Influences;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::LogicModule;
use crate::client::state::logic::move_logic_module::MoveLogicModule;

/// 1:1 translation of the `KickoffReturnLogicModule` class.
#[derive(Debug, Default)]
pub struct KickoffReturnLogicModule {
    move_logic: MoveLogicModule,
}

impl KickoffReturnLogicModule {
    /// java: `public KickoffReturnLogicModule(FantasyFootballClient pClient)`.
    pub fn new() -> Self {
        Self { move_logic: MoveLogicModule::new() }
    }

    /// java: `public InteractionResult playerInteraction(Player<?> player)`.
    pub fn player_interaction(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        let (has_player, is_active) = match client.game() {
            Some(game) => {
                let player_state = game.field_model.player_state(&player.id);
                (game.team_home.has_player(&player.id), player_state.map(|s| s.is_active()).unwrap_or(false))
            }
            None => return InteractionResult::ignore(),
        };
        if has_player && is_active {
            let ctx = self.action_context_for_player(client, player);
            return InteractionResult::select_action(ctx);
        }
        InteractionResult::ignore()
    }

    /// java: `public InteractionResult fieldInteraction(FieldCoordinate pCoordinate)`.
    pub fn field_interaction(&self, client: &mut FantasyFootballClient, coordinate: FieldCoordinate) -> InteractionResult {
        let has_move_square = match client.game() {
            Some(game) => game.field_model.get_move_square(coordinate).is_some(),
            None => false,
        };
        if has_move_square && self.move_logic.move_player(client, coordinate) {
            return InteractionResult::handled();
        }
        InteractionResult::ignore()
    }

    /// java: `protected ActionContext actionContext(Player<?> player)`.
    pub fn action_context_for_player(&self, client: &FantasyFootballClient, player: &Player) -> ActionContext {
        let mut action_context = ActionContext::new();
        let game = match client.game() {
            Some(g) => g,
            None => return action_context,
        };
        let acting_player = &game.acting_player;
        let player_state = game.field_model.player_state(&player.id);

        if acting_player.player_id.is_none() && player_state.map(|s| s.is_able_to_move()).unwrap_or(false) {
            action_context.add_action(ClientAction::MOVE);
        }
        if acting_player.player_id.as_deref() == Some(player.id.as_str()) {
            if acting_player.has_acted {
                action_context.add_influence(Influences::HAS_ACTED);
            }
            action_context.add_action(ClientAction::END_MOVE);
        }
        action_context
    }
}

impl LogicModule for KickoffReturnLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::KickoffReturn
    }

    /// java: `public Set<ClientAction> availableActions()`.
    fn available_actions(&self) -> std::collections::HashSet<ClientAction> {
        let mut actions = std::collections::HashSet::new();
        actions.insert(ClientAction::END_MOVE);
        actions.insert(ClientAction::MOVE);
        actions
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)` — see module
    /// doc gap; unsupported in the kickoff-return context.
    fn action_context(&self, _game: &Game, _acting_player: &ActingPlayer) -> ActionContext {
        unimplemented!("actionContext for acting player is not supported in kick off return context")
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)`.
    fn perform_available_action(&mut self, client: &mut FantasyFootballClient, player: &Player, action: ClientAction) {
        match action {
            ClientAction::MOVE => {
                client.communication_mut().send_acting_player(Some(player), PlayerAction::Move, false);
            }
            ClientAction::END_MOVE => {
                // java: `communication.sendActingPlayer(null, null, false);` — see
                // `LogicModule::deselect_acting_player`'s documented gap.
            }
            _ => {}
        }
    }

    /// java: `public void endTurn()`.
    fn end_turn(&mut self, client: &mut FantasyFootballClient) {
        let turn_mode = match client.game() {
            Some(game) => game.turn_mode,
            None => return,
        };
        client.communication_mut().send_end_turn(turn_mode);
        client.client_data_mut().set_end_turn_button_hidden(true);
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
    fn get_id_is_kickoff_return() {
        assert_eq!(KickoffReturnLogicModule::new().get_id(), ClientStateId::KickoffReturn);
    }

    #[test]
    fn available_actions_is_move_and_end_move() {
        let actions = KickoffReturnLogicModule::new().available_actions();
        assert_eq!(actions.len(), 2);
        assert!(actions.contains(&ClientAction::MOVE));
        assert!(actions.contains(&ClientAction::END_MOVE));
    }

    #[test]
    #[should_panic]
    fn action_context_for_acting_player_is_unsupported() {
        let module = KickoffReturnLogicModule::new();
        let game = make_game();
        let ap = ActingPlayer::new();
        let _ = module.action_context(&game, &ap);
    }

    #[test]
    fn action_context_for_player_adds_move_when_no_acting_player() {
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(2, 2));
        client.set_game(game);
        let module = KickoffReturnLogicModule::new();
        let player = client.game().unwrap().player("p1").unwrap().clone();
        let ctx = module.action_context_for_player(&client, &player);
        assert!(ctx.get_actions().contains(&ClientAction::MOVE));
    }

    #[test]
    fn player_interaction_ignores_when_not_home_team() {
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, false, "a1", FieldCoordinate::new(2, 2));
        client.set_game(game);
        let module = KickoffReturnLogicModule::new();
        let player = client.game().unwrap().player("a1").unwrap().clone();
        let result = module.player_interaction(&mut client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn field_interaction_ignores_without_move_square() {
        let mut client = make_client();
        client.set_game(make_game());
        let module = KickoffReturnLogicModule::new();
        let result = module.field_interaction(&mut client, FieldCoordinate::new(5, 5));
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn end_turn_hides_end_turn_button() {
        let mut module = KickoffReturnLogicModule::new();
        let mut client = make_client();
        client.set_game(make_game());
        module.end_turn(&mut client);
        assert!(client.client_data().is_end_turn_button_hidden());
    }

    #[test]
    fn perform_available_action_move_sends_command() {
        let mut module = KickoffReturnLogicModule::new();
        let mut client = make_client();
        client.set_game(make_game());
        let mut player = Player::default();
        player.id = "p1".to_string();
        module.perform_available_action(&mut client, &player, ClientAction::MOVE);
    }
}
