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

// Test mirror of com.fumbbl.ffb.server.util.WeatherCalcTest (1:1).
// The Java @ParameterizedTest CsvSource rows become one Rust test fn looping the same rows.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weather_for_roll() {
        let rows = [
            (2, Weather::SwelteringHeat),
            (3, Weather::VerySunny),
            (4, Weather::Nice),
            (5, Weather::Nice),
            (6, Weather::Nice),
            (7, Weather::Nice),
            (8, Weather::Nice),
            (9, Weather::Nice),
            (10, Weather::Nice),
            (11, Weather::PouringRain),
            (12, Weather::Blizzard),
        ];
        for (total, expected) in rows {
            assert_eq!(WeatherCalc::weather_for_roll(total), expected, "roll {total}");
        }
    }
}
