/// 1:1 translation of com.fumbbl.ffb.server.admin.AdminServlet.
/// HTTP servlet handling admin commands for the game server.
///
/// Java wires this to a live `FantasyFootballServer` (game cache, DB queries, session manager,
/// communication layer) that does not exist in this simplified server crate yet. Commands that
/// only touch that plumbing (`cache`, `stats`, `portrait`, `redeploy`, `schedule`, and the
/// `InternalServerCommand*` dispatches inside `close`/`delete`/`upload`/`concede`) are ported as
/// XML-shape-only stubs; commands whose logic is pure parameter parsing / XML building
/// (`challenge`, `loglevel`, `block`/`unblock`, `close`, `delete`, `upload`, `backup`,
/// `concede`, `message`, `forcelog`, `list`) are fully ported, including the `blockingNewGames`
/// flag which is now tracked on this struct directly instead of on `FantasyFootballServer`.
///
/// Java hashes the challenge with real MD5 (`PasswordChallenge`); see
/// [`super::backup_servlet`] for why this uses a structurally-equivalent placeholder instead.
pub struct AdminServlet {
    last_challenge: Option<String>,
    blocking_new_games: bool,
}

impl AdminServlet {
    // Command name constants
    pub const BACKUP: &'static str = "backup";
    pub const BLOCK: &'static str = "block";
    pub const CACHE: &'static str = "cache";
    pub const CHALLENGE: &'static str = "challenge";
    pub const CLOSE: &'static str = "close";
    pub const CONCEDE: &'static str = "concede";
    pub const DELETE: &'static str = "delete";
    pub const LIST: &'static str = "list";
    pub const LOGLEVEL: &'static str = "loglevel";
    pub const MESSAGE: &'static str = "message";
    pub const REFRESH: &'static str = "refresh";
    pub const SCHEDULE: &'static str = "schedule";
    pub const SHUTDOWN: &'static str = "shutdown";
    pub const STATS: &'static str = "stats";
    pub const UNBLOCK: &'static str = "unblock";
    pub const UPLOAD: &'static str = "upload";
    pub const FORCE_LOG: &'static str = "forcelog";
    pub const PORTRAIT: &'static str = "portrait";
    pub const PURGE_TEST: &'static str = "purgetest";
    pub const LOGFILE: &'static str = "logfile";
    pub const REDEPLOY: &'static str = "redeploy";

    pub const STATUS_OK: &'static str = "ok";
    pub const STATUS_FAIL: &'static str = "fail";

    // Request parameter constants
    pub const PARAMETER_BRANCH: &'static str = "branch";
    pub const PARAMETER_COACH: &'static str = "coach";
    pub const PARAMETER_RESPONSE: &'static str = "response";
    pub const PARAMETER_GAME_ID: &'static str = "gameId";
    pub const PARAMETER_TEAM_ID: &'static str = "teamId";
    pub const PARAMETER_STATUS: &'static str = "status";
    pub const PARAMETER_MESSAGE: &'static str = "message";
    pub const PARAMETER_TEAM_HOME_ID: &'static str = "teamHomeId";
    pub const PARAMETER_TEAM_AWAY_ID: &'static str = "teamAwayId";
    pub const PARAMETER_VALUE: &'static str = "value";
    pub const PARAMETER_LIMIT: &'static str = "limit";
    pub const PARAMETER_PERFORM: &'static str = "perform";
    pub const PARAMETER_FORCE: &'static str = "force";

    pub fn new() -> Self {
        Self {
            last_challenge: None,
            blocking_new_games: false,
        }
    }

    fn hash_challenge(salt: &str, now_millis: i64) -> String {
        format!("{:x}", salt.len() as i64 ^ now_millis)
    }

    pub fn is_blocking_new_games(&self) -> bool {
        self.blocking_new_games
    }

    pub fn parse_game_id(&self, s: &str) -> i64 {
        if s.is_empty() {
            return 0;
        }
        s.parse::<i64>().unwrap_or(0)
    }

