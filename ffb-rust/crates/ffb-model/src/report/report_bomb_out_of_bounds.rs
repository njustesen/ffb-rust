use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportBombOutOfBounds.java`.
/// No fields — the report carries only its identity.
#[derive(Debug, Clone, Default)]
pub struct ReportBombOutOfBounds;

impl ReportBombOutOfBounds {
    pub fn new() -> Self { Self }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
        })
    }

    pub fn from_json(_json: &serde_json::Value) -> Self {
        Self
    }
}

impl IReport for ReportBombOutOfBounds {
    fn get_id(&self) -> ReportId { ReportId::BOMB_OUT_OF_BOUNDS }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_id() {
        assert_eq!(ReportBombOutOfBounds::new().get_id(), ReportId::BOMB_OUT_OF_BOUNDS);
    }

    #[test]
    fn get_name() {
        assert_eq!(ReportBombOutOfBounds::new().get_name(), "bombOutOfBounds");
    }

    #[test]
    fn default_works() {
        let r = ReportBombOutOfBounds::default();
        assert_eq!(r.get_id(), ReportId::BOMB_OUT_OF_BOUNDS);
    }

    #[test]
    fn new_and_default_same_name() {
        assert_eq!(ReportBombOutOfBounds::new().get_name(), ReportBombOutOfBounds::default().get_name());
    }

    #[test]
    fn clone_preserves_id() {
        let r = ReportBombOutOfBounds::new();
        let c = r.clone();
        assert_eq!(c.get_id(), ReportId::BOMB_OUT_OF_BOUNDS);
    }

    #[test]
    fn serialization_round_trip() {
        let original = ReportBombOutOfBounds::new();
        let json = original.to_json_value();
        let restored = ReportBombOutOfBounds::from_json(&json);
        assert_eq!(restored.get_id(), original.get_id());
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = ReportBombOutOfBounds::new().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("bombOutOfBounds"));
    }
}
