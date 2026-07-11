/// 1:1 translation of com.fumbbl.ffb.server.admin.GameStateServlet.
///
/// Java looks games up via `GameCache`/`UtilBackup.loadGameState`, renders `SkillFactory`
/// behaviours, and calls `GameStateService.resetStepStack`. None of that cache/factory wiring
/// exists in this simplified server crate yet, so the lookup result is passed in by the caller
/// (`game_found`) rather than performed here — this focuses on the command routing, JSON
/// envelope shape, and challenge/response gate, matching Java method-for-method.
///
/// Java's challenge uses real MD5 (`PasswordChallenge`); see [`super::backup_servlet`] for why
/// this uses a structurally-equivalent placeholder hash instead.
pub struct GameStateServlet {
    last_challenge: Option<String>,
}

impl GameStateServlet {
    pub const BEHAVIOURS: &'static str = "behaviours";
    pub const CHALLENGE: &'static str = "challenge";
    pub const GET: &'static str = "get";
    pub const SET: &'static str = "set";
    pub const RESULT: &'static str = "result";
    pub const RESET: &'static str = "reset";

    pub fn new() -> Self {
        Self { last_challenge: None }
    }

    fn hash_challenge(salt: &str, now_millis: i64) -> String {
        format!("{:x}", salt.len() as i64 ^ now_millis)
    }

    /// Builds a fresh challenge and returns the
    /// `<admin><challenge>...</challenge><status>ok</status></admin>` XML response.
    pub fn handle_challenge(&mut self, admin_salt: &str, now_millis: i64) -> String {
        let challenge = Self::hash_challenge(admin_salt, now_millis);
        self.last_challenge = Some(challenge.clone());
        format!("<admin><challenge>{}</challenge><status>ok</status></admin>", challenge)
    }

    /// Validates `response` against the last issued challenge combined with `admin_password`.
    /// Consumes the challenge (one-shot).
    pub fn check_response(&mut self, response: &str, admin_password: &str) -> bool {
        let is_ok = match &self.last_challenge {
            Some(challenge) => format!("{}:{}", challenge, admin_password) == response,
            None => false,
        };
        self.last_challenge = None;
        is_ok
    }

    pub fn parse_game_id(&self, s: &str) -> i64 {
        if s.is_empty() {
            return 0;
        }
        s.parse::<i64>().unwrap_or(0)
    }

    /// Routes GET commands (other than [`Self::CHALLENGE`], which callers should dispatch to
    /// [`Self::handle_challenge`] before authorization). `game_found`/`game_json` stand in for
    /// the `GameCache`/`UtilBackup` lookup Java performs internally. Returns `(status_code,
    /// body)`.
    pub fn handle_get(
        &mut self,
        command: &str,
        game_id_string: &str,
        game_found: bool,
        game_json: Option<&str>,
    ) -> (u16, String) {
        let game_id = self.parse_game_id(game_id_string);
        match command {
            Self::GET | Self::RESULT | Self::BEHAVIOURS | Self::RESET => {
                if !game_found {
                    (404, format!("{{\"message\":\"Game '{}' not found\"}}", game_id))
                } else if command == Self::RESET {
                    (200, format!("{{\"message\":\"Game '{}' reset\"}}", game_id))
                } else {
                    (200, game_json.unwrap_or("{}").to_string())
                }
            }
            _ => (404, format!("{{\"message\":\"method '{}' not found\"}}", command)),
        }
    }

    /// Java's `handleSet` parses the posted JSON into a fresh `GameState`, closes existing
    /// sessions for that game, and re-adds it to the cache; none of that exists here, so this
    /// only validates that a body was provided and echoes the `{"status":"ok"}` envelope.
    pub fn handle_post(&mut self, command: &str, body: &str) -> (u16, String) {
        if command == Self::SET {
            if body.is_empty() {
                (400, "{\"message\":\"empty body\"}".to_string())
            } else {
                (200, "{\"status\":\"ok\"}".to_string())
            }
        } else {
            (404, format!("{{\"message\":\"method '{}' not found\"}}", command))
        }
    }
}

impl Default for GameStateServlet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let _ = GameStateServlet::new();
    }

    #[test]
    fn constants() {
        assert_eq!(GameStateServlet::CHALLENGE, "challenge");
        assert_eq!(GameStateServlet::GET, "get");
    }

    #[test]
    fn challenge_then_correct_response_is_accepted() {
        let mut servlet = GameStateServlet::new();
        let xml = servlet.handle_challenge("salt", 1000);
        let challenge = xml
            .split("<challenge>")
            .nth(1)
            .unwrap()
            .split("</challenge>")
            .next()
            .unwrap();
        let response = format!("{}:{}", challenge, "adminpw");
        assert!(servlet.check_response(&response, "adminpw"));
    }

    #[test]
    fn handle_get_missing_game_returns_404() {
        let mut servlet = GameStateServlet::new();
        let (status, body) = servlet.handle_get(GameStateServlet::GET, "42", false, None);
        assert_eq!(status, 404);
        assert!(body.contains("not found"));
    }

    #[test]
    fn handle_get_found_game_returns_json() {
        let mut servlet = GameStateServlet::new();
        let (status, body) = servlet.handle_get(GameStateServlet::GET, "42", true, Some("{\"half\":1}"));
        assert_eq!(status, 200);
        assert_eq!(body, "{\"half\":1}");
    }

    #[test]
    fn handle_get_reset_reports_reset_message() {
        let mut servlet = GameStateServlet::new();
        let (status, body) = servlet.handle_get(GameStateServlet::RESET, "42", true, None);
        assert_eq!(status, 200);
        assert!(body.contains("reset"));
    }

    #[test]
    fn handle_get_unknown_command_returns_404() {
        let mut servlet = GameStateServlet::new();
        let (status, body) = servlet.handle_get("unknown", "42", true, None);
        assert_eq!(status, 404);
        assert!(body.contains("not found"));
    }

    #[test]
    fn handle_post_set_with_body_returns_ok() {
        let mut servlet = GameStateServlet::new();
        let (status, body) = servlet.handle_post(GameStateServlet::SET, "{\"id\":1}");
        assert_eq!(status, 200);
        assert!(body.contains("ok"));
    }

    #[test]
    fn handle_post_set_with_empty_body_fails() {
        let mut servlet = GameStateServlet::new();
        let (status, _) = servlet.handle_post(GameStateServlet::SET, "");
        assert_eq!(status, 400);
    }
}
