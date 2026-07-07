/// Translation of com.fumbbl.ffb.server.injury.modification.bb2025.SlayerModification.
///
/// Extends AvOrInjModification. Extra gate: defender has the BigGuy keyword (PlayerType::BigGuy).
use ffb_model::enums::PlayerType;
use ffb_model::model::game::Game;
use ffb_model::model::SkillUse;
use crate::injury::InjuryContext;
use crate::injury::modification::{InjuryContextModification, ModificationParams};
use crate::injury::modification::av_or_inj_modification::AvOrInjModification;

pub struct SlayerModification {
    base: AvOrInjModification,
}

impl SlayerModification {
    pub fn new() -> Self { Self { base: AvOrInjModification::new() } }

    fn defender_is_big_guy(game: &Game, defender_id: Option<&str>) -> bool {
        defender_id
            .and_then(|id| game.player(id))
            .map(|p| p.player_type == PlayerType::BigGuy)
            .unwrap_or(false)
    }
}

impl Default for SlayerModification {
    fn default() -> Self { Self::new() }
}

impl InjuryContextModification for SlayerModification {
    fn skill_use(&self) -> SkillUse { SkillUse::ADD_ARMOUR_MODIFIER }
    fn valid_types(&self) -> &'static [&'static str] { &["Block"] }
    fn skill_id(&self) -> Option<u16> { self.base.skill_id() }
    fn set_skill_id(&mut self, id: u16) { self.base.set_skill_id(id); }

    fn try_armour_roll_modification(&self, params: &ModificationParams) -> bool {
        self.base.base_try_armour(params)
            && Self::defender_is_big_guy(params.game, params.new_context.defender_id.as_deref())
    }

    fn try_injury_modification(&self, game: &Game, ctx: &InjuryContext, injury_type_name: &str) -> bool {
        self.base.base_try_injury(game, ctx)
            && Self::defender_is_big_guy(game, ctx.defender_id.as_deref())
    }

    fn modify_injury_internal(&self, game: &Game, rng: &mut ffb_model::util::rng::GameRng, ctx: &mut InjuryContext) -> bool {
        self.base.modify_injury_internal(game, rng, ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{ApothecaryMode, Rules};
    use ffb_model::util::rng::GameRng;
    use ffb_model::model::game::Game;
    use crate::step::framework::test_team;

    #[test]
    fn valid_type_is_block() {
        assert!(SlayerModification::new().is_valid_type("Block"));
    }

    #[test]
    fn try_armour_false_without_big_guy_defender() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let mut rng = GameRng::new(1);
        let ctx = InjuryContext::new(ApothecaryMode::Defender);
        let params = ModificationParams::new(&game, &mut rng, ctx, "Block");
        assert!(!SlayerModification::new().try_armour_roll_modification(&params));
    }

    #[test]
    fn stab_is_not_valid_type() {
        assert!(!SlayerModification::new().is_valid_type("Stab"));
    }
    #[test]
    fn skill_use_is_add_armour_modifier() {
        use ffb_model::model::SkillUse;
        assert_eq!(SlayerModification::new().skill_use(), SkillUse::ADD_ARMOUR_MODIFIER);
    }
    #[test]
    fn try_injury_false_without_big_guy_defender() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let ctx = InjuryContext::new(ApothecaryMode::Defender);
        assert!(!SlayerModification::new().try_injury_modification(&game, &ctx, "Block"));
    }
}
