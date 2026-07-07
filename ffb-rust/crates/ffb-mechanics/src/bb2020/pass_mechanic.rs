use ffb_model::enums::{PassingDistance, TurnMode};
use ffb_model::model::{Game, Player};
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::util_player::UtilPlayer;
use crate::mechanic::{Mechanic, MechanicType};
use crate::modifiers::{PassModifier, StatBasedRollModifier};
use crate::pass_result::PassResult;
use crate::pass_mechanic::PassMechanic as PassMechanicTrait;

/// 1:1 translation of com.fumbbl.ffb.mechanics.bb2020.PassMechanic.
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
            "Q Q Q S S S S L L L L B B B".into(),
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
            "B B B                      ".into(),
        ]
    }

    fn minimum_roll(
        &self,
        thrower: &Player,
        distance: PassingDistance,
        modifiers: &[PassModifier],
        stat_based_roll_modifier: Option<&StatBasedRollModifier>,
    ) -> Option<i32> {
        let pa = thrower.passing_with_modifiers();
        if pa > 0 {
            let mut roll = pa + distance.modifier_2020() + modifiers.iter().map(|m| m.get_modifier()).sum::<i32>();
            if let Some(sbm) = stat_based_roll_modifier {
                roll += sbm.get_modifier();
            }
            Some(roll.max(2))
        } else {
            None
        }
    }

    fn minimum_roll_simple(
        &self,
        thrower: &Player,
        distance: PassingDistance,
        modifiers: &[PassModifier],
    ) -> Option<i32> {
        self.minimum_roll(thrower, distance, modifiers, None)
    }

    fn evaluate_pass(
        &self,
        thrower: &Player,
        roll: i32,
        distance: PassingDistance,
        modifiers: &[PassModifier],
        _bomb_action: bool,
        stat_based_roll_modifier: Option<&StatBasedRollModifier>,
    ) -> PassResult {
        let pa = thrower.passing_with_modifiers();
        let mut result_after_modifiers = roll - self.calculate_modifiers(modifiers) - distance.modifier_2020();
        if let Some(sbm) = stat_based_roll_modifier {
            result_after_modifiers += sbm.get_modifier();
        }
        if pa <= 0 || roll == 1 {
            if thrower.has_skill_property(NamedProperties::DONT_DROP_FUMBLES) {
                PassResult::SAVED_FUMBLE
            } else {
                PassResult::FUMBLE
            }
        } else if roll == 6 || result_after_modifiers >= pa {
            PassResult::ACCURATE
        } else if result_after_modifiers <= 1 {
            PassResult::WILDLY_INACCURATE
        } else {
            PassResult::INACCURATE
        }
    }

    fn evaluate_pass_simple(
        &self,
        thrower: &Player,
        roll: i32,
        distance: PassingDistance,
        modifiers: &[PassModifier],
        bomb_action: bool,
    ) -> PassResult {
        self.evaluate_pass(thrower, roll, distance, modifiers, bomb_action, None)
    }

    fn format_report_roll(&self, roll: i32, thrower: &Player) -> String {
        if thrower.passing_with_modifiers() > 0 {
            format!("Pass Roll [ {} ]", roll)
        } else {
            format!("Pass fumbled automatically as {} has no Passing Ability", thrower.name)
        }
    }

    fn format_roll_requirement(&self, distance: PassingDistance, formatted_modifiers: &str, thrower: &Player) -> String {
        if thrower.passing_with_modifiers() <= 0 {
            return String::new();
        }
        format!(
            " (Roll - {} {} {} >= PA {}+).",
            distance.modifier_2020(),
            distance.name(),
            formatted_modifiers,
            thrower.passing_with_modifiers()
        )
    }

    fn eligible_to_re_roll(&self, re_rolled_action_name: &str, thrower: &Player) -> bool {
        re_rolled_action_name != "pass" && thrower.passing_with_modifiers() > 0
    }

    fn pass_modifiers(&self, game: &Game, player: &Player) -> i32 {
        let players = UtilPlayer::find_tacklezone_players(game, &player.id);
        let mut zones = players.len() as i32;
        let ap = &game.acting_player;
        if game.turn_mode == TurnMode::DumpOff {
            if let Some(ap_id) = ap.player_id.as_deref() {
                if ap.standing_up && players.iter().any(|id| id.as_str() == ap_id) {
                    zones -= 1;
                }
            }
        }
        zones
    }
}

impl PassMechanic {
    pub fn new() -> Self { PassMechanic }
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
    fn roll_one_without_safe_pass_is_fumble() {
        let m = PassMechanic::new();
        let thrower = make_thrower();
        let result = m.evaluate_pass(&thrower, 1, PassingDistance::ShortPass, &[], false, None);
        assert_eq!(result, PassResult::FUMBLE);
    }

    #[test]
    fn roll_one_with_safe_pass_is_saved_fumble() {
        let m = PassMechanic::new();
        let mut thrower = make_thrower();
        thrower.starting_skills.push(SkillWithValue::new(SkillId::SafePass));
        let result = m.evaluate_pass(&thrower, 1, PassingDistance::ShortPass, &[], false, None);
        assert_eq!(result, PassResult::SAVED_FUMBLE);
    }

    #[test]
    fn no_passing_ability_with_safe_pass_is_saved_fumble() {
        let m = PassMechanic::new();
        let mut thrower = make_thrower();
        thrower.passing = -1;
        thrower.starting_skills.push(SkillWithValue::new(SkillId::SafePass));
        // pa <= 0 branch applies SafePass check
        let result = m.evaluate_pass(&thrower, 3, PassingDistance::ShortPass, &[], false, None);
        assert_eq!(result, PassResult::SAVED_FUMBLE);
    }

    #[test]
    fn no_passing_ability_without_safe_pass_is_fumble() {
        let m = PassMechanic::new();
        let mut thrower = make_thrower();
        thrower.passing = -1;
        let result = m.evaluate_pass(&thrower, 3, PassingDistance::ShortPass, &[], false, None);
        assert_eq!(result, PassResult::FUMBLE);
    }
}
