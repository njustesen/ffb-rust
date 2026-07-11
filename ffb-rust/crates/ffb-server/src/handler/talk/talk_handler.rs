/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandler.
///
/// Java's `TalkHandler` is an abstract class: the constructor takes the
/// recognized command set / privilege requirements, `handle()` parses and
/// validates a talk command, then dispatches to the abstract `handle(...)`
/// implemented by each concrete subclass. Rust has no inheritance, so this
/// struct holds the same configuration fields and exposes `handle()` as a
/// "parse + validate + resolve team" step; concrete `talk_handler_*.rs`
/// files call it, then run their own domain logic using the returned parts.
///
/// Java's `server.getCommunication().sendXxx(...)` calls have no wired
/// outbound-send equivalent in the Rust `ServerCommunication` yet (it only
/// dispatches *inbound* commands today) — helper methods below return the
/// info string(s) Java would have sent, so callers can feed them into
/// `SessionManager::send_to`/`send_all` once that wiring exists, and so the
/// logic stays fully testable without a live session.
use std::collections::HashSet;
use ffb_model::model::field_model::FieldModel;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::team::Team;
use ffb_model::enums::{PlayerState, PS_RESERVE, PS_STANDING};
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds};
use crate::model::received_command::SessionId;
use crate::net::session_manager::SessionManager;
use crate::handler::talk::talk_requirements::{Client, Environment, Privilege};

/// Java: `TalkHandler` constructor fields (`fCommands`, `commandPartsThreshold`,
/// `requiresOnePrivilegeOf`, `requiredClientMode`, `requiredEnvironment`).
pub struct TalkHandler {
    commands: HashSet<String>,
    pub command_parts_threshold: usize,
    requires_one_privilege_of: HashSet<Privilege>,
    required_client_mode: Client,
    required_environment: Environment,
}

impl TalkHandler {
    /// Java: `TalkHandler(Set<String> commands, int commandPartsThreshold, ...)`.
    /// The `commandAdapter.decorateCommands(commands)` call is the caller's
    /// responsibility (each concrete handler picks its own `CommandAdapter`).
    pub fn new(
        commands: HashSet<String>,
        command_parts_threshold: usize,
        required_client_mode: Client,
        required_environment: Environment,
        requires_one_privilege_of: HashSet<Privilege>,
    ) -> Self {
        Self {
            commands,
            command_parts_threshold,
            requires_one_privilege_of,
            required_client_mode,
            required_environment,
        }
    }

    /// Java: `handle(FantasyFootballServer, ClientCommandTalk, Session)` —
    /// validates requirements and returns the parsed command tokens for the
    /// caller to dispatch on. Returns `None` if the command doesn't apply
    /// (mirrors Java returning `false`).
    pub fn handle(
        &self,
        session_manager: &SessionManager,
        game_id: i64,
        game_exists: bool,
        session: SessionId,
        talk: &str,
        server_test_mode: bool,
        game_is_testing: bool,
    ) -> Option<Vec<String>> {
        let commands: Vec<String> = talk.split(' ').filter(|s| !s.is_empty()).map(String::from).collect();
        if commands.len() <= self.command_parts_threshold {
            return None;
        }

        if !self.handles(session_manager, game_id, game_exists, session, &commands[0], server_test_mode, game_is_testing) {
            return None;
        }

        Some(commands)
    }

    /// Java: `handles(FantasyFootballServer, String, Session)` — checks command
    /// match, client mode, environment, privileges.
    fn handles(
        &self,
        session_manager: &SessionManager,
        game_id: i64,
        game_exists: bool,
        session: SessionId,
        command: &str,
        server_test_mode: bool,
        game_is_testing: bool,
    ) -> bool {
        game_exists
            && self.commands.contains(command)
            && self.required_client_mode.is_met(session_manager, game_id, session)
            && self.required_environment.is_met(server_test_mode, game_is_testing)
            && (self.requires_one_privilege_of.is_empty()
                || self.requires_one_privilege_of.iter().any(|p| p.is_met(session_manager, session)))
    }

