use ffb_model::enums::PlayerStatKey;
use ffb_model::model::Game;
use crate::mechanic::{Mechanic, MechanicType};
use crate::modifiers::{PlayerStatLimit, StatBasedRollModifier};
use crate::modifiers::InjuryContext;

/// 1:1 translation of com.fumbbl.ffb.mechanics.StatsMechanic.
pub trait StatsMechanic: Mechanic {
    fn get_type(&self) -> MechanicType { MechanicType::STAT }

    fn draw_passing(&self) -> bool;
    fn stat_suffix(&self) -> String;
    fn armour_is_broken(&self, armour: i32, roll: &[i32; 2], context: &InjuryContext, game: &Game) -> bool;
    fn agility_modifier(&self, modifier: i32) -> StatBasedRollModifier;
    fn improvement_increases_value(&self) -> bool;
    fn apply_in_game_agility_injury(&self, agility: i32, decreases: i32) -> i32;
    fn limit(&self, key: PlayerStatKey) -> PlayerStatLimit;
    fn apply_lasting_injury(&self, starting_value: i32, key: PlayerStatKey) -> i32;
    fn stat_can_be_reduced_by_injury(&self, original_value: i32, current_value: i32) -> bool;

    /// 1:1 translation of reduceArmour (concrete protected method in Java abstract class).
    fn reduce_armour(&self, context: &InjuryContext, armour: i32, reduction_value: i32) -> i32 {
        // TODO: check NamedProperties.reducesArmourToFixedValue via context.getArmorModifiers()
        let _ = context;
        if armour > reduction_value {
            reduction_value
        } else {
            armour
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modifiers::InjuryContext;
    use ffb_model::enums::PlayerStatKey;
    use crate::modifiers::{PlayerStatLimit, StatBasedRollModifier};

    struct MinimalStats;
    impl Mechanic for MinimalStats {
        fn get_type(&self) -> MechanicType { MechanicType::STAT }
    }
    impl StatsMechanic for MinimalStats {
        fn draw_passing(&self) -> bool { false }
        fn stat_suffix(&self) -> String { String::new() }
        fn armour_is_broken(&self, _: i32, _: &[i32; 2], _: &InjuryContext, _: &Game) -> bool { false }
        fn agility_modifier(&self, m: i32) -> StatBasedRollModifier { StatBasedRollModifier::new("agility", m) }
        fn improvement_increases_value(&self) -> bool { true }
        fn apply_in_game_agility_injury(&self, a: i32, d: i32) -> i32 { a - d }
        fn limit(&self, _: PlayerStatKey) -> PlayerStatLimit { PlayerStatLimit::new(1, 10) }
        fn apply_lasting_injury(&self, v: i32, _: PlayerStatKey) -> i32 { v }
        fn stat_can_be_reduced_by_injury(&self, _: i32, _: i32) -> bool { true }
    }

    #[test]
    fn reduce_armour_returns_reduction_when_lower_than_armour() {
        let ctx = InjuryContext::block("d1".into(), "a1".into());
        let result = MinimalStats.reduce_armour(&ctx, 8, 5);
        assert_eq!(result, 5);
    }

    #[test]
    fn reduce_armour_returns_armour_when_not_higher_than_reduction() {
        let ctx = InjuryContext::block("d1".into(), "a1".into());
        assert_eq!(MinimalStats.reduce_armour(&ctx, 4, 8), 4);
        assert_eq!(MinimalStats.reduce_armour(&ctx, 5, 5), 5);
    }
}
