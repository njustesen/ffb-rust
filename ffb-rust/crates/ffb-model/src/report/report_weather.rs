use crate::enums::Weather;
use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportWeather.java`.
#[derive(Debug, Clone)]
pub struct ReportWeather {
    pub weather: Weather,
    pub weather_roll: Vec<i32>,
}

impl ReportWeather {
    pub fn new(weather: Weather, weather_roll: Vec<i32>) -> Self {
        Self { weather, weather_roll }
    }

    pub fn get_weather(&self) -> Weather { self.weather }
    pub fn get_weather_roll(&self) -> &[i32] { &self.weather_roll }
}

impl IReport for ReportWeather {
    fn get_id(&self) -> ReportId { ReportId::WEATHER }
}

impl ReportWeather {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "weather": self.weather.name(),
            "weatherRoll": self.weather_roll,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            weather: json["weather"].as_str()
                .and_then(Weather::from_name)
                .unwrap_or(Weather::Nice),
            weather_roll: json["weatherRoll"].as_array()
                .map(|a| a.iter().map(|v| v.as_i64().unwrap_or(0) as i32).collect())
                .unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportWeather {
        ReportWeather::new(Weather::Nice, vec![3, 4])
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::WEATHER);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "weather");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_weather(), Weather::Nice);
        assert_eq!(r.get_weather_roll(), &[3, 4]);
    }

    #[test]
    fn single_roll() {
        let r = ReportWeather::new(Weather::Nice, vec![5]);
        assert_eq!(r.get_weather_roll(), &[5]);
    }

    #[test]
    fn empty_roll() {
        let r = ReportWeather::new(Weather::Nice, vec![]);
        assert_eq!(r.get_weather_roll().len(), 0);
        assert_eq!(r.get_weather(), Weather::Nice);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportWeather::from_json(&json);
        assert_eq!(restored.weather, original.weather);
        assert_eq!(restored.weather_roll, original.weather_roll);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("weather"));
    }
}
