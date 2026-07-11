//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.HighKickLogicModule` (98 lines).
//!
//! Java's `HighKickLogicModule` lets the receiving coach place their catching player onto the
//! high-kick ball square. `playerInteraction`/`playerPeek` need `FantasyFootballClient` access
//! (game state + communication), so — matching the established `MoveLogicModule` convention —
//! they are translated as inherent methods taking `client` explicitly rather than trait
//! overrides (the `LogicModule` trait's own `player_interaction`/`player_peek` defaults have a
//! narrower signature with no client parameter; see `logic_module.rs`'s module doc).
//!
//! Because the trait's own `player_interaction`/`player_peek` defaults take `&self` (see
//! `logic_module.rs`), the inherent overrides here must also take `&self` (not `&mut self`) —
//! Rust's method lookup tries the `&self` self-kind (where it finds the trait default) before
//! ever trying `&mut self`, so a `&mut self` inherent method of the same name would silently be
//! shadowed by the trait default at every call site. `fOldCoordinate` is therefore held in a
//! `Cell` to allow the mutation `playerInteraction` needs while keeping the `&self` receiver.

use std::cell::Cell;
use std::collections::HashSet;

use ffb_model::enums::ClientStateId;
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::types::FieldCoordinate;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::LogicModule;

/// 1:1 translation of the `HighKickLogicModule` class.
#[derive(Debug, Default)]
pub struct HighKickLogicModule {
    /// java: `fOldCoordinate`. Held in a `Cell` — see module doc on the `&self` receiver
    /// requirement.
    old_coordinate: Cell<Option<FieldCoordinate>>,
}

impl HighKickLogicModule {
    /// java: `public HighKickLogicModule(FantasyFootballClient pClient)`.
    pub fn new() -> Self {
        Self { old_coordinate: Cell::new(None) }
    }

    /// java: `public InteractionResult playerInteraction(Player<?> pPlayer)` — see module doc
    /// regarding the trait-default signature.
    pub fn player_interaction(
        &self,
        client: &mut FantasyFootballClient,
        player: &Player,
    ) -> InteractionResult {
        if !self.is_player_selectable(client, Some(player)) {
            return InteractionResult::ignore();
        }
        let (old_player_id, ball_coordinate, player_coordinate) = match client.game() {
            Some(game) => {
                let old_player_id = game
                    .field_model
                    .ball_coordinate
                    .and_then(|coord| game.field_model.player_at(coord).cloned());
                (old_player_id, game.field_model.ball_coordinate, game.field_model.player_coordinate(&player.id))
            }
            None => (None, None, None),
        };
        let is_same_player = old_player_id.as_deref() == Some(player.id.as_str());
        if !is_same_player {
            if let (Some(old_player_id), Some(old_coordinate)) = (old_player_id.clone(), self.old_coordinate.get()) {
                if let Some(old_player) = client.game().and_then(|g| g.player(&old_player_id)).cloned() {
                    client.communication_mut().send_setup_player(&old_player, old_coordinate);
                }
            }
            self.old_coordinate.set(player_coordinate);
            if let Some(ball_coordinate) = ball_coordinate {
                client.communication_mut().send_setup_player(player, ball_coordinate);
            }
        } else if let Some(old_coordinate) = self.old_coordinate.get() {
            client.communication_mut().send_setup_player(player, old_coordinate);
            self.old_coordinate.set(None);
        }
        InteractionResult::handled()
    }

    /// java: `public InteractionResult playerPeek(Player<?> pPlayer)` — see module doc
    /// regarding the trait-default signature.
    pub fn player_peek(&self, client: &FantasyFootballClient, player: &Player) -> InteractionResult {
        if self.is_player_selectable(client, Some(player)) {
            InteractionResult::perform()
        } else {
            InteractionResult::reset()
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
                Some(state) => state.is_active() && game.team_home.has_player(&player.id),
                None => false,
            },
            None => false,
        }
    }
}

impl LogicModule for HighKickLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::HighKick
    }

    /// java: `public void setUp() { super.setUp(); fOldCoordinate = null; }`.
    fn set_up(&mut self, _client: &mut FantasyFootballClient) {
        self.old_coordinate.set(None);
    }

    /// java: `public void endTurn()`.
    fn end_turn(&mut self, client: &mut FantasyFootballClient) {
        let turn_mode = client.game().map(|g| g.turn_mode);
        if let Some(turn_mode) = turn_mode {
            client.communication_mut().send_end_turn(turn_mode);
        }
        client.client_data_mut().set_end_turn_button_hidden(true);
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
        panic!("actionContext for acting player is not supported in high kick context")
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
    fn get_id_is_high_kick() {
        assert_eq!(HighKickLogicModule::new().get_id(), ClientStateId::HighKick);
    }

    #[test]
    fn available_actions_is_empty() {
        assert!(HighKickLogicModule::new().available_actions().is_empty());
    }

    #[test]
    fn set_up_resets_old_coordinate() {
        let mut module = HighKickLogicModule::new();
        module.old_coordinate.set(Some(FieldCoordinate::new(3, 3)));
        let mut client = make_client();
        module.set_up(&mut client);
        assert!(module.old_coordinate.get().is_none());
    }

    #[test]
    fn player_interaction_places_ball_on_selected_player() {
        let module = HighKickLogicModule::new();
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(10, 5));
        client.set_game(game);
        let player = client.game().unwrap().player("p1").unwrap().clone();

        let result = module.player_interaction(&mut client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Handled
        );
        assert_eq!(module.old_coordinate.get(), Some(FieldCoordinate::new(1, 1)));
    }

    #[test]
    fn player_interaction_ignores_non_selectable_player() {
        let module = HighKickLogicModule::new();
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, false, "p2", FieldCoordinate::new(2, 2));
        client.set_game(game);
        let player = client.game().unwrap().player("p2").unwrap().clone();

        let result = module.player_interaction(&mut client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    #[should_panic(expected = "actionContext for acting player is not supported in high kick context")]
    fn action_context_panics() {
        let module = HighKickLogicModule::new();
        let game = make_game();
        let ap = ActingPlayer::new();
        module.action_context(&game, &ap);
    }

    #[test]
    fn is_player_selectable_requires_active_home_player() {
        let module = HighKickLogicModule::new();
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        add_player(&mut game, false, "p2", FieldCoordinate::new(2, 2));
        client.set_game(game);

        let home_player = client.game().unwrap().player("p1").unwrap().clone();
        let away_player = client.game().unwrap().player("p2").unwrap().clone();
        assert!(module.is_player_selectable(&client, Some(&home_player)));
        assert!(!module.is_player_selectable(&client, Some(&away_player)));
        assert!(!module.is_player_selectable(&client, None));
    }

    #[test]
    fn player_peek_performs_for_selectable_player() {
        let module = HighKickLogicModule::new();
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(1, 1));
        client.set_game(game);
        let player = client.game().unwrap().player("p1").unwrap().clone();
        assert_eq!(
            module.player_peek(&client, &player).get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Perform
        );
    }

    #[test]
    fn player_peek_resets_for_non_selectable_player() {
        let module = HighKickLogicModule::new();
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, false, "p2", FieldCoordinate::new(2, 2));
        client.set_game(game);
        let player = client.game().unwrap().player("p2").unwrap().clone();
        assert_eq!(
            module.player_peek(&client, &player).get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Reset
        );
    }
}