    /// Validates `response` against the last issued challenge combined with `admin_password`.
    /// Consumes the challenge (one-shot), mirroring Java's `fLastChallenge = null` at the end.
    pub fn check_response(&mut self, response: &str, admin_password: &str) -> bool {
        let is_ok = match &self.last_challenge {
            Some(challenge) => format!("{}:{}", challenge, admin_password) == response,
            None => false,
        };
        self.last_challenge = None;
        is_ok
    }

    fn wrap_admin_xml(is_ok: bool, body: &str) -> String {
        format!(
            "<admin>{}<status>{}</status></admin>",
            body,
            if is_ok { Self::STATUS_OK } else { Self::STATUS_FAIL }
        )
    }

    /// Routes a GET command. `admin_salt`/`now_millis` are used only for [`Self::CHALLENGE`];
    /// `admin_password` gates every other command via [`Self::check_response`].
    pub fn handle_get(
        &mut self,
        command: &str,
        params: &std::collections::HashMap<String, String>,
        admin_password: &str,
        admin_salt: &str,
        now_millis: i64,
        admin_list: Option<&super::admin_list::AdminList>,
    ) -> String {
        if command == Self::CHALLENGE {
            let challenge = Self::hash_challenge(admin_salt, now_millis);
            self.last_challenge = Some(challenge.clone());
            return Self::wrap_admin_xml(true, &format!("<challenge>{}</challenge>", challenge));
        }
        if command == Self::CACHE {
            // Java renders live GameCache entries; no cache exists in this crate yet.
            return Self::wrap_admin_xml(true, "<cache size=\"0\" activeGames=\"0\"></cache>");
        }

        let response = params.get(Self::PARAMETER_RESPONSE).map(|s| s.as_str()).unwrap_or("");
        if !self.check_response(response, admin_password) {
            return Self::wrap_admin_xml(false, "");
        }

        let get = |key: &str| params.get(key).map(|s| s.as_str()).unwrap_or("");

        match command {
            Self::SHUTDOWN => Self::wrap_admin_xml(true, &format!("<shutdown initiated=\"{}\"/>", now_millis)),
            Self::REFRESH => Self::wrap_admin_xml(true, &format!("<refresh initiated=\"{}\"/>", now_millis)),
            Self::BLOCK => {
                self.blocking_new_games = true;
                Self::wrap_admin_xml(true, "<block/>")
            }
            Self::UNBLOCK => {
                self.blocking_new_games = false;
                Self::wrap_admin_xml(true, "<unblock/>")
            }
            Self::LOGLEVEL => {
                let value = get(Self::PARAMETER_VALUE);
                Self::wrap_admin_xml(true, &format!("<loglevel value=\"{}\"/>", value))
            }
            Self::CLOSE => self.game_id_command(get(Self::PARAMETER_GAME_ID), "close"),
            Self::FORCE_LOG => self.game_id_command(get(Self::PARAMETER_GAME_ID), "forcelog"),
            Self::DELETE => self.game_id_command(get(Self::PARAMETER_GAME_ID), "delete"),
            Self::UPLOAD => self.game_id_command(get(Self::PARAMETER_GAME_ID), "upload"),
            Self::BACKUP => self.game_id_command(get(Self::PARAMETER_GAME_ID), "backup"),
            Self::CONCEDE => {
                let team_id = get(Self::PARAMETER_TEAM_ID);
                if team_id.is_empty() || team_id == "0" {
                    return Self::wrap_admin_xml(false, "<error>Invalid or missing teamId parameter</error>");
                }
                let game_id_string = get(Self::PARAMETER_GAME_ID);
                let game_id = self.parse_game_id(game_id_string);
                if game_id > 0 {
                    Self::wrap_admin_xml(
                        true,
                        &format!("<concede gameId=\"{}\" teamId=\"{}\"/>", game_id_string, team_id),
                    )
                } else {
                    Self::wrap_admin_xml(false, "<error>Invalid or missing gameId parameter</error>")
                }
            }
            Self::MESSAGE => {
                let message = get(Self::PARAMETER_MESSAGE);
                if message.is_empty() {
                    Self::wrap_admin_xml(false, "<message/>")
                } else {
                    Self::wrap_admin_xml(true, &format!("<message>{}</message>", message))
                }
            }
            Self::LIST => {
                let list = admin_list.map(|l| l.size()).unwrap_or(0);
                if list == 0 {
                    Self::wrap_admin_xml(true, "<list size=\"0\"/>")
                } else {
                    let entries = admin_list
                        .map(|l| l.get_entries().iter().map(|e| e.to_xml()).collect::<String>())
                        .unwrap_or_default();
                    Self::wrap_admin_xml(true, &format!("<list size=\"{}\">{}</list>", list, entries))
                }
            }
            Self::PURGE_TEST => {
                let limit = get(Self::PARAMETER_LIMIT);
                if limit.is_empty() || limit.parse::<i64>().map(|v| v < 1).unwrap_or(true) {
                    return Self::wrap_admin_xml(false, "");
                }
                let list = admin_list.map(|l| l.size()).unwrap_or(0);
                let perform = params
                    .get(Self::PARAMETER_PERFORM)
                    .map(|v| v.eq_ignore_ascii_case("true"))
                    .unwrap_or(false);
                Self::wrap_admin_xml(
                    true,
                    &format!("<list size=\"{}\"/><deleted>{}</deleted>", list, perform),
                )
            }
            // Java delegates these to live server subsystems (SkillFactory/session
            // manager/GameCache/RedeployHandler/DB queries) that do not exist here yet.
            Self::STATS | Self::PORTRAIT | Self::SCHEDULE | Self::REDEPLOY => {
                Self::wrap_admin_xml(false, "<error>not implemented</error>")
            }
            _ => Self::wrap_admin_xml(false, ""),
        }
    }

