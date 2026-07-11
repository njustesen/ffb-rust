/// 1:1 translation of com.fumbbl.ffb.server.request.fumbbl.FumbblRequestLoadTeam.
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

    /// Fetches FUMBBL_TEAM for [`Self::team_id`] via `team_url_template`. Java treats a missing
    /// team, or one whose XML has no name, as an invalid-team error; since the XML is not parsed
    /// into a real `Team` model here, this simply reports whether *some* non-empty team XML was
    /// returned.
    pub fn process(
        &mut self,
        client: &dyn super::util_fumbbl_request::HttpClient,
        team_url_template: &str,
    ) -> Result<Option<String>, String> {
        let url = super::util_fumbbl_request::UtilFumbblRequest::bind(team_url_template, &[self.team_id.as_str()]);
        self.set_request_url(url);
        super::util_fumbbl_request::UtilFumbblRequest::load_fumbbl_team(client, team_url_template, &self.team_id)
    }
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
    fn process_returns_team_xml_when_present() {
        let client = MockHttpClient {
            response: Ok("<team><name>The Reavers</name></team>".to_string()),
        };
        let mut r = FumbblRequestLoadTeam::new("coach".to_string(), "team1".to_string(), Some(true), vec![]);
        let team_xml = r.process(&client, "http://fumbbl/team/$1").unwrap();
        assert_eq!(r.get_request_url(), "http://fumbbl/team/team1");
        assert!(team_xml.is_some());
    }

    #[test]
    fn process_returns_none_for_empty_response() {
        let client = MockHttpClient { response: Ok(String::new()) };
        let mut r = FumbblRequestLoadTeam::new("coach".to_string(), "team1".to_string(), Some(true), vec![]);
        let team_xml = r.process(&client, "http://fumbbl/team/$1").unwrap();
        assert!(team_xml.is_none());
    }
}
