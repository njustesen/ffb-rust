/// 1:1 translation of com.fumbbl.ffb.server.request.fumbbl.FumbblRequestUploadResults.
/// Builds FumbblResult XML, POSTs multipart to FUMBBL_RESULT; on success saves replay + removes gamestate.
pub struct FumbblRequestUploadResults {
    pub upload_successful: bool,
    pub upload_status: String,
    request_url: String,
}

impl FumbblRequestUploadResults {
    pub fn new() -> Self {
        Self {
            upload_successful: false,
            upload_status: String::new(),
            request_url: String::new(),
        }
    }

    pub fn is_upload_successful(&self) -> bool {
        self.upload_successful
    }

    pub fn get_upload_status(&self) -> &str {
        &self.upload_status
    }

    pub fn get_request_url(&self) -> &str {
        &self.request_url
    }

    pub fn set_request_url(&mut self, url: String) {
        self.request_url = url;
    }

    /// Builds the FumbblResult XML and POSTs it via `postMultipartXml(url, challengeResponse,
    /// resultXml)` to `result_url`, then parses the `<result>`/`<description>` tags from the
    /// response to determine success.
    pub fn process(
        &mut self,
        client: &dyn super::util_fumbbl_request::HttpClient,
        result_url: &str,
        challenge_response: &str,
    ) -> Result<(), String> {
        let result_xml = super::fumbbl_result::FumbblResult::new().to_xml();
        self.set_request_url(result_url.to_string());
        let response_xml = client.post_multipart_xml(self.get_request_url(), challenge_response, &result_xml)?;
        if !response_xml.is_empty() {
            if let Some(result) = super::util_fumbbl_request::UtilFumbblRequest::extract_xml_tag_first(&response_xml, "result") {
                self.upload_successful = result.eq_ignore_ascii_case("success");
            }
            if let Some(description) =
                super::util_fumbbl_request::UtilFumbblRequest::extract_xml_tag_first(&response_xml, "description")
            {
                self.upload_status = description;
            }
        }
        Ok(())
    }
}

impl Default for FumbblRequestUploadResults {
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
        let r = FumbblRequestUploadResults::new();
        assert!(!r.is_upload_successful());
    }

    #[test]
    fn process_success_response_sets_flags() {
        let client = MockHttpClient {
            response: Ok("<response><result>success</result><description>ok</description></response>".to_string()),
        };
        let mut r = FumbblRequestUploadResults::new();
        r.process(&client, "http://fumbbl/result", "chal").unwrap();
        assert!(r.is_upload_successful());
        assert_eq!(r.get_upload_status(), "ok");
        assert_eq!(r.get_request_url(), "http://fumbbl/result");
    }

    #[test]
    fn process_failure_response_is_not_successful() {
        let client = MockHttpClient {
            response: Ok("<response><result>failure</result><description>bad teamvalue</description></response>".to_string()),
        };
        let mut r = FumbblRequestUploadResults::new();
        r.process(&client, "http://fumbbl/result", "chal").unwrap();
        assert!(!r.is_upload_successful());
        assert_eq!(r.get_upload_status(), "bad teamvalue");
    }
}
