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
