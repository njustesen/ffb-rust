/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerSetBall.
/// Abstract handler for `/set_ball` command — places the ball at a given coordinate.
use ffb_model::model::game::Game;
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds};
use crate::net::session_manager::SessionManager;
use crate::model::received_command::SessionId;
use super::talk_requirements::{Client, Environment, Privilege};

pub struct TalkHandlerSetBall {
    pub required_client: Client,
    pub required_environment: Environment,
    pub requires_one_privilege_of: Vec<Privilege>,
}

impl TalkHandlerSetBall {
    /// Java: `super("/set_ball", 2, ...)`.
    pub const COMMAND: &'static str = "/set_ball";
    pub const COMMAND_PARTS_THRESHOLD: usize = 2;

    pub fn new(
        required_client: Client,
        required_environment: Environment,
        requires_one_privilege_of: Vec<Privilege>,
    ) -> Self {
        Self { required_client, required_environment, requires_one_privilege_of }
    }

    /// Java: `handle(FantasyFootballServer, GameState, String[], Team, Session)` — parses
    /// `commands[1]`/`commands[2]` as x/y, mirrors the coordinate when the away coach issued
    /// the command, then moves the ball. Returns the info message that would be sent via
    /// `server.getCommunication().sendPlayerTalk(...)`. Malformed coordinates are ignored,
    /// matching the Java `catch (Exception e) { // ignored }`.
    pub fn handle(
        &self,
        game: &mut Game,
        session_manager: &SessionManager,
        game_id: i64,
        session: SessionId,
        commands: &[String],
    ) -> Option<String> {
        let x: i32 = commands.get(1)?.parse().ok()?;
        let y: i32 = commands.get(2)?.parse().ok()?;
        let mut coordinate = FieldCoordinate::new(x, y);
        if session_manager.get_session_of_away_coach(game_id) == Some(session) {
            coordinate = coordinate.transform();
        }
        Some(Self::move_ball_to_coordinate(game, coordinate))
    }

    /// Java: `TalkHandler.moveBallToCoordinate`. Duplicated locally because the abstract
    /// `TalkHandler` base class is owned by a concurrent translation pass.
    pub(super) fn move_ball_to_coordinate(game: &mut Game, coordinate: FieldCoordinate) -> String {
        if !FieldCoordinateBounds::FIELD.is_in_bounds(coordinate) {
            return format!("Coordinate {coordinate} is not on the pitch.");
        }
        game.field_model.ball_coordinate = Some(coordinate);
        game.field_model.ball_moving = game.field_model.player_at(coordinate).is_none();
        format!("Set ball to coordinate {coordinate}.")
    }
}

impl Default for TalkHandlerSetBall {
    fn default() -> Self {
        Self::new(Client::None, Environment::None, Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::ClientMode;
    use ffb_model::model::team::Team;

    fn team(name: &str) -> Team {
        Team {
            id: name.into(), name: name.into(), race: "Human".into(), roster_id: "human".into(),
            coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false,
        }
    }

    fn game() -> Game {
        let mut g = Game::new(team("Home"), team("Away"), Rules::Bb2020);
        g.id = 100;
        g
    }

    fn session_manager_with(home: SessionId, away: SessionId) -> SessionManager {
        let mut sm = SessionManager::new();
        let (tx1, _) = tokio::sync::mpsc::unbounded_channel();
        let (tx2, _) = tokio::sync::mpsc::unbounded_channel();
        sm.add_session(home, 100, "Home".into(), ClientMode::PLAYER, true, vec![], tx1);
        sm.add_session(away, 100, "Away".into(), ClientMode::PLAYER, false, vec![], tx2);
        sm
    }

    #[test]
    fn construct() {
        let h = TalkHandlerSetBall::new(Client::Spec, Environment::None, vec![Privilege::EditState]);
        assert_eq!(h.required_client, Client::Spec);
    }

    #[test]
    fn handle_moves_ball_to_coordinate() {
        let h = TalkHandlerSetBall::default();
        let mut g = game();
        let sm = session_manager_with(1, 2);
        let commands = vec!["/set_ball".to_string(), "5".to_string(), "6".to_string()];
        let info = h.handle(&mut g, &sm, 100, 1, &commands).unwrap();
        assert_eq!(g.field_model.ball_coordinate, Some(FieldCoordinate::new(5, 6)));
        assert!(info.contains("Set ball to coordinate"));
    }

    #[test]
    fn handle_mirrors_coordinate_for_away_session() {
        let h = TalkHandlerSetBall::default();
        let mut g = game();
        let sm = session_manager_with(1, 2);
        let commands = vec!["/set_ball".to_string(), "5".to_string(), "6".to_string()];
        h.handle(&mut g, &sm, 100, 2, &commands).unwrap();
        let expected = FieldCoordinate::new(5, 6).transform();
        assert_eq!(g.field_model.ball_coordinate, Some(expected));
    }

    #[test]
    fn handle_ignores_malformed_coordinate() {
        let h = TalkHandlerSetBall::default();
        let mut g = game();
        let sm = session_manager_with(1, 2);
        let commands = vec!["/set_ball".to_string(), "not_a_number".to_string()];
        assert!(h.handle(&mut g, &sm, 100, 1, &commands).is_none());
        assert!(g.field_model.ball_coordinate.is_none());
    }

    #[test]
    fn handle_rejects_out_of_bounds_coordinate() {
        let h = TalkHandlerSetBall::default();
        let mut g = game();
        let sm = session_manager_with(1, 2);
        let commands = vec!["/set_ball".to_string(), "99".to_string(), "99".to_string()];
        let info = h.handle(&mut g, &sm, 100, 1, &commands).unwrap();
        assert!(info.contains("not on the pitch"));
        assert!(g.field_model.ball_coordinate.is_none());
    }
}
