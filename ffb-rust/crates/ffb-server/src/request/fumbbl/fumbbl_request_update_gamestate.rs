/// 1:1 translation of com.fumbbl.ffb.server.request.fumbbl.FumbblRequestUpdateGamestate.
/// In non-testing mode: calls FUMBBL_GAMESTATE_UPDATE with half/turn/scores/spectators.
pub struct FumbblRequestUpdateGamestate {
    request_url: String,
}

impl FumbblRequestUpdateGamestate {
    pub fn new() -> Self {
        Self { request_url: String::new() }
    }

    pub fn get_request_url(&self) -> &str {
        &self.request_url
    }

    pub fn set_request_url(&mut self, url: String) {
        self.request_url = url;
    }

    /// Builds the FUMBBL_GAMESTATE_UPDATE URL with challenge/gameId/half/turn/scores/spectators
    /// bound in, fetches it, and returns the parsed gamestate response.
    #[allow(clippy::too_many_arguments)]
    pub fn process(
        &mut self,
        client: &dyn super::util_fumbbl_request::HttpClient,
        url_template: &str,
        challenge_response: &str,
        game_id: &str,
        half: &str,
        turn_nr: &str,
        home_score: &str,
        away_score: &str,
        spectators: &str,
    ) -> Result<Option<super::fumbbl_game_state::FumbblGameState>, String> {
        let url = super::util_fumbbl_request::UtilFumbblRequest::bind(
            url_template,
            &[challenge_response, game_id, half, turn_nr, home_score, away_score, spectators],
        );
        self.set_request_url(url);
        super::util_fumbbl_request::UtilFumbblRequest::process_fumbbl_game_state_request(
            client,
            self.get_request_url(),
        )
    }
}

impl Default for FumbblRequestUpdateGamestate {
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
        let _ = FumbblRequestUpdateGamestate::new();
    }

    #[test]
    fn process_builds_url_with_all_fields() {
        let client = MockHttpClient {
            response: Ok("<gamestate><result>ok</result></gamestate>".to_string()),
        };
        let mut r = FumbblRequestUpdateGamestate::new();
        let state = r
            .process(
                &client,
                "http://fumbbl/update/$1/$2/$3/$4/$5/$6/$7",
                "chal",
                "5",
                "1",
                "3",
                "2",
                "1",
                "4",
            )
            .unwrap()
            .unwrap();
        assert_eq!(r.get_request_url(), "http://fumbbl/update/chal/5/1/3/2/1/4");
        assert!(state.is_ok());
    }
}
