/// Translation of com.fumbbl.ffb.server.injury.modification.bb2020.SlayerModification.
///
/// Extends AvOrInjModification. Extra gate: defender has Strength >= 5.
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

    fn defender_has_st5_or_more(game: &Game, defender_id: Option<&str>) -> bool {
        defender_id
            .and_then(|id| game.player(id))
            .map(|p| p.strength_with_modifiers() >= 5)
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
            && Self::defender_has_st5_or_more(params.game, params.new_context.defender_id.as_deref())
    }

    fn try_injury_modification(&self, game: &Game, ctx: &InjuryContext, injury_type_name: &str) -> bool {
        self.base.base_try_injury(game, ctx)
            && Self::defender_has_st5_or_more(game, ctx.defender_id.as_deref())
    }

    fn modify_injury_internal(&self, game: &Game, rng: &mut ffb_model::util::rng::GameRng, ctx: &mut InjuryContext) -> bool {
        self.base.modify_injury_internal(game, rng, ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{ApothecaryMode, Rules, PS_STANDING, PlayerState};
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;
    use ffb_model::model::game::Game;
    use crate::step::framework::test_team;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    fn add_player(game: &mut Game, home: bool, id: &str, strength: i32) {
        let p = Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "p".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength, agility: 3, passing: 4, armour: 7,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0,
            race: None, ..Default::default()
        };
        if home { game.team_home.players.push(p); }
        else { game.team_away.players.push(p); }
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
        game.field_model.set_player_state(id, PlayerState::new(PS_STANDING));
    }

    #[test]
    fn valid_type_is_block() {
        assert!(SlayerModification::new().is_valid_type("Block"));
        assert!(!SlayerModification::new().is_valid_type("Stab"));
    }

    #[test]
    fn try_armour_false_when_no_defender() {
        let game = make_game();
        let mut rng = GameRng::new(1);
        let ctx = InjuryContext::new(ApothecaryMode::Defender);
        let params = ModificationParams::new(&game, &mut rng, ctx, "Block");
        assert!(!SlayerModification::new().try_armour_roll_modification(&params));
    }

    #[test]
    fn skill_use_is_add_armour_modifier() {
        assert_eq!(SlayerModification::new().skill_use(), SkillUse::ADD_ARMOUR_MODIFIER);
    }

    #[test]
    fn try_armour_false_when_defender_st4() {
        let mut game = make_game();
        add_player(&mut game, true, "act", 4);
        add_player(&mut game, false, "def", 4); // ST4 — below threshold
        game.acting_player.player_id = Some("act".into());

        let mut rng = GameRng::new(1);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        ctx.defender_id = Some("def".into());
        let params = ModificationParams::new(&game, &mut rng, ctx, "Block");
        assert!(!SlayerModification::new().try_armour_roll_modification(&params));
    }

    #[test]
    fn try_armour_true_when_defender_st5_and_acting_player_standing() {
        let mut game = make_game();
        add_player(&mut game, true, "act", 3);
        add_player(&mut game, false, "def", 5); // ST5 ✓
        game.acting_player.player_id = Some("act".into());

        let mut rng = GameRng::new(1);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        ctx.defender_id = Some("def".into());
        let params = ModificationParams::new(&game, &mut rng, ctx, "Block");
        assert!(SlayerModification::new().try_armour_roll_modification(&params));
    }

    #[test]
    fn try_injury_false_when_defender_st4() {
        let mut game = make_game();
        add_player(&mut game, true, "act", 3);
        add_player(&mut game, false, "def", 4);
        game.acting_player.player_id = Some("act".into());

        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        ctx.defender_id = Some("def".into());
        assert!(!SlayerModification::new().try_injury_modification(&game, &ctx, "Block"));
    }
}
