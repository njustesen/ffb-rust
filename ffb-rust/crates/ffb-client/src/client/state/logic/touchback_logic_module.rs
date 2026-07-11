//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.TouchbackLogicModule` (111 lines).
//!
//! Java's `TouchbackLogicModule` lets the receiving coach hand the ball to a standing player
//! (or, if no eligible player is on the pitch, drop it anywhere). The interaction methods need
//! `FantasyFootballClient` access, so — matching the established `MoveLogicModule` convention —
//! they are translated as inherent methods taking `client` explicitly rather than trait
//! overrides (see `logic_module.rs`'s module doc on the narrower trait-default signature).

use std::collections::HashSet;

use ffb_model::enums::ClientStateId;
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::property::NamedProperties;
use ffb_model::types::FieldCoordinate;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::LogicModule;

/// 1:1 translation of the `TouchbackLogicModule` class.
#[derive(Debug, Default)]
pub struct TouchbackLogicModule {
    /// java: `fTouchbackToAnyField`.
    touchback_to_any_field: bool,
}

impl TouchbackLogicModule {
    /// java: `public TouchbackLogicModule(FantasyFootballClient pClient)`.
    pub fn new() -> Self {
        Self { touchback_to_any_field: false }
    }

    /// java: `public InteractionResult playerPeek(Player<?> pPlayer)` — see module doc
    /// regarding the trait-default signature.
    pub fn player_peek(&self, client: &FantasyFootballClient, player: &Player) -> InteractionResult {
        if self.touchback_to_any_field || self.is_player_selectable(client, Some(player)) {
            InteractionResult::perform()
        } else {
            InteractionResult::reset()
        }
    }

    /// java: `public InteractionResult fieldPeek(FieldCoordinate pCoordinate)` — see module doc
    /// regarding the trait-default signature.
    pub fn field_peek(&self, _coordinate: FieldCoordinate) -> InteractionResult {
        if self.touchback_to_any_field {
            InteractionResult::perform()
        } else {
            InteractionResult::reset()
        }
    }

    /// java: `public InteractionResult playerInteraction(Player<?> pPlayer)` — see module doc
    /// regarding the trait-default signature.
    pub fn player_interaction(
        &self,
        client: &mut FantasyFootballClient,
        player: &Player,
    ) -> InteractionResult {
        if self.touchback_to_any_field || self.is_player_selectable(client, Some(player)) {
            let touchback_coordinate = client.game().and_then(|game| game.field_model.player_coordinate(&player.id));
            if let Some(touchback_coordinate) = touchback_coordinate {
                client.communication_mut().send_touchback(touchback_coordinate);
            }
            InteractionResult::handled()
        } else {
            InteractionResult::ignore()
        }
    }

    /// java: `public InteractionResult fieldInteraction(FieldCoordinate pCoordinate)` — see
    /// module doc regarding the trait-default signature.
    pub fn field_interaction(
        &self,
        client: &mut FantasyFootballClient,
        coordinate: FieldCoordinate,
    ) -> InteractionResult {
        if self.touchback_to_any_field {
            client.communication_mut().send_touchback(coordinate);
            InteractionResult::handled()
        } else {
            InteractionResult::ignore()
        }
    }

    /// java: `private boolean isPlayerSelectable(Player<?> pPlayer)`.
    fn is_player_selectable(&self, client: &FantasyFootballClient, player: Option<&Player>) -> bool {
        let player = match player {
            Some(p) => p,
            None => return false,
        };
        match client.game() {
            Some(game) => match game.field_model.player_state(&player.id) {
                Some(state) => {
                    state.has_tacklezones()
                        && game.team_home.has_player(&player.id)
                        && !player.has_skill_property(NamedProperties::PREVENT_HOLD_BALL)
                }
                None => false,
            },
            None => false,
        }
    }
}

impl LogicModule for TouchbackLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::Touchback
    }

    /// java: `public void setUp()` — checks if there are players on the field to give the ball
    /// to; `fTouchbackToAnyField` stays `true` unless a selectable home player is found.
    fn set_up(&mut self, client: &mut FantasyFootballClient) {
        self.touchback_to_any_field = true;
        if let Some(game) = client.game() {
            for player in &game.team_home.players.clone() {
                if self.is_player_selectable(client, Some(player)) {
                    self.touchback_to_any_field = false;
                    break;
                }
            }
        }
    }

    /// java: `public Set<ClientAction> availableActions()`.
    fn available_actions(&self) -> HashSet<ClientAction> {
        HashSet::new()
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action) {}`.
    fn perform_available_action(
        &mut self,
        _client: &mut FantasyFootballClient,
        _player: &Player,
        _action: ClientAction,
    ) {
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)` — always
    /// throws `UnsupportedOperationException` in Java; faithfully translated as a panic.
    fn action_context(&self, _game: &Game, _acting_player: &ActingPlayer) -> ActionContext {
        panic!("actionContext for acting player is not supported in touchback context")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::enums::PS_STANDING;
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
    fn get_id_is_touchback() {
        assert_eq!(TouchbackLogicModule::new().get_id(), ClientStateId::Touchback);
    }

    #[test]
    fn set_up_is_touchback_to_any_field_when_no_selectable_home_player() {
        let mut module = TouchbackLogicModule::new();
        let mut client = make_client();
        client.set_game(make_game());
        module.set_up(&mut client);
        assert!(module.touchback_to_any_field);
    }

    #[test]
    fn set_up_is_not_touchback_to_any_field_when_selectable_home_player_exists() {
        let mut module = TouchbackLogicModule::new();
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        client.set_game(game);
        module.set_up(&mut client);
        assert!(!module.touchback_to_any_field);
    }

    #[test]
    fn field_peek_perform_only_when_touchback_to_any_field() {
        let mut module = TouchbackLogicModule::new();
        assert_eq!(
            module.field_peek(FieldCoordinate::new(1, 1)).get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Reset
        );
        module.touchback_to_any_field = true;
        assert_eq!(
            module.field_peek(FieldCoordinate::new(1, 1)).get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Perform
        );
    }

    #[test]
    fn field_interaction_sends_touchback_only_when_touchback_to_any_field() {
        let mut module = TouchbackLogicModule::new();
        let mut client = make_client();
        client.set_game(make_game());
        assert_eq!(
            module.field_interaction(&mut client, FieldCoordinate::new(1, 1)).get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
        module.touchback_to_any_field = true;
        assert_eq!(
            module.field_interaction(&mut client, FieldCoordinate::new(1, 1)).get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Handled
        );
    }

    #[test]
    #[should_panic(expected = "actionContext for acting player is not supported in touchback context")]
    fn action_context_panics() {
        let module = TouchbackLogicModule::new();
        let game = make_game();
        let ap = ActingPlayer::new();
        module.action_context(&game, &ap);
    }
}
