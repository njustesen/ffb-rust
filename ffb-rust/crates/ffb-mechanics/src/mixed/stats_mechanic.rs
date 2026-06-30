use ffb_model::enums::PlayerStatKey;
use ffb_model::model::Game;
use crate::mechanic::{Mechanic, MechanicType};
use crate::modifiers::{InjuryContext, PlayerStatLimit, StatBasedRollModifier};
use crate::stats_drawing_modifier::StatsDrawingModifier;
use crate::stats_mechanic::StatsMechanic as StatsMechanicTrait;

/// 1:1 translation of com.fumbbl.ffb.mechanics.mixed.StatsMechanic (BB2020/BB2025).
pub struct StatsMechanic;

impl Default for StatsMechanic {
    fn default() -> Self { StatsMechanic }
}

impl Mechanic for StatsMechanic {
    fn get_type(&self) -> MechanicType { MechanicType::STAT }
}

impl StatsMechanicTrait for StatsMechanic {
    fn draw_passing(&self) -> bool { true }

    fn stat_suffix(&self) -> String { "+".to_string() }

    fn armour_is_broken(&self, armour: i32, roll: &[i32; 2], context: &InjuryContext, _game: &Game) -> bool {
        // TODO: context.armor_modifier_total(game) not yet available — using 0
        let reduced = self.reduce_armour(context, armour, 8);
        reduced <= (roll[0] + roll[1])
    }

    fn agility_modifier(&self, modifier: i32) -> StatBasedRollModifier {
        let _ = StatsDrawingModifier::positive_impairs(modifier);
        StatBasedRollModifier::new("agility", modifier)
    }

    fn improvement_increases_value(&self) -> bool { false }

    fn apply_in_game_agility_injury(&self, agility: i32, decreases: i32) -> i32 {
        agility + decreases
    }

    fn limit(&self, key: PlayerStatKey) -> PlayerStatLimit {
        match key {
            PlayerStatKey::Ma => PlayerStatLimit::new(1, 9),
            PlayerStatKey::St => PlayerStatLimit::new(1, 8),
            PlayerStatKey::Ag => PlayerStatLimit::new(1, 6),
            PlayerStatKey::Pa => PlayerStatLimit::new(1, 6),
            PlayerStatKey::Av => PlayerStatLimit::new(3, 11),
        }
    }

    fn apply_lasting_injury(&self, starting_value: i32, key: PlayerStatKey) -> i32 {
        let limit = self.limit(key);
        match key {
            PlayerStatKey::Ag | PlayerStatKey::Pa => (starting_value + 1).min(limit.get_max()),
            _ => (starting_value - 1).max(limit.get_min()),
        }
    }

    fn stat_can_be_reduced_by_injury(&self, _original_value: i32, _current_value: i32) -> bool {
        true
    }
}
