use crate::request::fumbbl::util_fumbbl_request::{HttpClient, UtilFumbblRequest};
use std::collections::HashMap;

/// 1:1 translation of com.fumbbl.ffb.server.admin.AdminConnector.
/// Command-line admin tool that connects to the admin servlet via HTTP.
///
/// Supported commands: backup, block, cache, stats, close, concede, delete, forcelog,
/// list, logfile, loglevel, message, portrait, purgetest, redeploy, refresh,
/// shutdown, schedule, unblock, upload.
///
/// Java reads the challenge/command URL templates (`ServerUrlProperty`) from an ini file and
/// hashes the challenge response with real MD5 (`PasswordChallenge`). No property-file loader or
/// MD5 crate is wired into this workspace yet, so [`Self::run`] takes the resolved challenge URL
/// and a map of per-command URL templates as parameters, and uses the same
/// structurally-equivalent placeholder hash as [`super::backup_servlet`].
pub struct AdminConnector;

pub const USAGE: &str = "java com.fumbbl.ffb.server.admin.AdminConnector backup <gameId>\n\
java com.fumbbl.ffb.server.admin.AdminConnector block\n\
java com.fumbbl.ffb.server.admin.AdminConnector cache\n\
java com.fumbbl.ffb.server.admin.AdminConnector stats\n\
java com.fumbbl.ffb.server.admin.AdminConnector close <gameId>\n\
java com.fumbbl.ffb.server.admin.AdminConnector concede <gameId> <teamId>\n\
java com.fumbbl.ffb.server.admin.AdminConnector delete <gameId>\n\
java com.fumbbl.ffb.server.admin.AdminConnector forcelog <gameId>\n\
java com.fumbbl.ffb.server.admin.AdminConnector list <status>\n\
java com.fumbbl.ffb.server.admin.AdminConnector list <gameId>\n\
java com.fumbbl.ffb.server.admin.AdminConnector logfile <gameId>\n\
java com.fumbbl.ffb.server.admin.AdminConnector loglevel <value>\n\
java com.fumbbl.ffb.server.admin.AdminConnector message <message>\n\
java com.fumbbl.ffb.server.admin.AdminConnector portrait <coach>\n\
java com.fumbbl.ffb.server.admin.AdminConnector purgetest <limit> <perform>\n\
java com.fumbbl.ffb.server.admin.AdminConnector redeploy <branch> <force>\n\
java com.fumbbl.ffb.server.admin.AdminConnector refresh\n\
java com.fumbbl.ffb.server.admin.AdminConnector shutdown\n\
java com.fumbbl.ffb.server.admin.AdminConnector schedule <teamHomeId> <teamAwayId>\n\
java com.fumbbl.ffb.server.admin.AdminConnector unblock\n\
java com.fumbbl.ffb.server.admin.AdminConnector upload <gameId>";

impl AdminConnector {
    pub fn new() -> Self {
        Self
    }

    /// `args[0]` is the subcommand; the rest are its own parameters. `url_templates` maps
    /// subcommand name to its resolved `ServerUrlProperty` template (`$1` = challenge response,
    /// `$2..` = the subcommand's own args), except `list` which is looked up under `list_id`
    /// (when `args[1]` parses as a positive game id) or `list_status` otherwise, matching
    /// `ADMIN_URL_LIST_ID`/`ADMIN_URL_LIST_STATUS` in Java.
    pub fn run(
        &self,
        args: &[String],
        client: &dyn HttpClient,
        challenge_url: &str,
        admin_password: &str,
        url_templates: &HashMap<String, String>,
    ) -> Result<String, String> {
        if args.is_empty() || args[0].is_empty() {
            return Ok(USAGE.to_string());
        }
        let command = args[0].as_str();

        let challenge_xml = client.fetch_page(challenge_url)?;
        let challenge = UtilFumbblRequest::extract_xml_tag_first(&challenge_xml, "challenge").unwrap_or_default();
        let response = format!("{}:{}", challenge, admin_password);

        let (template_key, extra_args): (&str, Vec<String>) = match command {
            "list" => {
                let game_id: i64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                if game_id > 0 {
                    ("list_id", vec![args[1].clone()])
                } else {
                    ("list_status", vec![args.get(1).cloned().unwrap_or_default()])
                }
            }
            "purgetest" => {
                let perform = args.get(2).cloned().unwrap_or_else(|| "false".to_string());
                ("purgetest", vec![args.get(1).cloned().unwrap_or_default(), perform])
            }
            "redeploy" => {
                let force = args.get(2).cloned().unwrap_or_else(|| "false".to_string());
                ("redeploy", vec![args.get(1).cloned().unwrap_or_default(), force])
            }
            other => (other, args[1..].to_vec()),
        };

        let template = url_templates
            .get(template_key)
            .ok_or_else(|| format!("no url template configured for '{}'", template_key))?;
        let mut bind_params: Vec<&str> = vec![response.as_str()];
        bind_params.extend(extra_args.iter().map(|s| s.as_str()));
        let url = UtilFumbblRequest::bind(template, &bind_params);

        if command == "logfile" {
            client.load_file(&url)
        } else {
            client.fetch_page(&url)
        }
    }
}

