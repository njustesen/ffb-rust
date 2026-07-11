/// 1:1 translation of com.fumbbl.ffb.server.request.fumbbl.FumbblRequestCheckAuthorization.
/// POSTs coach+password to FUMBBL_AUTH_RESPONSE; on OK dispatches JoinApproved command.
pub struct FumbblRequestCheckAuthorization {
    pub coach: String,
    pub password: String,
    pub game_id: i64,
    pub game_name: String,
    pub team_id: String,
    request_url: String,
}

impl FumbblRequestCheckAuthorization {
    pub fn new(coach: String, password: String, game_id: i64, game_name: String, team_id: String) -> Self {
        Self {
            coach,
            password,
            game_id,
            game_name,
            team_id,
            request_url: String::new(),
        }
    }

    pub fn get_coach(&self) -> &str {
        &self.coach
    }

    pub fn get_game_id(&self) -> i64 {
        self.game_id
    }

    pub fn get_request_url(&self) -> &str {
        &self.request_url
    }

    pub fn set_request_url(&mut self, url: String) {
        self.request_url = url;
    }

    /// Builds the FUMBBL_AUTH_RESPONSE URL from `auth_url_template` (a `$1`/`$2`-style template,
    /// see [`super::util_fumbbl_request::UtilFumbblRequest::bind`]), fetches it, and parses the
    /// `<response>` tag: "OK ..." means the password was accepted, and any tokens after "OK" are
    /// returned as account properties.
    pub fn process(
        &mut self,
        client: &dyn super::util_fumbbl_request::HttpClient,
        auth_url_template: &str,
    ) -> Result<(bool, Vec<String>), String> {
        let mut password_ok = false;
        let mut account_properties = Vec::new();

        if !self.coach.is_empty() && !self.password.is_empty() {
            let url = super::util_fumbbl_request::UtilFumbblRequest::bind(
                auth_url_template,
                &[self.coach.as_str(), self.password.as_str()],
            );
            self.set_request_url(url);
            let response_xml = client.fetch_page(self.get_request_url())?;
            if !response_xml.is_empty() {
                if let Some(response) =
                    super::util_fumbbl_request::UtilFumbblRequest::extract_xml_tag_last(&response_xml, "response")
                {
                    password_ok = response.starts_with("OK");
                    let mut segments = response.split(' ');
                    segments.next(); // "OK"
                    account_properties = segments.map(|s| s.to_string()).collect();
                }
            }
        }
        Ok((password_ok, account_properties))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::fumbbl::util_fumbbl_request::MockHttpClient;

    #[test]
    fn construct() {
        let r = FumbblRequestCheckAuthorization::new(
            "coach".to_string(),
            "pass".to_string(),
            1,
            "game".to_string(),
            "team".to_string(),
        );
        assert_eq!(r.get_coach(), "coach");
    }

    #[test]
    fn process_builds_url_from_coach_and_password() {
        let client = MockHttpClient {
            response: Ok("<response>OK</response>".to_string()),
        };
        let mut r = FumbblRequestCheckAuthorization::new(
            "coach".to_string(),
            "pass".to_string(),
            1,
            "game".to_string(),
            "team".to_string(),
        );
        r.process(&client, "http://fumbbl/auth/$1/$2").unwrap();
        assert_eq!(r.get_request_url(), "http://fumbbl/auth/coach/pass");
    }

    #[test]
    fn process_ok_response_returns_password_ok() {
        let client = MockHttpClient {
            response: Ok("<response>OK premium</response>".to_string()),
        };
        let mut r = FumbblRequestCheckAuthorization::new(
            "coach".to_string(),
            "pass".to_string(),
            1,
            "game".to_string(),
            "team".to_string(),
        );
        let (ok, props) = r.process(&client, "http://fumbbl/auth/$1/$2").unwrap();
        assert!(ok);
        assert_eq!(props, vec!["premium".to_string()]);
    }

    #[test]
    fn process_wrong_password_response_is_not_ok() {
        let client = MockHttpClient {
            response: Ok("<response>WRONG</response>".to_string()),
        };
        let mut r = FumbblRequestCheckAuthorization::new(
            "coach".to_string(),
            "pass".to_string(),
            1,
            "game".to_string(),
            "team".to_string(),
        );
        let (ok, _) = r.process(&client, "http://fumbbl/auth/$1/$2").unwrap();
        assert!(!ok);
    }

    #[test]
    fn process_missing_coach_or_password_skips_request() {
        let client = MockHttpClient { response: Ok(String::new()) };
        let mut r = FumbblRequestCheckAuthorization::new(
            String::new(),
            "pass".to_string(),
            1,
            "game".to_string(),
            "team".to_string(),
        );
        let (ok, props) = r.process(&client, "http://fumbbl/auth/$1/$2").unwrap();
        assert!(!ok);
        assert!(props.is_empty());
        assert_eq!(r.get_request_url(), "");
    }
}
