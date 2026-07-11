//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.SwoopLogicModule` (70 lines).
//!
//! Java's `SwoopLogicModule extends MoveLogicModule`, overriding `fieldInteraction`,
//! `playerInteraction`, and `playerPeek`; every other `MoveLogicModule` method is inherited
//! unchanged (delegated here to a held `MoveLogicModule` instance for `action_context`, per
//! the established batch convention — `getId`/`availableActions`/`performAvailableAction` are
//! *not* overridden in Java either, so those inherited defaults are reused verbatim too).

use ffb_model::enums::{ClientStateId, PlayerAction};
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::types::FieldCoordinate;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::LogicModule;
use crate::client::state::logic::move_logic_module::MoveLogicModule;

/// 1:1 translation of the `SwoopLogicModule` class.
#[derive(Debug, Default)]
pub struct SwoopLogicModule {
    move_logic: MoveLogicModule,
}

impl SwoopLogicModule {
    /// java: `public SwoopLogicModule(FantasyFootballClient pClient)`.
    pub fn new() -> Self {
        Self { move_logic: MoveLogicModule::new() }
    }

    /// java: `public InteractionResult fieldInteraction(FieldCoordinate pCoordinate)`.
    pub fn field_interaction(&self, client: &mut FantasyFootballClient, coordinate: FieldCoordinate) -> InteractionResult {
        let is_swoop = match client.game() {
            Some(game) => game.acting_player.player_action == Some(PlayerAction::Swoop),
            None => return InteractionResult::ignore(),
        };
        if is_swoop {
            self.send_swoop(client, coordinate);
            return InteractionResult::handled();
        }
        InteractionResult::ignore()
    }

    /// java: `public InteractionResult playerInteraction(Player<?> pPlayer)`.
    pub fn player_interaction(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        let (is_swoop, coordinate) = match client.game() {
            Some(game) => (
                game.acting_player.player_action == Some(PlayerAction::Swoop),
                game.field_model.player_coordinate(&player.id),
            ),
            None => return InteractionResult::ignore(),
        };
        if is_swoop {
            if let Some(coordinate) = coordinate {
                self.send_swoop(client, coordinate);
            }
            return InteractionResult::handled();
        }
        InteractionResult::ignore()
    }

    /// java: `public InteractionResult playerPeek(Player<?> pPlayer)`.
    pub fn player_peek(&self, client: &FantasyFootballClient, _player: &Player) -> InteractionResult {
        match client.game() {
            Some(game) if game.defender_id.is_none() && game.pass_coordinate.is_none() => InteractionResult::reset(),
            _ => InteractionResult::ignore(),
        }
    }

    /// java: `private void sendSwoop(Game game, ActingPlayer actingPlayer, FieldCoordinate destination)`.
    fn send_swoop(&self, client: &mut FantasyFootballClient, destination: FieldCoordinate) {
        let (source, acting_player_id) = match client.game() {
            Some(game) => {
                let acting_player = &game.acting_player;
                let source = acting_player.player_id.as_deref().and_then(|id| game.field_model.player_coordinate(id));
                (source, acting_player.player_id.clone())
            }
            None => return,
        };
        let source = match source {
            Some(s) => s,
            None => return,
        };
        if !source.is_adjacent(destination) {
            return;
        }
        // java: "Check if the destination is in one of the 4 cardinal directions from the
        // player" — `source.getY() == destination.getY() || source.getX() == destination.getX()`.
        if source.y == destination.y || source.x == destination.x {
            if let Some(id) = acting_player_id {
                client.communication_mut().send_swoop(id, destination);
            }
        }
    }
}

impl LogicModule for SwoopLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::Swoop
    }

    /// java: `public Set<ClientAction> availableActions()` — not overridden; inherited from
    /// `MoveLogicModule`.
    fn available_actions(&self) -> std::collections::HashSet<ClientAction> {
        self.move_logic.available_actions()
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)` — not
    /// overridden; inherited from `MoveLogicModule`.
    fn action_context(&self, game: &Game, acting_player: &ActingPlayer) -> ActionContext {
        self.move_logic.action_context(game, acting_player)
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)` —
    /// not overridden; inherited from `MoveLogicModule`.
    fn perform_available_action(&mut self, client: &mut FantasyFootballClient, player: &Player, action: ClientAction) {
        self.move_logic.perform_available_action(client, player, action);
    }

    /// java: `public void endTurn()` — not overridden; inherited from `MoveLogicModule`.
    fn end_turn(&mut self, client: &mut FantasyFootballClient) {
        self.move_logic.end_turn(client);
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
    fn get_id_is_swoop() {
        assert_eq!(SwoopLogicModule::new().get_id(), ClientStateId::Swoop);
    }

    #[test]
    fn available_actions_matches_move_logic_module() {
        let module = SwoopLogicModule::new();
        assert_eq!(module.available_actions().len(), MoveLogicModule::new().available_actions().len());
    }

    #[test]
    fn field_interaction_ignores_without_swoop_action() {
        let mut client = make_client();
        client.set_game(make_game());
        let module = SwoopLogicModule::new();
        let result = module.field_interaction(&mut client, FieldCoordinate::new(5, 5));
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn field_interaction_handled_when_swoop_action_set() {
        let mut client = make_client();
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(5, 5));
        game.acting_player.player_id = Some("p1".to_string());
        game.acting_player.player_action = Some(PlayerAction::Swoop);
        client.set_game(game);
        let module = SwoopLogicModule::new();
        let result = module.field_interaction(&mut client, FieldCoordinate::new(6, 5));
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Handled
        );
    }

    #[test]
    fn player_peek_resets_without_defender_or_pass_coordinate() {
        let mut client = make_client();
        client.set_game(make_game());
        let module = SwoopLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_peek(&client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Reset
        );
    }

    #[test]
    fn player_peek_ignores_when_defender_present() {
        let mut client = make_client();
        let mut game = make_game();
        game.defender_id = Some("d1".to_string());
        client.set_game(game);
        let module = SwoopLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_peek(&client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn player_interaction_ignores_without_swoop_action() {
        let mut client = make_client();
        client.set_game(make_game());
        let module = SwoopLogicModule::new();
        let mut player = Player::default();
        player.id = "p1".to_string();
        let result = module.player_interaction(&mut client, &player);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }
}
