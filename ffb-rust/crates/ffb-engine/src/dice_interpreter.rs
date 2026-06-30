/// 1:1 translation of com.fumbbl.ffb.server.DiceInterpreter (singleton utility).
/// Methods are static (no state); the Java singleton pattern collapses to associated functions.
use ffb_model::enums::Weather;

pub struct DiceInterpreter;

impl DiceInterpreter {
    pub fn new() -> Self { Self }

    // ── Weather ───────────────────────────────────────────────────────────────

    /// 1:1 translation of DiceInterpreter.interpretRollWeather(int[]).
    /// Sums the two dice and delegates to interpret_weather.
    pub fn interpret_roll_weather(roll: &[i32]) -> Weather {
        let total: i32 = roll.iter().sum();
        Self::interpret_weather(total)
    }

    /// 1:1 translation of DiceInterpreter.interpretWeather(int).
    /// 2d6 table: 2=SwelteringHeat, 3=VerySunny, 4–10=Nice, 11=PouringRain, 12=Blizzard.
    pub fn interpret_weather(total: i32) -> Weather {
        Weather::for_roll(total)
    }

    // ── Skill roll success ────────────────────────────────────────────────────

    /// 1:1 translation of DiceInterpreter.isSkillRollSuccessful(int, int).
    /// 6 always succeeds; 1 always fails; otherwise roll >= minimum.
    pub fn is_skill_roll_successful(roll: i32, minimum_roll: i32) -> bool {
        (roll == 6) || ((roll != 1) && (roll >= minimum_roll))
    }

    // ── Minimum rolls ─────────────────────────────────────────────────────────

    /// 1:1 translation of DiceInterpreter.minimumRollGoingForIt.
    /// Delegates to GoForItModifierFactory; kept here for API parity.
    /// Prefer GoForItModifierFactory::minimum_roll_going_for_it when modifiers are available.
    pub fn minimum_roll_going_for_it_no_modifiers() -> i32 {
        2
    }

    /// 1:1 translation of DiceInterpreter.minimumRollResistingFoulAppearance.
    pub fn minimum_roll_resisting_foul_appearance() -> i32 {
        2
    }

    /// 1:1 translation of DiceInterpreter.minimumRollDauntless(int, int).
    pub fn minimum_roll_dauntless(attacker_strength: i32, defender_strength: i32) -> i32 {
        6_i32.min(defender_strength - attacker_strength + 1)
    }

    /// 1:1 translation of DiceInterpreter.minimumRollChainsaw.
    pub fn minimum_roll_chainsaw() -> i32 {
        2
    }

    /// 1:1 translation of DiceInterpreter.minimumRollProjectileVomit.
    pub fn minimum_roll_projectile_vomit() -> i32 {
        2
    }

    /// 1:1 translation of DiceInterpreter.minimumRollConfusion(boolean).
    pub fn minimum_roll_confusion(good_conditions: bool) -> i32 {
        if good_conditions { 2 } else { 4 }
    }

    /// 1:1 translation of DiceInterpreter.minimumRollBloodLust.
    pub fn minimum_roll_blood_lust() -> i32 {
        2
    }

    /// 1:1 translation of DiceInterpreter.minimumRollAnimosity.
    pub fn minimum_roll_animosity() -> i32 {
        2
    }

    /// 1:1 translation of DiceInterpreter.minimumRollWeepingDagger.
    pub fn minimum_roll_weeping_dagger() -> i32 {
        4
    }

    /// 1:1 translation of DiceInterpreter.minimumRollTentaclesEscape(int, int).
    pub fn minimum_roll_tentacles_escape(tentacle_player_strength: i32, dodging_player_strength: i32) -> i32 {
        6 + tentacle_player_strength - dodging_player_strength
    }

    /// 1:1 translation of DiceInterpreter.minimumRollShadowingEscape(int, int).
    pub fn minimum_roll_shadowing_escape(shadowing_player_movement: i32, dodging_player_movement: i32) -> i32 {
        8 + shadowing_player_movement - dodging_player_movement
    }

    // ── Boolean roll checks ───────────────────────────────────────────────────

    /// 1:1 translation of DiceInterpreter.isRegenerationSuccessful(int).
    pub fn is_regeneration_successful(roll: i32) -> bool {
        roll >= 4
    }

