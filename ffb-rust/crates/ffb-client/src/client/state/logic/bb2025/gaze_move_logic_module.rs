//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.bb2025.GazeMoveLogicModule` (16 lines).
//!
//! Java's `GazeMoveLogicModule extends GazeLogicModule`, overriding only `getId()`. Translated
//! as a struct composing `GazeLogicModule` and delegating everything else to it.

use ffb_model::enums::ClientStateId;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::bb2025::gaze_logic_module::GazeLogicModule;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::LogicModule;

/// 1:1 translation of the `GazeMoveLogicModule` class.
#[derive(Debug, Default)]
pub struct GazeMoveLogicModule {
    gaze: GazeLogicModule,
}

impl GazeMoveLogicModule {
    /// java: `public GazeMoveLogicModule(FantasyFootballClient client)`.
    pub fn new() -> Self {
        Self { gaze: GazeLogicModule::new() }
    }

    /// java: `public InteractionResult playerInteraction(Player<?> player)` — inherited
    /// unchanged from `GazeLogicModule` (not overridden in Java).
    pub fn player_interaction(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        self.gaze.player_interaction(client, player)
    }

    /// java: `public InteractionResult playerPeek(Player<?> pPlayer)` — inherited unchanged
    /// from `GazeLogicModule` (not overridden in Java).
    pub fn player_peek(&self, client: &FantasyFootballClient, player: &Player) -> InteractionResult {
        self.gaze.player_peek(client, player)
    }
}

impl LogicModule for GazeMoveLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::GazeMove
    }

    /// java: `public Set<ClientAction> availableActions()` — inherited unchanged from
    /// `GazeLogicModule`/`MoveLogicModule` (not overridden in Java).
    fn available_actions(&self) -> std::collections::HashSet<ClientAction> {
        self.gaze.available_actions()
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)` — inherited
    /// unchanged (not overridden in Java).
    fn action_context(&self, game: &Game, acting_player: &ffb_model::model::acting_player::ActingPlayer) -> ActionContext {
        self.gaze.action_context(game, acting_player)
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)` —
    /// inherited unchanged (not overridden in Java).
    fn perform_available_action(&mut self, client: &mut FantasyFootballClient, player: &Player, action: ClientAction) {
        self.gaze.perform_available_action(client, player, action);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn get_id_is_gaze_move() {
        assert_eq!(GazeMoveLogicModule::new().get_id(), ClientStateId::GazeMove);
    }

    #[test]
    fn available_actions_delegates_to_gaze_logic() {
        let module = GazeMoveLogicModule::new();
        assert_eq!(module.available_actions(), GazeLogicModule::new().available_actions());
    }

    #[test]
    fn player_interaction_ignores_without_game() {
        let mut client = make_client();
        let module = GazeMoveLogicModule::new();
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
        let module = GazeMoveLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_peek(&client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn perform_available_action_no_op_without_game() {
        let mut module = GazeMoveLogicModule::new();
        let mut client = make_client();
        let mut player = Player::default();
        player.id = "p1".to_string();
        module.perform_available_action(&mut client, &player, ClientAction::MOVE);
    }
}