impl Default for AdminConnector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct RecordingClient {
        urls: RefCell<Vec<String>>,
        challenge_response: String,
        result_response: String,
    }

    impl HttpClient for RecordingClient {
        fn fetch_page(&self, url: &str) -> Result<String, String> {
            self.urls.borrow_mut().push(url.to_string());
            if url.contains("challenge") {
                Ok(self.challenge_response.clone())
            } else {
                Ok(self.result_response.clone())
            }
        }

        fn load_file(&self, url: &str) -> Result<String, String> {
            self.urls.borrow_mut().push(url.to_string());
            Ok(format!("Stored in: {}", self.result_response))
        }

        fn post_multipart_xml(&self, _url: &str, _challenge_response: &str, _result_xml: &str) -> Result<String, String> {
            unimplemented!("not exercised by AdminConnector")
        }

        fn post_authorized_form(&self, _url: &str, _challenge_response: &str, _key: &str, _payload: &str) -> Result<String, String> {
            unimplemented!("not exercised by AdminConnector")
        }

        fn post_file(&self, _url: &str, _file_path: &std::path::Path) -> Result<String, String> {
            unimplemented!("not exercised by AdminConnector")
        }
    }

    fn templates() -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("close".to_string(), "http://x/close/$1/$2".to_string());
        map.insert("list_id".to_string(), "http://x/list/id/$1/$2".to_string());
        map.insert("list_status".to_string(), "http://x/list/status/$1/$2".to_string());
        map.insert("purgetest".to_string(), "http://x/purgetest/$1/$2/$3".to_string());
        map.insert("shutdown".to_string(), "http://x/shutdown/$1".to_string());
        map.insert("logfile".to_string(), "http://x/logfile/$1/$2".to_string());
        map
    }

    #[test]
    fn construct() {
        let _ = AdminConnector::new();
    }

    #[test]
    fn run_with_no_args_returns_usage() {
        let client = RecordingClient {
            urls: RefCell::new(Vec::new()),
            challenge_response: String::new(),
            result_response: String::new(),
        };
        let result = AdminConnector::new()
            .run(&[], &client, "http://x/challenge", "adminpw", &templates())
            .unwrap();
        assert!(result.contains("AdminConnector"));
    }

    #[test]
    fn run_close_binds_response_and_game_id() {
        let client = RecordingClient {
            urls: RefCell::new(Vec::new()),
            challenge_response: "<challenge>abc</challenge>".to_string(),
            result_response: "<status>ok</status>".to_string(),
        };
        let args = vec!["close".to_string(), "42".to_string()];
        let result = AdminConnector::new()
            .run(&args, &client, "http://x/challenge", "adminpw", &templates())
            .unwrap();
        assert_eq!(result, "<status>ok</status>");
        assert_eq!(client.urls.borrow()[1], "http://x/close/abc:adminpw/42");
    }

    #[test]
    fn run_list_with_positive_game_id_uses_list_by_id_template() {
        let client = RecordingClient {
            urls: RefCell::new(Vec::new()),
            challenge_response: "<challenge>abc</challenge>".to_string(),
            result_response: String::new(),
        };
        let args = vec!["list".to_string(), "7".to_string()];
        AdminConnector::new()
            .run(&args, &client, "http://x/challenge", "adminpw", &templates())
            .unwrap();
        assert_eq!(client.urls.borrow()[1], "http://x/list/id/abc:adminpw/7");
    }

    #[test]
    fn run_list_with_status_uses_list_by_status_template() {
        let client = RecordingClient {
            urls: RefCell::new(Vec::new()),
            challenge_response: "<challenge>abc</challenge>".to_string(),
            result_response: String::new(),
        };
        let args = vec!["list".to_string(), "active".to_string()];
        AdminConnector::new()
            .run(&args, &client, "http://x/challenge", "adminpw", &templates())
            .unwrap();
        assert_eq!(client.urls.borrow()[1], "http://x/list/status/abc:adminpw/active");
    }

    #[test]
    fn run_purgetest_defaults_perform_to_false() {
        let client = RecordingClient {
            urls: RefCell::new(Vec::new()),
            challenge_response: "<challenge>abc</challenge>".to_string(),
            result_response: String::new(),
        };
        let args = vec!["purgetest".to_string(), "10".to_string()];
        AdminConnector::new()
            .run(&args, &client, "http://x/challenge", "adminpw", &templates())
            .unwrap();
        assert_eq!(client.urls.borrow()[1], "http://x/purgetest/abc:adminpw/10/false");
    }

    #[test]
    fn run_logfile_uses_load_file_not_fetch_page() {
        let client = RecordingClient {
            urls: RefCell::new(Vec::new()),
            challenge_response: "<challenge>abc</challenge>".to_string(),
            result_response: "app.log".to_string(),
        };
        let args = vec!["logfile".to_string(), "42".to_string()];
        let result = AdminConnector::new()
            .run(&args, &client, "http://x/challenge", "adminpw", &templates())
            .unwrap();
        assert_eq!(result, "Stored in: app.log");
        assert_eq!(client.urls.borrow()[1], "http://x/logfile/abc:adminpw/42");
    }

    #[test]
    fn run_unknown_template_errors() {
        let client = RecordingClient {
            urls: RefCell::new(Vec::new()),
            challenge_response: "<challenge>abc</challenge>".to_string(),
            result_response: String::new(),
        };
        let args = vec!["backup".to_string(), "1".to_string()];
        let result = AdminConnector::new().run(&args, &client, "http://x/challenge", "adminpw", &templates());
        assert!(result.is_err());
    }
}
