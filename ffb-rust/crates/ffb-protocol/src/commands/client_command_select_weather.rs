/// 1:1 translation of ClientCommandSelectWeather (Java fields: modifier, weatherName).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandSelectWeather {
    pub modifier: i32,
    pub weather_name: Option<String>,
}

impl ClientCommandSelectWeather {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_fields(modifier: i32, weather_name: impl Into<String>) -> Self {
        Self {
            modifier,
            weather_name: Some(weather_name.into()),
        }
    }

    pub fn get_modifier(&self) -> i32 {
        self.modifier
    }

    pub fn get_weather_name(&self) -> Option<&str> {
        self.weather_name.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_zero_modifier_and_no_name() {
        let cmd = ClientCommandSelectWeather::new();
        assert_eq!(cmd.get_modifier(), 0);
        assert!(cmd.get_weather_name().is_none());
    }

    #[test]
    fn with_fields_stores_values() {
        let cmd = ClientCommandSelectWeather::with_fields(2, "Nice");
        assert_eq!(cmd.get_modifier(), 2);
        assert_eq!(cmd.get_weather_name(), Some("Nice"));
    }
}
