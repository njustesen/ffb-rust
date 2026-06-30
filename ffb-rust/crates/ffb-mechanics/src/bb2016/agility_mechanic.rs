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
        let uses_strength = roll_modifiers.iter().any(|_m| {
            // TODO: check DodgeModifier::isUseStrength when DodgeModifier is fully translated
            false
        });
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
