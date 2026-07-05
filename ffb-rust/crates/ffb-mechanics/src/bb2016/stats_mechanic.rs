use ffb_model::enums::PlayerStatKey;
use ffb_model::model::Game;
use crate::mechanic::{Mechanic, MechanicType};
use crate::modifiers::{InjuryContext, PlayerStatLimit, StatBasedRollModifier};
use crate::stats_drawing_modifier::StatsDrawingModifier;
use crate::stats_mechanic::StatsMechanic as StatsMechanicTrait;

/// 1:1 translation of com.fumbbl.ffb.mechanics.bb2016.StatsMechanic.
pub struct StatsMechanic;

impl Default for StatsMechanic {
    fn default() -> Self { StatsMechanic }
}

impl Mechanic for StatsMechanic {
    fn get_type(&self) -> MechanicType { MechanicType::STAT }
}

impl StatsMechanicTrait for StatsMechanic {
    fn draw_passing(&self) -> bool { false }

    fn stat_suffix(&self) -> String { String::new() }

    fn armour_is_broken(&self, armour: i32, roll: &[i32; 2], context: &InjuryContext, _game: &Game) -> bool {
        // TODO: context.armor_modifier_total(game) not yet available — using 0
        let reduced = self.reduce_armour(context, armour, 7);
        reduced < (roll[0] + roll[1])
    }

    fn agility_modifier(&self, modifier: i32) -> StatBasedRollModifier {
        let _ = StatsDrawingModifier::positive_improves(modifier);
        StatBasedRollModifier::new("agility", modifier)
    }

    fn improvement_increases_value(&self) -> bool { true }

    fn apply_in_game_agility_injury(&self, agility: i32, decreases: i32) -> i32 {
        agility - decreases
    }

    fn limit(&self, key: PlayerStatKey) -> PlayerStatLimit {
        match key {
            PlayerStatKey::Ma | PlayerStatKey::St | PlayerStatKey::Ag | PlayerStatKey::Av => {
                PlayerStatLimit::new(1, 10)
            }
            PlayerStatKey::Pa => PlayerStatLimit::new(0, 0),
        }
    }

    fn apply_lasting_injury(&self, starting_value: i32, key: PlayerStatKey) -> i32 {
        let limit = self.limit(key);
        (starting_value - 1).max(limit.get_min())
    }

    fn stat_can_be_reduced_by_injury(&self, original_value: i32, current_value: i32) -> bool {
        (original_value - current_value) < 2
    }
}

impl StatsMechanic {
    pub fn new() -> Self { StatsMechanic }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::PlayerStatKey;
    use crate::stats_mechanic::StatsMechanic as StatsTrait;

    #[test]
    fn draw_passing_is_false() {
        assert!(!StatsMechanic.draw_passing());
    }

    #[test]
    fn stat_suffix_is_empty() {
        assert_eq!(StatsMechanic.stat_suffix(), "");
    }

    #[test]
    fn improvement_increases_value_is_true() {
        assert!(StatsMechanic.improvement_increases_value());
    }

    #[test]
    fn apply_in_game_agility_injury_decrements() {
        assert_eq!(StatsMechanic.apply_in_game_agility_injury(3, 1), 2);
    }

    #[test]
    fn apply_lasting_injury_decrements_and_floors_at_one() {
        // min is 1 for AG
        assert_eq!(StatsMechanic.apply_lasting_injury(4, PlayerStatKey::Ag), 3);
        assert_eq!(StatsMechanic.apply_lasting_injury(1, PlayerStatKey::Ag), 1);
    }

    #[test]
    fn stat_can_be_reduced_by_injury_true_when_diff_less_than_2() {
        assert!(StatsMechanic.stat_can_be_reduced_by_injury(3, 2));
    }

    #[test]
    fn stat_can_be_reduced_by_injury_false_when_diff_equals_2() {
        assert!(!StatsMechanic.stat_can_be_reduced_by_injury(3, 1));
    }
}
