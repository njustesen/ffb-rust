use ffb_model::enums::PassingDistance;
use ffb_model::model::{Game, Player};
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::util_player::UtilPlayer;
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
            if !bomb_action && thrower.has_skill_property(NamedProperties::DONT_DROP_FUMBLES) {
                PassResult::SAVED_FUMBLE
            } else {
                PassResult::FUMBLE
            }
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

    fn pass_modifiers(&self, game: &Game, player: &Player) -> i32 {
        UtilPlayer::find_tacklezones(game, &player.id) as i32
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

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerType, SkillId};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;

    fn make_thrower() -> Player {
        Player {
            id: "t".into(), name: "t".into(), nr: 1, position_id: "p".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 5, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        }
    }

    #[test]
    fn natural_one_without_safe_pass_is_fumble() {
        // Natural 1 always fumbles regardless of SafePass
        let m = PassMechanic::new();
        let thrower = make_thrower();
        let result = m.evaluate_pass(&thrower, 1, PassingDistance::ShortPass, &[], false, None);
        assert_eq!(result, PassResult::FUMBLE);
    }

    #[test]
    fn modified_fumble_with_safe_pass_is_saved_fumble() {
        let m = PassMechanic::new();
        let mut thrower = make_thrower();
        thrower.starting_skills.push(SkillWithValue::new(SkillId::SafePass));
        // LongPass modifier_2016=-1: roll=2 → 2+(-1)-0=1 <= 1 → modified fumble → SAVED_FUMBLE
        let result = m.evaluate_pass(&thrower, 2, PassingDistance::LongPass, &[], false, None);
        assert_eq!(result, PassResult::SAVED_FUMBLE);
    }

    #[test]
    fn modified_fumble_without_safe_pass_is_fumble() {
        let m = PassMechanic::new();
        let thrower = make_thrower();
        // LongPass modifier_2016=-1: roll=2 → modified fumble, no SafePass → FUMBLE
        let result = m.evaluate_pass(&thrower, 2, PassingDistance::LongPass, &[], false, None);
        assert_eq!(result, PassResult::FUMBLE);
    }

    #[test]
    fn bomb_action_modified_fumble_is_fumble_even_with_safe_pass() {
        let m = PassMechanic::new();
        let mut thrower = make_thrower();
        thrower.starting_skills.push(SkillWithValue::new(SkillId::SafePass));
        // bomb_action=true → SafePass doesn't apply to modified fumble
        let result = m.evaluate_pass(&thrower, 2, PassingDistance::LongPass, &[], true, None);
        assert_eq!(result, PassResult::FUMBLE);
    }
}
