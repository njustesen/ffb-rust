use crate::enums::Weather;

/// 1:1 translation of com.fumbbl.ffb.factory.WeatherFactory.
pub struct WeatherFactory;

impl Default for WeatherFactory {
    fn default() -> Self { Self }
}

impl WeatherFactory {
    pub fn for_name(&self, name: &str) -> Option<Weather> {
        Weather::from_name(name)
    }

    pub fn for_short_name(&self, short_name: &str) -> Option<Weather> {
        Weather::all().iter().copied().find(|w| w.short_name().eq_ignore_ascii_case(short_name))
    }

    pub fn initialize(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_nice_weather() {
        assert!(WeatherFactory.for_name("Nice Weather").is_some());
    }

    #[test]
    fn for_name_unknown() {
        assert_eq!(WeatherFactory.for_name("blizzard"), None);
    }

    #[test]
    fn for_short_name_nice_weather() {
        let result = WeatherFactory.for_short_name("Nice");
        assert!(result.is_some());
    }

    #[test]
    fn for_short_name_unknown() {
        assert_eq!(WeatherFactory.for_short_name("XXXX"), None);
    }
    #[test]
    fn for_name_empty_string_returns_none() {
        assert!(WeatherFactory.for_name("").is_none());
    }
}
