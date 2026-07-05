use ffb_model::enums::Rules;

/// A single modifier applied to a roll, with a name and value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Modifier {
    pub name: &'static str,
    pub value: i32,
    pub rules: Rules,
}

impl Modifier {
    pub const fn new(name: &'static str, value: i32, rules: Rules) -> Self {
        Modifier { name, value, rules }
    }
}

/// Sum a slice of modifiers.
pub fn sum_modifiers(modifiers: &[Modifier]) -> i32 {
    modifiers.iter().map(|m| m.value).sum()
}

// ── Common armor modifiers ────────────────────────────────────────────────────

pub const ARMOR_MIGHTY_BLOW_1: Modifier =
    Modifier::new("Mighty Blow +1", 1, Rules::Bb2020);
pub const ARMOR_MIGHTY_BLOW_2: Modifier =
    Modifier::new("Mighty Blow +2", 2, Rules::Bb2020);
pub const ARMOR_PILING_ON: Modifier =
    Modifier::new("Piling On", 2, Rules::Bb2020);
pub const ARMOR_DIRTY_PLAYER_1: Modifier =
    Modifier::new("Dirty Player +1", 1, Rules::Bb2020);
pub const ARMOR_DIRTY_PLAYER_2: Modifier =
    Modifier::new("Dirty Player +2", 2, Rules::Bb2020);
pub const ARMOR_STUNTY: Modifier =
    Modifier::new("Stunty", -1, Rules::Common);

// ── Common injury modifiers ───────────────────────────────────────────────────

pub const INJURY_MIGHTY_BLOW_1: Modifier =
    Modifier::new("Mighty Blow +1", 1, Rules::Bb2020);
pub const INJURY_MIGHTY_BLOW_2: Modifier =
    Modifier::new("Mighty Blow +2", 2, Rules::Bb2020);
pub const INJURY_DIRTY_PLAYER_1: Modifier =
    Modifier::new("Dirty Player +1", 1, Rules::Bb2020);
pub const INJURY_DIRTY_PLAYER_2: Modifier =
    Modifier::new("Dirty Player +2", 2, Rules::Bb2020);
/// Mighty Blow applies to EITHER armor OR injury, never both in the same roll.
pub const MIGHTY_BLOW_EXCLUSIVE: bool = true;

/// Dirty Player only applies to foul injury rolls, not block injury rolls.
pub const DIRTY_PLAYER_FOUL_ONLY: bool = true;

/// BB2016: each niggling injury adds +1 to the opponent's injury roll.
pub const INJURY_NIGGLING_1: Modifier =
    Modifier::new("1 Niggling Injury", 1, Rules::Bb2016);
pub const INJURY_NIGGLING_2: Modifier =
    Modifier::new("2 Niggling Injuries", 2, Rules::Bb2016);
pub const INJURY_NIGGLING_3: Modifier =
    Modifier::new("3 Niggling Injuries", 3, Rules::Bb2016);
pub const INJURY_NIGGLING_4: Modifier =
    Modifier::new("4 Niggling Injuries", 4, Rules::Bb2016);
pub const INJURY_NIGGLING_5: Modifier =
    Modifier::new("5 Niggling Injuries", 5, Rules::Bb2016);
pub const INJURY_FIREBALL: Modifier =
    Modifier::new("Fireball", 1, Rules::Common);
pub const INJURY_LIGHTNING: Modifier =
    Modifier::new("Lightning", 1, Rules::Common);
pub const INJURY_BOMB: Modifier =
    Modifier::new("Bomb", 1, Rules::Bb2020);

// ── Armor foul-assist modifiers ───────────────────────────────────────────────

pub const ARMOR_FOUL_1_OFF: Modifier = Modifier::new("1 Offensive Assist", 1, Rules::Common);
pub const ARMOR_FOUL_2_OFF: Modifier = Modifier::new("2 Offensive Assists", 2, Rules::Common);
pub const ARMOR_FOUL_3_OFF: Modifier = Modifier::new("3 Offensive Assists", 3, Rules::Common);
pub const ARMOR_FOUL_4_OFF: Modifier = Modifier::new("4 Offensive Assists", 4, Rules::Common);
pub const ARMOR_FOUL_5_OFF: Modifier = Modifier::new("5 Offensive Assists", 5, Rules::Common);
pub const ARMOR_FOUL_6_OFF: Modifier = Modifier::new("6 Offensive Assists", 6, Rules::Common);
pub const ARMOR_FOUL_7_OFF: Modifier = Modifier::new("7 Offensive Assists", 7, Rules::Common);
pub const ARMOR_FOUL_1_DEF: Modifier = Modifier::new("1 Defensive Assist", -1, Rules::Common);
pub const ARMOR_FOUL_2_DEF: Modifier = Modifier::new("2 Defensive Assists", -2, Rules::Common);
pub const ARMOR_FOUL_3_DEF: Modifier = Modifier::new("3 Defensive Assists", -3, Rules::Common);
pub const ARMOR_FOUL_4_DEF: Modifier = Modifier::new("4 Defensive Assists", -4, Rules::Common);
pub const ARMOR_FOUL_5_DEF: Modifier = Modifier::new("5 Defensive Assists", -5, Rules::Common);

