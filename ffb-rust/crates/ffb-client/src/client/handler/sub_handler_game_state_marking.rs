//! 1:1 translation of `com.fumbbl.ffb.client.handler.SubHandlerGameStateMarking`.
//!
//! Java holds a `FantasyFootballClient` reference to read `client.getGame()`,
//! `client.getMode()`, and the `SETTING_PLAYER_MARKING_TYPE` property. Those all
//! live on `FantasyFootballClient`, which is still a GUI stub with no working
//! fields (see `crates/ffb-client/src/client/FantasyFootballClient.rs`), so this
//! translation takes the equivalent values as explicit parameters instead of
//! reaching through a client reference.
//!
//! Deviation: Java's `FieldModel` separates transient (auto-generated/animation)
//! markers from persistent (coach-authored) ones via `TransientPlayerMarker`
//! (a `PlayerMarker` subclass) and a parallel `TransientFieldMarker` list. The
//! Rust `FieldModel` (`crates/ffb-model/src/model/field_model.rs`) has no such
//! split — `player_markers`/`field_markers` are single flat lists — so the "always
//! keep existing transient markers" step (`Arrays.stream(existingTransientPlayerMarkers)
//! .forEach(fieldModel::addTransient)`) has no distinct Rust counterpart and is
//! documented rather than silently dropped.

use ffb_model::model::client_mode::ClientMode;
use ffb_model::model::game::Game;

pub struct SubHandlerGameStateMarking;

impl SubHandlerGameStateMarking {
    pub fn new() -> Self {
        Self
    }

    /// Java: `handleNetCommand(ServerCommandGameState gameStateCommand)`.
    ///
    /// `existing_game` is Java's `client.getGame()`; `incoming_game` is
    /// `gameStateCommand.getGame()`. Returns the game the client should adopt
    /// (Java also performs `client.setGame(incomingGame)` as a side effect, which
    /// callers of this function are responsible for applying).
    ///
    /// `client_mode`/`is_manual_marking` stand in for `client.getMode()` and
    /// `SETTING_PLAYER_MARKING_TYPE_MANUAL.equals(client.getProperty(SETTING_PLAYER_MARKING_TYPE))`.
    pub fn handle_net_command(
        &self,
        existing_game: &Game,
        mut incoming_game: Game,
        client_mode: ClientMode,
        is_manual_marking: bool,
    ) -> Game {
        // Java: existingTransientPlayerMarkers / existingTransientFieldMarkers — no
        // distinct Rust representation exists (see module doc); only the persistent
        // marker lists below have a real counterpart.
        let existing_player_markers = existing_game.field_model.player_markers.clone();
        let existing_field_markers = existing_game.field_model.field_markers.clone();

        // Java: `boolean reconnecting = incomingGame.getStarted() != null;`
        let reconnecting = incoming_game.started.is_some();
        let is_initial_state = !reconnecting && existing_game.id == 0;
        let is_replay = client_mode == ClientMode::REPLAY;

        if is_initial_state || is_replay {
            incoming_game.field_model.field_markers = existing_field_markers;
        }

        if client_mode != ClientMode::PLAYER || is_initial_state || (!reconnecting && !is_manual_marking) {
            incoming_game.field_model.player_markers = existing_player_markers;
        }

        incoming_game
    }
}

impl Default for SubHandlerGameStateMarking {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    // Mirrors ffb-java ffb-client-logic SubHandlerGameStateMarkingTest
    // (Java: SubHandlerGameStateMarkingTest.java), one test fn per Java @Test.
    //
    // Java-only assertion (all tests): the transient-marker checks
    // (`getTransientFieldMarkers()`/`getTransientPlayerMarkers()`) — the Rust
    // FieldModel has no transient lists (see module doc), so those assertions
    // have no Rust counterpart.
    //
    // Java's `assertSame(incomingGame, result)` / `verify(client).setGame(incomingGame)`
    // are mirrored by asserting the returned game carries the incoming game's id
    // (INCOMING_GAME_ID); the Rust translation has no client to set the game on.
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::marking::field_marker::FieldMarker;
    use ffb_model::marking::player_marker::PlayerMarker;
    use ffb_model::model::team::Team;
    use ffb_model::types::FieldCoordinate;

    const INCOMING_GAME_ID: u64 = 99;

    fn incoming_player() -> PlayerMarker {
        PlayerMarker::with_player_id("incomingPlayer")
    }

    fn existing_player() -> PlayerMarker {
        PlayerMarker::with_player_id("existingPlayer")
    }

    fn incoming_field() -> FieldMarker {
        FieldMarker::with_all(FieldCoordinate::new(0, 0), "incomingField", "")
    }

    fn existing_field() -> FieldMarker {
        FieldMarker::with_all(FieldCoordinate::new(0, 0), "existingField", "")
    }

