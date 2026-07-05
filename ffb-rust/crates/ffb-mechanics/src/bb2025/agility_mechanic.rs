use std::collections::HashSet;
use ffb_model::model::{Game, Player};
use crate::mechanic::{Mechanic, MechanicType};
use crate::modifiers::{
    CatchModifier, DodgeModifier, GazeModifier, InterceptionModifier,
    JumpModifier, JumpUpModifier, PickupModifier, RightStuffModifier,
    RollModifier, StatBasedRollModifier,
};
use crate::wording::Wording;
use crate::agility_mechanic::AgilityMechanic as AgilityMechanicTrait;

/// 1:1 translation of com.fumbbl.ffb.mechanics.bb2025.AgilityMechanic.
pub struct AgilityMechanic;

impl AgilityMechanic {
    pub fn new() -> Self { Self }

    fn minimum_roll_internal(&self, agility: i32, modifier_total: i32, stat_modifier: i32) -> i32 {
        (agility + modifier_total - stat_modifier).max(2)
    }

    fn format_result(&self, agility: i32, roll_modifiers: &[RollModifier], stat_based_roll_modifier: Option<&StatBasedRollModifier>) -> String {
        let stat_modifier_str = match stat_based_roll_modifier {
            Some(sbm) => format!(" + {} {}", sbm.get_modifier(), sbm.get_report_string()),
            None => String::new(),
        };
        format!(" (Roll{}{}  >= {}+)", self.format_roll_modifiers(roll_modifiers), stat_modifier_str, agility.max(2))
    }
}

impl Default for AgilityMechanic {
    fn default() -> Self { Self::new() }
}

impl Mechanic for AgilityMechanic {
    fn get_type(&self) -> MechanicType { MechanicType::AGILITY }
}

impl AgilityMechanicTrait for AgilityMechanic {
    fn minimum_roll_jump_up(&self, player: &Player, modifiers: &HashSet<JumpUpModifier>) -> i32 {
        let modifier_total: i32 = modifiers.iter().map(|m| m.get_modifier()).sum();
        self.minimum_roll_internal(player.agility_with_modifiers(), modifier_total, 0)
    }

    fn minimum_roll_dodge(&self, _game: &Game, player: &Player, dodge_modifiers: &HashSet<DodgeModifier>) -> i32 {
        let modifier_total: i32 = dodge_modifiers.iter().map(|m| m.get_modifier()).sum();
        self.minimum_roll_internal(player.agility_with_modifiers(), modifier_total, 0)
    }

    fn minimum_roll_dodge_with_stat(&self, game: &Game, player: &Player, dodge_modifiers: &HashSet<DodgeModifier>, stat_based_roll_modifier: Option<&StatBasedRollModifier>) -> i32 {
        let modifier_total: i32 = dodge_modifiers.iter().map(|m| m.get_modifier()).sum();
        let stat_modifier = stat_based_roll_modifier.map(|s| s.get_modifier()).unwrap_or(0);
        self.minimum_roll_internal(player.agility_with_modifiers(), modifier_total, stat_modifier)
    }

    fn minimum_roll_pickup(&self, player: &Player, pickup_modifiers: &HashSet<PickupModifier>) -> i32 {
        let modifier_total: i32 = pickup_modifiers.iter().map(|m| m.get_modifier()).sum();
        self.minimum_roll_internal(player.agility_with_modifiers(), modifier_total, 0)
    }

    fn minimum_roll_interception(&self, player: &Player, interception_modifiers: &HashSet<InterceptionModifier>) -> i32 {
        let modifier_total: i32 = interception_modifiers.iter().map(|m| m.get_modifier()).sum();
        self.minimum_roll_internal(player.agility_with_modifiers(), modifier_total, 0)
    }

    fn minimum_roll_jump(&self, player: &Player, jump_modifiers: &HashSet<JumpModifier>) -> i32 {
        let modifier_total: i32 = jump_modifiers.iter().map(|m| m.get_modifier()).sum();
        self.minimum_roll_internal(player.agility_with_modifiers(), modifier_total, 0)
    }

    fn minimum_roll_hypnotic_gaze(&self, _player: &Player, _gaze_modifiers: &HashSet<GazeModifier>) -> i32 {
        3
    }

    fn minimum_roll_catch(&self, player: &Player, catch_modifiers: &HashSet<CatchModifier>) -> i32 {
        let modifier_total: i32 = catch_modifiers.iter().map(|m| m.get_modifier()).sum();
        self.minimum_roll_internal(player.agility_with_modifiers(), modifier_total, 0)
    }

