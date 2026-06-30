// 1:1 translation of com.fumbbl.ffb.server.util.SpecialRollCalc
//
// Pure special-roll minimum and success calculations extracted from DiceInterpreter.
// All functions are stateless and require no game context.

pub struct SpecialRollCalc;

impl SpecialRollCalc {
    pub fn new() -> Self {
        Self
    }

    // ── Skill minimum rolls ───────────────────────────────────────────────────

    /// Dauntless: must roll >= this on D6 to use own strength instead of being capped.
    pub fn minimum_roll_dauntless(attacker_strength: i32, defender_strength: i32) -> i32 {
        (defender_strength - attacker_strength + 1).min(6)
    }

    /// Tentacles: dodging player escapes on 2D6 >= this.
    pub fn minimum_roll_tentacles_escape(
        tentacle_player_strength: i32,
        dodging_player_strength: i32,
    ) -> i32 {
        6 + tentacle_player_strength - dodging_player_strength
    }

    /// Shadowing: dodging player escapes on 2D6 >= this.
    pub fn minimum_roll_shadowing_escape(
        shadowing_player_movement: i32,
        dodging_player_movement: i32,
    ) -> i32 {
        8 + shadowing_player_movement - dodging_player_movement
    }

    /// Chainsaw armour break: needs 2+ on D6.
    pub fn minimum_roll_chainsaw() -> i32 {
        2
    }

    /// Foul Appearance: opposing player must roll 2+ or cannot target this player.
    pub fn minimum_roll_resisting_foul_appearance() -> i32 {
        2
    }

    /// Confusion / Bone Head / Really Stupid: pass on 2+ (good cond) or 4+ (bad).
    pub fn minimum_roll_confusion(good_conditions: bool) -> i32 {
        if good_conditions { 2 } else { 4 }
    }

    /// Blood Lust (Vampire): needs 2+ or attacks teammate.
    pub fn minimum_roll_blood_lust() -> i32 {
        2
    }

    /// Animosity: must roll 2+ or refuses to hand off / pass to non-same-race player.
    pub fn minimum_roll_animosity() -> i32 {
        2
    }

    // ── Skill/event success checks ────────────────────────────────────────────

    /// Regeneration: 4+ on D6 brings the player back from SI/death.
    pub fn is_regeneration_successful(roll: i32) -> bool {
        roll >= 4
    }

    /// Pitch invasion: player is stunned when roll > 1 AND roll + fame_other_team >= 6.
    pub fn is_affected_by_pitch_invasion(roll: i32, fame_other_team: i32) -> bool {
        roll > 1 && (roll + fame_other_team) >= 6
    }

    /// Recovering from KO: player wakes up when roll > 1 AND roll + bloodweiser_babes > 3.
    pub fn is_recovering_from_knockout(roll: i32, bloodweiser_babes: i32) -> bool {
        roll > 1 && (roll + bloodweiser_babes) > 3
    }

    /// Always Hungry: on 2+ the player acts; on 1 the ball-carrier is eaten.
    pub fn is_always_hungry_successful(roll: i32) -> bool {
        roll >= 2
    }

    /// Escape from Always Hungry: on 2+ the player is released unharmed.
    pub fn is_escape_from_always_hungry_successful(roll: i32) -> bool {
        roll >= 2
    }

    /// Wild Animal / Exhausted: a roll of 1 means the player fails to act.
    pub fn is_exhausted(roll: i32) -> bool {
        roll == 1
    }

    /// Tentacles escape: sum of 2D6 >= minimum.
    pub fn is_tentacles_escape_successful(
        die1: i32,
        die2: i32,
        tentacle_str: i32,
        dodging_str: i32,
    ) -> bool {
        (die1 + die2) >= Self::minimum_roll_tentacles_escape(tentacle_str, dodging_str)
    }

    /// Shadowing escape: sum of 2D6 >= minimum.
    pub fn is_shadowing_escape_successful(
        die1: i32,
        die2: i32,
        shadow_mov: i32,
        dodging_mov: i32,
    ) -> bool {
        (die1 + die2) >= Self::minimum_roll_shadowing_escape(shadow_mov, dodging_mov)
    }

