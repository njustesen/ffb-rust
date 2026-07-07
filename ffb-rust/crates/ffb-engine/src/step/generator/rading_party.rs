/// Root-level abstract base for the RadingParty step sequence generator.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.RadingParty`.

#[derive(Debug, Clone, Default)]
pub struct RadingPartyParams {
    pub failure_label: Option<String>,
    pub success_label: Option<String>,
}

pub struct RadingParty;

impl RadingParty {
    pub fn new() -> Self { Self }
}

impl Default for RadingParty {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rading_party_params_default_no_labels() {
        let p = RadingPartyParams::default();
        assert!(p.failure_label.is_none());
        assert!(p.success_label.is_none());
    }

    #[test]
    fn rading_party_params_can_set_labels() {
        let p = RadingPartyParams {
            failure_label: Some("fail".to_string()),
            success_label: Some("ok".to_string()),
        };
        assert_eq!(p.failure_label.as_deref(), Some("fail"));
        assert_eq!(p.success_label.as_deref(), Some("ok"));
    }

    #[test]
    fn rading_party_struct_is_default() {
        let _ = RadingParty::default();
    }

    #[test]
    fn params_with_fields_set() {
        let p = RadingPartyParams {
            failure_label: Some("fail".into()),
            success_label: Some("ok".into()),
        };
        assert_eq!(p.failure_label.as_deref(), Some("fail"));
        assert_eq!(p.success_label.as_deref(), Some("ok"));
    }

    #[test]
    fn params_clone() {
        let p = RadingPartyParams { failure_label: Some("x".into()), success_label: None };
        let q = p.clone();
        assert_eq!(q.failure_label.as_deref(), Some("x"));
    }
}