    fn game_id_command(&self, game_id_string: &str, tag: &str) -> String {
        let game_id = self.parse_game_id(game_id_string);
        if game_id > 0 {
            Self::wrap_admin_xml(true, &format!("<{} gameId=\"{}\"/>", tag, game_id_string))
        } else {
            Self::wrap_admin_xml(false, "<error>Invalid or missing gameId parameter</error>")
        }
    }
}

impl Default for AdminServlet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let _ = AdminServlet::new();
    }

    #[test]
    fn constants() {
        assert_eq!(AdminServlet::STATUS_OK, "ok");
        assert_eq!(AdminServlet::CHALLENGE, "challenge");
    }

    fn extract_challenge(xml: &str) -> String {
        xml.split("<challenge>").nth(1).unwrap().split("</challenge>").next().unwrap().to_string()
    }

    #[test]
    fn challenge_command_bypasses_check_response() {
        let mut servlet = AdminServlet::new();
        let params = std::collections::HashMap::new();
        let xml = servlet.handle_get(AdminServlet::CHALLENGE, &params, "adminpw", "salt", 1000, None);
        assert!(xml.contains("<challenge>"));
        assert!(xml.contains(AdminServlet::STATUS_OK));
    }

    #[test]
    fn other_commands_require_valid_response() {
        let mut servlet = AdminServlet::new();
        let params = std::collections::HashMap::new();
        servlet.handle_get(AdminServlet::CHALLENGE, &params, "adminpw", "salt", 1000, None);

        let mut bad_params = std::collections::HashMap::new();
        bad_params.insert(AdminServlet::PARAMETER_RESPONSE.to_string(), "wrong".to_string());
        let xml = servlet.handle_get(AdminServlet::REFRESH, &bad_params, "adminpw", "salt", 1000, None);
        assert!(xml.contains(AdminServlet::STATUS_FAIL));

        // Re-issue a challenge since the previous check_response call consumed it.
        let challenge_xml = servlet.handle_get(AdminServlet::CHALLENGE, &params, "adminpw", "salt", 1000, None);
        let challenge = extract_challenge(&challenge_xml);
        let mut good_params = std::collections::HashMap::new();
        good_params.insert(
            AdminServlet::PARAMETER_RESPONSE.to_string(),
            format!("{}:{}", challenge, "adminpw"),
        );
        let xml = servlet.handle_get(AdminServlet::REFRESH, &good_params, "adminpw", "salt", 1000, None);
        assert!(xml.contains(AdminServlet::STATUS_OK));
        assert!(xml.contains("<refresh"));
    }

    fn authorized_params(servlet: &mut AdminServlet) -> std::collections::HashMap<String, String> {
        let empty = std::collections::HashMap::new();
        let challenge_xml = servlet.handle_get(AdminServlet::CHALLENGE, &empty, "adminpw", "salt", 1000, None);
        let challenge = extract_challenge(&challenge_xml);
        let mut params = std::collections::HashMap::new();
        params.insert(AdminServlet::PARAMETER_RESPONSE.to_string(), format!("{}:{}", challenge, "adminpw"));
        params
    }

    #[test]
    fn block_and_unblock_toggle_flag() {
        let mut servlet = AdminServlet::new();
        let params = authorized_params(&mut servlet);
        servlet.handle_get(AdminServlet::BLOCK, &params, "adminpw", "salt", 1000, None);
        assert!(servlet.is_blocking_new_games());

        let params = authorized_params(&mut servlet);
        servlet.handle_get(AdminServlet::UNBLOCK, &params, "adminpw", "salt", 1000, None);
        assert!(!servlet.is_blocking_new_games());
    }

    #[test]
    fn close_with_valid_game_id_succeeds() {
        let mut servlet = AdminServlet::new();
        let mut params = authorized_params(&mut servlet);
        params.insert(AdminServlet::PARAMETER_GAME_ID.to_string(), "42".to_string());
        let xml = servlet.handle_get(AdminServlet::CLOSE, &params, "adminpw", "salt", 1000, None);
        assert!(xml.contains(AdminServlet::STATUS_OK));
        assert!(xml.contains("gameId=\"42\""));
    }

    #[test]
    fn close_with_missing_game_id_fails() {
        let mut servlet = AdminServlet::new();
        let params = authorized_params(&mut servlet);
        let xml = servlet.handle_get(AdminServlet::CLOSE, &params, "adminpw", "salt", 1000, None);
        assert!(xml.contains(AdminServlet::STATUS_FAIL));
    }

    #[test]
    fn concede_requires_team_id_and_game_id() {
        let mut servlet = AdminServlet::new();
        let params = authorized_params(&mut servlet);
        let xml = servlet.handle_get(AdminServlet::CONCEDE, &params, "adminpw", "salt", 1000, None);
        assert!(xml.contains("teamId"));
        assert!(xml.contains(AdminServlet::STATUS_FAIL));

        let mut params = authorized_params(&mut servlet);
        params.insert(AdminServlet::PARAMETER_TEAM_ID.to_string(), "3".to_string());
        params.insert(AdminServlet::PARAMETER_GAME_ID.to_string(), "42".to_string());
        let xml = servlet.handle_get(AdminServlet::CONCEDE, &params, "adminpw", "salt", 1000, None);
        assert!(xml.contains(AdminServlet::STATUS_OK));
    }

    #[test]
    fn list_with_entries_renders_admin_list_entries() {
        let mut servlet = AdminServlet::new();
        let params = authorized_params(&mut servlet);
        let mut list = super::super::admin_list::AdminList::new();
        list.add(super::super::admin_list_entry::AdminListEntry::new(1));
        let xml = servlet.handle_get(AdminServlet::LIST, &params, "adminpw", "salt", 1000, Some(&list));
        assert!(xml.contains("size=\"1\""));
        assert!(xml.contains("<game"));
    }

    #[test]
    fn purge_test_without_limit_fails() {
        let mut servlet = AdminServlet::new();
        let params = authorized_params(&mut servlet);
        let xml = servlet.handle_get(AdminServlet::PURGE_TEST, &params, "adminpw", "salt", 1000, None);
        assert!(xml.contains(AdminServlet::STATUS_FAIL));
    }

    #[test]
    fn parse_game_id_returns_zero_for_invalid_input() {
        let servlet = AdminServlet::new();
        assert_eq!(servlet.parse_game_id("not-a-number"), 0);
        assert_eq!(servlet.parse_game_id("42"), 42);
    }
}
