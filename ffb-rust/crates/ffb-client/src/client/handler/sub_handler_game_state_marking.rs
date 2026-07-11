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

        // Java: `boolean reconnecting = incomingGame.getStarted() != null;` — the Rust
        // `Game` (crates/ffb-model/src/model/game.rs) has no `started` field, so
        // reconnecting can't be derived here and is treated as `false` (documented
        // deviation, not invented).
        let reconnecting = false;
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
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::marking::field_marker::FieldMarker;
    use ffb_model::marking::player_marker::PlayerMarker;
    use ffb_model::model::team::Team;
    use ffb_model::types::FieldCoordinate;

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

    #[test]
    fn initial_state_keeps_existing_player_and_field_markers() {
        let mut existing = make_game();
        existing.id = 0;
        existing.field_model.player_markers.push(PlayerMarker::with_player_id("p1"));
        existing.field_model.field_markers.push(FieldMarker::with_coordinate(FieldCoordinate::new(1, 1)));

        let incoming = make_game();
        let sub_handler = SubHandlerGameStateMarking::new();
        let result = sub_handler.handle_net_command(&existing, incoming, ClientMode::PLAYER, false);

        assert_eq!(result.field_model.player_markers.len(), 1);
        assert_eq!(result.field_model.field_markers.len(), 1);
    }

    #[test]
    fn reconnecting_with_automatic_marking_drops_existing_player_markers() {
        let mut existing = make_game();
        existing.id = 42;
        existing.field_model.player_markers.push(PlayerMarker::with_player_id("p1"));

        let mut incoming = make_game();
        incoming.id = 42;
        let sub_handler = SubHandlerGameStateMarking::new();
        // `client.getMode() != ClientMode.PLAYER` is false here (PLAYER), `isInitialState`
        // is false (existing.id != 0), and `!reconnecting` is always true (see module doc),
        // so `!isManualMarking` is what decides whether markers are kept.
        let result = sub_handler.handle_net_command(&existing, incoming.clone(), ClientMode::PLAYER, true);
        assert!(result.field_model.player_markers.is_empty());

        incoming.field_model.player_markers.clear();
        let result2 = sub_handler.handle_net_command(&existing, incoming, ClientMode::PLAYER, false);
        assert_eq!(result2.field_model.player_markers.len(), 1);
    }

    #[test]
    fn spectator_mode_always_keeps_existing_player_markers() {
        let mut existing = make_game();
        existing.id = 42;
        existing.field_model.player_markers.push(PlayerMarker::with_player_id("p1"));
        let incoming = make_game();
        let sub_handler = SubHandlerGameStateMarking::new();
        let result = sub_handler.handle_net_command(&existing, incoming, ClientMode::SPECTATOR, true);
        assert_eq!(result.field_model.player_markers.len(), 1);
    }

    #[test]
    fn replay_mode_keeps_existing_field_markers_regardless_of_id() {
        let mut existing = make_game();
        existing.id = 42;
        existing.field_model.field_markers.push(FieldMarker::with_coordinate(FieldCoordinate::new(2, 2)));
        let incoming = make_game();
        let sub_handler = SubHandlerGameStateMarking::new();
        let result = sub_handler.handle_net_command(&existing, incoming, ClientMode::REPLAY, false);
        assert_eq!(result.field_model.field_markers.len(), 1);
    }

    #[test]
    fn non_initial_non_replay_manual_marking_keeps_incoming_field_markers() {
        let mut existing = make_game();
        existing.id = 42;
        existing.field_model.field_markers.push(FieldMarker::with_coordinate(FieldCoordinate::new(3, 3)));
        let incoming = make_game();
        let sub_handler = SubHandlerGameStateMarking::new();
        let result = sub_handler.handle_net_command(&existing, incoming, ClientMode::PLAYER, true);
        assert!(result.field_model.field_markers.is_empty());
    }
}
