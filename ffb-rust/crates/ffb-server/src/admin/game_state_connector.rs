use crate::request::fumbbl::util_fumbbl_request::{HttpClient, UtilFumbblRequest};

/// 1:1 translation of com.fumbbl.ffb.server.admin.GameStateConnector.
/// Command-line tool for gamestate operations (behaviours, get, result, reset, set).
///
/// Java reads server properties from an ini file to resolve `ServerUrlProperty` URL templates
/// and hashes the challenge response with real MD5 (`PasswordChallenge`). No property-file
/// loader or MD5 crate is wired into this workspace yet, so [`Self::run`] takes the resolved
/// challenge URL / per-subcommand URL template / admin password as parameters instead of
/// reading them from an ini file, and uses the same structurally-equivalent placeholder hash as
/// [`super::backup_servlet`].
pub struct GameStateConnector;

pub const USAGE: &str = "java com.fumbbl.ffb.server.admin.GameStateConnector behaviours <gameId>\n\
java com.fumbbl.ffb.server.admin.GameStateConnector get <gameId> <fromDb> <includeLog>\n\
java com.fumbbl.ffb.server.admin.GameStateConnector result <gameId>\n\
java com.fumbbl.ffb.server.admin.GameStateConnector reset <gameId>\n\
java com.fumbbl.ffb.server.admin.GameStateConnector set <file>\n";

impl GameStateConnector {
    pub fn new() -> Self {
        Self
    }

    fn create_response(challenge: &str, admin_password: &str) -> String {
        format!("{}:{}", challenge, admin_password)
    }

    /// `args[0]` is the subcommand (`behaviours`/`get`/`set`/`result`/`reset`); the rest are its
    /// parameters. `url_template` is the resolved `ServerUrlProperty` template for that
    /// subcommand (`$1` = challenge response, `$2..` = the subcommand's own args).
    pub fn run(
        &self,
        args: &[String],
        client: &dyn HttpClient,
        challenge_url: &str,
        admin_password: &str,
        url_template: &str,
    ) -> Result<String, String> {
        if args.is_empty() || args[0].is_empty() {
            return Ok(USAGE.to_string());
        }

        let challenge_xml = client.fetch_page(challenge_url)?;
        let challenge = UtilFumbblRequest::extract_xml_tag_first(&challenge_xml, "challenge").unwrap_or_default();
        let response = Self::create_response(&challenge, admin_password);

        let mut bind_params: Vec<&str> = vec![response.as_str()];
        bind_params.extend(args[1..].iter().map(|s| s.as_str()));
        let url = UtilFumbblRequest::bind(url_template, &bind_params);

        client.fetch_page(&url)
    }
}

impl Default for GameStateConnector {
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
    }

    #[test]
    fn construct() {
        let _ = GameStateConnector::new();
    }

    #[test]
    fn run_with_no_args_returns_usage() {
        let client = RecordingClient {
            urls: RefCell::new(Vec::new()),
            challenge_response: String::new(),
            result_response: String::new(),
        };
        let result = GameStateConnector::new()
            .run(&[], &client, "http://x/challenge", "adminpw", "http://x/get/$1/$2")
            .unwrap();
        assert!(result.contains("GameStateConnector"));
        assert!(client.urls.borrow().is_empty());
    }

    #[test]
    fn run_fetches_challenge_then_binds_response_into_command_url() {
        let client = RecordingClient {
            urls: RefCell::new(Vec::new()),
            challenge_response: "<challenge>abc</challenge>".to_string(),
            result_response: "{\"half\":1}".to_string(),
        };
        let args = vec!["get".to_string(), "42".to_string()];
        let result = GameStateConnector::new()
            .run(&args, &client, "http://x/challenge", "adminpw", "http://x/get/$1/$2")
            .unwrap();
        assert_eq!(result, "{\"half\":1}");
        let urls = client.urls.borrow();
        assert_eq!(urls[0], "http://x/challenge");
        assert_eq!(urls[1], "http://x/get/abc:adminpw/42");
    }
}
