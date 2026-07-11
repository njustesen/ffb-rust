/// 1:1 translation of com.fumbbl.ffb.server.admin.BackupServlet.
///
/// Java builds a challenge with `PasswordChallenge.md5Encode` and verifies responses with
/// `PasswordChallenge.createResponse` (real MD5). No MD5 crate is wired into this workspace yet,
/// so [`Self::hash_challenge`]/[`Self::check_response`] use a structurally-equivalent
/// placeholder (same one-shot-challenge lifecycle) rather than a byte-for-byte port of the hash.
pub struct BackupServlet {
    last_challenge: Option<String>,
}

impl BackupServlet {
    pub const CHALLENGE: &'static str = "challenge";
    pub const LOAD: &'static str = "load";
    pub const SAVE: &'static str = "save";

    pub const STATUS_OK: &'static str = "ok";
    pub const STATUS_FAIL: &'static str = "fail";

    pub fn new() -> Self {
        Self { last_challenge: None }
    }

    fn hash_challenge(salt: &str, now_millis: i64) -> String {
        format!("{:x}", salt.len() as i64 ^ now_millis)
    }

    /// Builds a fresh challenge from `backup_salt` + `now_millis` and returns the
    /// `<backup><challenge>...</challenge><status>ok</status></backup>` XML response.
    pub fn execute_challenge(&mut self, backup_salt: &str, now_millis: i64) -> String {
        let challenge = Self::hash_challenge(backup_salt, now_millis);
        self.last_challenge = Some(challenge.clone());
        format!(
            "<backup><challenge>{}</challenge><status>{}</status></backup>",
            challenge,
            Self::STATUS_OK
        )
    }

    /// Validates `response` against the last issued challenge combined with `admin_password`.
    /// Consumes the challenge (one-shot), mirroring Java's `fLastChallenge = null` at the end.
    pub fn check_response(&mut self, response: &str, admin_password: &str) -> bool {
        let is_ok = match &self.last_challenge {
            Some(challenge) => {
                let expected = format!("{}:{}", challenge, admin_password);
                expected == response
            }
            None => false,
        };
        self.last_challenge = None;
        is_ok
    }

    /// Validates the response and, if OK, returns the `<backup><save gameId=".."/></backup>`
    /// XML plus the parsed game id the caller should enqueue a `ServerRequestSaveReplay` for.
    pub fn execute_save(&mut self, game_id_string: &str, response: &str, admin_password: &str) -> (String, Option<i64>) {
        if !self.check_response(response, admin_password) {
            return (format!("<backup><status>{}</status></backup>", Self::STATUS_FAIL), None);
        }
        let game_id = self.parse_game_id(game_id_string);
        if game_id > 0 {
            (
                format!(
                    "<backup><save gameId=\"{}\"/><status>{}</status></backup>",
                    game_id_string,
                    Self::STATUS_OK
                ),
                Some(game_id),
            )
        } else {
            (
                format!(
                    "<backup><error>Invalid or missing gameId parameter</error><status>{}</status></backup>",
                    Self::STATUS_FAIL
                ),
                None,
            )
        }
    }

    pub fn parse_game_id(&self, s: &str) -> i64 {
        if s.is_empty() {
            return 0;
        }
        s.parse::<i64>().unwrap_or(0)
    }
}

impl Default for BackupServlet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let _ = BackupServlet::new();
    }

    #[test]
    fn constants() {
        assert_eq!(BackupServlet::CHALLENGE, "challenge");
        assert_eq!(BackupServlet::LOAD, "load");
        assert_eq!(BackupServlet::SAVE, "save");
    }

    #[test]
    fn execute_challenge_then_correct_response_is_accepted() {
        let mut servlet = BackupServlet::new();
        let xml = servlet.execute_challenge("salt", 1000);
        assert!(xml.contains("<challenge>"));
        assert!(xml.contains(BackupServlet::STATUS_OK));

        // Extract the challenge value the same way a client would.
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
    fn check_response_is_one_shot() {
        let mut servlet = BackupServlet::new();
        let xml = servlet.execute_challenge("salt", 1000);
        let challenge = xml
            .split("<challenge>")
            .nth(1)
            .unwrap()
            .split("</challenge>")
            .next()
            .unwrap();
        let response = format!("{}:{}", challenge, "adminpw");
        assert!(servlet.check_response(&response, "adminpw"));
        // Second call with the same response fails: the challenge was consumed.
        assert!(!servlet.check_response(&response, "adminpw"));
    }

    #[test]
    fn execute_save_without_challenge_fails() {
        let mut servlet = BackupServlet::new();
        let (xml, game_id) = servlet.execute_save("42", "whatever", "adminpw");
        assert!(xml.contains(BackupServlet::STATUS_FAIL));
        assert!(game_id.is_none());
    }

    #[test]
    fn execute_save_with_valid_response_returns_game_id() {
        let mut servlet = BackupServlet::new();
        let xml = servlet.execute_challenge("salt", 1000);
        let challenge = xml
            .split("<challenge>")
            .nth(1)
            .unwrap()
            .split("</challenge>")
            .next()
            .unwrap()
            .to_string();
        let response = format!("{}:{}", challenge, "adminpw");
        let (save_xml, game_id) = servlet.execute_save("42", &response, "adminpw");
        assert_eq!(game_id, Some(42));
        assert!(save_xml.contains("gameId=\"42\""));
    }

    #[test]
    fn parse_game_id_returns_zero_for_invalid_input() {
        let servlet = BackupServlet::new();
        assert_eq!(servlet.parse_game_id("not-a-number"), 0);
        assert_eq!(servlet.parse_game_id(""), 0);
        assert_eq!(servlet.parse_game_id("42"), 42);
    }
}
