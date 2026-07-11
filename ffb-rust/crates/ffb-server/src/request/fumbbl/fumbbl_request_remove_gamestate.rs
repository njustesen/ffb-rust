/// 1:1 translation of com.fumbbl.ffb.server.request.fumbbl.FumbblRequestRemoveGamestate.
/// Calls FUMBBL_GAMESTATE_REMOVE; always calls server.closeResources(gameId).
pub struct FumbblRequestRemoveGamestate {
    request_url: String,
}

impl FumbblRequestRemoveGamestate {
    pub fn new() -> Self {
        Self { request_url: String::new() }
    }

    pub fn get_request_url(&self) -> &str {
        &self.request_url
    }

    pub fn set_request_url(&mut self, url: String) {
        self.request_url = url;
    }

    /// Builds the FUMBBL_GAMESTATE_REMOVE URL with the challenge response and game id bound in,
    /// fetches it, and returns the parsed gamestate response. Java always calls
    /// `server.closeResources(gameId)` after this regardless of outcome; that resource cleanup
    /// has no equivalent yet in this simplified server crate.
    pub fn process(
        &mut self,
        client: &dyn super::util_fumbbl_request::HttpClient,
        url_template: &str,
        challenge_response: &str,
        game_id: &str,
    ) -> Result<Option<super::fumbbl_game_state::FumbblGameState>, String> {
        let url = super::util_fumbbl_request::UtilFumbblRequest::bind(url_template, &[challenge_response, game_id]);
        self.set_request_url(url);
        super::util_fumbbl_request::UtilFumbblRequest::process_fumbbl_game_state_request(
            client,
            self.get_request_url(),
        )
    }
}

impl Default for FumbblRequestRemoveGamestate {
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
        let _ = FumbblRequestRemoveGamestate::new();
    }

    #[test]
    fn process_builds_url_with_challenge_and_game_id() {
        let client = MockHttpClient {
            response: Ok("<gamestate><result>ok</result></gamestate>".to_string()),
        };
        let mut r = FumbblRequestRemoveGamestate::new();
        let state = r.process(&client, "http://fumbbl/remove/$1/$2", "chal", "5").unwrap().unwrap();
        assert_eq!(r.get_request_url(), "http://fumbbl/remove/chal/5");
        assert!(state.is_ok());
    }
}
