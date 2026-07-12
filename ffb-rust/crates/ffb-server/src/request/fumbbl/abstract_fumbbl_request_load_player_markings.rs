/// 1:1 translation of com.fumbbl.ffb.server.request.fumbbl.AbstractFumbblRequestLoadPlayerMarkings.
use ffb_engine::marking::auto_marking_config::AutoMarkingConfig;

/// Java: the body of `loadAutomarkingConfig` after `UtilServerHttpClient.fetchPage`:
/// `JsonValue jsonValue = JsonValue.readFrom(response); if (jsonValue != null && !jsonValue.isNull())
/// config.initFrom(rules, jsonValue);`.
///
/// Returns `None` when the response is empty or the literal `"null"` (this crate's signal that
/// no config was returned — Java would hand back an empty config in that case). On a malformed
/// (non-JSON) body Java catches the exception and returns the empty config; that is mirrored by
/// returning `Some(AutoMarkingConfig::new())` after logging.
pub(crate) fn parse_markings_response(response: &str) -> Option<AutoMarkingConfig> {
    if response.is_empty() || response == "null" {
        return None;
    }
    match serde_json::from_str::<serde_json::Value>(response) {
        Ok(value) if !value.is_null() => Some(AutoMarkingConfig::from_json(&value)),
        Ok(_) => None,
        Err(e) => {
            // Java: `catch (Exception e) { source.logError(0, "Could not init auto marking config: " + e.getMessage()); }`
            log::error!("Could not init auto marking config: {e}");
            Some(AutoMarkingConfig::new())
        }
    }
}

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

    /// Java: `loadAutomarkingConfig(FantasyFootballServer, String coach, long id, GameRules)`.
    ///
    /// Builds the FUMBBL_PLAYER_MARKINGS URL for `coach`, fetches it, and parses the JSON body
    /// into an `AutoMarkingConfig` (Java: `config.initFrom(rules, jsonValue)`). Returns `None`
    /// when the response is empty/`"null"` (see [`parse_markings_response`]). The `rules`/`id`
    /// parameters Java threads through are only used for the `SkillFactory`/error logging, both
    /// of which are handled inside `AutoMarkingConfig::from_json`, so they are not needed here.
    pub fn load_automarking_config(
        &mut self,
        client: &dyn super::util_fumbbl_request::HttpClient,
        markings_url_template: &str,
        coach: &str,
    ) -> Result<Option<AutoMarkingConfig>, String> {
        let url = super::util_fumbbl_request::UtilFumbblRequest::bind(markings_url_template, &[coach]);
        self.set_request_url(url);
        let response = client.fetch_page(self.get_request_url())?;
        Ok(parse_markings_response(&response))
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
    fn load_automarking_config_builds_url_and_parses_config() {
        let client = MockHttpClient {
            response: Ok(r#"{"autoMarkingSeparator":"/","autoMarkingRecords":[{"skillArray":["Block"],"marking":"B"}]}"#.to_string()),
        };
        let mut r = AbstractFumbblRequestLoadPlayerMarkings::new();
        let config = r
            .load_automarking_config(&client, "http://fumbbl/markings/$1", "coach")
            .unwrap()
            .expect("config should parse");
        assert_eq!(r.get_request_url(), "http://fumbbl/markings/coach");
        assert_eq!(config.get_separator(), "/");
        assert_eq!(config.get_markings().len(), 1);
        assert_eq!(config.get_markings()[0].marking(), "B");
    }

    #[test]
    fn load_automarking_config_null_response_returns_none() {
        let client = MockHttpClient { response: Ok("null".to_string()) };
        let mut r = AbstractFumbblRequestLoadPlayerMarkings::new();
        let config = r
            .load_automarking_config(&client, "http://fumbbl/markings/$1", "coach")
            .unwrap();
        assert!(config.is_none());
    }

    #[test]
    fn parse_markings_response_empty_is_none() {
        assert!(parse_markings_response("").is_none());
        assert!(parse_markings_response("null").is_none());
    }

    #[test]
    fn parse_markings_response_malformed_json_yields_empty_config() {
        let config = parse_markings_response("{not valid json").expect("Java returns empty config on parse error");
        assert!(config.get_markings().is_empty());
    }
}