    /// Java: `setUp()` — existing game holds the existing markers, incoming game
    /// the incoming ones. `existing.id` stays 0 (Java's unstubbed mock default);
    /// "GameStateUpdate" tests overwrite it with 1 (Java: `given(existingGame.getId()).willReturn(1L)`).
    fn setup() -> (Game, Game) {
        let mut existing = make_game();
        existing.id = 0;
        existing.field_model.player_markers.push(existing_player());
        existing.field_model.field_markers.push(existing_field());

        let mut incoming = make_game();
        incoming.id = INCOMING_GAME_ID;
        incoming.field_model.player_markers.push(incoming_player());
        incoming.field_model.field_markers.push(incoming_field());
        (existing, incoming)
    }

    fn player_ids(game: &Game) -> Vec<String> {
        game.field_model
            .player_markers
            .iter()
            .map(|m| m.get_player_id().unwrap_or("").to_string())
            .collect()
    }

    fn make_team(id: &str) -> Team {
        Team {
            id: id.into(),
            name: "Team".into(),
            race: "Human".into(),
            roster_id: "human".into(),
            coach: "Coach".into(),
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
            special_rules: vec![],
            players: vec![],
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2020)
    }

    /// Java: `testManualReplayInitialGameState`.
    #[test]
    fn test_manual_replay_initial_game_state() {
        let (existing, incoming) = setup();
        let handler = SubHandlerGameStateMarking::new();

        let result = handler.handle_net_command(&existing, incoming, ClientMode::REPLAY, true);

        assert_eq!(result.id, INCOMING_GAME_ID);
        assert_eq!(result.field_model.field_markers, [existing_field()]);
        assert_eq!(player_ids(&result), ["existingPlayer"]);
    }

    /// Java: `testManualReplayGameStateUpdate`.
    #[test]
    fn test_manual_replay_game_state_update() {
        let (mut existing, incoming) = setup();
        existing.id = 1;
        let handler = SubHandlerGameStateMarking::new();

        let result = handler.handle_net_command(&existing, incoming, ClientMode::REPLAY, true);

        assert_eq!(result.id, INCOMING_GAME_ID);
        assert_eq!(result.field_model.field_markers, [existing_field()]);
        assert_eq!(player_ids(&result), ["existingPlayer"]);
    }

    /// Java: `testManualPlayerInitialGameState`.
    #[test]
    fn test_manual_player_initial_game_state() {
        let (existing, incoming) = setup();
        let handler = SubHandlerGameStateMarking::new();

        let result = handler.handle_net_command(&existing, incoming, ClientMode::PLAYER, true);

        assert_eq!(result.id, INCOMING_GAME_ID);
        assert_eq!(result.field_model.field_markers, [existing_field()]);
        assert_eq!(player_ids(&result), ["existingPlayer"]);
    }

    /// Java: `testManualPlayerGameStateUpdate`.
    #[test]
    fn test_manual_player_game_state_update() {
        let (mut existing, incoming) = setup();
        existing.id = 1;
        let handler = SubHandlerGameStateMarking::new();

        let result = handler.handle_net_command(&existing, incoming, ClientMode::PLAYER, true);

        assert_eq!(result.id, INCOMING_GAME_ID);
        assert_eq!(result.field_model.field_markers, [incoming_field()]);
        assert_eq!(player_ids(&result), ["incomingPlayer"]);
    }

    /// Java: `testManualSpectatorGameStateUpdate`.
    #[test]
    fn test_manual_spectator_game_state_update() {
        let (mut existing, incoming) = setup();
        existing.id = 1;
        let handler = SubHandlerGameStateMarking::new();

        let result = handler.handle_net_command(&existing, incoming, ClientMode::SPECTATOR, true);

        assert_eq!(result.id, INCOMING_GAME_ID);
        assert_eq!(result.field_model.field_markers, [incoming_field()]);
        assert_eq!(player_ids(&result), ["existingPlayer"]);
    }

    /// Java: `testAutomaticReplayInitialGameState`.
    #[test]
    fn test_automatic_replay_initial_game_state() {
        let (existing, incoming) = setup();
        let handler = SubHandlerGameStateMarking::new();

        let result = handler.handle_net_command(&existing, incoming, ClientMode::REPLAY, false);

        assert_eq!(result.id, INCOMING_GAME_ID);
        assert_eq!(result.field_model.field_markers, [existing_field()]);
        assert_eq!(player_ids(&result), ["existingPlayer"]);
    }