    fn minimum_roll_right_stuff(&self, player: &Player, right_stuff_modifiers: &HashSet<RightStuffModifier>) -> i32 {
        let modifier_total: i32 = right_stuff_modifiers.iter().map(|m| m.get_modifier()).sum();
        self.minimum_roll_internal(player.agility_with_modifiers(), modifier_total, 0)
    }

    fn minimum_roll_safe_throw(&self, player: &Player) -> i32 {
        self.minimum_roll_internal(player.agility_with_modifiers(), 0, 0)
    }

    fn minimum_roll(&self, base_value: i32, modifiers: &HashSet<RollModifier>) -> i32 {
        let modifier_total: i32 = modifiers.iter().map(|m| m.get_modifier()).sum();
        self.minimum_roll_internal(base_value, modifier_total, 0)
    }

    fn format_dodge_result(&self, roll_modifiers: &[RollModifier], player: &Player, stat_based_roll_modifier: Option<&StatBasedRollModifier>) -> String {
        self.format_result(player.agility_with_modifiers(), roll_modifiers, stat_based_roll_modifier)
    }

    fn format_jump_result(&self, roll_modifiers: &[RollModifier], player: &Player) -> String {
        self.format_result(player.agility_with_modifiers(), roll_modifiers, None)
    }

    fn format_jump_up_result(&self, roll_modifiers: &[RollModifier], player: &Player) -> String {
        self.format_result(player.agility_with_modifiers(), roll_modifiers, None)
    }

    fn format_safe_throw_result(&self, player: &Player) -> String {
        self.format_result(player.agility_with_modifiers(), &[], None)
    }

    fn format_right_stuff_result(&self, roll_modifiers: &[RollModifier], player: &Player) -> String {
        self.format_result(player.agility_with_modifiers(), roll_modifiers, None)
    }

    fn format_catch_result(&self, roll_modifiers: &[RollModifier], player: &Player) -> String {
        self.format_result(player.agility_with_modifiers(), roll_modifiers, None)
    }

    fn format_interception_result(&self, roll_modifiers: &[RollModifier], player: &Player) -> String {
        self.format_result(player.agility_with_modifiers(), roll_modifiers, None)
    }

    fn format_hypnotic_gaze_result(&self, roll_modifiers: &[RollModifier], player: &Player) -> String {
        // TODO: player.get_skill_int_value(NamedProperties.inflictsConfusion) when SkillProperty lookup is ready
        let variable_value = player.get_skill_int_value("inflictsConfusion");
        let base = if variable_value == 0 { player.agility_with_modifiers() } else { variable_value };
        self.format_result(base, roll_modifiers, None)
    }

    fn format_pickup_result(&self, roll_modifiers: &[RollModifier], player: &Player, is_secure_the_ball: bool) -> String {
        let base = if is_secure_the_ball { 2 } else { player.agility_with_modifiers() };
        self.format_result(base, roll_modifiers, None)
    }

    fn interception_wording(&self, _easy_intercept: bool) -> Wording {
        Wording::new("Interception", "intercept", "intercepts", "interceptor")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use crate::agility_mechanic::AgilityMechanic as Trait;

    fn player_with_agility(ag: i32) -> Player {
        Player {
            id: "p".into(), name: "p".into(), nr: 1,
            position_id: "pos".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: ag, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        }
    }

    #[test]
    fn minimum_roll_catch_ag3_no_modifiers() {
        assert_eq!(AgilityMechanic.minimum_roll_catch(&player_with_agility(3), &HashSet::new()), 3);
    }

    #[test]
    fn minimum_roll_catch_ag1_no_modifiers() {
        // floored at 2
        assert_eq!(AgilityMechanic.minimum_roll_catch(&player_with_agility(1), &HashSet::new()), 2);
    }

    #[test]
    fn minimum_roll_pickup_ag4_no_modifiers() {
        assert_eq!(AgilityMechanic.minimum_roll_pickup(&player_with_agility(4), &HashSet::new()), 4);
    }

    #[test]
    fn minimum_roll_hypnotic_gaze_is_3() {
        // bb2025 gaze always returns 3
        assert_eq!(AgilityMechanic.minimum_roll_hypnotic_gaze(&player_with_agility(5), &HashSet::new()), 3);
    }

    #[test]
    fn interception_wording_is_interception_regardless_of_easy() {
        // bb2025 always returns Interception wording (unlike bb2020 which returns Interference)
        let w_easy = AgilityMechanic.interception_wording(true);
        let w_hard = AgilityMechanic.interception_wording(false);
        assert_eq!(w_easy, w_hard);
    }
}
