/// 1:1 translation of `com.fumbbl.ffb.server.mechanic.InjuryModifierValues`.
///
/// Documented modifier values for skills that affect injury rolls.
///
/// Injury roll formula: total = d6 + d6 + sum(applicable modifiers)
/// Outcomes determined by `InjuryCalc::interpret_injury_total_bb2016/bb2020`.
pub struct InjuryModifierValues;

impl InjuryModifierValues {
    /// Mighty Blow default modifier for injury rolls (+1 to injury total).
    pub const MIGHTY_BLOW_DEFAULT: i32 = 1;

    /// Mighty Blow applies to EITHER armor OR injury roll, never both.
    pub const MIGHTY_BLOW_EXCLUSIVE: bool = true;

    /// Dirty Player default modifier for injury rolls (+1 to injury total).
    pub const DIRTY_PLAYER_DEFAULT: i32 = 1;

    /// Dirty Player only applies during foul actions.
    pub const DIRTY_PLAYER_FOUL_ONLY: bool = true;

    /// Each niggling injury adds this value to the opponent's injury roll (BB2016).
    pub const NIGGLING_INJURY_PER_STACK: i32 = 1;

    /// Fireball special effect adds this to injury rolls.
    pub const FIREBALL_MODIFIER: i32 = 1;

    /// Lightning special effect adds this to injury rolls.
    pub const LIGHTNING_MODIFIER: i32 = 1;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mighty_blow_default_is_1() {
        assert_eq!(InjuryModifierValues::MIGHTY_BLOW_DEFAULT, 1);
    }

    #[test]
    fn mighty_blow_is_exclusive() {
        assert!(InjuryModifierValues::MIGHTY_BLOW_EXCLUSIVE);
    }

    #[test]
    fn fireball_modifier_is_1() {
        assert_eq!(InjuryModifierValues::FIREBALL_MODIFIER, 1);
    }

    #[test]
    fn niggling_injury_per_stack_is_1() {
        assert_eq!(InjuryModifierValues::NIGGLING_INJURY_PER_STACK, 1);
    }

    #[test]
    fn dirty_player_default_is_1() {
        assert_eq!(InjuryModifierValues::DIRTY_PLAYER_DEFAULT, 1);
    }

    #[test]
    fn dirty_player_foul_only() {
        assert!(InjuryModifierValues::DIRTY_PLAYER_FOUL_ONLY);
    }

    #[test]
    fn lightning_modifier_is_1() {
        assert_eq!(InjuryModifierValues::LIGHTNING_MODIFIER, 1);
    }
}
