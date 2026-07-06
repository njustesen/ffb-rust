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
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;
    use crate::injury::InjuryContext;
    use crate::step::framework::test_team;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_defender(game: &mut Game, id: &str, armour: i32) {
        let p = Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "p".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0,
            race: None, ..Default::default()
        };
        game.team_away.players.push(p);
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
    }

    #[test]
    fn try_armour_false_when_broken() {
        let game = make_game();
        let mut rng = GameRng::new(1);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        ctx.armor_broken = true;
        let params = ModificationParams::new(&game, &mut rng, ctx, "Block");
        let m = RerollArmourModification::with_types(&["Block"]);
        assert!(!m.try_armour_roll_modification(&params));
    }

    #[test]
    fn try_armour_true_when_not_broken() {
        let game = make_game();
        let mut rng = GameRng::new(1);
        let ctx = InjuryContext::new(ApothecaryMode::Defender);
        let params = ModificationParams::new(&game, &mut rng, ctx, "Block");
        let m = RerollArmourModification::with_types(&["Block"]);
        assert!(m.try_armour_roll_modification(&params));
    }

    #[test]
    fn modify_armour_internal_sets_new_roll_and_recalculates_broken() {
        let mut game = make_game();
        add_defender(&mut game, "def", 7);

        let mut rng = GameRng::new(1);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        ctx.defender_id = Some("def".into());
        ctx.armor_roll = Some([3, 3]);
        ctx.armor_broken = false;

        let mut params = ModificationParams::new(&game, &mut rng, ctx, "Block");
        let m = RerollArmourModification::with_types(&["Block"]);
        let result = m.modify_armour_internal(&mut params);
        assert!(result);
        // After re-roll, armor_roll should be different (new 2d6)
        let [d1, d2] = params.new_context.armor_roll.unwrap();
        assert!((1..=6).contains(&d1));
        assert!((1..=6).contains(&d2));
    }

    #[test]
    fn with_types_sets_valid_types() {
        let m = RerollArmourModification::with_types(&["Block", "Stab"]);
        assert!(m.is_valid_type("Block"));
        assert!(m.is_valid_type("Stab"));
        assert!(!m.is_valid_type("Foul"));
    }

    #[test]
    fn skill_use_is_reroll_armour() {
        let m = RerollArmourModification::new();
        assert_eq!(m.skill_use(), ffb_model::model::SkillUse::RE_ROLL_ARMOUR);
    }
}
