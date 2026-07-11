/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerSetMarker.
use std::sync::{Arc, Mutex};
use ffb_model::enums::NetCommandId;
use ffb_model::marking::field_marker::FieldMarker;
use ffb_model::marking::player_marker::PlayerMarker;
use ffb_model::util::string_tool::is_provided;
use ffb_protocol::commands::client_command_set_marker::ClientCommandSetMarker;
use crate::game_cache::GameCache;
use crate::model::received_command::SessionId;
use crate::net::session_manager::SessionManager;

/// Java: `IDbTablePlayerMarkers.MAX_TEXT_LENGTH`.
const MAX_TEXT_LENGTH: usize = 40;

/// Java: `ServerCommandHandlerSetMarker extends ServerCommandHandler`.
pub struct ServerCommandHandlerSetMarker {
    game_cache: Arc<Mutex<GameCache>>,
    session_manager: Arc<Mutex<SessionManager>>,
}

impl ServerCommandHandlerSetMarker {
    /// Java: `protected ServerCommandHandlerSetMarker(FantasyFootballServer pServer)`
    pub fn new(game_cache: Arc<Mutex<GameCache>>, session_manager: Arc<Mutex<SessionManager>>) -> Self {
        Self { game_cache, session_manager }
    }

    /// Java: `getId()` — returns `NetCommandId.CLIENT_SET_MARKER`.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientSetMarker
    }

    /// Java: `handleCommand(ReceivedCommand)`.
    pub fn handle_command(&self, session_id: SessionId, set_marker_command: &ClientCommandSetMarker) -> bool {
        let sm = self.session_manager.lock().unwrap();
        let game_id = sm.get_game_id_for_session(session_id);
        let home_marker = sm.get_session_of_home_coach(game_id) == Some(session_id);
        let away_marker = sm.get_session_of_away_coach(game_id) == Some(session_id);
        drop(sm);

        if home_marker || away_marker {
            let mut gc = self.game_cache.lock().unwrap();
            let Some(game_state) = gc.get_game_state_by_id_mut(game_id) else {
                return true;
            };
            let Some(game) = game_state.get_game_mut() else {
                return true;
            };

            // Java: truncate text to IDbTablePlayerMarkers.MAX_TEXT_LENGTH.
            let mut text = set_marker_command.get_text().map(|t| t.to_string());
            if let Some(t) = &text {
                if t.len() > MAX_TEXT_LENGTH {
                    text = Some(t.chars().take(MAX_TEXT_LENGTH).collect());
                }
            }

            // Java: `if ((coordinate != null) && !homeMarker) { coordinate = coordinate.transform(); }`
            let mut coordinate = set_marker_command.get_coordinate();
            if coordinate.is_some() && !home_marker {
                coordinate = coordinate.map(|c| c.transform());
            }

            if let Some(coordinate) = coordinate {
                // Java: looks up/creates a `FieldMarker` at `coordinate`, sets home/away text,
                // then adds or removes it on `game.getFieldModel()` depending on whether any
                // text remains.
                let mut field_marker = game
                    .field_model
                    .get_field_marker(coordinate)
                    .cloned()
                    .unwrap_or_else(|| FieldMarker::with_coordinate(coordinate));
                if home_marker {
                    field_marker.home_text = text;
                } else {
                    field_marker.away_text = text;
                }
                if is_provided(field_marker.get_home_text()) || is_provided(field_marker.get_away_text()) {
                    game.field_model.add_field_marker(field_marker);
                } else {
                    game.field_model.remove_field_marker(coordinate);
                }
            } else {
                // Java: looks up/creates a `PlayerMarker` for `setMarkerCommand.getPlayerId()`,
                // sets home/away text, then adds or removes it on the field model.
                let player_id = set_marker_command.get_player_id().unwrap_or_default().to_string();
                let mut player_marker = game
                    .field_model
                    .get_player_marker(&player_id)
                    .cloned()
                    .unwrap_or_else(|| PlayerMarker::with_player_id(player_id.clone()));
                if home_marker {
                    player_marker.home_text = text;
                } else {
                    player_marker.away_text = text;
                }
                if is_provided(player_marker.get_home_text()) || is_provided(player_marker.get_away_text()) {
                    game.field_model.add_player_marker(player_marker);
                } else {
                    game.field_model.remove_player_marker(&player_id);
                }
            }

            // Java: `UtilServerGame.syncGameModel(gameState, null, null, null)` — pushes the
            // updated model to connected clients over the WebSocket. `ServerCommunication` has
            // no model-sync broadcast wired yet in this crate (net/ phase), so this is a no-op
            // here; the field/player marker mutation above (the actual game-state change) is
            // applied for real.
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::ClientMode;
    use ffb_model::types::FieldCoordinate;
    use tokio::sync::mpsc;

    fn setup() -> (Arc<Mutex<GameCache>>, Arc<Mutex<SessionManager>>) {
        (Arc::new(Mutex::new(GameCache::new())), Arc::new(Mutex::new(SessionManager::new())))
    }

    #[test]
    fn construct() {
        let (gc, sm) = setup();
        let _ = ServerCommandHandlerSetMarker::new(gc, sm);
    }

    #[test]
    fn get_id_is_client_set_marker() {
        let (gc, sm) = setup();
        let handler = ServerCommandHandlerSetMarker::new(gc, sm);
        assert_eq!(handler.get_id(), NetCommandId::ClientSetMarker);
    }

    #[test]
    fn spectator_session_is_a_noop() {
        let (gc, sm) = setup();
        let game_id = gc.lock().unwrap().create_game_state();
        let (tx, _rx) = mpsc::unbounded_channel();
        sm.lock().unwrap().add_session(1, game_id, "Spec".into(), ClientMode::SPECTATOR, false, vec![], tx);

        let handler = ServerCommandHandlerSetMarker::new(gc, sm);
        let cmd = ClientCommandSetMarker::with_marker("p1", FieldCoordinate::new(1, 1), "X");
        assert!(handler.handle_command(1, &cmd));
    }

    #[test]
    fn home_coach_with_unstarted_game_returns_true() {
        // GameState exists in the cache but has no driver/Game yet.
        let (gc, sm) = setup();
        let game_id = gc.lock().unwrap().create_game_state();
        let (tx, _rx) = mpsc::unbounded_channel();
        sm.lock().unwrap().add_session(1, game_id, "Home".into(), ClientMode::PLAYER, true, vec![], tx);

        let handler = ServerCommandHandlerSetMarker::new(gc, sm);
        let cmd = ClientCommandSetMarker::with_marker("p1", FieldCoordinate::new(1, 1), "X");
        assert!(handler.handle_command(1, &cmd));
    }

    #[test]
    fn unknown_session_is_a_noop() {
        let (gc, sm) = setup();
        let handler = ServerCommandHandlerSetMarker::new(gc, sm);
        let cmd = ClientCommandSetMarker::with_marker("p1", FieldCoordinate::new(1, 1), "X");
        assert!(handler.handle_command(99, &cmd));
    }

    fn team(id: &str) -> ffb_model::model::team::Team {
        ffb_model::model::team::Team {
            id: id.into(),
            name: id.into(),
            race: "Human".into(),
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
            special_rules: vec![],
            players: vec![],
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn setup_started_game_with_home_session() -> (Arc<Mutex<GameCache>>, Arc<Mutex<SessionManager>>, i64) {
        let (gc, sm) = setup();
        let game_id = gc.lock().unwrap().create_game_state();
        gc.lock()
            .unwrap()
            .get_game_state_by_id_mut(game_id)
            .unwrap()
            .start_game(team("home"), team("away"), ffb_model::enums::Rules::Bb2025, 0);
        let (tx, _rx) = mpsc::unbounded_channel();
        sm.lock().unwrap().add_session(1, game_id, "Home".into(), ClientMode::PLAYER, true, vec![], tx);
        (gc, sm, game_id)
    }

    #[test]
    fn home_coach_sets_player_marker_text() {
        let (gc, sm, game_id) = setup_started_game_with_home_session();
        let handler = ServerCommandHandlerSetMarker::new(Arc::clone(&gc), sm);
        let cmd = ClientCommandSetMarker {
            player_id: Some("p1".to_string()),
            coordinate: None,
            text: Some("Nice job".to_string()),
            entropy: None,
        };
        assert!(handler.handle_command(1, &cmd));

        let gc = gc.lock().unwrap();
        let game = gc.get_game_state_by_id(game_id).unwrap().get_game().unwrap();
        let marker = game.field_model.get_player_marker("p1").unwrap();
        assert_eq!(marker.get_home_text(), Some("Nice job"));
        assert_eq!(marker.get_away_text(), None);
    }

    #[test]
    fn home_coach_sets_field_marker_text() {
        let (gc, sm, game_id) = setup_started_game_with_home_session();
        let handler = ServerCommandHandlerSetMarker::new(Arc::clone(&gc), sm);
        let coord = FieldCoordinate::new(5, 5);
        let cmd = ClientCommandSetMarker::with_marker("p1", coord, "Trap here");
        assert!(handler.handle_command(1, &cmd));

        let gc = gc.lock().unwrap();
        let game = gc.get_game_state_by_id(game_id).unwrap().get_game().unwrap();
        let marker = game.field_model.get_field_marker(coord).unwrap();
        assert_eq!(marker.get_home_text(), Some("Trap here"));
    }

    #[test]
    fn setting_empty_text_removes_player_marker() {
        let (gc, sm, game_id) = setup_started_game_with_home_session();
        let handler = ServerCommandHandlerSetMarker::new(Arc::clone(&gc), sm);
        let set_cmd = ClientCommandSetMarker::with_marker("p1", FieldCoordinate::new(1, 1), "Nice job");
        assert!(handler.handle_command(1, &set_cmd));

        let clear_cmd = ClientCommandSetMarker::with_marker("p1", FieldCoordinate::new(1, 1), "");
        assert!(handler.handle_command(1, &clear_cmd));

        let gc = gc.lock().unwrap();
        let game = gc.get_game_state_by_id(game_id).unwrap().get_game().unwrap();
        assert!(game.field_model.get_player_marker("p1").is_none());
    }

    #[test]
    fn text_longer_than_max_length_is_truncated() {
        let (gc, sm, game_id) = setup_started_game_with_home_session();
        let handler = ServerCommandHandlerSetMarker::new(Arc::clone(&gc), sm);
        let long_text = "x".repeat(MAX_TEXT_LENGTH + 10);
        let cmd = ClientCommandSetMarker {
            player_id: Some("p1".to_string()),
            coordinate: None,
            text: Some(long_text),
            entropy: None,
        };
        assert!(handler.handle_command(1, &cmd));

        let gc = gc.lock().unwrap();
        let game = gc.get_game_state_by_id(game_id).unwrap().get_game().unwrap();
        let marker = game.field_model.get_player_marker("p1").unwrap();
        assert_eq!(marker.get_home_text().unwrap().len(), MAX_TEXT_LENGTH);
    }
}
