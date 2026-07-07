/// Translation of com.fumbbl.ffb.server.injury.modification.bb2025.ToxinConnoisseurModification.
///
/// Valid for Stab and StabForSpp (BB2025). Gate: not a casualty. ADD_INJURY_MODIFIER.
use ffb_model::model::game::Game;
use ffb_model::model::SkillUse;
use crate::injury::InjuryContext;
use crate::injury::modification::InjuryContextModification;

pub struct ToxinConnoisseurModification {
    skill_id: Option<u16>,
}

impl ToxinConnoisseurModification {
    pub fn new() -> Self { Self { skill_id: None } }
}

impl Default for ToxinConnoisseurModification {
    fn default() -> Self { Self::new() }
}

impl InjuryContextModification for ToxinConnoisseurModification {
    fn skill_use(&self) -> SkillUse { SkillUse::ADD_INJURY_MODIFIER }
    fn valid_types(&self) -> &'static [&'static str] { &["Stab", "StabForSpp"] }
    fn skill_id(&self) -> Option<u16> { self.skill_id }
    fn set_skill_id(&mut self, id: u16) { self.skill_id = Some(id); }

    fn try_injury_modification(&self, _game: &Game, ctx: &InjuryContext, _injury_type_name: &str) -> bool {
        !ctx.is_casualty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{ApothecaryMode, Rules, PS_SERIOUS_INJURY, PlayerState};
    use ffb_model::model::game::Game;
    use crate::step::framework::test_team;

    #[test]
    fn valid_types() {
        let m = ToxinConnoisseurModification::new();
        assert!(m.is_valid_type("Stab"));
        assert!(m.is_valid_type("StabForSpp"));
        assert!(!m.is_valid_type("Block"));
    }

    #[test]
    fn try_injury_false_when_casualty() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        ctx.injury = Some(PlayerState::new(PS_SERIOUS_INJURY));
        assert!(!ToxinConnoisseurModification::new().try_injury_modification(&game, &ctx, "Stab"));
    }

    #[test]
    fn try_injury_true_when_not_casualty() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let ctx = InjuryContext::new(ApothecaryMode::Defender);
        assert!(ToxinConnoisseurModification::new().try_injury_modification(&game, &ctx, "StabForSpp"));
    }
    #[test]
    fn skill_use_is_add_injury_modifier() {
        use ffb_model::model::SkillUse;
        assert_eq!(ToxinConnoisseurModification::new().skill_use(), SkillUse::ADD_INJURY_MODIFIER);
    }
    #[test]
    fn skill_id_starts_as_none() {
        assert!(ToxinConnoisseurModification::new().skill_id().is_none());
    }
}
