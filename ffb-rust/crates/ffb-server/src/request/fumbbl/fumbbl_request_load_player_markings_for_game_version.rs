/// 1:1 translation of com.fumbbl.ffb.server.request.fumbbl.FumbblRequestLoadPlayerMarkingsForGameVersion.
/// Loads player markings for a specific replay version; checks DB for sort mode preference.
pub struct FumbblRequestLoadPlayerMarkingsForGameVersion {
    pub index: i32,
    pub coach: String,
    request_url: String,
}

impl FumbblRequestLoadPlayerMarkingsForGameVersion {
    pub fn new(index: i32, coach: String) -> Self {
        Self { index, coach, request_url: String::new() }
    }

    pub fn get_index(&self) -> i32 {
        self.index
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

    /// Java: `FumbblRequestLoadPlayerMarkingsForGameVersion.process(ServerRequestProcessor)`.
    ///
    /// If `existing_config` is `Some` (Java: `sessionManager.getAutoMarking(session)` returned a
    /// cached `AutoMarkingConfig`), it is reused; otherwise the config is loaded from
    /// FUMBBL_PLAYER_MARKINGS for [`Self::coach`] and parsed via the shared
    /// [`parse_markings_response`](super::abstract_fumbbl_request_load_player_markings::parse_markings_response).
    /// Java's `updateSearchMode` (a DB lookup of the coach's marking-sort preference) and the
    /// final `InternalServerCommandCalculateAutomaticPlayerMarkings` dispatch back through
    /// `server.getCommunication().handleCommand(...)` are done downstream; that command is now
    /// dispatched for real from the `ServerCommandHandlerFactory` once built, but this request
    /// type's only caller (`ServerCommandHandlerLoadAutomaticPlayerMarkings`) still discards the
    /// parsed result — there is no `ServerRequestProcessor`→dispatch channel for it yet.
    pub fn process(
        &mut self,
        client: &dyn super::util_fumbbl_request::HttpClient,
        markings_url_template: &str,
        existing_config: Option<ffb_engine::marking::auto_marking_config::AutoMarkingConfig>,
    ) -> Result<Option<ffb_engine::marking::auto_marking_config::AutoMarkingConfig>, String> {
        if let Some(config) = existing_config {
            return Ok(Some(config));
        }
        let url = super::util_fumbbl_request::UtilFumbblRequest::bind(markings_url_template, &[self.coach.as_str()]);
        self.set_request_url(url);
        let response = client.fetch_page(self.get_request_url())?;
        Ok(super::abstract_fumbbl_request_load_player_markings::parse_markings_response(&response))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::fumbbl::util_fumbbl_request::MockHttpClient;

    #[test]
    fn construct() {
        let r = FumbblRequestLoadPlayerMarkingsForGameVersion::new(3, "coach".to_string());
        assert_eq!(r.get_index(), 3);
        assert_eq!(r.get_coach(), "coach");
    }

    #[test]
    fn process_fetches_and_parses_config_when_none_cached() {
        let client = MockHttpClient {
            response: Ok(r#"{"autoMarkingSeparator":"/","autoMarkingRecords":[{"skillArray":["Tackle"],"marking":"T"}]}"#.to_string()),
        };
        let mut r = FumbblRequestLoadPlayerMarkingsForGameVersion::new(3, "coach".to_string());
        let config = r.process(&client, "http://fumbbl/markings/$1", None).unwrap().expect("config should parse");
        assert_eq!(r.get_request_url(), "http://fumbbl/markings/coach");
        assert_eq!(config.get_separator(), "/");
        assert_eq!(config.get_markings().len(), 1);
        assert_eq!(config.get_markings()[0].marking(), "T");
    }

    #[test]
    fn process_reuses_cached_config_without_fetching() {
        let client = MockHttpClient { response: Err("should not be called".to_string()) };
        let mut r = FumbblRequestLoadPlayerMarkingsForGameVersion::new(3, "coach".to_string());
        let mut cached = ffb_engine::marking::auto_marking_config::AutoMarkingConfig::new();
        cached.set_separator("cached-sep");
        let config = r
            .process(&client, "http://fumbbl/markings/$1", Some(cached))
            .unwrap()
            .expect("cached config is returned");
        assert_eq!(config.get_separator(), "cached-sep");
        assert_eq!(r.get_request_url(), "");
    }
}
