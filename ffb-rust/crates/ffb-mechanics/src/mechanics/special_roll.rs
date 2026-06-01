/// Dauntless: minimum D6 roll to use own strength instead of being capped.
pub fn minimum_roll_dauntless(attacker_strength: i32, defender_strength: i32) -> i32 {
    (defender_strength - attacker_strength + 1).min(6)
}

/// Tentacles: dodging player escapes on 2D6 >= this.
pub fn minimum_roll_tentacles_escape(tentacle_strength: i32, dodging_strength: i32) -> i32 {
    6 + tentacle_strength - dodging_strength
}

/// Shadowing: dodging player escapes on 2D6 >= this.
pub fn minimum_roll_shadowing_escape(shadowing_movement: i32, dodging_movement: i32) -> i32 {
    8 + shadowing_movement - dodging_movement
}

/// Confusion / Bone Head / Really Stupid: 2+ (good conditions) or 4+ (bad).
pub fn minimum_roll_confusion(good_conditions: bool) -> i32 {
    if good_conditions { 2 } else { 4 }
}

/// Regeneration: 4+ brings the player back.
pub fn is_regeneration_successful(roll: i32) -> bool { roll >= 4 }

/// Pitch invasion: stunned when roll > 1 AND roll + fame_other_team >= 6.
pub fn is_affected_by_pitch_invasion(roll: i32, fame_other_team: i32) -> bool {
    roll > 1 && (roll + fame_other_team) >= 6
}

/// KO recovery: wakes up when roll > 1 AND roll + bloodweiser_babes > 3.
pub fn is_recovering_from_knockout(roll: i32, bloodweiser_babes: i32) -> bool {
    roll > 1 && (roll + bloodweiser_babes) > 3
}

/// Always Hungry: 2+ means the player acts; 1 = ball-carrier eaten.
pub fn is_always_hungry_successful(roll: i32) -> bool { roll >= 2 }

/// Escape from Always Hungry: 2+ = released unharmed.
pub fn is_escape_from_always_hungry_successful(roll: i32) -> bool { roll >= 2 }

/// Wild Animal / Exhausted: a roll of 1 means the player fails to act.
pub fn is_exhausted(roll: i32) -> bool { roll == 1 }

/// Tentacles escape: 2D6 sum >= minimum.
pub fn is_tentacles_escape_successful(die1: i32, die2: i32, tentacle_str: i32, dodging_str: i32) -> bool {
    (die1 + die2) >= minimum_roll_tentacles_escape(tentacle_str, dodging_str)
}

/// Shadowing escape: 2D6 sum >= minimum.
pub fn is_shadowing_escape_successful(die1: i32, die2: i32, shadow_mov: i32, dodging_mov: i32) -> bool {
    (die1 + die2) >= minimum_roll_shadowing_escape(shadow_mov, dodging_mov)
}

/// Bribes: 2+ avoids the sending-off.
pub fn is_bribes_successful(roll: i32) -> bool { roll > 1 }

/// Argue the Call: 6 overturns the sending-off.
pub fn is_argue_the_call_successful(roll: i32) -> bool { roll > 5 }

/// Argue the Call: coach is banned on a roll of 1.
pub fn is_coach_banned(roll: i32) -> bool { roll < 2 }

/// Stand up from prone: 4+ (not an auto-success on 6; 1 always fails).
pub fn is_stand_up_successful(roll: i32, modifier: i32) -> bool {
    roll > 1 && (roll + modifier) > 3
}

/// Loner / Animosity defection: roll 1–3 = defects.
pub fn is_player_defecting(roll: i32) -> bool { roll > 0 && roll < 4 }

/// Minimum roll for a Chainsaw kickback armour roll (always 2+).
pub fn minimum_roll_chainsaw() -> i32 { 2 }

/// Minimum roll to resist Foul Appearance (always 2+).
pub fn minimum_roll_resisting_foul_appearance() -> i32 { 2 }

/// Minimum roll for Blood Lust to not trigger (always 2+).
pub fn minimum_roll_blood_lust() -> i32 { 2 }

/// Minimum roll so Animosity does not trigger (always 2+).
pub fn minimum_roll_animosity() -> i32 { 2 }

/// Riot: < 4 advances the turn clock (+1); >= 4 goes back (-1).
pub fn interpret_riot_roll(riot_roll: i32) -> i32 {
    if riot_roll < 4 { 1 } else { -1 }
}

