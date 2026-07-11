/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.IdentityCommandAdapter.
use std::collections::HashSet;
use ffb_model::model::game::Game;
use ffb_model::model::team::Team;
use crate::net::session_manager::SessionManager;
use crate::model::received_command::SessionId;
use crate::handler::talk::command_adapter::CommandAdapter;

pub struct IdentityCommandAdapter;

impl IdentityCommandAdapter {
    pub fn new() -> Self { Self }
}

impl Default for IdentityCommandAdapter {
    fn default() -> Self { Self::new() }
}

impl CommandAdapter for IdentityCommandAdapter {
    /// Java: decorateCommands — no-op, returns input unchanged.
    fn decorate_commands(&self, input: HashSet<String>) -> HashSet<String> {
        input
    }

    /// Java: determineTeam — resolves team by checking session ownership.
    fn determine_team<'g>(
        &self,
        game: &'g Game,
        session_manager: &SessionManager,
        session: SessionId,
        _commands: &[String],
    ) -> Result<&'g Team, String> {
        let is_home = session_manager.get_session_of_home_coach(game.id as i64) == Some(session);
        Ok(if is_home { &game.team_home } else { &game.team_away })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;

    fn empty_team(name: &str) -> Team {
        Team {
            id: name.into(), name: name.into(), race: "Human".into(), roster_id: "human".into(),
            coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false,
        }
    }

    fn game() -> Game {
        let mut g = Game::new(empty_team("Home"), empty_team("Away"), Rules::Bb2020);
        g.id = 100;
        g
    }

    #[test]
    fn construct() { let _ = IdentityCommandAdapter::new(); }

    #[test]
    fn decorate_commands_is_noop() {
        let adapter = IdentityCommandAdapter::new();
        let mut input = HashSet::new();
        input.insert("move".to_string());
        let result = adapter.decorate_commands(input.clone());
        assert_eq!(result, input);
    }

    #[test]
    fn determine_team_returns_home_for_home_session() {
        use ffb_model::model::ClientMode;
        let g = game();
        let mut sm = SessionManager::new();
        let (tx, _) = tokio::sync::mpsc::unbounded_channel();
        sm.add_session(1, 100, "Home".into(), ClientMode::PLAYER, true, vec![], tx);
        let team = adapter_determine(&g, &sm, 1);
        assert_eq!(team.name, "Home");
    }

    #[test]
    fn determine_team_returns_away_for_non_home_session() {
        use ffb_model::model::ClientMode;
        let g = game();
        let mut sm = SessionManager::new();
        let (tx, _) = tokio::sync::mpsc::unbounded_channel();
        sm.add_session(2, 100, "Away".into(), ClientMode::PLAYER, false, vec![], tx);
        let team = adapter_determine(&g, &sm, 2);
        assert_eq!(team.name, "Away");
    }

    fn adapter_determine<'g>(g: &'g Game, sm: &SessionManager, session: SessionId) -> &'g Team {
        IdentityCommandAdapter::new().determine_team(g, sm, session, &[]).unwrap()
    }
}
