use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// Nested enum from `ReportWeatherMageResult.Effect`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum WeatherMageEffect {
    CHANGED,
    NO_CHANGE,
    NO_CHOICE,
}

impl WeatherMageEffect {
    pub fn get_name(self) -> &'static str {
        match self {
            WeatherMageEffect::CHANGED => "CHANGED",
            WeatherMageEffect::NO_CHANGE => "NO_CHANGE",
            WeatherMageEffect::NO_CHOICE => "NO_CHOICE",
        }
    }

    pub fn for_name(s: &str) -> Option<Self> {
        match s {
            "CHANGED" => Some(WeatherMageEffect::CHANGED),
            "NO_CHANGE" => Some(WeatherMageEffect::NO_CHANGE),
            "NO_CHOICE" => Some(WeatherMageEffect::NO_CHOICE),
            _ => None,
        }
    }
}

/// 1:1 translation of `ReportWeatherMageResult.java`.
/// `Weather` type is represented as a name string.
#[derive(Debug, Clone)]
pub struct ReportWeatherMageResult {
    pub modifier: i32,
    /// `newWeather` — Weather name string.
    pub new_weather: Option<String>,
    /// `oldWeather` — Weather name string.
    pub old_weather: Option<String>,
    pub effect: Option<WeatherMageEffect>,
}

impl ReportWeatherMageResult {
    pub fn new(
        modifier: i32,
        new_weather: Option<String>,
        effect: Option<WeatherMageEffect>,
        old_weather: Option<String>,
    ) -> Self {
        Self { modifier, new_weather, old_weather, effect }
    }

    pub fn get_modifier(&self) -> i32 { self.modifier }
    pub fn get_new_weather(&self) -> Option<&str> { self.new_weather.as_deref() }
    pub fn get_old_weather(&self) -> Option<&str> { self.old_weather.as_deref() }
    pub fn get_effect(&self) -> Option<WeatherMageEffect> { self.effect }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "modifier": self.modifier,
            "weather": self.new_weather,
            "oldWeather": self.old_weather,
            "effect": self.effect.map(|e| e.get_name()),
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            modifier: json["modifier"].as_i64().unwrap_or(0) as i32,
            new_weather: json["weather"].as_str().map(str::to_string),
            old_weather: json["oldWeather"].as_str().map(str::to_string),
            effect: json["effect"].as_str().and_then(WeatherMageEffect::for_name),
        }
    }
}

impl IReport for ReportWeatherMageResult {
    fn get_id(&self) -> ReportId { ReportId::WEATHER_MAGE_RESULT }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportWeatherMageResult {
        ReportWeatherMageResult::new(
            1,
            Some("NICE".into()),
            Some(WeatherMageEffect::CHANGED),
            Some("BLIZZARD".into()),
        )
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::WEATHER_MAGE_RESULT); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "weatherMageResult"); }

    #[test]
    fn get_modifier() { assert_eq!(make().get_modifier(), 1); }

    #[test]
    fn get_new_weather() { assert_eq!(make().get_new_weather(), Some("NICE")); }

    #[test]
    fn get_effect() { assert_eq!(make().get_effect(), Some(WeatherMageEffect::CHANGED)); }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportWeatherMageResult::from_json(&json);
        assert_eq!(restored.modifier, original.modifier);
        assert_eq!(restored.new_weather, original.new_weather);
        assert_eq!(restored.old_weather, original.old_weather);
        assert_eq!(restored.effect, original.effect);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("weatherMageResult"));
    }
}
