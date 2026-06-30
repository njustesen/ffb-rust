/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeBlock (166 lines).
/// ModificationAware: the most complex injury type. Handles block armor roll modes:
/// Regular, UseModifiersAgainstTeamMates, UseMightyBlow, UseClaws, UseClawsAndMightyBlow, etc.
/// Claws/MB interaction and CLAW_DOES_NOT_STACK game option are stubs (TODO).
use ffb_model::enums::{ApothecaryMode, PlayerState, PS_PRONE};
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use crate::injury::{InjuryContext, InjuryTypeServer, do_armor_roll, do_injury_roll};
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
}
impl ModificationAwareInjuryType for InjuryTypeBlock {
    fn armour_roll(&mut self, game: &Game, rng: &mut GameRng, _attacker_id: Option<&str>, defender_id: &str, roll: bool) {
        if roll && self.roll_armour {
            // TODO: add mode-specific armor modifiers (MightyBlow, Claws, team-mate modifiers) when ported
            // TODO: CLAW_DOES_NOT_STACK game option check
            // TODO: defender blocksLikeChainsaw / ignoresArmourModifiersFromSkills check
            let _ = self.mode;
            do_armor_roll(game, rng, &mut self.ctx, defender_id);
        }
    }
    fn injury_roll(&mut self, _game: &Game, rng: &mut GameRng, _attacker_id: Option<&str>, _defender_id: &str) {
        // TODO: add stunty and mode-specific injury modifiers when InjuryModifierFactory is ported
        do_injury_roll(rng, &mut self.ctx);
    }
    // savedByArmour: default PRONE
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    fn game_with_armor(armour: i32) -> Game {
        use std::collections::HashSet;
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(Player { id: "p1".into(), name: "p1".into(), nr: 1,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour, starting_skills: vec![], extra_skills: vec![],
            temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None });
        Game::new(home, crate::step::framework::test_team("away", 0), Rules::Bb2025)
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
        // armor_broken stays false when roll_armour=false
        assert!(!t.ctx.armor_broken);
    }
}
