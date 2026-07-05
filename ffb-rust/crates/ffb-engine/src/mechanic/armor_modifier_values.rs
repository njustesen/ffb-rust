/// 1:1 translation of `com.fumbbl.ffb.server.mechanic.ArmorModifierValues`.
///
/// Documented modifier values for skills that affect armor rolls.
///
/// Armor roll formula: total = d6 + d6 + sum(applicable modifiers)
/// Armor broken when:
///   BB2016: total > armour  (strict)
///   BB2020/BB2025: total >= armour  (inclusive)
pub struct ArmorModifierValues;

impl ArmorModifierValues {
    /// Mighty Blow default modifier for armor rolls. Default +1 all editions.
    pub const MIGHTY_BLOW_DEFAULT: i32 = 1;

    /// Dirty Player default modifier for armor rolls (foul actions only). Default +1.
    pub const DIRTY_PLAYER_DEFAULT: i32 = 1;

    /// Piling On modifier for armor rolls. Constant +2 in all editions.
    pub const PILING_ON: i32 = 2;

    /// Stunty imposes -1 to the armor roll against a Stunty player.
    pub const STUNTY: i32 = -1;

    /// Fixed armour cap applied by Chainsaw (BB2016).
    pub const FIXED_ARMOUR_CAP_BB2016: i32 = 7;

    /// Fixed armour cap applied by Chainsaw (BB2020/BB2025).
    pub const FIXED_ARMOUR_CAP_BB2020: i32 = 8;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mighty_blow_default_is_1() {
        assert_eq!(ArmorModifierValues::MIGHTY_BLOW_DEFAULT, 1);
    }

    #[test]
    fn piling_on_is_2() {
        assert_eq!(ArmorModifierValues::PILING_ON, 2);
    }

    #[test]
    fn stunty_is_minus_1() {
        assert_eq!(ArmorModifierValues::STUNTY, -1);
    }

    #[test]
    fn fixed_armour_cap_bb2016_is_7() {
        assert_eq!(ArmorModifierValues::FIXED_ARMOUR_CAP_BB2016, 7);
    }

    #[test]
    fn fixed_armour_cap_bb2020_is_8() {
        assert_eq!(ArmorModifierValues::FIXED_ARMOUR_CAP_BB2020, 8);
    }
}
