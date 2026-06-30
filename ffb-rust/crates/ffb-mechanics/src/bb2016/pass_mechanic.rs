use ffb_model::enums::PassingDistance;
use ffb_model::model::{Game, Player};
use crate::mechanic::{Mechanic, MechanicType};
use crate::modifiers::{PassModifier, StatBasedRollModifier};
use crate::pass_result::PassResult;
use crate::pass_mechanic::PassMechanic as PassMechanicTrait;

/// 1:1 translation of com.fumbbl.ffb.mechanics.bb2016.PassMechanic.
pub struct PassMechanic;

impl Default for PassMechanic {
    fn default() -> Self { PassMechanic }
}

impl Mechanic for PassMechanic {
    fn get_type(&self) -> MechanicType { MechanicType::PASS }
}

impl PassMechanicTrait for PassMechanic {
    fn throwing_range_table(&self) -> Vec<String> {
        vec![
            "T Q Q Q S S S L L L L B B B".into(),
            "Q Q Q Q S S S L L L L B B B".into(),
            "Q Q Q S S S S L L L L B B  ".into(),
            "Q Q S S S S S L L L B B B  ".into(),
            "S S S S S S L L L L B B B  ".into(),
            "S S S S S L L L L B B B    ".into(),
            "S S S S L L L L L B B B    ".into(),
            "L L L L L L L L B B B      ".into(),
            "L L L L L L L B B B B      ".into(),
            "L L L L L B B B B B        ".into(),
            "L L L B B B B B B          ".into(),
            "B B B B B B B              ".into(),
            "B B B B B                  ".into(),
            "B B                        ".into(),
        ]
    }

    fn minimum_roll(
        &self,
        thrower: &Player,
        distance: PassingDistance,
        modifiers: &[PassModifier],
        _stat_based_roll_modifier: Option<&StatBasedRollModifier>,
    ) -> Option<i32> {
        Some(self.minimum_roll_internal(thrower, distance, modifiers))
    }

    fn minimum_roll_simple(
        &self,
        thrower: &Player,
        distance: PassingDistance,
        modifiers: &[PassModifier],
    ) -> Option<i32> {
        Some(self.minimum_roll_internal(thrower, distance, modifiers))
    }

    fn evaluate_pass(
        &self,
        thrower: &Player,
        roll: i32,
        distance: PassingDistance,
        modifiers: &[PassModifier],
        bomb_action: bool,
        _stat_based_roll_modifier: Option<&StatBasedRollModifier>,
    ) -> PassResult {
        self.evaluate_pass_simple(thrower, roll, distance, modifiers, bomb_action)
    }

    fn evaluate_pass_simple(
        &self,
        thrower: &Player,
        roll: i32,
        distance: PassingDistance,
        modifiers: &[PassModifier],
        bomb_action: bool,
    ) -> PassResult {
        let minimum_roll = self.minimum_roll_internal(thrower, distance, modifiers);
        if roll == 6 {
            PassResult::ACCURATE
        } else if roll == 1 {
            PassResult::FUMBLE
        } else if self.is_modified_fumble(roll, distance, modifiers) {
            if !bomb_action {
                // TODO: thrower.has_skill_property(NamedProperties.dontDropFumbles) → SAVED_FUMBLE
            }
            PassResult::FUMBLE
        } else if roll < minimum_roll {
            PassResult::INACCURATE
        } else {
            PassResult::ACCURATE
        }
    }

    fn format_report_roll(&self, roll: i32, _thrower: &Player) -> String {
        format!("Pass Roll [ {} ]", roll)
    }

    fn format_roll_requirement(&self, distance: PassingDistance, formatted_modifiers: &str, thrower: &Player) -> String {
        let ag = thrower.agility_with_modifiers().min(6);
        let m2016 = distance.modifier_2016();
        let sign = if m2016 >= 0 { " + " } else { " - " };
        format!(
            " (AG{}{}{} {}{}+ Roll > 6).",
            ag, sign, m2016.abs(), distance.name(), formatted_modifiers
        )
    }

    fn eligible_to_re_roll(&self, re_rolled_action_name: &str, _thrower: &Player) -> bool {
        re_rolled_action_name != "pass"
    }

    fn pass_modifiers(&self, _game: &Game, _player: &Player) -> i32 {
        // TODO: UtilPlayer::find_tacklezones(game, player)
        0
    }
}

impl PassMechanic {
    pub fn new() -> Self { PassMechanic }

    fn minimum_roll_internal(&self, thrower: &Player, distance: PassingDistance, modifiers: &[PassModifier]) -> i32 {
        let modifier_total = self.calculate_modifiers(modifiers);
        let m2016 = distance.modifier_2016();
        let base1 = (2 - (m2016 - modifier_total)).max(2);
        let base2 = (7 - thrower.agility_with_modifiers().min(6) - m2016 + modifier_total).max(2);
        base1.max(base2)
    }

    fn is_modified_fumble(&self, roll: i32, distance: PassingDistance, modifiers: &[PassModifier]) -> bool {
        (roll + distance.modifier_2016() - self.calculate_modifiers(modifiers)) <= 1
    }
}
