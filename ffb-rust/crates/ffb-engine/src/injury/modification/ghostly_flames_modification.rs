/// Translation of com.fumbbl.ffb.server.injury.modification.GhostlyFlamesModification.
///
/// Valid for Chainsaw. Gate: armor not broken, acting player is blitzing AND is the
/// attacker AND has tacklezones. prepareArmourParams strips chainsaw-specific static
/// modifiers before applying GhostlyFlames's own modifier.
use ffb_model::model::SkillUse;
use crate::injury::modification::{InjuryContextModification, ModificationParams};

pub struct GhostlyFlamesModification {
    skill_id: Option<u16>,
}

impl GhostlyFlamesModification {
    pub fn new() -> Self { Self { skill_id: None } }
}

impl Default for GhostlyFlamesModification {
    fn default() -> Self { Self::new() }
}

impl InjuryContextModification for GhostlyFlamesModification {
    fn skill_use(&self) -> SkillUse { SkillUse::INCREASE_CHAINSAW_DAMAGE }
    fn valid_types(&self) -> &'static [&'static str] { &["Chainsaw"] }
    fn skill_id(&self) -> Option<u16> { self.skill_id }
    fn set_skill_id(&mut self, id: u16) { self.skill_id = Some(id); }

    /// Java: !armorBroken && isBlitzing && actingPlayer == attacker && hasTacklezones.
    fn try_armour_roll_modification(&self, params: &ModificationParams) -> bool {
        !params.new_context.armor_broken
            && self.acting_player_is_blitzing_attacker(params.game, &params.new_context)
            && self.acting_player_has_tacklezones(params.game)
    }

    /// Java: strip StaticArmourModifier where isChainsaw() == true; then call super.
    /// Rust: strip modifiers whose name indicates they are the chainsaw +3 modifier.
    fn prepare_armour_params(&self, params: &mut ModificationParams) {
        params.new_context.armor_modifiers.retain(|m| !m.name.contains("Chainsaw"));
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
    fn valid_type_is_chainsaw() {
        let m = GhostlyFlamesModification::new();
        assert!(m.is_valid_type("Chainsaw"));
        assert!(!m.is_valid_type("Block"));
    }

    #[test]
    fn try_armour_false_when_broken() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let mut rng = GameRng::new(1);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        ctx.armor_broken = true;
        let params = ModificationParams::new(&game, &mut rng, ctx, "Chainsaw");
        assert!(!GhostlyFlamesModification::new().try_armour_roll_modification(&params));
    }

    #[test]
    fn skill_use_is_increase_chainsaw_damage() {
        use ffb_model::model::SkillUse;
        assert_eq!(GhostlyFlamesModification::new().skill_use(), SkillUse::INCREASE_CHAINSAW_DAMAGE);
    }
}