    // ── Bribery / post-match events ───────────────────────────────────────────

    /// Bribes: 2+ on D6 avoids the sending-off.
    pub fn is_bribes_successful(roll: i32) -> bool {
        roll > 1
    }

    /// Argue the Call: 6 on D6 overturns the sending-off.
    pub fn is_argue_the_call_successful(roll: i32) -> bool {
        roll > 5
    }

    /// Argue the Call: coach is banned when they roll 1.
    pub fn is_coach_banned(roll: i32) -> bool {
        roll < 2
    }

    /// Stand up from prone: 4+ on D6 (modifier may apply from cards etc.).
    /// Unlike a normal skill roll: 1 always fails; 6 does NOT auto-succeed (rule book).
    pub fn is_stand_up_successful(roll: i32, modifier: i32) -> bool {
        roll > 1 && (roll + modifier) > 3
    }

    /// Loner / player defecting on Animosity: roll 1–3 = defects (fails).
    pub fn is_player_defecting(roll: i32) -> bool {
        roll > 0 && roll < 4
    }

    // ── Kickoff events ────────────────────────────────────────────────────────

    /// Riot: D6 < 4 → turn clock advances (+1 turn); D6 >= 4 → goes back (-1 turn).
    /// Returns +1 (advance) or -1 (go back).
    pub fn interpret_riot_roll(riot_roll: i32) -> i32 {
        if riot_roll < 4 { 1 } else { -1 }
    }

    /// True when two dice show the same value (used for doubles detection).
    pub fn is_double(die1: i32, die2: i32) -> bool {
        die1 == die2
    }
}

impl Default for SpecialRollCalc {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimum_roll_dauntless_equal_strength() {
        // def=3, att=3 → min(6, 1) = 1
        assert_eq!(SpecialRollCalc::minimum_roll_dauntless(3, 3), 1);
    }

    #[test]
    fn minimum_roll_dauntless_capped_at_6() {
        // def=6, att=1 → min(6, 6) = 6
        assert_eq!(SpecialRollCalc::minimum_roll_dauntless(1, 6), 6);
    }

    #[test]
    fn minimum_roll_dauntless_negative_clamped() {
        // def=2, att=5 → min(6, -2) = -2 (no floor in Java source, mirrors it)
        assert_eq!(SpecialRollCalc::minimum_roll_dauntless(5, 2), -2);
    }

    #[test]
    fn minimum_roll_tentacles_escape_equal_strength() {
        // 6 + 3 - 3 = 6
        assert_eq!(SpecialRollCalc::minimum_roll_tentacles_escape(3, 3), 6);
    }

    #[test]
    fn minimum_roll_shadowing_escape_equal_movement() {
        // 8 + 6 - 6 = 8
        assert_eq!(SpecialRollCalc::minimum_roll_shadowing_escape(6, 6), 8);
    }

    #[test]
    fn minimum_roll_chainsaw_is_2() {
        assert_eq!(SpecialRollCalc::minimum_roll_chainsaw(), 2);
    }

    #[test]
    fn minimum_roll_confusion_good_is_2() {
        assert_eq!(SpecialRollCalc::minimum_roll_confusion(true), 2);
    }

    #[test]
    fn minimum_roll_confusion_bad_is_4() {
        assert_eq!(SpecialRollCalc::minimum_roll_confusion(false), 4);
    }

    #[test]
    fn regeneration_succeeds_on_4_plus() {
        assert!(SpecialRollCalc::is_regeneration_successful(4));
        assert!(SpecialRollCalc::is_regeneration_successful(6));
        assert!(!SpecialRollCalc::is_regeneration_successful(3));
    }

    #[test]
    fn pitch_invasion_roll_1_is_never_stunned() {
        assert!(!SpecialRollCalc::is_affected_by_pitch_invasion(1, 5));
    }

    #[test]
    fn pitch_invasion_roll_plus_fame_under_6_not_stunned() {
        // roll=3, fame=2 → 3+2=5 < 6 → false
        assert!(!SpecialRollCalc::is_affected_by_pitch_invasion(3, 2));
    }

