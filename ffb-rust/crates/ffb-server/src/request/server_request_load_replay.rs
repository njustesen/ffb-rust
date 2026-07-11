/// 1:1 translation of com.fumbbl.ffb.server.request.ServerRequestLoadReplay.
pub struct ServerRequestLoadReplay {
    pub game_id: i64,
    pub replay_to_command_nr: i32,
    pub mode: i32,
    pub team_id: String,
    pub coach: String,
    request_url: String,
}

impl ServerRequestLoadReplay {
    pub const LOAD_GAME: i32 = 1;
    pub const DELETE_GAME: i32 = 2;
    pub const UPLOAD_GAME: i32 = 3;

    pub fn new(
        game_id: i64,
        replay_to_command_nr: i32,
        mode: i32,
        team_id: String,
        coach: String,
    ) -> Self {
        Self {
            game_id,
            replay_to_command_nr,
            mode,
            team_id,
            coach,
            request_url: String::new(),
        }
    }

    pub fn get_game_id(&self) -> i64 {
        self.game_id
    }

    pub fn get_replay_to_command_nr(&self) -> i32 {
        self.replay_to_command_nr
    }

    pub fn get_mode(&self) -> i32 {
        self.mode
    }

    pub fn get_request_url(&self) -> &str {
        &self.request_url
    }

    pub fn set_request_url(&mut self, url: String) {
        self.request_url = url;
    }

    /// Builds the BACKUP_URL_LOAD URL for [`Self::game_id`], fetches it, and returns the raw
    /// JSON payload if present. Java parses this into a `GameState` and, depending on
    /// [`Self::mode`], dispatches `InternalServerCommandReplayLoaded`,
    /// `InternalServerCommandDeleteGame`, or `InternalServerCommandUploadGame`; that
    /// session/command dispatch has no equivalent yet in this simplified server crate, so the
    /// caller is expected to act on the returned JSON per [`Self::mode`].
    pub fn process(
        &mut self,
        client: &dyn super::fumbbl::util_fumbbl_request::HttpClient,
        load_url_template: &str,
    ) -> Result<Option<String>, String> {
        let url = super::fumbbl::util_fumbbl_request::UtilFumbblRequest::bind(
            load_url_template,
            &[self.game_id.to_string().as_str()],
        );
        self.set_request_url(url);
        let json_string = client.fetch_page(self.get_request_url())?;
        if json_string.is_empty() {
            return Ok(None);
        }
        Ok(Some(json_string))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::fumbbl::util_fumbbl_request::MockHttpClient;

    #[test]
    fn construct() {
        let r = ServerRequestLoadReplay::new(1, 0, ServerRequestLoadReplay::LOAD_GAME, String::new(), String::new());
        assert_eq!(r.get_game_id(), 1);
        assert_eq!(r.get_mode(), ServerRequestLoadReplay::LOAD_GAME);
    }

    #[test]
    fn process_builds_url_and_returns_json() {
        let client = MockHttpClient {
            response: Ok("{\"half\":1}".to_string()),
        };
        let mut r = ServerRequestLoadReplay::new(42, 0, ServerRequestLoadReplay::LOAD_GAME, String::new(), String::new());
        let json = r.process(&client, "http://backup/load/$1").unwrap();
        assert_eq!(r.get_request_url(), "http://backup/load/42");
        assert_eq!(json, Some("{\"half\":1}".to_string()));
    }

    #[test]
    fn process_empty_response_returns_none() {
        let client = MockHttpClient { response: Ok(String::new()) };
        let mut r = ServerRequestLoadReplay::new(42, 0, ServerRequestLoadReplay::DELETE_GAME, String::new(), String::new());
        let json = r.process(&client, "http://backup/load/$1").unwrap();
        assert!(json.is_none());
    }
}
