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
}
