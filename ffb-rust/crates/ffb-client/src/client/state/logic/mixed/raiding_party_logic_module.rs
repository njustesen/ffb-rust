//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.mixed.RaidingPartyLogicModule` (89 lines).
//!
//! Java's `RaidingPartyLogicModule extends LogicModule` directly (no intermediate base class).

use std::collections::HashSet;

use ffb_model::enums::ClientStateId;
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::player_choice_mode::PlayerChoiceMode;
use ffb_model::types::FieldCoordinate;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::LogicModule;

/// 1:1 translation of the `RaidingPartyLogicModule` class.
#[derive(Debug, Default)]
pub struct RaidingPartyLogicModule;

impl RaidingPartyLogicModule {
    /// java: `public RaidingPartyLogicModule(FantasyFootballClient pClient)`.
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
            let ctx = match client.game() {
                Some(game) => self.action_context(game, &acting_player),
                None => ActionContext::new(),
            };
            return InteractionResult::select_action(ctx);
        }
        InteractionResult::ignore()
    }

    /// java: `public InteractionResult playerPeek(Player<?> player)`.
    pub fn player_peek(&self, client: &FantasyFootballClient, player: &Player) -> InteractionResult {
        let acting_player = match client.game() {
            Some(game) => &game.acting_player,
            None => return InteractionResult::invalid(),
        };
        if acting_player.player_id.as_deref() == Some(player.id.as_str()) {
            InteractionResult::reset()
        } else {
            InteractionResult::invalid()
        }
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

impl LogicModule for RaidingPartyLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::RaidingParty
    }

    /// java: `public Set<ClientAction> availableActions()`.
    fn available_actions(&self) -> HashSet<ClientAction> {
        let mut actions = HashSet::new();
        actions.insert(ClientAction::RAIDING_PARTY);
        actions
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)`.
    fn action_context(&self, _game: &Game, _acting_player: &ActingPlayer) -> ActionContext {
        let mut action_context = ActionContext::new();
        action_context.add_action(ClientAction::RAIDING_PARTY);
        action_context
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)`.
    fn perform_available_action(&mut self, client: &mut FantasyFootballClient, _player: &Player, action: ClientAction) {
        if action == ClientAction::RAIDING_PARTY {
            client.communication_mut().send_player_choice(PlayerChoiceMode::RAIDING_PARTY, &[]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
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
    fn get_id_is_raiding_party() {
        assert_eq!(RaidingPartyLogicModule::new().get_id(), ClientStateId::RaidingParty);
    }

    #[test]
    fn available_actions_is_raiding_party_only() {
        let actions = RaidingPartyLogicModule::new().available_actions();
        assert_eq!(actions.len(), 1);
        assert!(actions.contains(&ClientAction::RAIDING_PARTY));
    }

    #[test]
    fn action_context_always_adds_raiding_party() {
        let module = RaidingPartyLogicModule::new();
        let game = make_game();
        let ap = ActingPlayer::new();
        let ctx = module.action_context(&game, &ap);
        assert_eq!(ctx.get_actions(), &vec![ClientAction::RAIDING_PARTY]);
    }

    #[test]
    fn player_peek_invalid_without_game() {
        let client = make_client();
        let module = RaidingPartyLogicModule::new();
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
        let module = RaidingPartyLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
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
        let module = RaidingPartyLogicModule::new();
        let result = module.field_interaction(&mut client, FieldCoordinate::new(3, 3));
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn field_peek_invalid_without_move_square() {
        let mut client = make_client();
        client.set_game(make_game());
        let module = RaidingPartyLogicModule::new();
        let result = module.field_peek(&client, FieldCoordinate::new(3, 3));
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Invalid
        );
    }

    #[test]
    fn perform_available_action_sends_player_choice() {
        let mut module = RaidingPartyLogicModule::new();
        let mut client = make_client();
        let mut player = Player::default();
        player.id = "p1".to_string();
        module.perform_available_action(&mut client, &player, ClientAction::RAIDING_PARTY);
        assert!(!client.communication().is_stopped());
    }
}
