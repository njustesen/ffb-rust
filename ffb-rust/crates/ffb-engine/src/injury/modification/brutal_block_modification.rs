/// Translation of com.fumbbl.ffb.server.injury.modification.BrutalBlockModification.
///
/// Valid for Block injury type. Applies when acting player has tacklezones and the
/// injury is not a casualty. Adds +1 to the injury roll (ADD_INJURY_MODIFIER).
use ffb_model::model::game::Game;
use ffb_model::model::SkillUse;
use crate::injury::InjuryContext;
use crate::injury::modification::{InjuryContextModification, ModificationParams};

pub struct BrutalBlockModification {
    skill_id: Option<u16>,
}

impl BrutalBlockModification {
    pub fn new() -> Self { Self { skill_id: None } }
}

impl Default for BrutalBlockModification {
    fn default() -> Self { Self::new() }
}

impl InjuryContextModification for BrutalBlockModification {
    fn skill_use(&self) -> SkillUse { SkillUse::ADD_INJURY_MODIFIER }
    fn valid_types(&self) -> &'static [&'static str] { &["Block"] }
    fn skill_id(&self) -> Option<u16> { self.skill_id }
    fn set_skill_id(&mut self, id: u16) { self.skill_id = Some(id); }

    fn try_injury_modification(&self, game: &Game, ctx: &InjuryContext, _injury_type_name: &str) -> bool {
        !ctx.is_casualty() && self.acting_player_has_tacklezones(game)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{ApothecaryMode, Rules};
    use crate::step::framework::test_team;

    fn make() -> BrutalBlockModification { BrutalBlockModification::new() }

    #[test]
    fn valid_type_is_block() {
        assert!(make().is_valid_type("Block"));
        assert!(!make().is_valid_type("Stab"));
    }

    #[test]
    fn skill_use_is_add_injury_modifier() {
        assert_eq!(make().skill_use(), SkillUse::ADD_INJURY_MODIFIER);
    }

    #[test]
    fn try_injury_modification_false_when_casualty() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        use ffb_model::enums::{PlayerState, PS_SERIOUS_INJURY};
        ctx.injury = Some(PlayerState::new(PS_SERIOUS_INJURY));
        assert!(!make().try_injury_modification(&game, &ctx, "Block"));
    }
    #[test]
    fn skill_id_starts_as_none() {
        assert!(make().skill_id().is_none());
    }
    #[test]
    fn try_injury_modification_false_when_no_active_player() {
        // No acting player in game → no tacklezones → false even for non-casualty
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let ctx = InjuryContext::new(ApothecaryMode::Defender);
        assert!(!make().try_injury_modification(&game, &ctx, "Block"));
    }
}
