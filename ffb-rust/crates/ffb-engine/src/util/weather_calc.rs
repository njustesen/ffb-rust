// 1:1 translation of com.fumbbl.ffb.server.util.WeatherCalc
//
// Pure weather table calculation.
// Mirrors Java DiceInterpreter.interpretWeather().

use ffb_model::enums::Weather;

pub struct WeatherCalc;

impl WeatherCalc {
    pub fn new() -> Self {
        Self
    }

    /// Map a 2D6 sum to the resulting weather.
    /// 2=SwelteringHeat, 3=VerySunny, 4–10=Nice, 11=PouringRain, 12=Blizzard.
    pub fn weather_for_roll(total: i32) -> Weather {
        match total {
            2 => Weather::SwelteringHeat,
            3 => Weather::VerySunny,
            11 => Weather::PouringRain,
            12 => Weather::Blizzard,
            _ => Weather::Nice,
        }
    }
}

impl Default for WeatherCalc {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roll_2_is_sweltering_heat() {
        assert_eq!(WeatherCalc::weather_for_roll(2), Weather::SwelteringHeat);
    }

    #[test]
    fn roll_3_is_very_sunny() {
        assert_eq!(WeatherCalc::weather_for_roll(3), Weather::VerySunny);
    }

    #[test]
    fn rolls_4_through_10_are_nice() {
        for total in 4..=10 {
            assert_eq!(
                WeatherCalc::weather_for_roll(total),
                Weather::Nice,
                "total={total}"
            );
        }
    }

    #[test]
    fn roll_11_is_pouring_rain() {
        assert_eq!(WeatherCalc::weather_for_roll(11), Weather::PouringRain);
    }

    #[test]
    fn roll_12_is_blizzard() {
        assert_eq!(WeatherCalc::weather_for_roll(12), Weather::Blizzard);
    }

    #[test]
    fn boundary_4_is_nice() {
        assert_eq!(WeatherCalc::weather_for_roll(4), Weather::Nice);
    }

    #[test]
    fn boundary_10_is_nice() {
        assert_eq!(WeatherCalc::weather_for_roll(10), Weather::Nice);
    }
}