    /// 1:1 translation of DiceInterpreter.isAffectedByPitchInvasion(int, int).
    pub fn is_affected_by_pitch_invasion(roll: i32, fame_other_team: i32) -> bool {
        (roll > 1) && (roll + fame_other_team >= 6)
    }

    /// 1:1 translation of DiceInterpreter.isRecoveringFromKnockout(int, int).
    pub fn is_recovering_from_knockout(roll: i32, bloodweiser_babes: i32) -> bool {
        (roll > 1) && ((roll + bloodweiser_babes) > 3)
    }

    /// 1:1 translation of DiceInterpreter.isAlwaysHungrySuccessful(int).
    pub fn is_always_hungry_successful(roll: i32) -> bool {
        roll >= 2
    }

    /// 1:1 translation of DiceInterpreter.isEscapeFromAlwaysHungrySuccessful(int).
    pub fn is_escape_from_always_hungry_successful(roll: i32) -> bool {
        roll >= 2
    }

    /// 1:1 translation of DiceInterpreter.isExhausted(int).
    pub fn is_exhausted(roll: i32) -> bool {
        roll == 1
    }

    /// 1:1 translation of DiceInterpreter.isStandUpSuccessful(int, int).
    /// roll > 1 AND roll + modifier > 3 (i.e. roll + modifier >= 4).
    pub fn is_stand_up_successful(roll: i32, modifier: i32) -> bool {
        roll > 1 && roll + modifier > 3
    }

    /// 1:1 translation of DiceInterpreter.isPlayerDefecting(int).
    pub fn is_player_defecting(roll: i32) -> bool {
        roll > 0 && roll < 4
    }

    /// 1:1 translation of DiceInterpreter.isTentaclesEscapeSuccessful(int[], int, int).
    pub fn is_tentacles_escape_successful(roll: &[i32], tentacle_player_strength: i32, dodging_player_strength: i32) -> bool {
        roll.len() > 1
            && (roll[0] + roll[1]) >= Self::minimum_roll_tentacles_escape(tentacle_player_strength, dodging_player_strength)
    }

    /// 1:1 translation of DiceInterpreter.isShadowingEscapeSuccessful(int[], int, int).
    pub fn is_shadowing_escape_successful(roll: &[i32], shadowing_player_movement: i32, dodging_player_movement: i32) -> bool {
        roll.len() > 1
            && (roll[0] + roll[1]) >= Self::minimum_roll_shadowing_escape(shadowing_player_movement, dodging_player_movement)
    }

    // ── Post-match rolls ──────────────────────────────────────────────────────

    /// 1:1 translation of DiceInterpreter.interpretFanFactorRoll(int[], int, int).
    /// Returns -1, 0, or +1 modifier to dedicated fans.
    pub fn interpret_fan_factor_roll(fan_factor_roll: &[i32], fan_factor: i32, score_diff: i32) -> i32 {
        let fan_factor_total: i32 = fan_factor_roll.iter().sum();
        let mut modifier = 0;
        if score_diff >= 0 && fan_factor_total > fan_factor {
            modifier = 1;
        }
        if score_diff <= 0 && fan_factor_total < fan_factor {
            modifier = -1;
        }
        modifier
    }

    /// 1:1 translation of DiceInterpreter.interpretMasterChefRoll(int[]).
    /// Returns number of re-rolls stolen (roll > 3 per die).
    pub fn interpret_master_chef_roll(master_chef_roll: &[i32]) -> i32 {
        master_chef_roll.iter().filter(|&&r| r > 3).count() as i32
    }
}

impl Default for DiceInterpreter {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Weather;

    // ── Weather ───────────────────────────────────────────────────────────────

    #[test]
    fn interpret_roll_weather_sweltering_heat() {
        assert_eq!(DiceInterpreter::interpret_roll_weather(&[1, 1]), Weather::SwelteringHeat);
    }

    #[test]
    fn interpret_roll_weather_very_sunny() {
        assert_eq!(DiceInterpreter::interpret_roll_weather(&[1, 2]), Weather::VerySunny);
    }

    #[test]
    fn interpret_roll_weather_nice() {
        assert_eq!(DiceInterpreter::interpret_roll_weather(&[3, 4]), Weather::Nice);
    }

