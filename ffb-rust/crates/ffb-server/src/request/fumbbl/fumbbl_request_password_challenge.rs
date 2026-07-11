/// 1:1 translation of com.fumbbl.ffb.server.request.fumbbl.FumbblRequestPasswordChallenge.
/// Fetches FUMBBL_AUTH_CHALLENGE URL, extracts challenge from XML, sends to session.
pub struct FumbblRequestPasswordChallenge {
    pub coach: String,
    request_url: String,
}

impl FumbblRequestPasswordChallenge {
    pub fn new(coach: String) -> Self {
        Self { coach, request_url: String::new() }
    }

    pub fn get_coach(&self) -> &str {
        &self.coach
    }

    pub fn get_request_url(&self) -> &str {
        &self.request_url
    }

    pub fn set_request_url(&mut self, url: String) {
        self.request_url = url;
    }

    /// Builds the FUMBBL_AUTH_CHALLENGE URL for the coach, fetches it, and extracts the
    /// `<challenge>` value from the response (mirrors Java's `_PATTERN_CHALLENGE` matcher loop).
    pub fn process(
        &mut self,
        client: &dyn super::util_fumbbl_request::HttpClient,
        challenge_url_template: &str,
    ) -> Result<Option<String>, String> {
        let url = super::util_fumbbl_request::UtilFumbblRequest::bind(challenge_url_template, &[self.coach.as_str()]);
        self.set_request_url(url);
        let response_xml = client.fetch_page(self.get_request_url())?;
        if response_xml.is_empty() {
            return Ok(None);
        }
        Ok(super::util_fumbbl_request::UtilFumbblRequest::extract_xml_tag_first(
            &response_xml,
            "challenge",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::fumbbl::util_fumbbl_request::MockHttpClient;

    #[test]
    fn construct() {
        let r = FumbblRequestPasswordChallenge::new("coach".to_string());
        assert_eq!(r.get_coach(), "coach");
    }

    #[test]
    fn process_builds_url_and_extracts_challenge() {
        let client = MockHttpClient {
            response: Ok("<challenge>abc123</challenge>".to_string()),
        };
        let mut r = FumbblRequestPasswordChallenge::new("coach".to_string());
        let challenge = r.process(&client, "http://fumbbl/auth/challenge/$1").unwrap();
        assert_eq!(challenge, Some("abc123".to_string()));
        assert_eq!(r.get_request_url(), "http://fumbbl/auth/challenge/coach");
    }

    #[test]
    fn process_empty_response_returns_none() {
        let client = MockHttpClient { response: Ok(String::new()) };
        let mut r = FumbblRequestPasswordChallenge::new("coach".to_string());
        let challenge = r.process(&client, "http://fumbbl/auth/challenge/$1").unwrap();
        assert!(challenge.is_none());
    }
}
