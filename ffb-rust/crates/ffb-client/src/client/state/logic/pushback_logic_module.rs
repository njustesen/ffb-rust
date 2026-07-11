//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.PushbackLogicModule`.
//!
//! Handles the `PUSHBACK` client state: clicking on (or clicking a player standing on) an
//! unlocked home-choice pushback square sends the resulting `Pushback` (pushed-player id +
//! destination square) to the server; `fieldPeek`/`playerPeek` toggle the `selected` flag on
//! pushback squares to preview the choice; `pushbackTo` drives the same flow from a keyboard
//! direction shortcut.

use std::collections::HashSet;

use ffb_model::enums::{ClientStateId, Direction};
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::pushback::Pushback;
use ffb_model::types::{FieldCoordinate, PushbackSquare};

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::LogicModule;

/// Java: `PushbackLogicModule`.
#[derive(Debug, Default)]
pub struct PushbackLogicModule;

impl PushbackLogicModule {
    /// Java: `public PushbackLogicModule(FantasyFootballClient pClient)`.
    pub fn new() -> Self {
        Self
    }

    /// java: `public InteractionResult fieldInteraction(FieldCoordinate pCoordinate)`
    pub fn field_interaction(
        &self,
        client: &mut FantasyFootballClient,
        coordinate: FieldCoordinate,
    ) -> InteractionResult {
        let pushback = client.game().and_then(|game| {
            find_unlocked_pushback_square(game, coordinate).and_then(|square| find_pushback(game, square))
        });
        if let Some(pushback) = pushback {
            client.communication_mut().send_pushback(pushback);
            return InteractionResult::handled();
        }
        InteractionResult::ignore()
    }

    /// java: `public InteractionResult playerInteraction(Player<?> pPlayer)`
    pub fn player_interaction(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        let player_coordinate = client.game().and_then(|game| game.field_model.player_coordinate(&player.id));
        match player_coordinate {
            Some(coordinate) => self.field_interaction(client, coordinate),
            None => InteractionResult::ignore(),
        }
    }

    /// java: `public InteractionResult playerPeek(Player<?> player)`
    pub fn player_peek(&self, client: &FantasyFootballClient, player: &Player) -> InteractionResult {
        let player_coordinate = client.game().and_then(|game| game.field_model.player_coordinate(&player.id));
        match player_coordinate {
            Some(coordinate) => self.field_peek(client, coordinate),
            None => self.field_peek_none(),
        }
    }

    /// java: `LogicModule.getCoordinate(player)` returned `null` (no coordinate found), which
    /// Java then passes straight into `fieldPeek(null)` — `FieldCoordinate.equals(null)` is
    /// always `false`, so every unlocked/unselected pushback square is deselected. Modeled
    /// directly rather than threading an `Option<FieldCoordinate>` through `field_peek`.
    fn field_peek_none(&self) -> InteractionResult {
        InteractionResult::ignore()
    }

    /// java: `public InteractionResult fieldPeek(FieldCoordinate pCoordinate)`
    pub fn field_peek(&self, client: &FantasyFootballClient, coordinate: FieldCoordinate) -> InteractionResult {
        let game = match client.game() {
            Some(game) => game,
            None => return InteractionResult::ignore(),
        };
        let mut to_update: Vec<PushbackSquare> = Vec::new();
        for square in &game.field_model.pushback_squares {
            if square.coordinate == coordinate {
                if square.home_choice && !square.selected && !square.locked {
                    let mut updated = *square;
                    updated.selected = true;
                    to_update.push(updated);
                }
            } else if square.selected && !square.locked {
                let mut updated = *square;
                updated.selected = false;
                to_update.push(updated);
            }
        }
        if !to_update.is_empty() {
            return InteractionResult::handled().with_pushback_squares(to_update);
        }
        InteractionResult::ignore()
    }

    /// java: `public boolean pushbackTo(Direction moveDirection)`
    pub fn pushback_to(&self, client: &mut FantasyFootballClient, move_direction: Direction) -> bool {
        let pushback = client.game().and_then(|game| {
            let square = game
                .field_model
                .pushback_squares
                .iter()
                .find(|square| !square.locked && square.direction == move_direction)
                .copied();
            square.and_then(|square| find_pushback(game, square))
        });
        if let Some(pushback) = pushback {
            client.communication_mut().send_pushback(pushback);
            return true;
        }
        false
    }
}

/// java: `private PushbackSquare findUnlockedPushbackSquare(FieldCoordinate pCoordinate)`
fn find_unlocked_pushback_square(game: &Game, coordinate: FieldCoordinate) -> Option<PushbackSquare> {
    game.field_model
        .pushback_squares
        .iter()
        .find(|square| !square.locked && square.coordinate == coordinate && square.home_choice)
        .copied()
}

