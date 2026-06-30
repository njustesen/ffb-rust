use ffb_model::model::Game;
use crate::modifiers::catch_modifier::CatchModifier;
use crate::modifiers::interception_modifier::InterceptionModifier;
use crate::modifiers::pass_modifier::PassModifier;
use crate::modifiers::dodge_modifier::DodgeModifier;
use crate::modifiers::pickup_modifier::PickupModifier;
use crate::modifiers::jump_modifier::JumpModifier;
use crate::modifiers::jump_up_modifier::JumpUpModifier;
use crate::modifiers::gaze_modifier::GazeModifier;
use crate::modifiers::go_for_it_modifier::GoForItModifier;
use crate::modifiers::right_stuff_modifier::RightStuffModifier;
use crate::modifiers::armor_modifier::ArmorModifier;
use crate::modifiers::injury_modifier::InjuryModifier;

/// 1:1 translation of com.fumbbl.ffb.modifiers.ModifierAggregator.
/// Aggregates modifiers from SkillFactory and active cards.
/// SkillFactory and UtilCards not yet translated; each method returns empty vec.
pub struct ModifierAggregator {
    // TODO: skill_factory: SkillFactory (not yet translated)
    // TODO: game reference
}

impl ModifierAggregator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn init(&mut self, _game: &Game) {
        // TODO: store game reference and initialize skill_factory via game.get_factory(FactoryType::SKILL)
    }

    pub fn get_catch_modifiers(&self) -> Vec<CatchModifier> {
        // TODO: merge skill_factory.get_skills().flat_map(|s| s.get_catch_modifiers())
        //       with UtilCards::find_all_cards(game).flat_map(|c| c.roll_modifiers()).filter_map(CatchModifier)
        Vec::new()
    }

    pub fn get_interception_modifiers(&self) -> Vec<InterceptionModifier> {
        Vec::new()
    }

    pub fn get_pass_modifiers(&self) -> Vec<PassModifier> {
        Vec::new()
    }

    pub fn get_dodge_modifiers(&self) -> Vec<DodgeModifier> {
        Vec::new()
    }

    pub fn get_pickup_modifiers(&self) -> Vec<PickupModifier> {
        Vec::new()
    }

    pub fn get_jump_modifiers(&self) -> Vec<JumpModifier> {
        Vec::new()
    }

    pub fn get_jump_up_modifiers(&self) -> Vec<JumpUpModifier> {
        Vec::new()
    }

    pub fn get_gaze_modifiers(&self) -> Vec<GazeModifier> {
        Vec::new()
    }

    pub fn get_go_for_it_modifiers(&self) -> Vec<GoForItModifier> {
        Vec::new()
    }

    pub fn get_right_stuff_modifiers(&self) -> Vec<RightStuffModifier> {
        Vec::new()
    }

    pub fn get_armour_modifiers(&self) -> Vec<Box<dyn ArmorModifier>> {
        Vec::new()
    }

    pub fn get_injury_modifiers(&self) -> Vec<Box<dyn InjuryModifier>> {
        Vec::new()
    }
}

impl Default for ModifierAggregator {
    fn default() -> Self { Self::new() }
}
