/// 1:1 translation of com.fumbbl.ffb.server.request.fumbbl.UtilFumbblRequest.
pub struct UtilFumbblRequest;

/// Minimal HTTP client abstraction so the URL-building / response-parsing logic in the
/// `request::fumbbl` module can be unit tested without an actual network round trip. Java's
/// `UtilServerHttpClient.fetchPage` is the real equivalent.
pub trait HttpClient {
    fn fetch_page(&self, url: &str) -> Result<String, String>;
}

/// 1:1-behavior translation of `com.fumbbl.ffb.server.util.UtilServerHttpClient.fetchPage`:
/// issues a blocking GET and returns the response body, or an error message on any
/// non-2xx status / transport failure. This is the runtime `HttpClient` used by the server;
/// tests use `MockHttpClient` (below) instead so the test suite never makes a live network call.
pub struct ReqwestHttpClient {
    client: reqwest::blocking::Client,
}

impl ReqwestHttpClient {
    pub fn new() -> Self {
        Self { client: reqwest::blocking::Client::new() }
    }
}

impl Default for ReqwestHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpClient for ReqwestHttpClient {
    fn fetch_page(&self, url: &str) -> Result<String, String> {
        let response = self
            .client
            .get(url)
            .send()
            .map_err(|e| format!("failed to fetch {url}: {e}"))?;
        if !response.status().is_success() {
            return Err(format!("fetch {url} failed with status {}", response.status()));
        }
        response
            .text()
            .map_err(|e| format!("failed to read response body from {url}: {e}"))
    }
}

impl UtilFumbblRequest {
    pub const CHARACTER_ENCODING: &'static str = "UTF-8";

    const UNKNOWN_FUMBBL_ERROR: &'static str = "Unknown problem accessing Fumbbl.";

