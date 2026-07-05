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
    fn get_id() { assert_eq!(make().get_id(), ReportId::WEATHER_MAGE_ROLL); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "weatherMageRoll"); }

    #[test]
    fn get_weather_roll() { assert_eq!(make().get_weather_roll(), &[3, 4]); }
}
