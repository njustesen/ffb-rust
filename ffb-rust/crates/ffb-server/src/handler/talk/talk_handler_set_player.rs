/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerSetPlayer.
/// Abstract handler for `/set_player` command — places a player at a given coordinate.
use ffb_model::model::game::Game;
use ffb_model::model::team::Team;
use ffb_model::enums::PlayerState;
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds};
use crate::net::session_manager::SessionManager;
use crate::model::received_command::SessionId;
use super::talk_requirements::{Client, Environment, Privilege};

pub struct TalkHandlerSetPlayer {
    pub required_client: Client,
    pub required_environment: Environment,
    pub requires_one_privilege_of: Vec<Privilege>,
}

impl TalkHandlerSetPlayer {
    /// Java: `super("/set_player", 3, ...)`.
    pub const COMMAND: &'static str = "/set_player";
    pub const COMMAND_PARTS_THRESHOLD: usize = 3;

    pub fn new(
        required_client: Client,
        required_environment: Environment,
        requires_one_privilege_of: Vec<Privilege>,
    ) -> Self {
        Self { required_client, required_environment, requires_one_privilege_of }
    }

    /// Java: `handle(...)` — `commands[1]` is the player number, `commands[2]`/`commands[3]`
    /// are the target x/y, mirrored when the away coach issued the command. Malformed input
    /// or an unknown player number is ignored, matching Java's `catch (Exception e)`.
    pub fn handle(
        &self,
        game: &mut Game,
        team: &Team,
        session_manager: &SessionManager,
        game_id: i64,
        session: SessionId,
        commands: &[String],
    ) -> Option<String> {
        let nr: i32 = commands.get(1)?.parse().ok()?;
        let player_id = team.player_by_nr(nr)?.id.clone();
        let x: i32 = commands.get(2)?.parse().ok()?;
        let y: i32 = commands.get(3)?.parse().ok()?;
        let mut coordinate = FieldCoordinate::new(x, y);
        if session_manager.get_session_of_away_coach(game_id) == Some(session) {
            coordinate = coordinate.transform();
        }
        Some(Self::move_player_to_coordinate(game, &player_id, coordinate))
    }

    /// Java: `TalkHandler.movePlayerToCoordinate`. Duplicated locally because the abstract
    /// `TalkHandler` base class is owned by a concurrent translation pass.
    pub(super) fn move_player_to_coordinate(game: &mut Game, player_id: &str, coordinate: FieldCoordinate) -> String {
        if !FieldCoordinateBounds::FIELD.is_in_bounds(coordinate) {
            return format!("Coordinate {coordinate} is not on the pitch.");
        }
        if let Some(occupying) = game.field_model.player_at(coordinate) {
            if occupying != player_id {
                let name = game
                    .team_home
                    .player(occupying)
                    .or_else(|| game.team_away.player(occupying))
                    .map(|p| p.name.clone())
                    .unwrap_or_default();
                return format!("Coordinate {coordinate} already occupied by {name}.");
            }
        }
        game.field_model.set_player_coordinate(player_id, coordinate);
        let player_name = game
            .team_home
            .player(player_id)
            .or_else(|| game.team_away.player(player_id))
            .map(|p| p.name.clone())
            .unwrap_or_default();
        let mut info = format!("Set player {player_name} to coordinate {coordinate}.");
        if let Some(state) = game.field_model.player_state(player_id) {
            if coordinate.is_box_coordinate() {
                game.field_model.set_player_state(player_id, state.change_base(ffb_model::enums::PS_RESERVE));
                info.push_str(&format!("\nSet playerState of {player_name} to RESERVE."));
            } else if state.base() == ffb_model::enums::PS_RESERVE {
                info.push_str(&format!("\nSet playerState of {player_name} to STANDING."));
                game.field_model.set_player_state(player_id, state.change_base(ffb_model::enums::PS_STANDING));
            }
        }
        info
    }
}

impl Default for TalkHandlerSetPlayer {
    fn default() -> Self {
        Self::new(Client::None, Environment::None, Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::ClientMode;
    use ffb_model::model::player::Player;

    fn player(id: &str, nr: i32) -> Player {
        Player { id: id.into(), name: format!("Player{nr}"), nr, ..Player::default() }
    }

    fn team(name: &str, p: Player) -> Team {
        Team {
            id: name.into(), name: name.into(), race: "Human".into(), roster_id: "human".into(),
            coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![p], vampire_lord: false, necromancer: false,
        }
    }

    fn game() -> Game {
        let mut g = Game::new(team("Home", player("p1", 1)), team("Away", player("p2", 1)), Rules::Bb2020);
        g.id = 100;
        g.field_model.set_player_coordinate("p1", FieldCoordinate::new(0, 0));
        g.field_model.set_player_state("p1", PlayerState::new(ffb_model::enums::PS_RESERVE));
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
        let h = TalkHandlerSetPlayer::new(Client::Spec, Environment::None, vec![Privilege::EditState]);
        assert_eq!(h.required_client, Client::Spec);
    }

    #[test]
    fn handle_moves_player_and_promotes_from_reserve() {
        let h = TalkHandlerSetPlayer::default();
        let mut g = game();
        let team = g.team_home.clone();
        let sm = session_manager_with(1, 2);
        let commands = vec!["/set_player".into(), "1".into(), "5".into(), "6".into()];
        let info = h.handle(&mut g, &team, &sm, 100, 1, &commands).unwrap();
        assert_eq!(g.field_model.player_coordinate("p1"), Some(FieldCoordinate::new(5, 6)));
        assert!(info.contains("Set playerState of Player1 to STANDING"));
    }

    #[test]
    fn handle_rejects_occupied_coordinate() {
        let h = TalkHandlerSetPlayer::default();
        let mut g = game();
        g.field_model.set_player_coordinate("p2", FieldCoordinate::new(5, 6));
        let team = g.team_home.clone();
        let sm = session_manager_with(1, 2);
        let commands = vec!["/set_player".into(), "1".into(), "5".into(), "6".into()];
        let info = h.handle(&mut g, &team, &sm, 100, 1, &commands).unwrap();
        assert!(info.contains("already occupied"));
    }

    #[test]
    fn handle_unknown_player_number_returns_none() {
        let h = TalkHandlerSetPlayer::default();
        let mut g = game();
        let team = g.team_home.clone();
        let sm = session_manager_with(1, 2);
        let commands = vec!["/set_player".into(), "99".into(), "5".into(), "6".into()];
        assert!(h.handle(&mut g, &team, &sm, 100, 1, &commands).is_none());
    }
}
