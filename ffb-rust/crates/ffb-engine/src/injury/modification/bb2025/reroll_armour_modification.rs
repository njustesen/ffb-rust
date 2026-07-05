/// Translation of com.fumbbl.ffb.server.injury.modification.bb2025.RerollArmourModification.
///
/// Abstract base for BB2025 armour re-roll modifications. Gate: armor not broken.
/// modify_armour_internal: re-roll 2d6 armour. skill_use = RE_ROLL_ARMOUR.
/// Concrete subclasses provide valid_types; this file also serves as a concrete type
/// via new() → Block valid type for the case where no subtype overrides it.
use ffb_model::model::game::Game;
use ffb_model::model::SkillUse;
use ffb_model::util::rng::GameRng;
use crate::injury::InjuryContext;
use crate::injury::modification::{InjuryContextModification, ModificationParams};

pub struct RerollArmourModification {
    skill_id: Option<u16>,
    valid_types: &'static [&'static str],
}

impl RerollArmourModification {
    pub fn new() -> Self { Self { skill_id: None, valid_types: &[] } }
    pub fn with_types(valid_types: &'static [&'static str]) -> Self {
        Self { skill_id: None, valid_types }
    }
}

impl Default for RerollArmourModification {
    fn default() -> Self { Self::new() }
}

impl InjuryContextModification for RerollArmourModification {
    fn skill_use(&self) -> SkillUse { SkillUse::RE_ROLL_ARMOUR }
    fn valid_types(&self) -> &'static [&'static str] { self.valid_types }
    fn skill_id(&self) -> Option<u16> { self.skill_id }
    fn set_skill_id(&mut self, id: u16) { self.skill_id = Some(id); }

    fn try_armour_roll_modification(&self, params: &ModificationParams) -> bool {
        !params.new_context.armor_broken
    }

    /// Java: re-roll 2d6 armour, always return true.
    fn modify_armour_internal(&self, params: &mut ModificationParams) -> bool {
        let d1 = params.rng.d6();
        let d2 = params.rng.d6();
        params.new_context.set_armor_roll([d1, d2]);
        // recalculate armor_broken after re-roll
        if let Some(defender_id) = params.new_context.defender_id.clone() {
            let armor_value = params.game.player(&defender_id).map(|p| p.armour).unwrap_or(7);
            params.new_context.armor_broken =
                ffb_mechanics::mechanics::armor_broken(armor_value, [d1, d2], &params.new_context.armor_modifiers);
        }
        true
    }
}

/// Trait extension helper used by subtype implementations (KrumpAndSmash, LoneFouler, etc.)
/// that inherit all behaviour but override valid_types.
pub trait RerollArmourBehaviour: InjuryContextModification {
    fn base_try_armour(&self, params: &ModificationParams) -> bool {
        !params.new_context.armor_broken
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{ApothecaryMode, Rules};
    use ffb_model::util::rng::GameRng;
    use ffb_model::model::game::Game;
    use crate::injury::InjuryContext;
    use crate::step::framework::test_team;

    #[test]
    fn try_armour_false_when_broken() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let mut rng = GameRng::new(1);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        ctx.armor_broken = true;
        let params = ModificationParams::new(&game, &mut rng, ctx, "Block");
        let m = RerollArmourModification::with_types(&["Block"]);
        assert!(!m.try_armour_roll_modification(&params));
    }

    #[test]
    fn try_armour_true_when_not_broken() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let mut rng = GameRng::new(1);
        let ctx = InjuryContext::new(ApothecaryMode::Defender);
        let params = ModificationParams::new(&game, &mut rng, ctx, "Block");
        let m = RerollArmourModification::with_types(&["Block"]);
        assert!(m.try_armour_roll_modification(&params));
    }
}
