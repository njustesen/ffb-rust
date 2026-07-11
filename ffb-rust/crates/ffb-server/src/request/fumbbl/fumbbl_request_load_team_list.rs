/// 1:1 translation of com.fumbbl.ffb.server.request.fumbbl.FumbblRequestLoadTeamList.
/// Fetches FUMBBL_TEAMS URL for the coach, parses XML into TeamList, sends via communication.
pub struct FumbblRequestLoadTeamList {
    pub coach: String,
    request_url: String,
}

impl FumbblRequestLoadTeamList {
    pub fn new(coach: String) -> Self {
        Self { coach, request_url: String::new() }
    }

    pub fn get_coach(&self) -> &str {
        &self.coach
    }

    pub fn get_request_url(&self) -> &str {
        &self.request_url
    }

    pub fn set_request_url(&mut self, url: String) {
        self.request_url = url;
    }

    /// Builds the FUMBBL_TEAMS URL for [`Self::coach`] and fetches it. Java parses the response
    /// XML into a `TeamList` and sends it to the session; that model/communication layer does
    /// not exist yet in this simplified server crate, so this returns the raw XML for the caller
    /// to hand off once that plumbing lands.
    pub fn process(
        &mut self,
        client: &dyn super::util_fumbbl_request::HttpClient,
        teams_url_template: &str,
    ) -> Result<Option<String>, String> {
        let url = super::util_fumbbl_request::UtilFumbblRequest::bind(teams_url_template, &[self.coach.as_str()]);
        self.set_request_url(url);
        let teams_xml = client.fetch_page(self.get_request_url())?;
        if teams_xml.is_empty() {
            return Ok(None);
        }
        Ok(Some(teams_xml))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::fumbbl::util_fumbbl_request::MockHttpClient;

    #[test]
    fn construct() {
        let r = FumbblRequestLoadTeamList::new("coach".to_string());
        assert_eq!(r.get_coach(), "coach");
    }

    #[test]
    fn process_builds_url_and_returns_xml() {
        let client = MockHttpClient {
            response: Ok("<teamList></teamList>".to_string()),
        };
        let mut r = FumbblRequestLoadTeamList::new("coach".to_string());
        let xml = r.process(&client, "http://fumbbl/teams/$1").unwrap();
        assert_eq!(r.get_request_url(), "http://fumbbl/teams/coach");
        assert_eq!(xml, Some("<teamList></teamList>".to_string()));
    }

    #[test]
    fn process_empty_response_returns_none() {
        let client = MockHttpClient { response: Ok(String::new()) };
        let mut r = FumbblRequestLoadTeamList::new("coach".to_string());
        assert!(r.process(&client, "http://fumbbl/teams/$1").unwrap().is_none());
    }
}
