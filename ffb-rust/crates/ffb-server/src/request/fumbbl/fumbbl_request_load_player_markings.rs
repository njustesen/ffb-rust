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

    /// Java: `FumbblRequestLoadPlayerMarkings.process(ServerRequestProcessor)` — the part that
    /// loads the auto-marking config. Fetches the FUMBBL_PLAYER_MARKINGS URL for `coach` and
    /// parses the JSON body into an `AutoMarkingConfig` via the shared
    /// [`parse_markings_response`](super::abstract_fumbbl_request_load_player_markings::parse_markings_response)
    /// (Java: `loadAutomarkingConfig` on the base class).
    ///
    /// Java then adds the config to the session's marking config, applies `sort_mode`, and
    /// dispatches `InternalServerCommandApplyAutomatedPlayerMarkings` back through
    /// `server.getCommunication().handleCommand(...)`. In this crate that dispatch happens from
    /// the `ServerCommandHandlerFactory`'s `ApplyAutomatedPlayerMarkings` arm once the command
    /// is built; there is no `ServerRequestProcessor`→dispatch channel wiring for this request
    /// type yet (its only caller, `MarkerLoadingService`, discards the parsed result — a
    /// documented gap), so this method just returns the parsed config for that caller to use.
    pub fn process(
        &mut self,
        client: &dyn super::util_fumbbl_request::HttpClient,
        markings_url_template: &str,
        coach: &str,
    ) -> Result<Option<ffb_engine::marking::auto_marking_config::AutoMarkingConfig>, String> {
        let url = super::util_fumbbl_request::UtilFumbblRequest::bind(markings_url_template, &[coach]);
        self.set_request_url(url);
        let response = client.fetch_page(self.get_request_url())?;
        Ok(super::abstract_fumbbl_request_load_player_markings::parse_markings_response(&response))
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
    fn process_builds_url_and_parses_config() {
        let client = MockHttpClient {
            response: Ok(r#"{"autoMarkingSeparator":"-","autoMarkingRecords":[{"skillArray":["Block"],"marking":"B","gainedOnly":true}]}"#.to_string()),
        };
        let mut r = FumbblRequestLoadPlayerMarkings::new();
        let config = r.process(&client, "http://fumbbl/markings/$1", "coach").unwrap().expect("config should parse");
        assert_eq!(r.get_request_url(), "http://fumbbl/markings/coach");
        assert_eq!(config.get_separator(), "-");
        assert_eq!(config.get_markings().len(), 1);
        assert_eq!(config.get_markings()[0].marking(), "B");
        assert!(config.get_markings()[0].is_gained_only());
    }

    #[test]
    fn process_null_response_returns_none() {
        let client = MockHttpClient { response: Ok("null".to_string()) };
        let mut r = FumbblRequestLoadPlayerMarkings::new();
        assert!(r.process(&client, "http://fumbbl/markings/$1", "coach").unwrap().is_none());
    }
}
