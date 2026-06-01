use serde::{Deserialize, Serialize};

/// Current weather on the pitch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Weather {
    SwelteringHeat,
    VerySunny,
    Nice,
    PouringRain,
    Blizzard,
    Intro,
}

impl Weather {
    pub fn name(self) -> &'static str {
        match self {
            Weather::SwelteringHeat => "Sweltering Heat",
            Weather::VerySunny => "Very Sunny",
            Weather::Nice => "Nice Weather",
            Weather::PouringRain => "Pouring Rain",
            Weather::Blizzard => "Blizzard",
            Weather::Intro => "Intro",
        }
    }

    pub fn short_name(self) -> &'static str {
        match self {
            Weather::SwelteringHeat => "heat",
            Weather::VerySunny => "sunny",
            Weather::Nice => "nice",
            Weather::PouringRain => "rain",
            Weather::Blizzard => "blizzard",
            Weather::Intro => "intro",
        }
    }

    pub fn from_name(name: &str) -> Option<Weather> {
        match name {
            "Sweltering Heat" => Some(Weather::SwelteringHeat),
            "Very Sunny" => Some(Weather::VerySunny),
            "Nice Weather" => Some(Weather::Nice),
            "Pouring Rain" => Some(Weather::PouringRain),
            "Blizzard" => Some(Weather::Blizzard),
            "Intro" => Some(Weather::Intro),
            _ => None,
        }
    }

    /// Map a 2D6 sum to weather. Mirrors Java DiceInterpreter.interpretWeather().
    /// 2=SwelteringHeat, 3=VerySunny, 11=PouringRain, 12=Blizzard, 4–10=Nice.
    pub fn for_roll(total: i32) -> Weather {
        match total {
            2  => Weather::SwelteringHeat,
            3  => Weather::VerySunny,
            11 => Weather::PouringRain,
            12 => Weather::Blizzard,
            _  => Weather::Nice,
        }
    }

    pub fn all() -> &'static [Weather] {
        &[
            Weather::SwelteringHeat,
            Weather::VerySunny,
            Weather::Nice,
            Weather::PouringRain,
            Weather::Blizzard,
            Weather::Intro,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_roll_all_buckets() {
        assert_eq!(Weather::for_roll(2), Weather::SwelteringHeat);
        assert_eq!(Weather::for_roll(3), Weather::VerySunny);
        assert_eq!(Weather::for_roll(4), Weather::Nice);
        assert_eq!(Weather::for_roll(10), Weather::Nice);
        assert_eq!(Weather::for_roll(11), Weather::PouringRain);
        assert_eq!(Weather::for_roll(12), Weather::Blizzard);
    }

    #[test]
    fn round_trip_name() {
        for w in Weather::all() {
            assert_eq!(Weather::from_name(w.name()), Some(*w));
        }
    }

    #[test]
    fn serde_round_trip() {
        let w = Weather::Blizzard;
        let json = serde_json::to_string(&w).unwrap();
        let back: Weather = serde_json::from_str(&json).unwrap();
        assert_eq!(w, back);
    }

    // ── WeatherCalcTest parity — all 11 roll values 2..=12 ─────────────────────

    #[test]
    fn weather_for_roll_sweltering_heat() {
        assert_eq!(Weather::for_roll(2), Weather::SwelteringHeat);
    }

    #[test]
    fn weather_for_roll_very_sunny() {
        assert_eq!(Weather::for_roll(3), Weather::VerySunny);
    }

    #[test]
    fn weather_for_roll_nice_all_middle_values() {
        for total in 4..=10 {
            assert_eq!(Weather::for_roll(total), Weather::Nice, "total={total}");
        }
    }

    #[test]
    fn weather_for_roll_pouring_rain() {
        assert_eq!(Weather::for_roll(11), Weather::PouringRain);
    }

    #[test]
    fn weather_for_roll_blizzard() {
        assert_eq!(Weather::for_roll(12), Weather::Blizzard);
    }

    #[test]
    fn weather_for_roll_nice_boundaries() {
        assert_eq!(Weather::for_roll(4), Weather::Nice);
        assert_eq!(Weather::for_roll(10), Weather::Nice);
    }
}
