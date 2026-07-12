/// 1:1 translation of com.fumbbl.ffb.server.net.commands.InternalServerCommandAddLoadedTeam.
use ffb_model::model::team::Team;
use super::internal_server_command::InternalServerCommand;

pub struct InternalServerCommandAddLoadedTeam {
    /// Java: `gameState` — modeled here as its id (`gameState.getId()`, passed to `super`).
    pub game_id: i64,
    pub coach: String,
    pub home_team: Option<bool>,
    /// Java: `team` — the real `Team` parsed from the FUMBBL response in
    /// `FumbblRequestLoadTeam` before this command is built.
    pub team: Team,
    /// Account properties (list of strings).
    pub account_properties: Vec<String>,
}

impl InternalServerCommandAddLoadedTeam {
    /// Java constructor order: `(gameState, coach, homeTeam, team, accountProperties)`.
    pub fn new(
        game_id: i64,
        coach: String,
        home_team: Option<bool>,
        team: Team,
        account_properties: Vec<String>,
    ) -> Self {
        Self { game_id, coach, home_team, team, account_properties }
    }

    pub fn get_coach(&self) -> &str {
        &self.coach
    }

    pub fn get_home_team(&self) -> Option<bool> {
        self.home_team
    }

    /// Java: `getTeam()`.
    pub fn get_team(&self) -> &Team {
        &self.team
    }

    pub fn get_account_properties(&self) -> &[String] {
        &self.account_properties
    }
}

impl InternalServerCommand for InternalServerCommandAddLoadedTeam {
    fn get_id(&self) -> &'static str {
        "internalServerAddLoadedTeam"
    }

    fn get_game_id(&self) -> i64 {
        self.game_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn team(id: &str) -> Team {
        Team {
            id: id.into(), name: id.into(), race: "Human".into(), roster_id: "human".into(),
            coach: "coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false,
        }
    }

    #[test]
    fn construct() {
        let _ = InternalServerCommandAddLoadedTeam::new(42, "coach".to_string(), Some(true), team("t1"), vec![]);
    }

    #[test]
    fn get_id() {
        let c = InternalServerCommandAddLoadedTeam::new(1, "c".to_string(), None, team("t1"), vec![]);
        assert_eq!(c.get_id(), "internalServerAddLoadedTeam");
    }

    #[test]
    fn get_game_id() {
        let c = InternalServerCommandAddLoadedTeam::new(99, "c".to_string(), None, team("t1"), vec![]);
        assert_eq!(c.get_game_id(), 99);
    }

    #[test]
    fn is_internal() {
        let c = InternalServerCommandAddLoadedTeam::new(1, "c".to_string(), None, team("t1"), vec![]);
        assert!(c.is_internal());
    }

    #[test]
    fn get_coach() {
        let c = InternalServerCommandAddLoadedTeam::new(1, "myCoach".to_string(), None, team("t1"), vec![]);
        assert_eq!(c.get_coach(), "myCoach");
    }

    #[test]
    fn get_team_returns_carried_team() {
        let c = InternalServerCommandAddLoadedTeam::new(1, "c".to_string(), None, team("reavers"), vec![]);
        assert_eq!(c.get_team().id, "reavers");
    }
}