    #[test]
    fn interpret_roll_weather_pouring_rain() {
        assert_eq!(DiceInterpreter::interpret_roll_weather(&[5, 6]), Weather::PouringRain);
    }

    #[test]
    fn interpret_roll_weather_blizzard() {
        assert_eq!(DiceInterpreter::interpret_roll_weather(&[6, 6]), Weather::Blizzard);
    }

    #[test]
    fn interpret_weather_all_nice_values() {
        for total in 4..=10 {
            assert_eq!(DiceInterpreter::interpret_weather(total), Weather::Nice, "total={total}");
        }
    }

    // ── is_skill_roll_successful ──────────────────────────────────────────────

    #[test]
    fn roll_six_always_succeeds() {
        assert!(DiceInterpreter::is_skill_roll_successful(6, 6));
        assert!(DiceInterpreter::is_skill_roll_successful(6, 7));
    }

    #[test]
    fn roll_one_always_fails() {
        assert!(!DiceInterpreter::is_skill_roll_successful(1, 1));
        assert!(!DiceInterpreter::is_skill_roll_successful(1, 2));
    }

    #[test]
    fn roll_meets_minimum() {
        assert!(DiceInterpreter::is_skill_roll_successful(4, 4));
        assert!(!DiceInterpreter::is_skill_roll_successful(3, 4));
    }

    // ── Minimum rolls ─────────────────────────────────────────────────────────

    #[test]
    fn minimum_roll_dauntless_capped_at_six() {
        assert_eq!(DiceInterpreter::minimum_roll_dauntless(1, 10), 6);
        assert_eq!(DiceInterpreter::minimum_roll_dauntless(3, 5), 3);
    }

    #[test]
    fn minimum_roll_confusion_good_conditions() {
        assert_eq!(DiceInterpreter::minimum_roll_confusion(true), 2);
        assert_eq!(DiceInterpreter::minimum_roll_confusion(false), 4);
    }

    #[test]
    fn minimum_roll_tentacles_escape_formula() {
        assert_eq!(DiceInterpreter::minimum_roll_tentacles_escape(5, 3), 8);
    }

    #[test]
    fn minimum_roll_shadowing_escape_formula() {
        assert_eq!(DiceInterpreter::minimum_roll_shadowing_escape(6, 4), 10);
    }

    // ── Boolean checks ────────────────────────────────────────────────────────

    #[test]
    fn is_regeneration_successful_threshold_four() {
        assert!(DiceInterpreter::is_regeneration_successful(4));
        assert!(!DiceInterpreter::is_regeneration_successful(3));
    }

    #[test]
    fn is_recovering_from_knockout_roll_one_always_fails() {
        assert!(!DiceInterpreter::is_recovering_from_knockout(1, 10));
    }

    #[test]
    fn is_recovering_from_knockout_roll_four_no_babes() {
        assert!(DiceInterpreter::is_recovering_from_knockout(4, 0));
        assert!(!DiceInterpreter::is_recovering_from_knockout(3, 0));
    }

    #[test]
    fn is_affected_by_pitch_invasion_roll_one_fails() {
        assert!(!DiceInterpreter::is_affected_by_pitch_invasion(1, 5));
    }

    #[test]
    fn tentacles_escape_requires_two_dice() {
        assert!(!DiceInterpreter::is_tentacles_escape_successful(&[5], 3, 3));
        assert!(DiceInterpreter::is_tentacles_escape_successful(&[5, 5], 3, 4));
    }

    // ── Post-match ────────────────────────────────────────────────────────────

    #[test]
    fn interpret_fan_factor_roll_winning_gains() {
        assert_eq!(DiceInterpreter::interpret_fan_factor_roll(&[4, 5], 5, 1), 1);
    }

    #[test]
    fn interpret_fan_factor_roll_losing_loses() {
        assert_eq!(DiceInterpreter::interpret_fan_factor_roll(&[2, 1], 5, -1), -1);
    }

    #[test]
    fn interpret_master_chef_roll_counts_high_dice() {
        // rolls: 4, 2, 5 → 4 > 3 and 5 > 3 → 2 rerolls stolen
        assert_eq!(DiceInterpreter::interpret_master_chef_roll(&[4, 2, 5]), 2);
    }
}
