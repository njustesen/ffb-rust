/// 1:1 translation of com.fumbbl.ffb.server.request.fumbbl.FumbblRequestLoadTeam.
use ffb_model::model::team::Team;
use ffb_model::xml::XmlHandler;

pub struct FumbblRequestLoadTeam {
    pub coach: String,
    pub team_id: String,
    pub home_team: Option<bool>,
    pub account_properties: Vec<String>,
    request_url: String,
}

impl FumbblRequestLoadTeam {
    pub fn new(
        coach: String,
        team_id: String,
        home_team: Option<bool>,
        account_properties: Vec<String>,
    ) -> Self {
        Self {
            coach,
            team_id,
            home_team,
            account_properties,
            request_url: String::new(),
        }
    }

    pub fn get_coach(&self) -> &str {
        &self.coach
    }

    pub fn get_team_id(&self) -> &str {
        &self.team_id
    }

    pub fn get_account_properties(&self) -> &[String] {
        &self.account_properties
    }

    pub fn get_request_url(&self) -> &str {
        &self.request_url
    }

    pub fn set_request_url(&mut self, url: String) {
        self.request_url = url;
    }

    /// Java: `FumbblRequestLoadTeam.process(ServerRequestProcessor)`.
    ///
    /// Fetches FUMBBL_TEAM for [`Self::team_id`] via `team_url_template` and inflates the XML
    /// into a real `Team` (Java: `team = UtilFumbblRequest.loadFumbblTeam(server, getTeamId())`,
    /// where `loadFumbblTeam` XML-parses into a `Team`). Java then treats a missing team, or one
    /// whose XML has no name, as an invalid-team error (`handleInvalidTeam`); that is modeled
    /// here as `Ok(None)` (the caller reports the invalid team), while a valid parse yields
    /// `Ok(Some(team))`.
    ///
    /// On success Java dispatches `InternalServerCommandAddLoadedTeam(gameState, coach, homeTeam,
    /// team, accountProperties)` through `server.getCommunication().handleCommand(...)`. That
    /// command now carries the real `Team` and its `ServerCommandHandlerFactory` arm is wired,
    /// but this request type has no `ServerRequestProcessor`→dispatch caller in this crate yet
    /// (it is exercised only directly), so the built command is not enqueued from here.
    pub fn process(
        &mut self,
        client: &dyn super::util_fumbbl_request::HttpClient,
        team_url_template: &str,
    ) -> Result<Option<Team>, String> {
        let url = super::util_fumbbl_request::UtilFumbblRequest::bind(team_url_template, &[self.team_id.as_str()]);
        self.set_request_url(url);
        let team_xml =
            super::util_fumbbl_request::UtilFumbblRequest::load_fumbbl_team(client, team_url_template, &self.team_id)?;
        Ok(team_xml.and_then(|xml| parse_team_xml(&xml)))
    }
}

/// Java: `UtilFumbblRequest.loadFumbblTeam`'s XML→`Team` inflation —
/// `XmlHandler.parse(game, xmlSource, new Team(source))`. Mirrors `team_cache::map_to_team`'s
/// parse-then-downcast pattern (the only real XML→`Team` deserializer in this crate). A team
/// whose XML yields no name is treated as invalid (Java's `!StringTool.isProvided(team.getName())`
/// guard) and reported as `None`.
fn parse_team_xml(xml: &str) -> Option<Team> {
    let empty = Team {
        id: String::new(), name: String::new(), race: String::new(),
        roster_id: String::new(), coach: String::new(),
        rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
        prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
        cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0,
        team_value: 0, treasury: 0, special_rules: vec![], players: vec![],
        vampire_lord: false, necromancer: false,
    };
    let parsed = XmlHandler::parse(None, xml, Box::new(empty));
    let team = parsed.into_any().downcast::<Team>().ok().map(|t| *t)?;
    if team.name.is_empty() {
        return None;
    }
    Some(team)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::fumbbl::util_fumbbl_request::MockHttpClient;

    #[test]
    fn construct() {
        let r = FumbblRequestLoadTeam::new("coach".to_string(), "team1".to_string(), Some(true), vec![]);
        assert_eq!(r.get_coach(), "coach");
        assert_eq!(r.get_team_id(), "team1");
    }

    #[test]
    fn process_returns_parsed_team_when_present() {
        let client = MockHttpClient {
            response: Ok(r#"<team id="team1"><name>The Reavers</name><coach>coach</coach></team>"#.to_string()),
        };
        let mut r = FumbblRequestLoadTeam::new("coach".to_string(), "team1".to_string(), Some(true), vec![]);
        let team = r.process(&client, "http://fumbbl/team/$1").unwrap().expect("team should parse");
        assert_eq!(r.get_request_url(), "http://fumbbl/team/team1");
        assert_eq!(team.name, "The Reavers");
        assert_eq!(team.id, "team1");
    }

    #[test]
    fn process_returns_none_for_empty_response() {
        let client = MockHttpClient { response: Ok(String::new()) };
        let mut r = FumbblRequestLoadTeam::new("coach".to_string(), "team1".to_string(), Some(true), vec![]);
        let team = r.process(&client, "http://fumbbl/team/$1").unwrap();
        assert!(team.is_none());
    }

    #[test]
    fn process_returns_none_for_team_without_name() {
        // Java's `!StringTool.isProvided(team.getName())` invalid-team guard.
        let client = MockHttpClient {
            response: Ok(r#"<team id="team1"><coach>coach</coach></team>"#.to_string()),
        };
        let mut r = FumbblRequestLoadTeam::new("coach".to_string(), "team1".to_string(), Some(true), vec![]);
        let team = r.process(&client, "http://fumbbl/team/$1").unwrap();
        assert!(team.is_none());
    }
}
