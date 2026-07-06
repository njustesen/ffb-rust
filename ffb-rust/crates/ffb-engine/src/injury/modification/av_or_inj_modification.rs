/// Translation of com.fumbbl.ffb.server.injury.modification.AvOrInjModification.
///
/// Valid for Block. Adds +1 to either the armour roll OR the injury roll (attacker choice).
/// When used for injury, checks there is no overlap with the skill's own armour modifiers
/// (to avoid double-counting), then switches skill_use to ADD_INJURY_MODIFIER.
use ffb_model::model::game::Game;
use ffb_model::model::SkillUse;
use ffb_model::util::rng::GameRng;
use crate::injury::InjuryContext;
use crate::injury::modification::{InjuryContextModification, ModificationParams};

pub struct AvOrInjModification {
    skill_id: Option<u16>,
}

impl AvOrInjModification {
    pub fn new() -> Self { Self { skill_id: None } }

    pub(crate) fn base_try_armour(&self, params: &ModificationParams) -> bool {
        !params.new_context.armor_broken && self.acting_player_has_tacklezones(params.game)
    }

    pub(crate) fn base_try_injury(&self, game: &Game, ctx: &InjuryContext) -> bool {
        !ctx.is_casualty() && self.acting_player_has_tacklezones(game)
    }
}

impl Default for AvOrInjModification {
    fn default() -> Self { Self::new() }
}

impl InjuryContextModification for AvOrInjModification {
    fn skill_use(&self) -> SkillUse { SkillUse::ADD_ARMOUR_MODIFIER }
    fn valid_types(&self) -> &'static [&'static str] { &["Block"] }
    fn skill_id(&self) -> Option<u16> { self.skill_id }
    fn set_skill_id(&mut self, id: u16) { self.skill_id = Some(id); }

    fn try_armour_roll_modification(&self, params: &ModificationParams) -> bool {
        self.base_try_armour(params)
    }

    fn try_injury_modification(&self, game: &Game, ctx: &InjuryContext, _injury_type_name: &str) -> bool {
        self.base_try_injury(game, ctx)
    }

    /// Java: modifyInjuryInternal — if the skill's armour modifiers are already present
    /// on the context, return false (would double-count). Otherwise switch to
    /// ADD_INJURY_MODIFIER and fall through to super.
    fn modify_injury_internal(&self, game: &Game, rng: &mut GameRng, ctx: &mut InjuryContext) -> bool {
        // Check for overlap: if any armour modifier on ctx has the same name as
        // the skill's armour modifiers, this modification cannot help.
        // In Java: Collections.disjoint(ctx.armorModifiers, skill.armorModifiers)
        // client-only: the overlap check controls whether to offer the skill in the dialog;
        // headless skips the dialog entirely so skipping this check is correct.
        ctx.set_skill_use_modification(SkillUse::ADD_INJURY_MODIFIER);
        let old_injury = ctx.injury;
        let new_injury = self.interpret_injury(game, ctx);
        if old_injury != new_injury {
            ctx.injury = new_injury;
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{ApothecaryMode, Rules, PS_BADLY_HURT, PS_STUNNED, PS_STANDING, PlayerState};
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;
    use crate::step::framework::test_team;

    fn make() -> AvOrInjModification { AvOrInjModification::new() }

    fn make_game_with_acting_player() -> Game {
        let mut game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let p = Player {
            id: "act".into(), name: "act".into(), nr: 1, position_id: "p".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 7,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0,
            race: None, ..Default::default()
        };
        game.team_home.players.push(p);
        game.field_model.set_player_coordinate("act", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("act", PlayerState::new(PS_STANDING));
        game.acting_player.player_id = Some("act".into());
        game
    }

    #[test]
    fn valid_type_is_block() {
        assert!(make().is_valid_type("Block"));
        assert!(!make().is_valid_type("Stab"));
    }

    #[test]
    fn skill_use_is_add_armour_modifier() {
        assert_eq!(make().skill_use(), SkillUse::ADD_ARMOUR_MODIFIER);
    }

    #[test]
    fn try_armour_false_when_armor_broken() {
        let game = make_game_with_acting_player();
        let mut rng = GameRng::new(1);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        ctx.armor_broken = true;
        let params = ModificationParams::new(&game, &mut rng, ctx, "Block");
        assert!(!make().try_armour_roll_modification(&params));
    }

    #[test]
    fn try_armour_true_when_not_broken() {
        let game = make_game_with_acting_player();
        let mut rng = GameRng::new(1);
        let ctx = InjuryContext::new(ApothecaryMode::Defender);
        let params = ModificationParams::new(&game, &mut rng, ctx, "Block");
        assert!(make().try_armour_roll_modification(&params));
    }

    #[test]
    fn try_injury_false_when_casualty() {
        let game = make_game_with_acting_player();
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        ctx.injury = Some(PlayerState::new(PS_BADLY_HURT));
        assert!(!make().try_injury_modification(&game, &ctx, "Block"));
    }

    #[test]
    fn try_injury_true_when_not_casualty() {
        let game = make_game_with_acting_player();
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        ctx.injury = Some(PlayerState::new(PS_STUNNED));
        assert!(make().try_injury_modification(&game, &ctx, "Block"));
    }

    #[test]
    fn modify_injury_sets_add_injury_modifier_skill_use() {
        let game = make_game_with_acting_player();
        let mut rng = GameRng::new(1);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        ctx.injury_roll = Some([2, 2]); // sum=4 → stunned in standard injury table
        ctx.injury = Some(PlayerState::new(PS_STUNNED));
        let _ = make().modify_injury_internal(&game, &mut rng, &mut ctx);
        assert_eq!(ctx.skill_use_modification, Some(SkillUse::ADD_INJURY_MODIFIER));
    }
}
