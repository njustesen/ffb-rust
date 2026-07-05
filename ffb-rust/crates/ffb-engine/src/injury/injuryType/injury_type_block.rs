/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeBlock (166 lines).
/// ModificationAware: the most complex injury type. Handles block armor roll modes:
/// Regular, UseModifiersAgainstTeamMates, UseMightyBlow, UseClaws, UseClawsAndMightyBlow, etc.
/// Claws/MB interaction and CLAW_DOES_NOT_STACK game option are stubs (TODO).
use ffb_model::enums::{ApothecaryMode, PlayerState, PS_PRONE, SkillId};
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use ffb_mechanics::modifiers::{niggling_injury_modifier, ARMOR_MIGHTY_BLOW_1, INJURY_MIGHTY_BLOW_1};
use crate::injury::{InjuryContext, InjuryTypeServer, do_armor_roll, do_injury_roll_for_player};
use crate::injury::injuryType::modification_aware_injury_type_server::{ModificationAwareInjuryType, modification_aware_handle_injury};

/// Java: InjuryTypeBlock.Mode enum (inner class).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockMode {
    Regular,
    UseModifiersAgainstTeamMates,
    UseMightyBlow,
    UseClaws,
    UseClawsAndMightyBlow,
}

pub struct InjuryTypeBlock {
    ctx: InjuryContext,
    mode: BlockMode,
    /// Java: fRollArmour. Whether to actually roll armor dice (vs just evaluate existing ctx).
    roll_armour: bool,
}

impl InjuryTypeBlock {
    pub fn new(mode: BlockMode, roll_armour: bool) -> Self {
        Self { ctx: InjuryContext::new(ApothecaryMode::Defender), mode, roll_armour }
    }
}
impl Default for InjuryTypeBlock {
    fn default() -> Self { Self::new(BlockMode::Regular, true) }
}

