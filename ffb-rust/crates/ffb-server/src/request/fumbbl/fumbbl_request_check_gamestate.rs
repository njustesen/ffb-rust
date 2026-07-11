/// 1:1 translation of com.fumbbl.ffb.server.request.fumbbl.FumbblRequestCheckGamestate.
pub struct FumbblRequestCheckGamestate {
    request_url: String,
}

impl FumbblRequestCheckGamestate {
    pub fn new() -> Self {
        Self { request_url: String::new() }
    }

    pub fn get_request_url(&self) -> &str {
        &self.request_url
    }

    pub fn set_request_url(&mut self, url: String) {
        self.request_url = url;
    }

    /// Builds the FUMBBL_GAMESTATE_CHECK (or FUMBBL_GAMESTATE_OPTIONS when `testing`) URL from
    /// `url_template` with the home/away team ids bound in, fetches it, and returns the parsed
    /// gamestate response.
    pub fn process(
        &mut self,
        client: &dyn super::util_fumbbl_request::HttpClient,
        url_template: &str,
        home_team_id: &str,
        away_team_id: &str,
    ) -> Result<Option<super::fumbbl_game_state::FumbblGameState>, String> {
        let url = super::util_fumbbl_request::UtilFumbblRequest::bind(url_template, &[home_team_id, away_team_id]);
        self.set_request_url(url);
        super::util_fumbbl_request::UtilFumbblRequest::process_fumbbl_game_state_request(
            client,
            self.get_request_url(),
        )
    }
}

impl Default for FumbblRequestCheckGamestate {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::fumbbl::util_fumbbl_request::MockHttpClient;

    #[test]
    fn construct() {
        let _ = FumbblRequestCheckGamestate::new();
    }

    #[test]
    fn process_builds_url_with_team_ids() {
        let client = MockHttpClient {
            response: Ok("<gamestate><result>ok</result></gamestate>".to_string()),
        };
        let mut r = FumbblRequestCheckGamestate::new();
        let state = r
            .process(&client, "http://fumbbl/check/$1/$2", "10", "20")
            .unwrap()
            .unwrap();
        assert_eq!(r.get_request_url(), "http://fumbbl/check/10/20");
        assert!(state.is_ok());
    }

    #[test]
    fn process_not_ok_result_is_reported() {
        let client = MockHttpClient {
            response: Ok("<gamestate><result>fail</result><description>bad</description></gamestate>".to_string()),
        };
        let mut r = FumbblRequestCheckGamestate::new();
        let state = r
            .process(&client, "http://fumbbl/check/$1/$2", "10", "20")
            .unwrap()
            .unwrap();
        assert!(!state.is_ok());
    }
}
