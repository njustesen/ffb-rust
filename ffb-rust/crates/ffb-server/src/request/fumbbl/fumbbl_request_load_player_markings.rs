/// 1:1 translation of com.fumbbl.ffb.server.request.fumbbl.FumbblRequestLoadPlayerMarkings.
pub struct FumbblRequestLoadPlayerMarkings {
    request_url: String,
}

impl FumbblRequestLoadPlayerMarkings {
    pub fn new() -> Self {
        Self { request_url: String::new() }
    }

    pub fn get_request_url(&self) -> &str {
        &self.request_url
    }

    pub fn set_request_url(&mut self, url: String) {
        self.request_url = url;
    }

    /// Loads the auto-marking config JSON for `coach` (see
    /// `AbstractFumbblRequestLoadPlayerMarkings::load_automarking_config`). Java then adds it to
    /// the session's marking config, applies `sort_mode`, and dispatches
    /// `InternalServerCommandApplyAutomatedPlayerMarkings`; that session/command plumbing does
    /// not exist yet in this simplified server crate.
    pub fn process(
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

impl Default for FumbblRequestLoadPlayerMarkings {
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
        let _ = FumbblRequestLoadPlayerMarkings::new();
    }

    #[test]
    fn process_builds_url_and_returns_config_json() {
        let client = MockHttpClient {
            response: Ok("{\"markings\":[1,2]}".to_string()),
        };
        let mut r = FumbblRequestLoadPlayerMarkings::new();
        let json = r.process(&client, "http://fumbbl/markings/$1", "coach").unwrap();
        assert_eq!(r.get_request_url(), "http://fumbbl/markings/coach");
        assert_eq!(json, Some("{\"markings\":[1,2]}".to_string()));
    }
}
