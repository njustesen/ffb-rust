use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportWeatherMageRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportWeatherMageRoll {
    /// `fWeatherRoll` — the dice rolls.
    pub weather_roll: Vec<i32>,
}

impl ReportWeatherMageRoll {
    pub fn new(weather_roll: Vec<i32>) -> Self {
        Self { weather_roll }
    }

    pub fn get_weather_roll(&self) -> &[i32] { &self.weather_roll }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "weatherRoll": self.weather_roll,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            weather_roll: json["weatherRoll"].as_array()
                .map(|a| a.iter().map(|v| v.as_i64().unwrap_or(0) as i32).collect())
                .unwrap_or_default(),
        }
    }
}

impl IReport for ReportWeatherMageRoll {
    fn get_id(&self) -> ReportId { ReportId::WEATHER_MAGE_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportWeatherMageRoll {
        ReportWeatherMageRoll::new(vec![3, 4])
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportWeatherMageRoll::from_json(&json);
        assert_eq!(restored.weather_roll, original.weather_roll);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("weatherMageRoll"));
    }
}