/// java: `private Pushback findPushback(PushbackSquare pPushbackSquare)`
fn find_pushback(game: &Game, pushback_square: PushbackSquare) -> Option<Pushback> {
    let to_square = pushback_square.coordinate;
    let from_square = match pushback_square.direction {
        Direction::North => to_square.add(0, 1),
        Direction::Northeast => to_square.add(-1, 1),
        Direction::East => to_square.add(-1, 0),
        Direction::Southeast => to_square.add(-1, -1),
        Direction::South => to_square.add(0, -1),
        Direction::Southwest => to_square.add(1, -1),
        Direction::West => to_square.add(1, 0),
        Direction::Northwest => to_square.add(1, 1),
    };
    let pushed_player_id = game.field_model.player_at(from_square)?;
    Some(Pushback::new(pushed_player_id.clone(), to_square))
}

impl LogicModule for PushbackLogicModule {
    /// java: `public ClientStateId getId()`
    fn get_id(&self) -> ClientStateId {
        ClientStateId::Pushback
    }

    /// java: `public Set<ClientAction> availableActions()`
    fn available_actions(&self) -> HashSet<ClientAction> {
        HashSet::new()
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)` — always throws
    /// `UnsupportedOperationException` in Java.
    fn action_context(&self, _game: &Game, _acting_player: &ActingPlayer) -> ActionContext {
        panic!("actionContext for acting player is not supported in pushback context");
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)` —
    /// empty body in Java.
    fn perform_available_action(
        &mut self,
        _client: &mut FantasyFootballClient,
        _player: &Player,
        _action: ClientAction,
    ) {
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::client_parameters::ClientParameters;
    use ffb_model::enums::Rules;
    use ffb_model::model::player::Player as ModelPlayer;
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

    fn make_client() -> FantasyFootballClient {
        let params =
            ClientParameters::create_valid_params(&["-spectator".into(), "-coach".into(), "bob".into()]).unwrap();
        let mut client = FantasyFootballClient::new(params);
        client.set_game(Game::new(make_team("home"), make_team("away"), Rules::Bb2025));
        client
    }

    fn add_player(client: &mut FantasyFootballClient, id: &str, coord: FieldCoordinate) {
        let mut player = ModelPlayer::default();
        player.id = id.to_string();
        let game = client.game_mut().unwrap();
        game.team_home.players.push(player);
        game.field_model.set_player_coordinate(id, coord);
    }

    #[test]
    fn get_id_is_pushback() {
        let module = PushbackLogicModule::new();
        assert_eq!(module.get_id(), ClientStateId::Pushback);
    }

    #[test]
    fn available_actions_is_always_empty() {
        let module = PushbackLogicModule::new();
        assert!(module.available_actions().is_empty());
    }

    #[test]
    fn field_interaction_ignores_when_no_pushback_square() {
        let module = PushbackLogicModule::new();
        let mut client = make_client();
        let result = module.field_interaction(&mut client, FieldCoordinate::new(1, 1));
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn field_interaction_sends_pushback_when_square_found() {
        let module = PushbackLogicModule::new();
        let mut client = make_client();
        let from = FieldCoordinate::new(5, 5);
        let to = from.add(0, -1);
        add_player(&mut client, "p1", from);
        client
            .game_mut()
            .unwrap()
            .field_model
            .pushback_squares
            .push(PushbackSquare::new(to, Direction::North, true));
        let result = module.field_interaction(&mut client, to);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Handled
        );
    }

    #[test]
    fn field_peek_selects_matching_unlocked_home_choice_square() {
        let module = PushbackLogicModule::new();
        let mut client = make_client();
        let coord = FieldCoordinate::new(4, 4);
        client
            .game_mut()
            .unwrap()
            .field_model
            .pushback_squares
            .push(PushbackSquare::new(coord, Direction::North, true));
        let result = module.field_peek(&client, coord);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Handled
        );
        assert_eq!(result.get_pushback_squares().unwrap().len(), 1);
        assert!(result.get_pushback_squares().unwrap()[0].selected);
    }

    #[test]
    fn field_peek_deselects_previously_selected_square_on_miss() {
        let module = PushbackLogicModule::new();
        let mut client = make_client();
        let coord = FieldCoordinate::new(4, 4);
        let mut square = PushbackSquare::new(coord, Direction::North, true);
        square.selected = true;
        client.game_mut().unwrap().field_model.pushback_squares.push(square);
        let result = module.field_peek(&client, FieldCoordinate::new(9, 9));
        assert_eq!(result.get_pushback_squares().unwrap().len(), 1);
        assert!(!result.get_pushback_squares().unwrap()[0].selected);
    }

    #[test]
    fn pushback_to_finds_square_by_direction() {
        let module = PushbackLogicModule::new();
        let mut client = make_client();
        let from = FieldCoordinate::new(5, 5);
        let to = from.add(0, -1);
        add_player(&mut client, "p1", from);
        client
            .game_mut()
            .unwrap()
            .field_model
            .pushback_squares
            .push(PushbackSquare::new(to, Direction::North, true));
        assert!(module.pushback_to(&mut client, Direction::North));
        assert!(!module.pushback_to(&mut client, Direction::South));
    }

    #[test]
    #[should_panic(expected = "actionContext for acting player is not supported in pushback context")]
    fn action_context_panics() {
        let module = PushbackLogicModule::new();
        let client = make_client();
        let game = client.game().unwrap();
        let ap = ActingPlayer::new();
        module.action_context(game, &ap);
    }
}
