/// Translation of com.fumbbl.ffb.server.injury.modification.bb2025.LoneFoulerModification.
///
/// Extends RerollArmourModification for foul injury types. Extra gate: no offensive or
/// defensive foul assists.
use ffb_model::model::SkillUse;
use ffb_model::util::util_player::UtilPlayer;
use crate::injury::modification::{InjuryContextModification, ModificationParams};
use crate::injury::modification::bb2025::reroll_armour_modification::RerollArmourModification;

pub struct LoneFoulerModification {
    inner: RerollArmourModification,
}

const VALID: &[&str] = &["Foul", "FoulForSpp", "FoulWithChainsaw", "FoulForSppWithChainsaw"];

impl LoneFoulerModification {
    pub fn new() -> Self { Self { inner: RerollArmourModification::with_types(VALID) } }
}

impl Default for LoneFoulerModification {
    fn default() -> Self { Self::new() }
}

impl InjuryContextModification for LoneFoulerModification {
    fn skill_use(&self) -> SkillUse { self.inner.skill_use() }
    fn valid_types(&self) -> &'static [&'static str] { VALID }
    fn skill_id(&self) -> Option<u16> { self.inner.skill_id() }
    fn set_skill_id(&mut self, id: u16) { self.inner.set_skill_id(id); }

    /// Java: no offensive OR defensive assists AND super.tryArmourRollModification (not broken).
    fn try_armour_roll_modification(&self, params: &ModificationParams) -> bool {
        let attacker_id = match params.new_context.attacker_id.as_deref() {
            Some(id) => id,
            None => return false,
        };
        let defender_id = match params.new_context.defender_id.as_deref() {
            Some(id) => id,
            None => return false,
        };
        let offensive = UtilPlayer::find_offensive_foul_assists(params.game, attacker_id, defender_id);
        let defensive = UtilPlayer::find_defensive_foul_assists(params.game, attacker_id, defender_id);
        let no_assists = offensive == 0 && defensive == 0;
        no_assists && self.inner.try_armour_roll_modification(params)
    }

    fn modify_armour_internal(&self, params: &mut ModificationParams) -> bool {
        self.inner.modify_armour_internal(params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{ApothecaryMode, Rules};
    use ffb_model::model::game::Game;
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;
    use crate::step::framework::test_team;
    use crate::injury::InjuryContext;

    fn make() -> LoneFoulerModification { LoneFoulerModification::new() }

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_player(game: &mut Game, home: bool, id: &str, coord: FieldCoordinate) {
        let p = Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "p".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 7,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0,
            race: None, ..Default::default()
        };
        if home { game.team_home.players.push(p); }
        else { game.team_away.players.push(p); }
        game.field_model.set_player_coordinate(id, coord);
    }

    #[test]
    fn valid_foul_types() {
        let m = make();
        assert!(m.is_valid_type("Foul"));
        assert!(m.is_valid_type("FoulForSpp"));
        assert!(m.is_valid_type("FoulWithChainsaw"));
        assert!(!m.is_valid_type("Block"));
    }

    #[test]
    fn skill_use_is_reroll_armour() {
        assert_eq!(make().skill_use(), SkillUse::RE_ROLL_ARMOUR);
    }

    #[test]
    fn try_armour_false_when_no_attacker_id() {
        let game = make_game();
        let mut rng = GameRng::new(1);
        let ctx = InjuryContext::new(ApothecaryMode::Defender); // no attacker_id
        let params = ModificationParams::new(&game, &mut rng, ctx, "Foul");
        assert!(!make().try_armour_roll_modification(&params));
    }

    #[test]
    fn try_armour_false_when_no_defender_id() {
        let game = make_game();
        let mut rng = GameRng::new(1);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        ctx.attacker_id = Some("att".into());
        // no defender_id set
        let params = ModificationParams::new(&game, &mut rng, ctx, "Foul");
        assert!(!make().try_armour_roll_modification(&params));
    }

    #[test]
    fn try_armour_true_when_no_assists_and_not_broken() {
        let mut game = make_game();
        add_player(&mut game, true, "att", FieldCoordinate::new(10, 5));
        add_player(&mut game, false, "def", FieldCoordinate::new(10, 6));
        // No adjacent players providing assists

        let mut rng = GameRng::new(1);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        ctx.attacker_id = Some("att".into());
        ctx.defender_id = Some("def".into());
        ctx.armor_broken = false;
        let params = ModificationParams::new(&game, &mut rng, ctx, "Foul");
        assert!(make().try_armour_roll_modification(&params));
    }

    #[test]
    fn try_armour_false_when_armor_broken_even_with_no_assists() {
        let mut game = make_game();
        add_player(&mut game, true, "att", FieldCoordinate::new(10, 5));
        add_player(&mut game, false, "def", FieldCoordinate::new(10, 6));

        let mut rng = GameRng::new(1);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        ctx.attacker_id = Some("att".into());
        ctx.defender_id = Some("def".into());
        ctx.armor_broken = true;
        let params = ModificationParams::new(&game, &mut rng, ctx, "Foul");
        assert!(!make().try_armour_roll_modification(&params));
    }
}
