/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.DecoratingCommandAdapter.
use std::collections::HashSet;
use ffb_model::model::game::Game;
use ffb_model::model::team::Team;
use crate::net::session_manager::SessionManager;
use crate::model::received_command::SessionId;
use crate::handler::talk::command_adapter::CommandAdapter;

const HOME: &str = "home";
const AWAY: &str = "away";
const TEAM_DELIM: &str = "_";

pub struct DecoratingCommandAdapter;

impl DecoratingCommandAdapter {
    pub fn new() -> Self { Self }
}

impl Default for DecoratingCommandAdapter {
    fn default() -> Self { Self::new() }
}

impl CommandAdapter for DecoratingCommandAdapter {
    /// Java: decorateCommands — expands each command into _home and _away variants.
    fn decorate_commands(&self, input: HashSet<String>) -> HashSet<String> {
        input
            .into_iter()
            .flat_map(|command| {
                vec![
                    format!("{command}{TEAM_DELIM}{HOME}"),
                    format!("{command}{TEAM_DELIM}{AWAY}"),
                ]
            })
            .collect()
    }

    /// Java: determineTeam — parses _home/_away suffix to resolve team.
    fn determine_team<'g>(
        &self,
        game: &'g Game,
        _session_manager: &SessionManager,
        _session: SessionId,
        commands: &[String],
    ) -> Result<&'g Team, String> {
        let command = commands.first().map(String::as_str).unwrap_or("");
        if command.is_empty() {
            return Err("No command given".to_string());
        }

        let command_parts: Vec<&str> = command.split(TEAM_DELIM).collect();
        if command_parts.len() < 2 {
            return Err(format!("Unsupported format for command: {command}"));
        }

        match command_parts[command_parts.len() - 1].to_lowercase().as_str() {
            HOME => Ok(&game.team_home),
            AWAY => Ok(&game.team_away),
            other => Err(format!("Invalid team: {other}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::game::Game;
    use ffb_model::model::team::Team;
    use crate::net::session_manager::SessionManager;

    fn empty_team(name: &str) -> Team {
        Team {
            id: name.into(),
            name: name.into(),
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

    fn game_with_team_names() -> Game {
        Game::new(empty_team("Home Team"), empty_team("Away Team"), Rules::Bb2020)
    }

    #[test]
    fn construct() { let _ = DecoratingCommandAdapter::new(); }

    #[test]
    fn decorate_commands_expands_into_home_and_away() {
        let adapter = DecoratingCommandAdapter::new();
        let mut input = HashSet::new();
        input.insert("move".to_string());
        let result = adapter.decorate_commands(input);
        assert_eq!(result.len(), 2);
        assert!(result.contains("move_home"));
        assert!(result.contains("move_away"));
    }

    #[test]
    fn determine_team_resolves_home_suffix() {
        let adapter = DecoratingCommandAdapter::new();
        let game = game_with_team_names();
        let sm = SessionManager::new();
        let commands = vec!["move_home".to_string()];
        let team = adapter.determine_team(&game, &sm, 1, &commands).unwrap();
        assert_eq!(team.name, "Home Team");
    }

    #[test]
    fn determine_team_resolves_away_suffix() {
        let adapter = DecoratingCommandAdapter::new();
        let game = game_with_team_names();
        let sm = SessionManager::new();
        let commands = vec!["move_away".to_string()];
        let team = adapter.determine_team(&game, &sm, 1, &commands).unwrap();
        assert_eq!(team.name, "Away Team");
    }

    #[test]
    fn determine_team_rejects_invalid_suffix() {
        let adapter = DecoratingCommandAdapter::new();
        let game = game_with_team_names();
        let sm = SessionManager::new();
        let commands = vec!["move_north".to_string()];
        assert!(adapter.determine_team(&game, &sm, 1, &commands).is_err());
    }

    #[test]
    fn determine_team_rejects_missing_delimiter() {
        let adapter = DecoratingCommandAdapter::new();
        let game = game_with_team_names();
        let sm = SessionManager::new();
        let commands = vec!["move".to_string()];
        assert!(adapter.determine_team(&game, &sm, 1, &commands).is_err());
    }
}
