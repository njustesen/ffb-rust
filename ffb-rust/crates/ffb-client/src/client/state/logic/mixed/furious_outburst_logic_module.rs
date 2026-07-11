//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.mixed.FuriousOutburstLogicModule`
//! (88 lines).
//!
//! Java's `FuriousOutburstLogicModule extends LogicModule` directly, with no fields.
//!
//! Documented gap:
//! - `client.getCommunication().sendActingPlayer(null, null, false)` in the `END_MOVE` branch
//!   of `performAvailableAction` — see `LogicModule::deselect_acting_player`'s documented gap
//!   (no "null action" variant exists in the translated `PlayerAction` enum); left as a no-op.

use ffb_model::enums::ClientStateId;
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

/// 1:1 translation of the `FuriousOutburstLogicModule` class.
#[derive(Debug, Default)]
pub struct FuriousOutburstLogicModule;

impl FuriousOutburstLogicModule {
    /// java: `public FuriousOutburstLogicModule(FantasyFootballClient pClient)`.
    pub fn new() -> Self {
        Self
    }

    /// java: `public InteractionResult fieldInteraction(FieldCoordinate pCoordinate)`.
    pub fn field_interaction(&self, client: &mut FantasyFootballClient, coordinate: FieldCoordinate) -> InteractionResult {
        if is_eligible(client, coordinate) {
            client.communication_mut().send_field_coordinate(coordinate);
            InteractionResult::handled()
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
        let acting_player = &game.acting_player;
        if acting_player.player_id.as_deref() == Some(player.id.as_str()) {
            InteractionResult::select_action(self.action_context(game, acting_player))
        } else {
            InteractionResult::ignore()
        }
    }

    /// java: `public InteractionResult fieldPeek(FieldCoordinate pCoordinate)`.
    pub fn field_peek(&self, client: &FantasyFootballClient, coordinate: FieldCoordinate) -> InteractionResult {
        if is_eligible(client, coordinate) {
            InteractionResult::perform()
        } else {
            InteractionResult::invalid()
        }
    }
}

/// java: `private boolean isEligible(FieldCoordinate coordinate)`.
fn is_eligible(client: &FantasyFootballClient, coordinate: FieldCoordinate) -> bool {
    client.game().map(|g| g.field_model.get_move_square(coordinate).is_some()).unwrap_or(false)
}

impl LogicModule for FuriousOutburstLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::FuriousOutburst
    }

    /// java: `public Set<ClientAction> availableActions()`.
    fn available_actions(&self) -> std::collections::HashSet<ClientAction> {
        let mut actions = std::collections::HashSet::new();
        actions.insert(ClientAction::END_MOVE);
        actions
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)`.
    fn action_context(&self, _game: &Game, acting_player: &ActingPlayer) -> ActionContext {
        let mut action_context = ActionContext::new();
        action_context.add_action(ClientAction::END_MOVE);
        if acting_player.has_acted {
            action_context.add_influence(Influences::HAS_ACTED);
        }
        action_context
    }

    /// java: `protected void performAvailableAction(Player<?> pPlayer, ClientAction action)`.
    fn perform_available_action(&mut self, _client: &mut FantasyFootballClient, _player: &Player, action: ClientAction) {
        match action {
            ClientAction::END_MOVE => {
                // java: `client.getCommunication().sendActingPlayer(null, null, false);` — see
                // module doc gap.
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
    fn get_id_is_furious_outburst() {
        assert_eq!(FuriousOutburstLogicModule::new().get_id(), ClientStateId::FuriousOutburst);
    }

    #[test]
    fn available_actions_is_end_move_only() {
        let actions = FuriousOutburstLogicModule::new().available_actions();
        assert_eq!(actions.len(), 1);
        assert!(actions.contains(&ClientAction::END_MOVE));
    }

    #[test]
    fn action_context_adds_influence_when_acted() {
        let module = FuriousOutburstLogicModule::new();
        let game = make_game();
        let mut ap = ActingPlayer::new();
        ap.has_acted = true;
        let ctx = module.action_context(&game, &ap);
        assert!(ctx.get_actions().contains(&ClientAction::END_MOVE));
        assert!(ctx.get_influences().contains(&Influences::HAS_ACTED));
    }

    #[test]
    fn action_context_no_influence_without_acted() {
        let module = FuriousOutburstLogicModule::new();
        let game = make_game();
        let ap = ActingPlayer::new();
        let ctx = module.action_context(&game, &ap);
        assert!(!ctx.get_influences().contains(&Influences::HAS_ACTED));
    }

    #[test]
    fn field_interaction_ignores_without_move_square() {
        let mut client = make_client();
        client.set_game(make_game());
        let module = FuriousOutburstLogicModule::new();
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
        let module = FuriousOutburstLogicModule::new();
        let result = module.field_peek(&client, FieldCoordinate::new(1, 1));
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Invalid
        );
    }

    #[test]
    fn player_interaction_ignores_without_game() {
        let client = make_client();
        let module = FuriousOutburstLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_interaction(&client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn player_interaction_selects_action_for_acting_player() {
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        game.acting_player.player_id = Some("p1".to_string());
        client.set_game(game);
        let module = FuriousOutburstLogicModule::new();
        let player = client.game().unwrap().player("p1").unwrap().clone();
        let result = module.player_interaction(&client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::SelectAction
        );
    }

    #[test]
    fn perform_available_action_is_no_op() {
        let mut module = FuriousOutburstLogicModule::new();
        let mut client = make_client();
        let mut player = Player::default();
        player.id = "p1".to_string();
        module.perform_available_action(&mut client, &player, ClientAction::END_MOVE);
    }
}
