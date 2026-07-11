/// 1:1 translation of com.fumbbl.ffb.server.request.fumbbl.FumbblRequestCreateGamestate.
/// In non-testing mode: calls FUMBBL_GAMESTATE_CREATE with challenge + gameId + teamIds.
pub struct FumbblRequestCreateGamestate {
    request_url: String,
}

impl FumbblRequestCreateGamestate {
    pub fn new() -> Self {
        Self { request_url: String::new() }
    }

    pub fn get_request_url(&self) -> &str {
        &self.request_url
    }

    pub fn set_request_url(&mut self, url: String) {
        self.request_url = url;
    }

    /// Builds the FUMBBL_GAMESTATE_CREATE URL from `url_template` with the auth challenge
    /// response, game id and home/away team ids bound in, fetches it, and returns the parsed
    /// gamestate response.
    pub fn process(
        &mut self,
        client: &dyn super::util_fumbbl_request::HttpClient,
        url_template: &str,
        challenge_response: &str,
        game_id: &str,
        home_team_id: &str,
        away_team_id: &str,
    ) -> Result<Option<super::fumbbl_game_state::FumbblGameState>, String> {
        let url = super::util_fumbbl_request::UtilFumbblRequest::bind(
            url_template,
            &[challenge_response, game_id, home_team_id, away_team_id],
        );
        self.set_request_url(url);
        super::util_fumbbl_request::UtilFumbblRequest::process_fumbbl_game_state_request(
            client,
            self.get_request_url(),
        )
    }
}

impl Default for FumbblRequestCreateGamestate {
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
        let _ = FumbblRequestCreateGamestate::new();
    }

    #[test]
    fn process_builds_url_with_challenge_and_ids() {
        let client = MockHttpClient {
            response: Ok("<gamestate><result>ok</result></gamestate>".to_string()),
        };
        let mut r = FumbblRequestCreateGamestate::new();
        let state = r
            .process(&client, "http://fumbbl/create/$1/$2/$3/$4", "chal", "5", "10", "20")
            .unwrap()
            .unwrap();
        assert_eq!(r.get_request_url(), "http://fumbbl/create/chal/5/10/20");
        assert!(state.is_ok());
    }

    #[test]
    fn process_failure_reports_error() {
        let client = MockHttpClient {
            response: Ok("<gamestate><result>fail</result></gamestate>".to_string()),
        };
        let mut r = FumbblRequestCreateGamestate::new();
        let state = r
            .process(&client, "http://fumbbl/create/$1/$2/$3/$4", "chal", "5", "10", "20")
            .unwrap()
            .unwrap();
        assert!(!state.is_ok());
    }
}
