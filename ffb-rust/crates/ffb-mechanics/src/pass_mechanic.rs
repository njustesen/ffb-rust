use ffb_model::enums::{PassingDistance, Weather};
use ffb_model::types::FieldCoordinate;
use ffb_model::model::{Game, Player};
use crate::mechanic::{Mechanic, MechanicType};
use crate::modifiers::{PassModifier, StatBasedRollModifier};
use crate::pass_result::PassResult;

/// 1:1 translation of com.fumbbl.ffb.mechanics.PassMechanic.
pub trait PassMechanic: Mechanic {
    fn get_type(&self) -> MechanicType { MechanicType::PASS }

    fn throwing_range_table(&self) -> Vec<String>;
    fn minimum_roll(&self, thrower: &Player, distance: PassingDistance, modifiers: &[PassModifier], stat_based_roll_modifier: Option<&StatBasedRollModifier>) -> Option<i32>;
    fn minimum_roll_simple(&self, thrower: &Player, distance: PassingDistance, modifiers: &[PassModifier]) -> Option<i32>;
    fn evaluate_pass(&self, thrower: &Player, roll: i32, distance: PassingDistance, modifiers: &[PassModifier], bomb_action: bool, stat_based_roll_modifier: Option<&StatBasedRollModifier>) -> PassResult;
    fn evaluate_pass_simple(&self, thrower: &Player, roll: i32, distance: PassingDistance, modifiers: &[PassModifier], bomb_action: bool) -> PassResult;
    fn format_report_roll(&self, roll: i32, thrower: &Player) -> String;
    fn format_roll_requirement(&self, distance: PassingDistance, formatted_modifiers: &str, thrower: &Player) -> String;
    fn eligible_to_re_roll(&self, re_rolled_action_name: &str, thrower: &Player) -> bool;
    fn pass_modifiers(&self, game: &Game, player: &Player) -> i32;

    /// 1:1 translation of calculateModifiers (concrete method in Java abstract class).
    fn calculate_modifiers(&self, pass_modifiers: &[PassModifier]) -> i32 {
        pass_modifiers.iter().map(|m| m.get_modifier()).sum()
    }

    /// 1:1 translation of findPassingDistance (concrete method in Java abstract class).
    fn find_passing_distance(&self, game: &Game, from_coordinate: Option<FieldCoordinate>, to_coordinate: Option<FieldCoordinate>, throw_team_mate: bool) -> Option<PassingDistance> {
        let (from, to) = match (from_coordinate, to_coordinate) {
            (Some(f), Some(t)) => (f, t),
            _ => return None,
        };
        let delta_y = (to.y - from.y).unsigned_abs() as usize;
        let delta_x = (to.x - from.x).unsigned_abs() as usize;
        if delta_y >= 14 || delta_x >= 14 {
            return None;
        }
        let table = self.throwing_range_table();
        if delta_y >= table.len() { return None; }
        let row = table[delta_y].as_bytes();
        let char_idx = delta_x * 2;
        if char_idx >= row.len() { return None; }
        let shortcut = row[char_idx] as char;
        let distance = PassingDistance::for_shortcut(shortcut)?;
        if (throw_team_mate || game.field_model.weather == Weather::Blizzard)
            && (distance == PassingDistance::LongBomb || distance == PassingDistance::LongPass)
        {
            return None;
        }
        Some(distance)
    }
}
