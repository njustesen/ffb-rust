/// HTTP client utilities for FUMBBL API calls — 1:1 translation of Java
/// `com.fumbbl.ffb.server.util.UtilServerHttpClient`.
///
/// **Genuinely still a stub (Phase AAC correction).** `TRANSLATION_TRACKER.md` previously
/// marked this file `✓`, which was wrong — none of the three methods below (`get`/`post`/
/// `post_form`, which don't even match Java's real method names: `fetchPage`/`loadFile`/
/// `postMultipartXml`/`postAuthorizedForm`/`post`) do anything but `todo!()`, and — per this
/// crate's own architecture doc (`CLAUDE.md`'s "Engine output channels" section) —
/// `ffb-engine` is deliberately networking-free: "The Rust engine uses two output channels
/// instead of Java's direct networking calls" (`GameEvent`/`AgentPrompt`), with real HTTP
/// calls living one layer up in `ffb-server` (see `ffb-server/src/request/fumbbl/
/// util_fumbbl_request.rs`'s `HttpClient` trait + `ReqwestHttpClient`, which already is the
/// real, tested implementation of this same Java class's concern). `ffb-engine` has no
/// `reqwest` dependency and no caller anywhere in this crate actually invokes `get`/`post`/
/// `post_form` (only a doc-comment mention in `step_riotous_rookies.rs`'s no-op
/// `rookieName()` fallback) — so there is nothing to wire these into from this layer, and
/// duplicating `ReqwestHttpClient` here would just be a second, divergent implementation of
/// the same HTTP concern. Left as an honest `todo!()` stub; do not mark this file `✓` again
/// unless `ffb-engine` grows a real caller that needs it.
pub struct UtilServerHttpClient;

impl UtilServerHttpClient {
    pub fn get(url: &str) -> Result<String, String> {
        let _ = url;
        todo!("no caller in this crate; real HTTP lives in ffb-server's ReqwestHttpClient")
    }

    pub fn post(url: &str, body: &str) -> Result<String, String> {
        let _ = (url, body);
        todo!("no caller in this crate; real HTTP lives in ffb-server's ReqwestHttpClient")
    }

    pub fn post_form(url: &str, params: &[(&str, &str)]) -> Result<String, String> {
        let _ = (url, params);
        todo!("no caller in this crate; real HTTP lives in ffb-server's ReqwestHttpClient")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_exists() {
        let _name = std::any::type_name::<UtilServerHttpClient>();
        assert!(_name.contains("UtilServerHttpClient"));
    }

    #[test]
    fn test_struct_is_unit() {
        // UtilServerHttpClient is a utility struct with only static methods
        let _size = std::mem::size_of::<UtilServerHttpClient>();
        assert_eq!(_size, 0);
    }
}
