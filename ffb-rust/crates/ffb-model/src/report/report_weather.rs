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
}
