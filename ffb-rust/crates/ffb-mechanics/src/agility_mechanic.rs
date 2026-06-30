use std::collections::HashSet;
use ffb_model::model::Player;
use ffb_model::model::Game;
use crate::mechanic::{Mechanic, MechanicType};
use crate::modifiers::{
    CatchModifier, DodgeModifier, GazeModifier, InterceptionModifier,
    JumpModifier, JumpUpModifier, PickupModifier, RightStuffModifier,
    RollModifier, StatBasedRollModifier,
};
use crate::wording::Wording;

/// 1:1 translation of com.fumbbl.ffb.mechanics.AgilityMechanic.
pub trait AgilityMechanic: Mechanic {
    fn get_type(&self) -> MechanicType { MechanicType::AGILITY }

    fn minimum_roll_jump_up(&self, player: &Player, modifiers: &HashSet<JumpUpModifier>) -> i32;
    fn minimum_roll_dodge(&self, game: &Game, player: &Player, dodge_modifiers: &HashSet<DodgeModifier>) -> i32;
    fn minimum_roll_dodge_with_stat(&self, game: &Game, player: &Player, dodge_modifiers: &HashSet<DodgeModifier>, stat_based_roll_modifier: Option<&StatBasedRollModifier>) -> i32;
    fn minimum_roll_pickup(&self, player: &Player, pickup_modifiers: &HashSet<PickupModifier>) -> i32;
    fn minimum_roll_interception(&self, player: &Player, interception_modifiers: &HashSet<InterceptionModifier>) -> i32;
    fn minimum_roll_jump(&self, player: &Player, jump_modifiers: &HashSet<JumpModifier>) -> i32;
    fn minimum_roll_hypnotic_gaze(&self, player: &Player, gaze_modifiers: &HashSet<GazeModifier>) -> i32;
    fn minimum_roll_catch(&self, player: &Player, catch_modifiers: &HashSet<CatchModifier>) -> i32;
    fn minimum_roll_right_stuff(&self, player: &Player, right_stuff_modifiers: &HashSet<RightStuffModifier>) -> i32;
    fn minimum_roll_safe_throw(&self, player: &Player) -> i32;
    fn minimum_roll(&self, base_value: i32, modifiers: &HashSet<RollModifier>) -> i32;

    /// format_dodge_result — also accepts stat_based_roll_modifier for BB2020/BB2025.
    fn format_dodge_result(&self, roll_modifiers: &[RollModifier], player: &Player, stat_based_roll_modifier: Option<&StatBasedRollModifier>) -> String;
    fn format_jump_result(&self, roll_modifiers: &[RollModifier], player: &Player) -> String;
    fn format_jump_up_result(&self, roll_modifiers: &[RollModifier], player: &Player) -> String;
    fn format_safe_throw_result(&self, player: &Player) -> String;
    fn format_right_stuff_result(&self, roll_modifiers: &[RollModifier], player: &Player) -> String;
    fn format_catch_result(&self, roll_modifiers: &[RollModifier], player: &Player) -> String;
    fn format_interception_result(&self, roll_modifiers: &[RollModifier], player: &Player) -> String;
    fn format_hypnotic_gaze_result(&self, roll_modifiers: &[RollModifier], player: &Player) -> String;
    /// is_secure_the_ball: BB2025 uses base roll 2 instead of agility when SecureTheBall is active.
    fn format_pickup_result(&self, roll_modifiers: &[RollModifier], player: &Player, is_secure_the_ball: bool) -> String;
    fn interception_wording(&self, easy_intercept: bool) -> Wording;

    /// 1:1 translation of formatRollModifiers (concrete protected method in Java abstract class).
    fn format_roll_modifiers(&self, roll_modifiers: &[RollModifier]) -> String {
        let mut modifiers = String::new();
        for rm in roll_modifiers {
            if rm.get_modifier() > 0 {
                modifiers.push_str(" - ");
            } else {
                modifiers.push_str(" + ");
            }
            if !rm.is_modifier_included() {
                modifiers.push_str(&rm.get_modifier().unsigned_abs().to_string());
                modifiers.push(' ');
            }
            modifiers.push_str(rm.get_report_string());
        }
        modifiers
    }
}
