/// Translation of com.fumbbl.ffb.server.injury.modification.SavageMaulingModification.
///
/// Valid for many injury types (Block, BlockStunned, BlockProne, Foul, FoulForSpp,
/// ProjectileVomit, Stab). Re-rolls the injury. allowed = always true (same-team ok).
/// Gate: not casualty, OR spotted foul (injury dice equal + foul type), OR
/// (same team + not AnimalSavagery + not stunned + has tacklezones).
use ffb_model::enums::ApothecaryMode;
use ffb_model::model::game::Game;
use ffb_model::model::SkillUse;
use ffb_model::util::rng::GameRng;
use crate::injury::InjuryContext;
use crate::injury::modification::{InjuryContextModification, ModificationParams};

pub struct SavageMaulingModification {
    skill_id: Option<u16>,
}

impl SavageMaulingModification {
    pub fn new() -> Self { Self { skill_id: None } }

    fn is_spotted_foul(ctx: &InjuryContext, injury_type_name: &str) -> bool {
        if let Some([d1, d2]) = ctx.injury_roll {
            let is_foul = matches!(injury_type_name, "Foul" | "FoulForSpp" | "FoulWithChainsaw" | "FoulForSppWithChainsaw");
            d1 == d2 && is_foul
        } else {
            false
        }
    }
}

impl Default for SavageMaulingModification {
    fn default() -> Self { Self::new() }
}

const VALID: &[&str] = &["Block", "BlockStunned", "BlockProne", "Foul", "FoulForSpp", "ProjectileVomit", "Stab"];

impl InjuryContextModification for SavageMaulingModification {
    fn skill_use(&self) -> SkillUse { SkillUse::RE_ROLL_INJURY }
    fn valid_types(&self) -> &'static [&'static str] { VALID }
    fn skill_id(&self) -> Option<u16> { self.skill_id }
    fn set_skill_id(&mut self, id: u16) { self.skill_id = Some(id); }

    fn allowed_for_attacker_and_defender_teams(&self, _game: &Game, _ctx: &InjuryContext) -> bool {
        true
    }

    fn try_injury_modification(&self, game: &Game, ctx: &InjuryContext, injury_type_name: &str) -> bool {
        if !ctx.is_casualty() {
            return true;
        }
        if Self::is_spotted_foul(ctx, injury_type_name) {
            return true;
        }
        // same team + not animal savagery + not stunned + has tacklezones
        let same_team = !self.different_teams(game, ctx);
        let not_savagery = ctx.apothecary_mode != ApothecaryMode::AnimalSavagery;
        let not_stunned = ctx.injury.map(|s| !s.is_stunned()).unwrap_or(true);
        same_team && not_savagery && not_stunned && self.acting_player_has_tacklezones(game)
    }

    fn modify_injury_internal(&self, game: &Game, rng: &mut GameRng, ctx: &mut InjuryContext) -> bool {
        let d1 = rng.d6();
        let d2 = rng.d6();
        ctx.set_injury_roll([d1, d2]);
        ctx.injury = self.interpret_injury(game, ctx);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, PS_STUNNED, PlayerState};
    use crate::step::framework::test_team;

    fn make() -> SavageMaulingModification { SavageMaulingModification::new() }

    #[test]
    fn valid_types() {
        let m = make();
        assert!(m.is_valid_type("Block"));
        assert!(m.is_valid_type("Foul"));
        assert!(m.is_valid_type("Stab"));
        assert!(!m.is_valid_type("Chainsaw"));
    }

    #[test]
    fn allows_same_team() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let ctx = InjuryContext::new(ApothecaryMode::Defender);
        assert!(make().allowed_for_attacker_and_defender_teams(&game, &ctx));
    }

    #[test]
    fn try_injury_true_when_not_casualty() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        ctx.injury = Some(PlayerState::new(PS_STUNNED));
        assert!(make().try_injury_modification(&game, &ctx, "Block"));
    }
}
