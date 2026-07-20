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

// NOTE: tests duplicating SpecialRollCalcTest.java live in the 1:1 mirror module
// (ffb-engine/src/util/special_roll_calc.rs); redundant copies were removed here.
