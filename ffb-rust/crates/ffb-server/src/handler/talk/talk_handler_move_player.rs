/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerMovePlayer.
/// Abstract handler for /move_player command — moves a player by direction and distance.
///
/// Java's `handle()` resolves the acting session's away-coach status via
/// `server.getSessionManager().getSessionOfAwayCoach(game.getId())` to mirror the
/// direction before applying it; that lookup is passed in explicitly here
/// (`game_id`, `session`, `session_manager`) since Rust has no `FantasyFootballServer`/
/// `GameState` bundle yet. Any parse/lookup failure along the way is swallowed,
/// mirroring Java's `catch (Exception e) { // ignored }`.
use std::collections::HashSet;
use ffb_model::enums::Direction;
use ffb_model::model::field_model::FieldModel;
use ffb_model::model::team::Team;
use crate::handler::talk::command_adapter::CommandAdapter;
use crate::handler::talk::talk_handler::TalkHandler;
use crate::handler::talk::talk_requirements::{Client, Environment, Privilege};
use crate::model::received_command::SessionId;
use crate::net::session_manager::SessionManager;

pub struct TalkHandlerMovePlayer {
    base: TalkHandler,
}

impl TalkHandlerMovePlayer {
    /// Java: `TalkHandlerMovePlayer(CommandAdapter, Client, Environment, Privilege...)`.
    pub fn new(
        command_adapter: &dyn CommandAdapter,
        required_client: Client,
        required_env: Environment,
        requires_one_privilege_of: HashSet<Privilege>,
    ) -> Self {
        let mut commands = HashSet::new();
        commands.insert("/move_player".to_string());
        let commands = command_adapter.decorate_commands(commands);
        Self {
            base: TalkHandler::new(commands, 3, required_client, required_env, requires_one_privilege_of),
        }
    }

    pub fn base(&self) -> &TalkHandler { &self.base }

