/// Translation of com.fumbbl.ffb.server.injury.modification.CrushingBlowModification.
use ffb_model::model::SkillUse;
use crate::injury::modification::{InjuryContextModification, ModificationParams};

pub struct CrushingBlowModification {
    skill_id: Option<u16>,
}

impl CrushingBlowModification {
    pub fn new() -> Self { Self { skill_id: None } }
}

impl Default for CrushingBlowModification {
    fn default() -> Self { Self::new() }
}

impl InjuryContextModification for CrushingBlowModification {
    fn skill_use(&self) -> SkillUse { SkillUse::ADD_ARMOUR_MODIFIER }
    fn valid_types(&self) -> &'static [&'static str] { &["Block"] }
    fn skill_id(&self) -> Option<u16> { self.skill_id }
    fn set_skill_id(&mut self, id: u16) { self.skill_id = Some(id); }

    /// Java: (!armorBroken || (armorBroken && mbUsed)) && hasTacklezones.
    /// mbUsed = true when a MightyBlow modifier (affectsEitherArmourOrInjuryOnBlock) is present.
    /// In Rust, all Mighty Blow modifiers have "Mighty Blow" in their name — name check is exact.
    fn try_armour_roll_modification(&self, params: &ModificationParams) -> bool {
        let mb_present = params.new_context.armor_modifiers.iter()
            .any(|m| m.name.contains("Mighty Blow"));
        let gate = !params.new_context.armor_broken || mb_present;
        gate && self.acting_player_has_tacklezones(params.game)
    }

    /// Java: if armor broken, strip MB modifiers; then call super (= applyArmourModification).
    fn prepare_armour_params(&self, params: &mut ModificationParams) {
        if params.new_context.armor_broken {
            params.new_context.armor_modifiers.retain(|m|
                !m.name.contains("Mighty Blow") && !m.name.contains("mightyBlow")
            );
        }
        self.apply_armour_modification(params);
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
    fn valid_type_is_block() {
        assert!(CrushingBlowModification::new().is_valid_type("Block"));
        assert!(!CrushingBlowModification::new().is_valid_type("Stab"));
    }

    #[test]
    fn try_armour_false_when_broken_without_mb() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let mut rng = GameRng::new(1);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        ctx.armor_broken = true;
        let params = ModificationParams::new(&game, &mut rng, ctx, "Block");
        assert!(!CrushingBlowModification::new().try_armour_roll_modification(&params));
    }

    #[test]
    fn try_armour_false_when_not_broken_but_no_active_player() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let mut rng = GameRng::new(1);
        let ctx = InjuryContext::new(ApothecaryMode::Defender);
        let params = ModificationParams::new(&game, &mut rng, ctx, "Block");
        // No acting player → no tacklezones → false
        assert!(!CrushingBlowModification::new().try_armour_roll_modification(&params));
    }
}