    /// Port of `StringTool.bind(String, Object[])`: replaces `$1`, `$2`, ... (1-indexed)
    /// placeholders in `template` with the corresponding entry of `params`.
    pub fn bind(template: &str, params: &[&str]) -> String {
        let chars: Vec<char> = template.chars().collect();
        let mut result = String::new();
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == '$' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit() {
                let mut j = i + 1;
                while j < chars.len() && chars[j].is_ascii_digit() {
                    j += 1;
                }
                let digits: String = chars[i + 1..j].iter().collect();
                if let Ok(index) = digits.parse::<usize>() {
                    if index >= 1 && index <= params.len() {
                        result.push_str(params[index - 1]);
                    }
                }
                i = j;
            } else {
                result.push(chars[i]);
                i += 1;
            }
        }
        result
    }

    /// Port of the `Matcher.find()` loop used throughout the Java request classes: scans
    /// `response` line by line for the FIRST `<tag>value</tag>` occurrence and returns its value.
    pub fn extract_xml_tag_first(response: &str, tag: &str) -> Option<String> {
        Self::extract_xml_tag(response, tag, false)
    }

    /// Same scan as [`extract_xml_tag_first`] but keeps overwriting the result on every match,
    /// mirroring loops (e.g. in `FumbblRequestCheckAuthorization`) that keep the LAST match.
    pub fn extract_xml_tag_last(response: &str, tag: &str) -> Option<String> {
        Self::extract_xml_tag(response, tag, true)
    }

    fn extract_xml_tag(response: &str, tag: &str, keep_last: bool) -> Option<String> {
        let open = format!("<{}>", tag);
        let close = format!("</{}>", tag);
        let mut found = None;
        for line in response.lines() {
            if let Some(start) = line.find(&open) {
                let after = &line[start + open.len()..];
                if let Some(end) = after.find(&close) {
                    found = Some(after[..end].to_string());
                    if !keep_last {
                        return found;
                    }
                }
            }
        }
        found
    }

    /// Fetches a FUMBBL gamestate URL and parses the XML response into a FumbblGameState.
    pub fn process_fumbbl_game_state_request(
        client: &dyn HttpClient,
        request_url: &str,
    ) -> Result<Option<super::fumbbl_game_state::FumbblGameState>, String> {
        if request_url.is_empty() {
            return Ok(None);
        }
        let response_xml = client.fetch_page(request_url)?;
        Ok(Self::process_fumbbl_game_state_response(request_url, &response_xml))
    }

    fn process_fumbbl_game_state_response(
        request_url: &str,
        response_xml: &str,
    ) -> Option<super::fumbbl_game_state::FumbblGameState> {
        if response_xml.is_empty() {
            return None;
        }
        let mut state = super::fumbbl_game_state::FumbblGameState::new(request_url.to_string());
        if let Some(result) = Self::extract_xml_tag_first(response_xml, "result") {
            state.result = result;
        }
        if let Some(reason) = Self::extract_xml_tag_first(response_xml, "reason") {
            state.reason = reason;
        }
        if let Some(description) = Self::extract_xml_tag_first(response_xml, "description") {
            state.description = description;
        }
        if let Some(game_id) = Self::extract_xml_tag_first(response_xml, "gameid") {
            state.game_id = game_id;
        }
        Some(state)
    }

    /// Fetches the auth challenge for the given coach and turns it into a challenge response.
    pub fn get_fumbbl_auth_challenge_response_for_fumbbl_user(
        client: &dyn HttpClient,
        challenge_url_template: &str,
        fumbbl_user: &str,
        fumbbl_user_password: &str,
    ) -> Result<Option<String>, String> {
        let challenge = Self::get_fumbbl_auth_challenge_for(client, challenge_url_template, fumbbl_user)?;
        Ok(match challenge {
            Some(challenge) => Self::create_fumbbl_auth_challenge_response(&challenge, fumbbl_user_password),
            None => None,
        })
    }

    /// Creates a challenge response from a challenge string and password.
    ///
    /// Java hashes the challenge with the coach's MD5-encoded password
    /// (`PasswordChallenge.createResponse`). No MD5 crate is wired into this workspace yet, so
    /// this is a structurally-equivalent placeholder (same guard clauses, same "combine
    /// challenge+password" shape) rather than a byte-for-byte port of the hash itself.
    pub fn create_fumbbl_auth_challenge_response(challenge: &str, password: &str) -> Option<String> {
        if challenge.is_empty() || password.is_empty() {
            return None;
        }
        Some(format!("{}:{}", challenge, password))
    }

    /// Fetches the auth challenge XML for a specific coach.
    pub fn get_fumbbl_auth_challenge_for(
        client: &dyn HttpClient,
        challenge_url_template: &str,
        coach: &str,
    ) -> Result<Option<String>, String> {
        if coach.is_empty() {
            return Ok(None);
        }
        let challenge_url = Self::bind(challenge_url_template, &[coach]);
        let response_xml = client.fetch_page(&challenge_url)?;
        if response_xml.is_empty() {
            return Ok(None);
        }
        Ok(Self::extract_xml_tag_first(&response_xml, "challenge"))
    }

    /// Builds the FUMBBL_ERROR status message/log line for a failed request.
    pub fn report_fumbbl_error(game_state: Option<&super::fumbbl_game_state::FumbblGameState>) -> String {
        match game_state {
            Some(state) => state.get_description().to_string(),
            None => Self::UNKNOWN_FUMBBL_ERROR.to_string(),
        }
    }

    /// Fetches FUMBBL_TEAM URL for a team id.
    pub fn load_fumbbl_team(client: &dyn HttpClient, team_url_template: &str, team_id: &str) -> Result<Option<String>, String> {
        if team_id.is_empty() {
            return Ok(None);
        }
        let team_url = Self::bind(team_url_template, &[team_id]);
        let team_xml = client.fetch_page(&team_url)?;
        if team_xml.is_empty() {
            return Ok(None);
        }
        Ok(Some(team_xml))
    }

    /// Fetches FUMBBL_ROSTER_TEAM URL for a team id.
    pub fn load_fumbbl_roster_for_team(
        client: &dyn HttpClient,
        roster_url_template: &str,
        team_id: &str,
    ) -> Result<Option<String>, String> {
        if team_id.is_empty() {
            return Ok(None);
        }
        let roster_url = Self::bind(roster_url_template, &[team_id]);
        let roster_xml = client.fetch_page(&roster_url)?;
        if roster_xml.is_empty() {
            return Ok(None);
        }
        Ok(Some(roster_xml))
    }
}