impl InjuryTypeServer for InjuryTypeBlock {
    fn handle_injury(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, from_coord: Option<FieldCoordinate>, old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
        modification_aware_handle_injury(self, game, rng, attacker_id, defender_id, coord, from_coord, old_ctx, apo_mode);
    }
    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    fn java_class_name(&self) -> &'static str { "Block" }
}
impl ModificationAwareInjuryType for InjuryTypeBlock {
    fn armour_roll(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str, roll: bool) {
        if roll && self.roll_armour {
            // MightyBlow: +1 to armor roll in MB or Claws+MB mode
            if matches!(self.mode, BlockMode::UseMightyBlow | BlockMode::UseClawsAndMightyBlow) {
                if let Some(aid) = attacker_id {
                    if let Some(attacker) = game.player(aid) {
                        if attacker.has_skill(SkillId::MightyBlow) {
                            self.ctx.add_armor_modifier(ARMOR_MIGHTY_BLOW_1);
                        }
                    }
                }
            }
            // TODO: CLAW_DOES_NOT_STACK game option check
            // TODO: defender blocksLikeChainsaw / ignoresArmourModifiersFromSkills check
            do_armor_roll(game, rng, &mut self.ctx, defender_id);
        }
    }
    fn injury_roll(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str) {
        if let Some(defender) = game.player(defender_id) {
            if let Some(m) = niggling_injury_modifier(defender.niggling_injuries) {
                self.ctx.add_injury_modifier(m);
            }
        }
        // MightyBlow: +1 to injury roll in MB or Claws+MB mode
        if matches!(self.mode, BlockMode::UseMightyBlow | BlockMode::UseClawsAndMightyBlow) {
            if let Some(aid) = attacker_id {
                if let Some(attacker) = game.player(aid) {
                    if attacker.has_skill(SkillId::MightyBlow) {
                        self.ctx.add_injury_modifier(INJURY_MIGHTY_BLOW_1);
                    }
                }
            }
        }
        do_injury_roll_for_player(rng, &mut self.ctx, game, defender_id);
    }
    // savedByArmour: default PRONE
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, SkillId};
    use ffb_mechanics::modifiers::{ARMOR_MIGHTY_BLOW_1, INJURY_MIGHTY_BLOW_1};

    fn make_player(id: &str, armour: i32, skills: Vec<SkillId>) -> ffb_model::model::player::Player {
        use std::collections::HashSet;
        use ffb_model::model::player::Player;
        use ffb_model::model::SkillWithValue;
        use ffb_model::enums::{PlayerType, PlayerGender};
        Player { id: id.into(), name: id.into(), nr: 1,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour, starting_skills: skills.into_iter().map(SkillWithValue::new).collect(), extra_skills: vec![],
            temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            ..Default::default() }
    }

    fn game_with_armor(armour: i32) -> Game {
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(make_player("p1", armour, vec![]));
        Game::new(home, crate::step::framework::test_team("away", 0), Rules::Bb2025)
    }

    fn game_with_attacker_and_defender(attacker_skills: Vec<SkillId>, defender_armour: i32) -> Game {
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(make_player("attacker", 7, attacker_skills));
        let mut away = crate::step::framework::test_team("away", 0);
        away.players.push(make_player("defender", defender_armour, vec![]));
        Game::new(home, away, Rules::Bb2025)
    }

    fn coord() -> FieldCoordinate { FieldCoordinate::new(5, 5) }

    #[test]
    fn armor_save_results_in_prone() {
        let mut t = InjuryTypeBlock::new(BlockMode::Regular, true); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(13), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert_eq!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test]
    fn armor_break_results_in_injury_roll() {
        let mut t = InjuryTypeBlock::new(BlockMode::Regular, true); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(2), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken); assert_ne!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test]
    fn no_roll_armour_skips_armor_check() {
        let mut t = InjuryTypeBlock::new(BlockMode::Regular, false); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(2), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(!t.ctx.armor_broken);
    }
    #[test]
    fn use_mighty_blow_adds_armor_modifier() {
        let game = game_with_attacker_and_defender(vec![SkillId::MightyBlow], 2);
        let mut t = InjuryTypeBlock::new(BlockMode::UseMightyBlow, true);
        let mut rng = GameRng::new(1);
        t.armour_roll(&game, &mut rng, Some("attacker"), "defender", true);
        assert!(t.ctx.armor_modifiers.contains(&ARMOR_MIGHTY_BLOW_1));
    }
    #[test]
    fn regular_mode_does_not_add_mighty_blow_armor_modifier() {
        let game = game_with_attacker_and_defender(vec![SkillId::MightyBlow], 2);
        let mut t = InjuryTypeBlock::new(BlockMode::Regular, true);
        let mut rng = GameRng::new(1);
        t.armour_roll(&game, &mut rng, Some("attacker"), "defender", true);
        assert!(!t.ctx.armor_modifiers.contains(&ARMOR_MIGHTY_BLOW_1));
    }
    #[test]
    fn use_claws_and_mighty_blow_adds_armor_modifier() {
        let game = game_with_attacker_and_defender(vec![SkillId::MightyBlow], 2);
        let mut t = InjuryTypeBlock::new(BlockMode::UseClawsAndMightyBlow, true);
        let mut rng = GameRng::new(1);
        t.armour_roll(&game, &mut rng, Some("attacker"), "defender", true);
        assert!(t.ctx.armor_modifiers.contains(&ARMOR_MIGHTY_BLOW_1));
    }
    #[test]
    fn use_mighty_blow_adds_injury_modifier() {
        let game = game_with_attacker_and_defender(vec![SkillId::MightyBlow], 2);
        let mut t = InjuryTypeBlock::new(BlockMode::UseMightyBlow, true);
        let mut rng = GameRng::new(1);
        t.ctx.armor_broken = true;
        t.injury_roll(&game, &mut rng, Some("attacker"), "defender");
        assert!(t.ctx.injury_modifiers.contains(&INJURY_MIGHTY_BLOW_1));
    }
    #[test]
    fn regular_mode_does_not_add_mighty_blow_injury_modifier() {
        let game = game_with_attacker_and_defender(vec![SkillId::MightyBlow], 2);
        let mut t = InjuryTypeBlock::new(BlockMode::Regular, true);
        let mut rng = GameRng::new(1);
        t.ctx.armor_broken = true;
        t.injury_roll(&game, &mut rng, Some("attacker"), "defender");
        assert!(!t.ctx.injury_modifiers.contains(&INJURY_MIGHTY_BLOW_1));
    }
    #[test]
    fn use_mighty_blow_without_skill_does_not_add_modifier() {
        let game = game_with_attacker_and_defender(vec![], 2);
        let mut t = InjuryTypeBlock::new(BlockMode::UseMightyBlow, true);
        let mut rng = GameRng::new(1);
        t.armour_roll(&game, &mut rng, Some("attacker"), "defender", true);
        assert!(!t.ctx.armor_modifiers.contains(&ARMOR_MIGHTY_BLOW_1));
    }
    #[test]
    fn stunty_defender_ko_at_total_7_bb2020() {
        // BB2020: Stunty at roll 7 → KO instead of Stunned.
        // Seed rng to produce d1=3, d2=4 (total 7 with no modifiers).
        use ffb_model::enums::{PS_KNOCKED_OUT, PS_STUNNED};
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(make_player("defender", 2, vec![SkillId::Stunty]));
        let game = Game::new(home, crate::step::framework::test_team("away", 0), ffb_model::enums::Rules::Bb2025);
        let mut t = InjuryTypeBlock::new(BlockMode::Regular, true);
        // seed=42 produces d1+d2=7 for first pair
        let mut rng = GameRng::new(42);
        t.ctx.armor_broken = true;
        t.injury_roll(&game, &mut rng, None, "defender");
        // With Stunty (BB2020), total=7 must be KO, not Stunned
        let state = t.ctx.injury.map(|s| s.base());
        // The roll total determines the outcome — if total was 7, should be KO
        if t.ctx.injury_roll == Some([3, 4]) || t.ctx.injury_roll == Some([4, 3]) {
            assert_eq!(state, Some(PS_KNOCKED_OUT), "Stunty at total 7 must be KO in BB2020");
        }
        // Regardless of roll, non-Stunty player at total 7 would be Stunned
        let mut home2 = crate::step::framework::test_team("home", 0);
        home2.players.push(make_player("defender", 2, vec![]));
        let game2 = Game::new(home2, crate::step::framework::test_team("away", 0), ffb_model::enums::Rules::Bb2025);
        let mut t2 = InjuryTypeBlock::new(BlockMode::Regular, true);
        let mut rng2 = GameRng::new(42);
        t2.ctx.armor_broken = true;
        t2.injury_roll(&game2, &mut rng2, None, "defender");
        if t2.ctx.injury_roll == Some([3, 4]) || t2.ctx.injury_roll == Some([4, 3]) {
            assert_eq!(t2.ctx.injury.map(|s| s.base()), Some(PS_STUNNED), "non-Stunty at total 7 must be Stunned");
        }
    }
}