/// True when two dice show the same value.
pub fn is_double(die1: i32, die2: i32) -> bool { die1 == die2 }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dauntless_str3_vs_4_minimum2() {
        assert_eq!(minimum_roll_dauntless(3, 4), 2);
    }

    #[test]
    fn dauntless_str2_vs_6_is5() {
        assert_eq!(minimum_roll_dauntless(2, 6), 5);
    }

    #[test]
    fn dauntless_str1_vs_6_is6() {
        assert_eq!(minimum_roll_dauntless(1, 6), 6);
    }

    #[test]
    fn dauntless_capped_at_6() {
        assert_eq!(minimum_roll_dauntless(1, 100), 6);
    }

    #[test]
    fn tentacles_equal_str_minimum6() {
        assert_eq!(minimum_roll_tentacles_escape(3, 3), 6);
    }

    #[test]
    fn tentacles_stronger_opponent_higher_minimum() {
        assert_eq!(minimum_roll_tentacles_escape(5, 3), 8);
    }

    #[test]
    fn tentacles_escape_success() {
        assert!(is_tentacles_escape_successful(4, 4, 3, 3));
    }

    #[test]
    fn tentacles_escape_failure() {
        assert!(!is_tentacles_escape_successful(2, 3, 5, 3));
    }

    #[test]
    fn shadowing_equal_movement_minimum8() {
        assert_eq!(minimum_roll_shadowing_escape(4, 4), 8);
    }

    #[test]
    fn shadowing_faster_shadow_higher_minimum() {
        assert_eq!(minimum_roll_shadowing_escape(6, 4), 10);
    }

    #[test]
    fn shadowing_escape_success_and_failure() {
        assert!(is_shadowing_escape_successful(5, 5, 4, 4));   // 10 >= 8
        assert!(!is_shadowing_escape_successful(2, 3, 6, 4));  // 5 < 10
    }

    #[test]
    fn constant_minimum_rolls_are_2() {
        assert_eq!(minimum_roll_chainsaw(), 2);
        assert_eq!(minimum_roll_resisting_foul_appearance(), 2);
        assert_eq!(minimum_roll_blood_lust(), 2);
        assert_eq!(minimum_roll_animosity(), 2);
    }

    #[test]
    fn confusion_good_is_2_bad_is_4() {
        assert_eq!(minimum_roll_confusion(true), 2);
        assert_eq!(minimum_roll_confusion(false), 4);
    }

    #[test]
    fn regeneration_4plus() {
        assert!(!is_regeneration_successful(3));
        assert!(is_regeneration_successful(4));
        assert!(is_regeneration_successful(6));
    }

    #[test]
    fn pitch_invasion_1_is_safe() {
        assert!(!is_affected_by_pitch_invasion(1, 3));
    }

    #[test]
    fn pitch_invasion_2_plus_3_fame_not_affected() {
        assert!(!is_affected_by_pitch_invasion(2, 3)); // 2+3=5 < 6
    }

    #[test]
    fn pitch_invasion_3_plus_3_fame_is_affected() {
        assert!(is_affected_by_pitch_invasion(3, 3));
    }

    #[test]
    fn knockout_recovery_1_never_recovers() {
        assert!(!is_recovering_from_knockout(1, 3));
    }

    #[test]
    fn knockout_recovery_2_with_2_babes() {
        assert!(is_recovering_from_knockout(2, 2));
        assert!(!is_recovering_from_knockout(2, 0));
    }

    #[test]
    fn always_hungry_fails_on_1() {
        assert!(!is_always_hungry_successful(1));
        assert!(is_always_hungry_successful(2));
    }

    #[test]
    fn escape_from_always_hungry_succeeds_on_2() {
        assert!(!is_escape_from_always_hungry_successful(1));
        assert!(is_escape_from_always_hungry_successful(2));
        assert!(is_escape_from_always_hungry_successful(6));
    }

    #[test]
    fn exhausted_only_on_1() {
        assert!(is_exhausted(1));
        assert!(!is_exhausted(2));
    }

    #[test]
    fn bribes_2plus_succeed() {
        assert!(!is_bribes_successful(1));
        assert!(is_bribes_successful(2));
    }

    #[test]
    fn argue_call_only_6() {
        assert!(!is_argue_the_call_successful(5));
        assert!(is_argue_the_call_successful(6));
    }

    #[test]
    fn coach_banned_on_1() {
        assert!(is_coach_banned(1));
        assert!(!is_coach_banned(2));
    }

    #[test]
    fn stand_up_1_always_fails() {
        assert!(!is_stand_up_successful(1, 0));
    }

    #[test]
    fn stand_up_4plus_succeeds() {
        assert!(!is_stand_up_successful(3, 0));
        assert!(is_stand_up_successful(4, 0));
    }

    #[test]
    fn stand_up_modifier_adjusts() {
        assert!(is_stand_up_successful(3, 1));
        assert!(!is_stand_up_successful(3, -1));
    }

    #[test]
    fn player_defecting_1_to_3() {
        assert!(is_player_defecting(1));
        assert!(is_player_defecting(3));
        assert!(!is_player_defecting(4));
    }

    #[test]
    fn riot_roll_below_4_advances() {
        assert_eq!(interpret_riot_roll(3), 1);
        assert_eq!(interpret_riot_roll(4), -1);
    }

    #[test]
    fn double_same_values() {
        assert!(is_double(3, 3));
        assert!(!is_double(2, 3));
    }

    // ── Individual parity tests (mirrors SpecialRollCalcTest one-method-per-case style) ─

    #[test]
    fn shadowing_escape_failure_standalone() {
        assert!(!is_shadowing_escape_successful(2, 3, 6, 4)); // 5 < 10
    }

    #[test]
    fn chainsaw_minimum_is_2_standalone() {
        assert_eq!(minimum_roll_chainsaw(), 2);
    }

    #[test]
    fn foul_appearance_minimum_is_2_standalone() {
        assert_eq!(minimum_roll_resisting_foul_appearance(), 2);
    }

    #[test]
    fn blood_lust_minimum_is_2_standalone() {
        assert_eq!(minimum_roll_blood_lust(), 2);
    }

    #[test]
    fn animosity_minimum_is_2_standalone() {
        assert_eq!(minimum_roll_animosity(), 2);
    }

    #[test]
    fn always_hungry_succeeds_on_2() {
        assert!(is_always_hungry_successful(2));
    }

    #[test]
    fn player_defecting_0_is_false() {
        assert!(!is_player_defecting(0));
    }

    #[test]
    fn riot_roll_extremes() {
        assert_eq!(interpret_riot_roll(1), 1);  // minimum advancing
        assert_eq!(interpret_riot_roll(6), -1); // maximum going back
    }
}
