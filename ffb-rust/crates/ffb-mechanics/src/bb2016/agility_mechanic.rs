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

/// 1:1 translation of com.fumbbl.ffb.mechanics.bb2016.AgilityMechanic.
pub struct AgilityMechanic;

impl AgilityMechanic {
    pub fn new() -> Self { Self }

    /// 1:1 translation of getAgilityRollBase: `7 - min(agility, 6)`.
    fn agility_roll_base(&self, agility: i32) -> i32 {
        7 - agility.min(6)
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
        (self.agility_roll_base(player.agility_with_modifiers()) + modifier_total).max(2)
    }

    fn minimum_roll_dodge(&self, _game: &Game, player: &Player, dodge_modifiers: &HashSet<DodgeModifier>) -> i32 {
        let modifier_total: i32 = dodge_modifiers.iter().map(|m| m.get_modifier()).sum();
        let statistic = if dodge_modifiers.iter().any(|m| m.is_use_strength()) {
            player.strength_with_modifiers()
        } else {
            player.agility_with_modifiers()
        };
        (self.agility_roll_base(statistic) - 1 + modifier_total).max(2)
    }

    fn minimum_roll_dodge_with_stat(&self, game: &Game, player: &Player, dodge_modifiers: &HashSet<DodgeModifier>, _stat_based_roll_modifier: Option<&StatBasedRollModifier>) -> i32 {
        self.minimum_roll_dodge(game, player, dodge_modifiers)
    }

    fn minimum_roll_pickup(&self, player: &Player, pickup_modifiers: &HashSet<PickupModifier>) -> i32 {
        let modifier_total: i32 = pickup_modifiers.iter().map(|m| m.get_modifier()).sum();
        (self.agility_roll_base(player.agility_with_modifiers()) - 1 + modifier_total).max(2)
    }

    fn minimum_roll_interception(&self, player: &Player, interception_modifiers: &HashSet<InterceptionModifier>) -> i32 {
        let modifier_total: i32 = interception_modifiers.iter().map(|m| m.get_modifier()).sum();
        (self.agility_roll_base(player.agility_with_modifiers()) + 2 + modifier_total).max(2)
    }

    fn minimum_roll_jump(&self, player: &Player, jump_modifiers: &HashSet<JumpModifier>) -> i32 {
        let modifier_total: i32 = jump_modifiers.iter().map(|m| m.get_modifier()).sum();
        (self.agility_roll_base(player.agility_with_modifiers()) + modifier_total).max(2)
    }

    fn minimum_roll_hypnotic_gaze(&self, player: &Player, gaze_modifiers: &HashSet<GazeModifier>) -> i32 {
        let modifier_total: i32 = gaze_modifiers.iter().map(|m| m.get_modifier()).sum();
        (self.agility_roll_base(player.agility_with_modifiers()) + modifier_total).max(2)
    }

    fn minimum_roll_catch(&self, player: &Player, catch_modifiers: &HashSet<CatchModifier>) -> i32 {
        let modifier_total: i32 = catch_modifiers.iter().map(|m| m.get_modifier()).sum();
        (self.agility_roll_base(player.agility_with_modifiers()) + modifier_total).max(2)
    }

    fn minimum_roll_right_stuff(&self, player: &Player, right_stuff_modifiers: &HashSet<RightStuffModifier>) -> i32 {
        let modifier_total: i32 = right_stuff_modifiers.iter().map(|m| m.get_modifier()).sum();
        (self.agility_roll_base(player.agility_with_modifiers()) + modifier_total).max(2)
    }

    fn minimum_roll_safe_throw(&self, player: &Player) -> i32 {
        self.agility_roll_base(player.agility_with_modifiers()).max(2)
    }

    fn minimum_roll(&self, _base_value: i32, _modifiers: &HashSet<RollModifier>) -> i32 {
        unimplemented!("BB2016 AgilityMechanic.minimumRoll(int, Set) throws UnsupportedOperationException in Java")
    }

    fn format_dodge_result(&self, roll_modifiers: &[RollModifier], player: &Player, _stat_based_roll_modifier: Option<&StatBasedRollModifier>) -> String {
        // headless: Break Tackle format — RollModifier doesn't carry is_use_strength flag;
        // blocked until format interface carries DodgeModifier directly for report generation.
        let uses_strength = false;
        let mut result = String::new();
        if uses_strength {
            result.push_str(&format!(" using Break Tackle (ST {}", player.strength_with_modifiers().min(6)));
        } else {
            result.push_str(&format!(" (AG {}", player.agility_with_modifiers().min(6)));
        }
        result.push_str(" + 1 Dodge");
        result.push_str(&self.format_roll_modifiers(roll_modifiers));
        result.push_str(" + Roll > 6).");
        result
    }