    /// Java: `handle(FantasyFootballServer, GameState, String[], Team, Session)` — moves
    /// the player named by `commands[1]` (roster number) `commands[3]` squares in the
    /// direction named by `commands[2]`, mirroring the direction if the away coach issued
    /// the command. Returns the info message(s) Java would have sent via
    /// `movePlayerToCoordinate` (which itself sends via `communication.sendPlayerTalk`).
    pub fn handle(
        &self,
        field_model: &mut FieldModel,
        commands: &[String],
        team: &Team,
        game_id: i64,
        session: SessionId,
        session_manager: &SessionManager,
    ) -> Vec<String> {
        let nr: i32 = match commands.get(1).and_then(|s| s.parse().ok()) {
            Some(nr) => nr,
            None => return Vec::new(),
        };
        let player = match team.player_by_nr(nr) {
            Some(p) => p,
            None => return Vec::new(),
        };
        let start_coordinate = match field_model.player_coordinate(&player.id) {
            Some(c) => c,
            None => return Vec::new(),
        };
        let mut direction = match commands.get(2).and_then(|s| Direction::from_name(s)) {
            Some(d) => d,
            None => return Vec::new(),
        };
        if session_manager.get_session_of_away_coach(game_id) == Some(session) {
            direction = direction.transform();
        }
        let distance: i32 = match commands.get(3).and_then(|s| s.parse().ok()) {
            Some(d) => d,
            None => return Vec::new(),
        };
        let coordinate = start_coordinate.step(direction, distance);
        let player_id = player.id.clone();
        let player_name = player.name.clone();
        self.base.move_player_to_coordinate(field_model, &player_id, &player_name, coordinate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::model::ClientMode;
    use ffb_model::types::FieldCoordinate;
    use crate::handler::talk::identity_command_adapter::IdentityCommandAdapter;
    use std::collections::HashSet as Set;

    fn make_player(id: &str, nr: i32) -> ffb_model::model::player::Player {
        ffb_model::model::player::Player {
            id: id.into(), name: format!("Player{nr}"), nr, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Set::new(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None, is_big_guy: false,
            ..Default::default()
        }
    }

    fn make_team(players: Vec<ffb_model::model::player::Player>) -> Team {
        Team {
            id: "t".into(), name: "Team".into(), race: "Human".into(), roster_id: "human".into(),
            coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players, vampire_lord: false, necromancer: false,
        }
    }

    fn handler() -> TalkHandlerMovePlayer {
        let adapter = IdentityCommandAdapter::new();
        TalkHandlerMovePlayer::new(&adapter, Client::Player, Environment::None, HashSet::new())
    }

    fn session_manager_with(home: SessionId, away: SessionId, game_id: i64) -> SessionManager {
        let mut sm = SessionManager::new();
        let (tx1, _) = tokio::sync::mpsc::unbounded_channel();
        let (tx2, _) = tokio::sync::mpsc::unbounded_channel();
        sm.add_session(home, game_id, "Home".into(), ClientMode::PLAYER, true, vec![], tx1);
        sm.add_session(away, game_id, "Away".into(), ClientMode::PLAYER, false, vec![], tx2);
        sm
    }

    #[test]
    fn construct() { let _ = handler(); }

    #[test]
    fn handle_moves_player_by_direction_and_distance() {
        let h = handler();
        let team = make_team(vec![make_player("p1", 1)]);
        let mut fm = FieldModel::default();
        fm.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        let sm = session_manager_with(1, 2, 100);
        let commands = vec!["/move_player".to_string(), "1".to_string(), "East".to_string(), "2".to_string()];
        let info = h.handle(&mut fm, &commands, &team, 100, 1, &sm);
        assert_eq!(fm.player_coordinate("p1"), Some(FieldCoordinate::new(7, 5)));
        assert!(!info.is_empty());
    }

    #[test]
    fn handle_mirrors_direction_for_away_coach_session() {
        let h = handler();
        let team = make_team(vec![make_player("p1", 1)]);
        let mut fm = FieldModel::default();
        fm.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        let sm = session_manager_with(1, 2, 100);
        let commands = vec!["/move_player".to_string(), "1".to_string(), "East".to_string(), "2".to_string()];
        let info = h.handle(&mut fm, &commands, &team, 100, 2, &sm);
        assert_eq!(fm.player_coordinate("p1"), Some(FieldCoordinate::new(3, 5)));
        assert!(!info.is_empty());
    }

    #[test]
    fn handle_ignores_unknown_player_number() {
        let h = handler();
        let team = make_team(vec![make_player("p1", 1)]);
        let mut fm = FieldModel::default();
        fm.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        let sm = session_manager_with(1, 2, 100);
        let commands = vec!["/move_player".to_string(), "99".to_string(), "East".to_string(), "2".to_string()];
        let info = h.handle(&mut fm, &commands, &team, 100, 1, &sm);
        assert!(info.is_empty());
        assert_eq!(fm.player_coordinate("p1"), Some(FieldCoordinate::new(5, 5)));
    }

    #[test]
    fn handle_ignores_unknown_direction() {
        let h = handler();
        let team = make_team(vec![make_player("p1", 1)]);
        let mut fm = FieldModel::default();
        fm.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        let sm = session_manager_with(1, 2, 100);
        let commands = vec!["/move_player".to_string(), "1".to_string(), "nowhere".to_string(), "2".to_string()];
        let info = h.handle(&mut fm, &commands, &team, 100, 1, &sm);
        assert!(info.is_empty());
        assert_eq!(fm.player_coordinate("p1"), Some(FieldCoordinate::new(5, 5)));
    }

    #[test]
    fn handle_ignores_missing_coordinate_for_player_not_on_field() {
        let h = handler();
        let team = make_team(vec![make_player("p1", 1)]);
        let mut fm = FieldModel::default();
        let sm = session_manager_with(1, 2, 100);
        let commands = vec!["/move_player".to_string(), "1".to_string(), "East".to_string(), "2".to_string()];
        let info = h.handle(&mut fm, &commands, &team, 100, 1, &sm);
        assert!(info.is_empty());
    }
}