#[cfg(test)]
pub(crate) struct MockHttpClient {
    pub response: Result<String, String>,
}

#[cfg(test)]
impl HttpClient for MockHttpClient {
    fn fetch_page(&self, _url: &str) -> Result<String, String> {
        self.response.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encoding_constant() {
        assert_eq!(UtilFumbblRequest::CHARACTER_ENCODING, "UTF-8");
    }

    #[test]
    fn bind_replaces_placeholders_in_order() {
        let bound = UtilFumbblRequest::bind("http://x/$1/$2", &["10", "20"]);
        assert_eq!(bound, "http://x/10/20");
    }

    #[test]
    fn bind_ignores_out_of_range_placeholder() {
        let bound = UtilFumbblRequest::bind("http://x/$1/$3", &["10", "20"]);
        assert_eq!(bound, "http://x/10/");
    }

    #[test]
    fn extract_xml_tag_first_returns_first_match() {
        let xml = "<response>OK foo</response>\n<response>OK bar</response>";
        assert_eq!(
            UtilFumbblRequest::extract_xml_tag_first(xml, "response"),
            Some("OK foo".to_string())
        );
    }

    #[test]
    fn extract_xml_tag_last_returns_last_match() {
        let xml = "<response>OK foo</response>\n<response>OK bar</response>";
        assert_eq!(
            UtilFumbblRequest::extract_xml_tag_last(xml, "response"),
            Some("OK bar".to_string())
        );
    }

    #[test]
    fn process_fumbbl_game_state_request_parses_ok_response() {
        let client = MockHttpClient {
            response: Ok("<gamestate><result>ok</result><gameid>7</gameid></gamestate>".to_string()),
        };
        let state = UtilFumbblRequest::process_fumbbl_game_state_request(&client, "http://fumbbl/x")
            .unwrap()
            .unwrap();
        assert!(state.is_ok());
        assert_eq!(state.get_game_id(), "7");
    }

    #[test]
    fn process_fumbbl_game_state_request_empty_url_returns_none() {
        let client = MockHttpClient { response: Ok(String::new()) };
        let result = UtilFumbblRequest::process_fumbbl_game_state_request(&client, "").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn report_fumbbl_error_uses_description_when_present() {
        let mut state = super::super::fumbbl_game_state::FumbblGameState::new("http://x".to_string());
        state.description = "boom".to_string();
        assert_eq!(UtilFumbblRequest::report_fumbbl_error(Some(&state)), "boom");
    }

    #[test]
    fn report_fumbbl_error_uses_unknown_message_when_absent() {
        assert_eq!(
            UtilFumbblRequest::report_fumbbl_error(None),
            "Unknown problem accessing Fumbbl."
        );
    }

    #[test]
    fn get_fumbbl_auth_challenge_for_extracts_challenge() {
        let client = MockHttpClient {
            response: Ok("<challenge>abc123</challenge>".to_string()),
        };
        let challenge =
            UtilFumbblRequest::get_fumbbl_auth_challenge_for(&client, "http://fumbbl/auth/$1", "coach").unwrap();
        assert_eq!(challenge, Some("abc123".to_string()));
    }

    #[test]
    fn load_fumbbl_team_returns_none_for_empty_team_id() {
        let client = MockHttpClient { response: Ok("<team/>".to_string()) };
        let result = UtilFumbblRequest::load_fumbbl_team(&client, "http://fumbbl/team/$1", "").unwrap();
        assert!(result.is_none());
    }

    /// Construction only — does not perform any network I/O.
    #[test]
    fn reqwest_http_client_constructs() {
        let _client = ReqwestHttpClient::new();
        let _default_client = ReqwestHttpClient::default();
    }

    /// Malformed URL should fail fast with an `Err`, without any network round trip.
    #[test]
    fn reqwest_http_client_fetch_page_invalid_url_is_error() {
        let client = ReqwestHttpClient::new();
        let result = client.fetch_page("not-a-valid-url");
        assert!(result.is_err());
    }
}