    fn format_jump_result(&self, roll_modifiers: &[RollModifier], player: &Player) -> String {
        format!(" (AG {}{}+ Roll > 6).",
            player.agility_with_modifiers().min(6),
            self.format_roll_modifiers(roll_modifiers))
    }

    fn format_jump_up_result(&self, roll_modifiers: &[RollModifier], player: &Player) -> String {
        format!(" (AG {}{}+ Roll > 6).",
            player.agility_with_modifiers().min(6),
            self.format_roll_modifiers(roll_modifiers))
    }

    fn format_safe_throw_result(&self, player: &Player) -> String {
        format!(" (AG {} + Roll > 6).", player.agility_with_modifiers().min(6))
    }

    fn format_right_stuff_result(&self, roll_modifiers: &[RollModifier], player: &Player) -> String {
        format!(" (AG {}{}+ Roll > 6).",
            player.agility_with_modifiers().min(6),
            self.format_roll_modifiers(roll_modifiers))
    }

    fn format_catch_result(&self, roll_modifiers: &[RollModifier], player: &Player) -> String {
        format!(" (AG {}{}+ Roll > 6).",
            player.agility_with_modifiers().min(6),
            self.format_roll_modifiers(roll_modifiers))
    }

    fn format_interception_result(&self, roll_modifiers: &[RollModifier], player: &Player) -> String {
        format!(" (AG {} - 2 Interception{}+ Roll > 6).",
            player.agility_with_modifiers().min(6),
            self.format_roll_modifiers(roll_modifiers))
    }

    fn format_hypnotic_gaze_result(&self, roll_modifiers: &[RollModifier], player: &Player) -> String {
        format!(" (AG {}{}+ Roll > 6).",
            player.agility_with_modifiers().min(6),
            self.format_roll_modifiers(roll_modifiers))
    }

    fn format_pickup_result(&self, roll_modifiers: &[RollModifier], player: &Player, _is_secure_the_ball: bool) -> String {
        format!(" (AG {} + 1 Pickup{}+ Roll > 6).",
            player.agility_with_modifiers().min(6),
            self.format_roll_modifiers(roll_modifiers))
    }

    fn interception_wording(&self, _easy_intercept: bool) -> Wording {
        Wording::new("Interception", "intercept", "intercepts", "interceptor")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agility_mechanic::AgilityMechanic as Trait;

    fn player_with_agility(ag: i32) -> Player {
        use ffb_model::enums::{PlayerType, PlayerGender};
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
            is_big_guy: false,
            ..Default::default()
        }
    }

    #[test]
    fn minimum_roll_catch_ag3_no_modifiers() {
        // agility_roll_base(3) = 7 - 3 = 4; no modifier → 4
        let m = AgilityMechanic::new();
        let p = player_with_agility(3);
        assert_eq!(m.minimum_roll_catch(&p, &HashSet::new()), 4);
    }

    #[test]
    fn minimum_roll_dodge_ag3_no_modifiers() {
        // base = 7 - 3 = 4; dodge gets -1 → 3
        let m = AgilityMechanic::new();
        let p = player_with_agility(3);
        let game = ffb_model::model::Game::new(
            ffb_model::model::Team { id: "h".into(), name: "h".into(), race: "H".into(), roster_id: "r".into(), coach: "c".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0, special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false },
            ffb_model::model::Team { id: "a".into(), name: "a".into(), race: "H".into(), roster_id: "r".into(), coach: "c".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0, special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false },
            ffb_model::enums::Rules::Bb2016,
        );
        assert_eq!(m.minimum_roll_dodge(&game, &p, &HashSet::new()), 3);
    }

    #[test]
    fn minimum_roll_interception_ag3_no_modifiers() {
        // base = 7 - 3 = 4; +2 interception → 6
        let m = AgilityMechanic::new();
        let p = player_with_agility(3);
        assert_eq!(m.minimum_roll_interception(&p, &HashSet::new()), 6);
    }

    #[test]
    fn minimum_roll_safe_throw_ag3() {
        // base = 7 - 3 = 4; safe throw = base → 4
        let m = AgilityMechanic::new();
        let p = player_with_agility(3);
        assert_eq!(m.minimum_roll_safe_throw(&p), 4);
    }

    #[test]
    fn minimum_roll_never_below_two() {
        // AG 6 → base = 1; dodge = 1 - 1 = 0 → clamped to 2
        let m = AgilityMechanic::new();
        let p = player_with_agility(6);
        let game = ffb_model::model::Game::new(
            ffb_model::model::Team { id: "h".into(), name: "h".into(), race: "H".into(), roster_id: "r".into(), coach: "c".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0, special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false },
            ffb_model::model::Team { id: "a".into(), name: "a".into(), race: "H".into(), roster_id: "r".into(), coach: "c".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0, special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false },
            ffb_model::enums::Rules::Bb2016,
        );
        assert_eq!(m.minimum_roll_dodge(&game, &p, &HashSet::new()), 2);
    }
}