    /// Java: `findPlayersInCommand(Team, String[])` — finds players referenced
    /// in talk command args.
    pub fn find_players_in_command<'t>(&self, team: &'t Team, commands: &[String]) -> Vec<&'t Player> {
        let mut players: Vec<&Player> = Vec::new();
        if !commands.is_empty() && self.command_parts_threshold < commands.len() {
            if commands[self.command_parts_threshold].eq_ignore_ascii_case("all") {
                for p in &team.players {
                    players.push(p);
                }
            } else {
                for token in &commands[self.command_parts_threshold..] {
                    if let Ok(nr) = token.parse::<i32>() {
                        if let Some(player) = team.player_by_nr(nr) {
                            if !players.iter().any(|p| p.id == player.id) {
                                players.push(player);
                            }
                        }
                    }
                }
            }
        }
        players
    }

    /// Java: `putPlayerIntoBox(GameState, ServerCommunication, Player, PlayerState, String, SeriousInjury)`
    /// — moves a player to the dugout box. Returns the info message Java would
    /// have sent via `communication.sendPlayerTalk`.
    pub fn put_player_into_box(
        &self,
        game: &mut Game,
        player_id: &str,
        player_state: PlayerState,
        box_name: &str,
    ) -> String {
        game.field_model.set_player_state(player_id, player_state);
        ffb_model::util::util_box::UtilBox::put_player_into_box(game, player_id);
        let name = game.player(player_id).map(|p| p.name.clone()).unwrap_or_default();
        format!("Player {name} moved into box {box_name}.")
    }

    /// Java: `handleSpecs(FantasyFootballServer, GameState, Session, boolean)` —
    /// builds the spectator listing message lines. `spectators` must already be
    /// sorted case-insensitively by the caller (Java sorts via `SpecsComparator`).
    pub fn handle_specs(&self, spectators: &[String], issued_by_spec: bool) -> Vec<String> {
        if spectators.is_empty() {
            vec!["There are no spectators.".to_string()]
        } else if issued_by_spec && spectators.len() == 1 {
            vec!["You are the only spectator of this game.".to_string()]
        } else {
            let mut info = Vec::with_capacity(spectators.len() + 1);
            info.push(format!("{} spectators are watching this game:", spectators.len()));
            info.extend(spectators.iter().cloned());
            info
        }
    }

    /// Java: `playSoundAfterCooldown(FantasyFootballServer, GameState, String, SoundId)`
    /// — decides whether the sound should play given the last time this coach
    /// triggered a sound, returning the new cooldown timestamp to store if so.
    /// The Java `GameState` per-coach cooldown map has no Rust equivalent yet,
    /// so the caller passes/receives the timestamp explicitly.
    pub fn play_sound_after_cooldown(
        &self,
        last_cooldown_time: i64,
        spectator_cooldown_ms: Option<i64>,
        current_time_ms: i64,
    ) -> Option<i64> {
        match spectator_cooldown_ms {
            Some(cooldown) => {
                if current_time_ms > last_cooldown_time + cooldown {
                    Some(current_time_ms)
                } else {
                    None
                }
            }
            None => Some(last_cooldown_time),
        }
    }

    /// Java: `movePlayerToCoordinate(FantasyFootballServer, GameState, Player, FieldCoordinate)`
    /// — repositions a player on the field. Returns the info message(s) Java
    /// would have sent via `communication.sendPlayerTalk`.
    pub fn move_player_to_coordinate(
        &self,
        field_model: &mut FieldModel,
        player_id: &str,
        player_name: &str,
        coordinate: FieldCoordinate,
    ) -> Vec<String> {
        if !FieldCoordinateBounds::FIELD.is_in_bounds(coordinate) {
            return vec![format!("Coordinate {coordinate:?} is not on the pitch.")];
        }

        if let Some(occupying) = field_model.player_at(coordinate) {
            if occupying != player_id {
                return vec![format!("Coordinate {coordinate:?} already occupied by {occupying}.")];
            }
        }

        field_model.set_player_coordinate(player_id, coordinate);
        let mut info = vec![format!("Set player {player_name} to coordinate {coordinate:?}.")];

        let player_state = field_model.player_state(player_id).unwrap_or(PlayerState::new(PS_STANDING));
        let mut player_state_base = player_state.base();
        if coordinate.is_box_coordinate() {
            player_state_base = PS_RESERVE;
            info.push(format!("Set playerState of {player_name} to RESERVE."));
        } else if player_state_base == PS_RESERVE {
            info.push(format!("Set playerState of {player_name} to STANDING."));
            player_state_base = PS_STANDING;
        }
        field_model.set_player_state(player_id, player_state.change_base(player_state_base));
        info
    }

    /// Java: `moveBallToCoordinate(FantasyFootballServer, GameState, FieldCoordinate)`
    /// — repositions the ball on the field.
    pub fn move_ball_to_coordinate(&self, field_model: &mut FieldModel, coordinate: FieldCoordinate) -> Vec<String> {
        if !FieldCoordinateBounds::FIELD.is_in_bounds(coordinate) {
            return vec![format!("Coordinate {coordinate:?} is not on the pitch.")];
        }

        field_model.ball_coordinate = Some(coordinate);
        field_model.ball_moving = field_model.player_at(coordinate).is_none();
        vec![format!("Set ball to coordinate {coordinate:?}.")]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerType, PlayerGender, Rules, PS_KNOCKED_OUT};
    use std::collections::HashSet as Set;

    fn handler(threshold: usize) -> TalkHandler {
        let mut commands = HashSet::new();
        commands.insert("box".to_string());
        TalkHandler::new(commands, threshold, Client::None, Environment::None, HashSet::new())
    }

    fn make_player(id: &str, nr: i32) -> Player {
        Player {
            id: id.into(), name: format!("Player{nr}"), nr, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Set::new(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None, is_big_guy: false,
            ..Default::default()
        }
    }

    fn make_team(players: Vec<Player>) -> Team {
        Team {
            id: "t".into(), name: "Team".into(), race: "Human".into(), roster_id: "human".into(),
            coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players, vampire_lord: false, necromancer: false,
        }
    }

    #[test]
    fn handle_returns_none_when_below_threshold() {
        let h = handler(2);
        let sm = SessionManager::new();
        let result = h.handle(&sm, 100, true, 1, "box 1", false, false);
        assert!(result.is_none());
    }

    #[test]
    fn handle_returns_none_when_command_unrecognized() {
        let h = handler(0);
        let sm = SessionManager::new();
        let result = h.handle(&sm, 100, true, 1, "unknown 1", false, false);
        assert!(result.is_none());
    }

    #[test]
    fn handle_returns_none_when_game_missing() {
        let h = handler(0);
        let sm = SessionManager::new();
        let result = h.handle(&sm, 100, false, 1, "box 1", false, false);
        assert!(result.is_none());
    }

    #[test]
    fn handle_returns_parsed_commands_when_valid() {
        let h = handler(0);
        let sm = SessionManager::new();
        let result = h.handle(&sm, 100, true, 1, "box  1", false, false);
        assert_eq!(result, Some(vec!["box".to_string(), "1".to_string()]));
    }

    #[test]
    fn find_players_in_command_all_returns_every_player() {
        let h = handler(1);
        let team = make_team(vec![make_player("p1", 1), make_player("p2", 2)]);
        let commands = vec!["box".to_string(), "all".to_string()];
        let players = h.find_players_in_command(&team, &commands);
        assert_eq!(players.len(), 2);
    }

    #[test]
    fn find_players_in_command_by_number() {
        let h = handler(1);
        let team = make_team(vec![make_player("p1", 1), make_player("p2", 2), make_player("p3", 3)]);
        let commands = vec!["box".to_string(), "2".to_string(), "3".to_string()];
        let players = h.find_players_in_command(&team, &commands);
        assert_eq!(players.len(), 2);
        assert!(players.iter().any(|p| p.id == "p2"));
        assert!(players.iter().any(|p| p.id == "p3"));
    }

    #[test]
    fn handle_specs_no_spectators() {
        let h = handler(0);
        let info = h.handle_specs(&[], false);
        assert_eq!(info, vec!["There are no spectators.".to_string()]);
    }

    #[test]
    fn handle_specs_only_self() {
        let h = handler(0);
        let info = h.handle_specs(&["Alice".to_string()], true);
        assert_eq!(info, vec!["You are the only spectator of this game.".to_string()]);
    }

    #[test]
    fn handle_specs_lists_multiple() {
        let h = handler(0);
        let info = h.handle_specs(&["Alice".to_string(), "Bob".to_string()], false);
        assert_eq!(info[0], "2 spectators are watching this game:");
        assert_eq!(info[1], "Alice");
        assert_eq!(info[2], "Bob");
    }

    #[test]
    fn play_sound_after_cooldown_blocks_within_window() {
        let h = handler(0);
        let result = h.play_sound_after_cooldown(1000, Some(5000), 3000);
        assert_eq!(result, None);
    }

    #[test]
    fn play_sound_after_cooldown_allows_after_window() {
        let h = handler(0);
        let result = h.play_sound_after_cooldown(1000, Some(5000), 7000);
        assert_eq!(result, Some(7000));
    }

    #[test]
    fn play_sound_after_cooldown_no_cooldown_configured() {
        let h = handler(0);
        let result = h.play_sound_after_cooldown(1000, None, 7000);
        assert_eq!(result, Some(1000));
    }

    #[test]
    fn move_player_to_coordinate_out_of_bounds() {
        let h = handler(0);
        let mut fm = FieldModel::default();
        let info = h.move_player_to_coordinate(&mut fm, "p1", "Joe", FieldCoordinate::new(-1, -1));
        assert!(info[0].contains("not on the pitch"));
    }

    #[test]
    fn move_player_to_coordinate_sets_standing_when_leaving_reserve() {
        let h = handler(0);
        let mut fm = FieldModel::default();
        fm.set_player_state("p1", PlayerState::new(PS_RESERVE));
        let info = h.move_player_to_coordinate(&mut fm, "p1", "Joe", FieldCoordinate::new(5, 5));
        assert_eq!(fm.player_coordinate("p1"), Some(FieldCoordinate::new(5, 5)));
        assert_eq!(fm.player_state("p1").unwrap().base(), PS_STANDING);
        assert!(info.iter().any(|s| s.contains("STANDING")));
    }

    #[test]
    fn move_player_to_coordinate_rejects_occupied_square() {
        let h = handler(0);
        let mut fm = FieldModel::default();
        fm.set_player_coordinate("other", FieldCoordinate::new(5, 5));
        let info = h.move_player_to_coordinate(&mut fm, "p1", "Joe", FieldCoordinate::new(5, 5));
        assert!(info[0].contains("already occupied"));
        assert_eq!(fm.player_coordinate("p1"), None);
    }

    #[test]
    fn move_ball_to_coordinate_updates_field_model() {
        let h = handler(0);
        let mut fm = FieldModel::default();
        let info = h.move_ball_to_coordinate(&mut fm, FieldCoordinate::new(6, 6));
        assert_eq!(fm.ball_coordinate, Some(FieldCoordinate::new(6, 6)));
        assert!(fm.ball_moving);
        assert!(info[0].contains("Set ball to coordinate"));
    }

    #[test]
    fn put_player_into_box_updates_state_and_message() {
        let h = handler(0);
        let team = make_team(vec![make_player("h1", 1)]);
        let mut game = Game::new(team, make_team(vec![]), Rules::Bb2025);
        game.field_model.set_player_coordinate("h1", FieldCoordinate::new(5, 5));
        let msg = h.put_player_into_box(&mut game, "h1", PlayerState::new(PS_KNOCKED_OUT), "KO");
        assert!(msg.contains("moved into box KO"));
        assert!(game.field_model.player_coordinate("h1").unwrap().is_box_coordinate());
    }
}
