/// 1:1 translation of `com.fumbbl.ffb.server.mechanic.WeatherModifierValues`.
///
/// Documented modifier values for weather effects on rolls.
///
/// Sign convention (positive = harder for the rolling player, negative = easier):
/// - Dodge/catch/pass: modifier is ADDED to the roll target (min 2).
///   +1 = harder; -1 = easier.
pub struct WeatherModifierValues;

impl WeatherModifierValues {
    /// Pouring Rain: +1 to catch and pickup targets (harder to handle the ball).
    pub const POURING_RAIN_CATCH: i32 = 1;

    /// Blizzard (BB2016): +0 to pass rolls (no pass modifier in BB2016 Blizzard).
    pub const BLIZZARD_PASS_BB2016: i32 = 0;

    /// Blizzard (BB2025): +1 to GFI minimum roll (needs 3+ instead of 2+).
    pub const BLIZZARD_GFI_BB2025: i32 = 1;

    /// Very Sunny: +1 to pass rolls (harder to pass in bright sunlight).
    pub const VERY_SUNNY_PASS: i32 = 1;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pouring_rain_catch_is_1() {
        assert_eq!(WeatherModifierValues::POURING_RAIN_CATCH, 1);
    }

    #[test]
    fn blizzard_pass_bb2016_is_0() {
        assert_eq!(WeatherModifierValues::BLIZZARD_PASS_BB2016, 0);
    }

    #[test]
    fn blizzard_gfi_bb2025_is_1() {
        assert_eq!(WeatherModifierValues::BLIZZARD_GFI_BB2025, 1);
    }

    #[test]
    fn very_sunny_pass_is_1() {
        assert_eq!(WeatherModifierValues::VERY_SUNNY_PASS, 1);
    }
    #[test]
    fn blizzard_and_pouring_rain_are_different_effects() {
        assert_ne!(WeatherModifierValues::BLIZZARD_GFI_BB2025, WeatherModifierValues::BLIZZARD_PASS_BB2016);
    }
}