    #[test]
    fn pitch_invasion_roll_plus_fame_at_6_is_stunned() {
        // roll=4, fame=2 → 4+2=6 ≥ 6 → true
        assert!(SpecialRollCalc::is_affected_by_pitch_invasion(4, 2));
    }

    #[test]
    fn recovering_from_ko_roll_1_fails() {
        assert!(!SpecialRollCalc::is_recovering_from_knockout(1, 2));
    }

    #[test]
    fn recovering_from_ko_needs_total_above_3() {
        // roll=2, babes=1 → 2+1=3, not >3 → false
        assert!(!SpecialRollCalc::is_recovering_from_knockout(2, 1));
        // roll=2, babes=2 → 2+2=4 > 3 → true
        assert!(SpecialRollCalc::is_recovering_from_knockout(2, 2));
    }

    #[test]
    fn always_hungry_roll_1_fails() {
        assert!(!SpecialRollCalc::is_always_hungry_successful(1));
    }

    #[test]
    fn always_hungry_roll_2_succeeds() {
        assert!(SpecialRollCalc::is_always_hungry_successful(2));
    }

    #[test]
    fn is_exhausted_only_on_1() {
        assert!(SpecialRollCalc::is_exhausted(1));
        assert!(!SpecialRollCalc::is_exhausted(2));
    }

    #[test]
    fn tentacles_escape_sufficient_roll_succeeds() {
        // min = 6+3-3=6; die1+die2=6 → true
        assert!(SpecialRollCalc::is_tentacles_escape_successful(3, 3, 3, 3));
    }

    #[test]
    fn shadowing_escape_roll_too_low_fails() {
        // min = 8+5-5=8; roll=3+3=6 → false
        assert!(!SpecialRollCalc::is_shadowing_escape_successful(3, 3, 5, 5));
    }

    #[test]
    fn bribes_roll_1_fails() {
        assert!(!SpecialRollCalc::is_bribes_successful(1));
    }

    #[test]
    fn bribes_roll_2_plus_succeeds() {
        assert!(SpecialRollCalc::is_bribes_successful(2));
    }

    #[test]
    fn argue_the_call_only_succeeds_on_6() {
        assert!(!SpecialRollCalc::is_argue_the_call_successful(5));
        assert!(SpecialRollCalc::is_argue_the_call_successful(6));
    }

    #[test]
    fn coach_banned_on_1() {
        assert!(SpecialRollCalc::is_coach_banned(1));
        assert!(!SpecialRollCalc::is_coach_banned(2));
    }

    #[test]
    fn stand_up_roll_1_always_fails() {
        assert!(!SpecialRollCalc::is_stand_up_successful(1, 3));
    }

    #[test]
    fn stand_up_needs_total_above_3() {
        // roll=3, modifier=0 → 3>1 AND 3+0=3, not >3 → false
        assert!(!SpecialRollCalc::is_stand_up_successful(3, 0));
        // roll=4, modifier=0 → 4>1 AND 4>3 → true
        assert!(SpecialRollCalc::is_stand_up_successful(4, 0));
    }

    #[test]
    fn player_defecting_rolls_1_to_3() {
        assert!(SpecialRollCalc::is_player_defecting(1));
        assert!(SpecialRollCalc::is_player_defecting(3));
        assert!(!SpecialRollCalc::is_player_defecting(4));
    }

    #[test]
    fn riot_roll_below_4_advances_turn() {
        assert_eq!(SpecialRollCalc::interpret_riot_roll(3), 1);
    }

    #[test]
    fn riot_roll_4_or_above_goes_back() {
        assert_eq!(SpecialRollCalc::interpret_riot_roll(4), -1);
        assert_eq!(SpecialRollCalc::interpret_riot_roll(6), -1);
    }

    #[test]
    fn is_double_same_values() {
        assert!(SpecialRollCalc::is_double(3, 3));
    }

    #[test]
    fn is_double_different_values() {
        assert!(!SpecialRollCalc::is_double(2, 5));
    }
}
