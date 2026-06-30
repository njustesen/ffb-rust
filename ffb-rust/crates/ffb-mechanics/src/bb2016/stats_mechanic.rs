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
