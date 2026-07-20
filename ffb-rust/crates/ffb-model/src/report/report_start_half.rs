use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportStartHalf.java`.
#[derive(Debug, Clone)]
pub struct ReportStartHalf {
    pub half: i32,
}

impl ReportStartHalf {
    pub fn new(half: i32) -> Self {
        Self { half }
    }

    pub fn get_half(&self) -> i32 { self.half }
}

impl IReport for ReportStartHalf {
    fn get_id(&self) -> ReportId { ReportId::START_HALF }
}

impl ReportStartHalf {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "half": self.half,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            half: json["half"].as_i64().unwrap_or(0) as i32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportStartHalf {
        ReportStartHalf::new(1)
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportStartHalf::from_json(&json);
        assert_eq!(restored.half, original.half);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("startHalf"));
    }
}
