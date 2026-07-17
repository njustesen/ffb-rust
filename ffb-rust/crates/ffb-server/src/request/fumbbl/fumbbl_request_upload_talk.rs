/// 1:1 translation of com.fumbbl.ffb.server.request.fumbbl.FumbblRequestUploadTalk.
/// POSTs talk JSON to FUMBBL_TALK URL.
pub struct FumbblRequestUploadTalk {
    request_url: String,
}

impl FumbblRequestUploadTalk {
    pub fn new() -> Self {
        Self { request_url: String::new() }
    }

    pub fn get_request_url(&self) -> &str {
        &self.request_url
    }

    pub fn set_request_url(&mut self, url: String) {
        self.request_url = url;
    }

    /// Sets the FUMBBL_TALK request url and POSTs `chat_json` to it via
    /// `postAuthorizedForm(url, challengeResponse, "chat", chatJson)`. Java swallows any error
    /// by logging it; this mirrors that by never returning `Err`.
    pub fn process(
        &mut self,
        client: &dyn super::util_fumbbl_request::HttpClient,
        talk_url: &str,
        challenge_response: &str,
        chat_json: &str,
    ) -> String {
        self.set_request_url(talk_url.to_string());
        match client.post_authorized_form(self.get_request_url(), challenge_response, "chat", chat_json) {
            Ok(response) => response,
            Err(err) => err,
        }
    }
}

impl Default for FumbblRequestUploadTalk {
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
        let _ = FumbblRequestUploadTalk::new();
    }

    #[test]
    fn process_sets_request_url_and_returns_response() {
        let client = MockHttpClient { response: Ok("ok".to_string()) };
        let mut r = FumbblRequestUploadTalk::new();
        let response = r.process(&client, "http://fumbbl/talk", "chal", "{\"msg\":\"hi\"}");
        assert_eq!(r.get_request_url(), "http://fumbbl/talk");
        assert_eq!(response, "ok");
    }

    #[test]
    fn process_returns_error_message_on_failure() {
        let client = MockHttpClient { response: Err("boom".to_string()) };
        let mut r = FumbblRequestUploadTalk::new();
        let response = r.process(&client, "http://fumbbl/talk", "chal", "{}");
        assert_eq!(response, "boom");
    }
}