    /// Java: `testAutomaticPlayerInitialGameState`.
    #[test]
    fn test_automatic_player_initial_game_state() {
        let (existing, incoming) = setup();
        let handler = SubHandlerGameStateMarking::new();

        let result = handler.handle_net_command(&existing, incoming, ClientMode::PLAYER, false);

        assert_eq!(result.id, INCOMING_GAME_ID);
        assert_eq!(result.field_model.field_markers, [existing_field()]);
        assert_eq!(player_ids(&result), ["existingPlayer"]);
    }

    /// Java: `testAutomaticPlayerGameStateUpdate`.
    #[test]
    fn test_automatic_player_game_state_update() {
        let (mut existing, incoming) = setup();
        existing.id = 1;
        let handler = SubHandlerGameStateMarking::new();

        let result = handler.handle_net_command(&existing, incoming, ClientMode::PLAYER, false);

        assert_eq!(result.id, INCOMING_GAME_ID);
        assert_eq!(result.field_model.field_markers, [incoming_field()]);
        assert_eq!(player_ids(&result), ["existingPlayer"]);
    }

    /// Java: `testAutomaticSpectatorGameStateUpdate`.
    #[test]
    fn test_automatic_spectator_game_state_update() {
        let (mut existing, incoming) = setup();
        existing.id = 1;
        let handler = SubHandlerGameStateMarking::new();

        let result = handler.handle_net_command(&existing, incoming, ClientMode::SPECTATOR, false);

        assert_eq!(result.id, INCOMING_GAME_ID);
        assert_eq!(result.field_model.field_markers, [incoming_field()]);
        assert_eq!(player_ids(&result), ["existingPlayer"]);
    }

    /// Java: `testAutomaticReplayGameStateUpdate`.
    #[test]
    fn test_automatic_replay_game_state_update() {
        let (mut existing, incoming) = setup();
        existing.id = 1;
        let handler = SubHandlerGameStateMarking::new();

        let result = handler.handle_net_command(&existing, incoming, ClientMode::REPLAY, false);

        assert_eq!(result.id, INCOMING_GAME_ID);
        assert_eq!(result.field_model.field_markers, [existing_field()]);
        assert_eq!(player_ids(&result), ["existingPlayer"]);
    }

    /// Java: `testManualPlayerReconnecting` (`given(incomingGame.getStarted()).willReturn(new Date())`).
    #[test]
    fn test_manual_player_reconnecting() {
        let (existing, mut incoming) = setup();
        // Java: incomingGame.getStarted() returns a Date → reconnecting == true.
        incoming.started = Some("1".to_owned());
        let handler = SubHandlerGameStateMarking::new();

        let result = handler.handle_net_command(&existing, incoming, ClientMode::PLAYER, true);

        assert_eq!(result.id, INCOMING_GAME_ID);
        assert_eq!(result.field_model.field_markers, [incoming_field()]);
        assert_eq!(player_ids(&result), ["incomingPlayer"]);
    }

    /// Java: `testManualSpectatorReconnecting`.
    #[test]
    fn test_manual_spectator_reconnecting() {
        let (existing, mut incoming) = setup();
        // Java: incomingGame.getStarted() returns a Date → reconnecting == true.
        incoming.started = Some("1".to_owned());
        let handler = SubHandlerGameStateMarking::new();

        let result = handler.handle_net_command(&existing, incoming, ClientMode::SPECTATOR, true);

        assert_eq!(result.id, INCOMING_GAME_ID);
        assert_eq!(result.field_model.field_markers, [incoming_field()]);
        assert_eq!(player_ids(&result), ["existingPlayer"]);
    }

    /// Java: `testAutomaticPlayerReconnecting`.
    #[test]
    fn test_automatic_player_reconnecting() {
        let (existing, mut incoming) = setup();
        // Java: incomingGame.getStarted() returns a Date → reconnecting == true.
        incoming.started = Some("1".to_owned());
        let handler = SubHandlerGameStateMarking::new();

        let result = handler.handle_net_command(&existing, incoming, ClientMode::PLAYER, false);

        assert_eq!(result.id, INCOMING_GAME_ID);
        assert_eq!(result.field_model.field_markers, [incoming_field()]);
        assert_eq!(player_ids(&result), ["incomingPlayer"]);
    }

    /// Java: `testAutomaticSpectatorReconnecting`.
    #[test]
    fn test_automatic_spectator_reconnecting() {
        let (existing, mut incoming) = setup();
        // Java: incomingGame.getStarted() returns a Date → reconnecting == true.
        incoming.started = Some("1".to_owned());
        let handler = SubHandlerGameStateMarking::new();

        let result = handler.handle_net_command(&existing, incoming, ClientMode::SPECTATOR, false);

        assert_eq!(result.id, INCOMING_GAME_ID);
        assert_eq!(result.field_model.field_markers, [incoming_field()]);
        assert_eq!(player_ids(&result), ["existingPlayer"]);
    }
}
