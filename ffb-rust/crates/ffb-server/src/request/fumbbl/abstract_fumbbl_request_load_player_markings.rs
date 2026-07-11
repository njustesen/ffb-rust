/// 1:1 translation of com.fumbbl.ffb.server.request.fumbbl.AbstractFumbblRequestLoadPlayerMarkings.
pub struct AbstractFumbblRequestLoadPlayerMarkings {
    request_url: String,
}

impl AbstractFumbblRequestLoadPlayerMarkings {
    pub fn new() -> Self {
        Self { request_url: String::new() }
    }

    pub fn get_request_url(&self) -> &str {
        &self.request_url
    }

    pub fn set_request_url(&mut self, url: String) {
        self.request_url = url;
    }

    /// Fetches the FUMBBL_PLAYER_MARKINGS URL for `coach` and returns the raw JSON response.
    ///
    /// Java parses the JSON into an `AutoMarkingConfig` via `JsonValue.readFrom` +
    /// `config.initFrom(rules, jsonValue)`; that config type does not exist in this simplified
    /// server crate yet, so this returns the raw JSON text for the caller to parse once that
    /// model lands.
    pub fn load_automarking_config(
        &mut self,
        client: &dyn super::util_fumbbl_request::HttpClient,
        markings_url_template: &str,
        coach: &str,
    ) -> Result<Option<String>, String> {
        let url = super::util_fumbbl_request::UtilFumbblRequest::bind(markings_url_template, &[coach]);
        self.set_request_url(url);
        let response = client.fetch_page(self.get_request_url())?;
        if response.is_empty() || response == "null" {
            return Ok(None);
        }
        Ok(Some(response))
    }
}

impl Default for AbstractFumbblRequestLoadPlayerMarkings {
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
        let r = AbstractFumbblRequestLoadPlayerMarkings::new();
        assert_eq!(r.get_request_url(), "");
    }

    #[test]
    fn load_automarking_config_builds_url_and_returns_json() {
        let client = MockHttpClient {
            response: Ok("{\"markings\":[]}".to_string()),
        };
        let mut r = AbstractFumbblRequestLoadPlayerMarkings::new();
        let json = r
            .load_automarking_config(&client, "http://fumbbl/markings/$1", "coach")
            .unwrap();
        assert_eq!(r.get_request_url(), "http://fumbbl/markings/coach");
        assert_eq!(json, Some("{\"markings\":[]}".to_string()));
    }

    #[test]
    fn load_automarking_config_null_response_returns_none() {
        let client = MockHttpClient { response: Ok("null".to_string()) };
        let mut r = AbstractFumbblRequestLoadPlayerMarkings::new();
        let json = r
            .load_automarking_config(&client, "http://fumbbl/markings/$1", "coach")
            .unwrap();
        assert!(json.is_none());
    }
}
