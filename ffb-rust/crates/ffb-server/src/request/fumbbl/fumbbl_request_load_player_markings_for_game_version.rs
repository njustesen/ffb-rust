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

    /// If `existing_config_json` is `None` (no cached `AutoMarkingConfig` for this session yet),
    /// loads it from FUMBBL_PLAYER_MARKINGS for [`Self::coach`]; otherwise reuses it, mirroring
    /// Java's `sessionManager.getAutoMarking(session)` cache check. Java's `updateSearchMode`
    /// (a DB lookup of the coach's marking-sort preference) and the final
    /// `InternalServerCommandCalculateAutomaticPlayerMarkings` dispatch have no equivalent yet
    /// in this simplified server crate.
    pub fn process(
        &mut self,
        client: &dyn super::util_fumbbl_request::HttpClient,
        markings_url_template: &str,
        existing_config_json: Option<String>,
    ) -> Result<Option<String>, String> {
        if let Some(config) = existing_config_json {
            return Ok(Some(config));
        }
        let url = super::util_fumbbl_request::UtilFumbblRequest::bind(markings_url_template, &[self.coach.as_str()]);
        self.set_request_url(url);
        let response = client.fetch_page(self.get_request_url())?;
        if response.is_empty() || response == "null" {
            return Ok(None);
        }
        Ok(Some(response))
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
    fn process_fetches_config_when_none_cached() {
        let client = MockHttpClient {
            response: Ok("{\"markings\":[]}".to_string()),
        };
        let mut r = FumbblRequestLoadPlayerMarkingsForGameVersion::new(3, "coach".to_string());
        let json = r.process(&client, "http://fumbbl/markings/$1", None).unwrap();
        assert_eq!(r.get_request_url(), "http://fumbbl/markings/coach");
        assert_eq!(json, Some("{\"markings\":[]}".to_string()));
    }

    #[test]
    fn process_reuses_cached_config_without_fetching() {
        let client = MockHttpClient { response: Err("should not be called".to_string()) };
        let mut r = FumbblRequestLoadPlayerMarkingsForGameVersion::new(3, "coach".to_string());
        let json = r
            .process(&client, "http://fumbbl/markings/$1", Some("cached".to_string()))
            .unwrap();
        assert_eq!(json, Some("cached".to_string()));
        assert_eq!(r.get_request_url(), "");
    }
}