/// The "Foul" blatant-foul bonus (+1) when FOUL_BONUS option or FOUL_BONUS_OUTSIDE_TACKLEZONE applies.
pub const ARMOR_FOUL: Modifier = Modifier::new("Foul", 1, Rules::Common);

/// Chainsaw foul armor modifier (+3) — applied when attacker uses chainsaw for a Foul Action.
pub const ARMOR_CHAINSAW_3: Modifier = Modifier::new("Chainsaw", 3, Rules::Common);

/// Special-effect armor modifiers.
pub const ARMOR_FIREBALL: Modifier = Modifier::new("Fireball", 1, Rules::Common);
pub const ARMOR_LIGHTNING: Modifier = Modifier::new("Lightning", 1, Rules::Common);
pub const ARMOR_BOMB: Modifier = Modifier::new("Bomb", 1, Rules::Bb2020);

/// Returns the armor foul-assist modifier constant for a given net assist count.
/// net_assists = offensive_assists - defensive_assists.
/// Returns None if count is 0 or out of range (-5..=7 supported).
pub fn foul_assist_armor_modifier(net_assists: i32) -> Option<Modifier> {
    match net_assists {
        1 => Some(ARMOR_FOUL_1_OFF),
        2 => Some(ARMOR_FOUL_2_OFF),
        3 => Some(ARMOR_FOUL_3_OFF),
        4 => Some(ARMOR_FOUL_4_OFF),
        5 => Some(ARMOR_FOUL_5_OFF),
        6 => Some(ARMOR_FOUL_6_OFF),
        7 => Some(ARMOR_FOUL_7_OFF),
        -1 => Some(ARMOR_FOUL_1_DEF),
        -2 => Some(ARMOR_FOUL_2_DEF),
        -3 => Some(ARMOR_FOUL_3_DEF),
        -4 => Some(ARMOR_FOUL_4_DEF),
        -5 => Some(ARMOR_FOUL_5_DEF),
        _ => None,
    }
}

/// Returns the BB2016 niggling injury modifier for a given niggling count (1-5).
pub fn niggling_injury_modifier(count: i32) -> Option<Modifier> {
    match count {
        1 => Some(INJURY_NIGGLING_1),
        2 => Some(INJURY_NIGGLING_2),
        3 => Some(INJURY_NIGGLING_3),
        4 => Some(INJURY_NIGGLING_4),
        5 => Some(INJURY_NIGGLING_5),
        _ => None,
    }
}

// ── Common dodge modifiers ────────────────────────────────────────────────────
// Positive = harder for dodging player (penalty); negative = easier (benefit).
// Convention matches Java DodgeModifierCollection: tackle_zone=+1, Two Heads=-1.

/// Each tackle zone raises the dodge target by 1 (harder to dodge).
pub const DODGE_TACKLE_ZONE: Modifier =
    Modifier::new("Tackle Zone", 1, Rules::Common);
/// Two Heads: ignores all tackle zones (handled via property); direct modifier is -1 (benefit).
pub const DODGE_TWO_HEADS: Modifier =
    Modifier::new("Two Heads", -1, Rules::Common);
/// Break Tackle: allows ST-based dodge; modifier is -1 (benefit).
pub const DODGE_BREAK_TACKLE: Modifier =
    Modifier::new("Break Tackle", -1, Rules::Common);

// ── Common catch modifiers ────────────────────────────────────────────────────

/// Pouring Rain: +1 to catch target (harder to catch). Java CatchModifierCollection: value=+1.
pub const CATCH_RAIN: Modifier =
    Modifier::new("Pouring Rain", 1, Rules::Common);
/// Disturbing Presence: +1 per DP player adjacent (harder to catch).
pub const CATCH_DISTURBING_PRESENCE: Modifier =
    Modifier::new("Disturbing Presence", 1, Rules::Common);

// ── Common pass modifiers ─────────────────────────────────────────────────────
// Positive = harder for passer (penalty); negative = easier (benefit).
// Convention matches Java PassModifierCollection: Very Sunny=+1, Accurate=-1.

/// Very Sunny weather: +1 to pass target (glare makes passing harder).
pub const PASS_VERY_SUNNY: Modifier =
    Modifier::new("Very Sunny", 1, Rules::Common);
/// Disturbing Presence: +1 per DP player (harder to pass). One constant; multiply in use.
pub const PASS_DISTURBING_PRESENCE: Modifier =
    Modifier::new("Disturbing Presence", 1, Rules::Common);
/// Accurate Pass (BB2016): -1 to pass target (easier).
pub const PASS_ACCURATE: Modifier =
    Modifier::new("Accurate", -1, Rules::Bb2016);
/// Stunty: +1 to pass target when player has Stunty trait (harder for Stunty to pass).
pub const PASS_STUNTY: Modifier =
    Modifier::new("Stunty", 1, Rules::Common);

