/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerMoveBall.
/// Abstract handler for /move_ball command — moves ball by direction and distance.
use std::collections::HashSet;
use ffb_model::model::field_model::FieldModel;
use ffb_model::enums::Direction;
use crate::handler::talk::command_adapter::CommandAdapter;
use crate::handler::talk::talk_handler::TalkHandler;
use crate::handler::talk::talk_requirements::{Client, Environment, Privilege};
use crate::model::received_command::SessionId;
use crate::net::session_manager::SessionManager;

pub struct TalkHandlerMoveBall {
    base: TalkHandler,
}

impl TalkHandlerMoveBall {
    /// Java: `TalkHandlerMoveBall(CommandAdapter, Client, Environment, Privilege...)`.
    pub fn new(
        command_adapter: &dyn CommandAdapter,
        required_client: Client,
        required_env: Environment,
        requires_one_privilege_of: HashSet<Privilege>,
    ) -> Self {
        let mut commands = HashSet::new();
        commands.insert("/move_ball".to_string());
        let commands = command_adapter.decorate_commands(commands);
        Self {
            base: TalkHandler::new(commands, 2, required_client, required_env, requires_one_privilege_of),
        }
    }

    pub fn base(&self) -> &TalkHandler { &self.base }

    /// Java: `handle(FantasyFootballServer, GameState, String[], Team, Session)` — moves the
    /// ball by the direction/distance named in `commands[1..2]`, mirroring the away-coach
    /// direction flip and the try/catch-all-and-ignore around parsing. Returns the info
    /// message(s) `moveBallToCoordinate` would have sent (see `talk_handler.rs` doc), or an
    /// empty vec if anything failed to parse (Java's swallowed exception).
    pub fn handle(
        &self,
        field_model: &mut FieldModel,
        commands: &[String],
        session_manager: &SessionManager,
        game_id: i64,
        session: SessionId,
    ) -> Vec<String> {
        let start_coordinate = match field_model.ball_coordinate {
            Some(c) => c,
            None => return Vec::new(),
        };

        let direction = match Direction::from_name(&commands[1]) {
            Some(d) => d,
            None => return Vec::new(),
        };

        let direction = if session_manager.get_session_of_away_coach(game_id) == Some(session) {
            direction.transform()
        } else {
            direction
        };

        let distance: i32 = match commands[2].parse() {
            Ok(d) => d,
            Err(_) => return Vec::new(),
        };

        let coordinate = start_coordinate.step(direction, distance);
        self.base.move_ball_to_coordinate(field_model, coordinate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::types::FieldCoordinate;
    use crate::handler::talk::identity_command_adapter::IdentityCommandAdapter;

    fn handler() -> TalkHandlerMoveBall {
        let adapter = IdentityCommandAdapter::new();
        TalkHandlerMoveBall::new(&adapter, Client::Player, Environment::None, HashSet::new())
    }

    #[test]
    fn construct() { let _ = handler(); }

    #[test]
    fn handle_moves_ball_by_direction_and_distance() {
        let h = handler();
        let sm = SessionManager::new();
        let mut fm = FieldModel::default();
        fm.ball_coordinate = Some(FieldCoordinate::new(5, 5));
        let commands = vec!["/move_ball".to_string(), "East".to_string(), "2".to_string()];
        let info = h.handle(&mut fm, &commands, &sm, 100, 1);
        assert_eq!(fm.ball_coordinate, Some(FieldCoordinate::new(7, 5)));
        assert!(!info.is_empty());
    }

    #[test]
    fn handle_flips_direction_for_away_coach_session() {
        let h = handler();
        let mut sm = SessionManager::new();
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        sm.add_session(2, 100, "Away".into(), ffb_model::model::ClientMode::PLAYER, false, vec![], tx);
        let mut fm = FieldModel::default();
        fm.ball_coordinate = Some(FieldCoordinate::new(5, 5));
        let commands = vec!["/move_ball".to_string(), "East".to_string(), "2".to_string()];
        let info = h.handle(&mut fm, &commands, &sm, 100, 2);
        // East transformed (mirrored) becomes West.
        assert_eq!(fm.ball_coordinate, Some(FieldCoordinate::new(3, 5)));
        assert!(!info.is_empty());
    }

    #[test]
    fn handle_ignores_unknown_direction() {
        let h = handler();
        let sm = SessionManager::new();
        let mut fm = FieldModel::default();
        fm.ball_coordinate = Some(FieldCoordinate::new(5, 5));
        let commands = vec!["/move_ball".to_string(), "nowhere".to_string(), "2".to_string()];
        let info = h.handle(&mut fm, &commands, &sm, 100, 1);
        assert!(info.is_empty());
        assert_eq!(fm.ball_coordinate, Some(FieldCoordinate::new(5, 5)));
    }

    #[test]
    fn handle_ignores_unparseable_distance() {
        let h = handler();
        let sm = SessionManager::new();
        let mut fm = FieldModel::default();
        fm.ball_coordinate = Some(FieldCoordinate::new(5, 5));
        let commands = vec!["/move_ball".to_string(), "East".to_string(), "far".to_string()];
        let info = h.handle(&mut fm, &commands, &sm, 100, 1);
        assert!(info.is_empty());
    }

    #[test]
    fn handle_ignores_when_ball_has_no_coordinate() {
        let h = handler();
        let sm = SessionManager::new();
        let mut fm = FieldModel::default();
        let commands = vec!["/move_ball".to_string(), "East".to_string(), "2".to_string()];
        let info = h.handle(&mut fm, &commands, &sm, 100, 1);
        assert!(info.is_empty());
    }
}