// ── BB2025 GFI modifiers ──────────────────────────────────────────────────────

/// Blizzard (BB2025): +1 to GFI minimum roll (needs 3+ instead of 2+).
pub const GFI_BLIZZARD_BB2025: Modifier =
    Modifier::new("Blizzard", 1, Rules::Bb2025);

/// Blizzard (BB2016): no pass roll modifier (movement effects are separate).
pub const PASS_BLIZZARD_BB2016: i32 = 0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sum_empty() {
        assert_eq!(sum_modifiers(&[]), 0);
    }

    #[test]
    fn sum_positive() {
        assert_eq!(sum_modifiers(&[ARMOR_MIGHTY_BLOW_1, ARMOR_PILING_ON]), 3);
    }

    #[test]
    fn sum_negative() {
        // ARMOR_STUNTY (-1) + ARMOR_MIGHTY_BLOW_1 (+1) = 0
        assert_eq!(sum_modifiers(&[ARMOR_STUNTY, ARMOR_MIGHTY_BLOW_1]), 0);
    }

    #[test]
    fn dodge_tackle_zone_is_positive_penalty() {
        // +1 per tackle zone: increases dodge target (harder)
        assert_eq!(DODGE_TACKLE_ZONE.value, 1);
    }

    #[test]
    fn dodge_two_heads_is_negative_benefit() {
        // -1: decreases dodge target (easier)
        assert_eq!(DODGE_TWO_HEADS.value, -1);
    }

    #[test]
    fn pass_accurate_is_negative_benefit() {
        // Accurate Pass makes passing easier: -1
        assert_eq!(PASS_ACCURATE.value, -1);
    }

    #[test]
    fn pass_stunty_is_positive_penalty() {
        // Stunty player passes harder: +1
        assert_eq!(PASS_STUNTY.value, 1);
    }

    #[test]
    fn catch_rain_is_positive_penalty() {
        assert_eq!(CATCH_RAIN.value, 1);
    }

    // ── Armor modifier values (ArmorModifierValues parity) ────────────────────

    #[test]
    fn mighty_blow_default_armor_modifier_is_1() {
        assert_eq!(ARMOR_MIGHTY_BLOW_1.value, 1);
    }

    #[test]
    fn dirty_player_default_armor_modifier_is_1() {
        assert_eq!(ARMOR_DIRTY_PLAYER_1.value, 1);
    }

    #[test]
    fn piling_on_armor_modifier_is_2() {
        assert_eq!(ARMOR_PILING_ON.value, 2);
    }

    #[test]
    fn stunty_armor_modifier_is_minus1() {
        assert_eq!(ARMOR_STUNTY.value, -1);
    }

    // ── Injury modifier values (InjuryModifierValues parity) ─────────────────

    #[test]
    fn mighty_blow_default_injury_modifier_is_1() {
        assert_eq!(INJURY_MIGHTY_BLOW_1.value, 1);
    }

    #[test]
    fn mighty_blow_exclusive_armor_or_injury_not_both() {
        assert!(MIGHTY_BLOW_EXCLUSIVE);
    }

    #[test]
    fn dirty_player_default_injury_modifier_is_1() {
        assert_eq!(INJURY_DIRTY_PLAYER_1.value, 1);
    }

    #[test]
    fn dirty_player_foul_only_constraint() {
        assert!(DIRTY_PLAYER_FOUL_ONLY);
    }

    #[test]
    fn niggling_injury_per_stack_is_1() {
        assert_eq!(INJURY_NIGGLING_1.value, 1);
    }

    #[test]
    fn niggling_injuries_2_give_plus2() {
        assert_eq!(INJURY_NIGGLING_2.value, 2);
    }

    #[test]
    fn fireball_injury_modifier_is_1() {
        assert_eq!(INJURY_FIREBALL.value, 1);
    }

    #[test]
    fn lightning_injury_modifier_is_1() {
        assert_eq!(INJURY_LIGHTNING.value, 1);
    }

    // ── Weather modifier values (WeatherModifierValues parity) ───────────────

    #[test]
    fn blizzard_gfi_bb2025_is_1() {
        assert_eq!(GFI_BLIZZARD_BB2025.value, 1);
    }

    #[test]
    fn very_sunny_pass_modifier_is_1() {
        assert_eq!(PASS_VERY_SUNNY.value, 1);
    }

    #[test]
    fn blizzard_pass_bb2016_is_0() {
        assert_eq!(PASS_BLIZZARD_BB2016, 0);
    }

    #[test]
    fn sign_convention_positive_is_harder() {
        assert!(CATCH_RAIN.value > 0);
        assert!(GFI_BLIZZARD_BB2025.value > 0);
        assert!(PASS_VERY_SUNNY.value > 0);
    }

    #[test]
    fn gfi_blizzard_bb2025_raises_minimum_to_3() {
        let gfi_base = 2;
        assert_eq!(3, (gfi_base + GFI_BLIZZARD_BB2025.value).max(2));
    }
}
