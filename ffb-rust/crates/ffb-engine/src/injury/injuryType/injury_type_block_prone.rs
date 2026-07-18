/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeBlockProne.
/// ModificationAware: armor roll + injury roll. savedByArmour -> PRONE.
///
/// Java's `injuryRoll()` here checks `pDefender.getSkillWithProperty(isHurtMoreEasily)` (Stunty)
/// and, if present, adds that skill's registered injury modifiers to the context — but Stunty's
/// registered modifier is `new StaticInjuryModifier("Stunty", 0, false)`, i.e. value **0**; it
/// exists purely as a marker so `RollMechanic.interpretInjuryRoll` can detect Stunty via the
/// modifier list. `do_injury_roll_for_player` (called below) instead detects Stunty directly via
/// `defender.has_skill(SkillId::Stunty)`, so no marker-modifier equivalent is needed here.
/// Java's `injuryRoll()` never touches niggling injuries for this type (unlike plain `Block`,
/// which calls `factory.getNigglingInjuryModifier` explicitly) — no niggling modifier is added.
use ffb_model::enums::{ApothecaryMode, PlayerState, PS_PRONE};
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use crate::injury::{InjuryContext, InjuryTypeServer, do_armor_roll, do_injury_roll_for_player};
use crate::injury::injuryType::modification_aware_injury_type_server::{ModificationAwareInjuryType, modification_aware_handle_injury};

pub struct InjuryTypeBlockProne { ctx: InjuryContext }
impl InjuryTypeBlockProne { pub fn new() -> Self { Self { ctx: InjuryContext::new(ApothecaryMode::Defender) } } }
impl Default for InjuryTypeBlockProne { fn default() -> Self { Self::new() } }

impl InjuryTypeServer for InjuryTypeBlockProne {
    fn handle_injury(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, from_coord: Option<FieldCoordinate>, old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
        modification_aware_handle_injury(self, game, rng, attacker_id, defender_id, coord, from_coord, old_ctx, apo_mode);
    }
    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    /// Java: `BlockProne.isCausedByOpponent()` — true.
    fn is_caused_by_opponent(&self) -> bool { true }
    /// Java: `new BlockProne()` constructor passes `SendToBoxReason.BLOCKED`.
    fn send_to_box_reason(&self) -> Option<ffb_model::enums::SendToBoxReason> {
        Some(ffb_model::enums::SendToBoxReason::Blocked)
    }
}
impl ModificationAwareInjuryType for InjuryTypeBlockProne {
    fn armour_roll(&mut self, game: &Game, rng: &mut GameRng, _attacker_id: Option<&str>, defender_id: &str, _roll: bool) {
        do_armor_roll(game, rng, &mut self.ctx, defender_id);
    }
    fn injury_roll(&mut self, game: &Game, rng: &mut GameRng, _attacker_id: Option<&str>, defender_id: &str) {
        do_injury_roll_for_player(rng, &mut self.ctx, game, defender_id);
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
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default() });
        Game::new(home, crate::step::framework::test_team("away", 0), Rules::Bb2025)
    }
    fn coord() -> FieldCoordinate { FieldCoordinate::new(5, 5) }
    #[test]
    fn armor_save_results_in_prone() {
        let mut t = InjuryTypeBlockProne::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(13), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert_eq!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test]
    fn armor_break_results_in_injury_roll() {
        let mut t = InjuryTypeBlockProne::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(2), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken); assert_ne!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }

    #[test]
    fn initial_context_has_no_injury() {
        let t = InjuryTypeBlockProne::new();
        assert!(!t.ctx.armor_broken);
        assert!(t.ctx.injury.is_none());
    }
    #[test]
    fn send_to_box_reason_is_blocked() {
        use ffb_model::enums::SendToBoxReason;
        assert_eq!(InjuryTypeBlockProne::new().send_to_box_reason(), Some(SendToBoxReason::Blocked));
    }
    #[test]
    fn default_equivalent_to_new() {
        let t1 = InjuryTypeBlockProne::new();
        let t2 = InjuryTypeBlockProne::default();
        assert_eq!(t1.ctx.armor_broken, t2.ctx.armor_broken);
        assert!(t1.ctx.injury.is_none() && t2.ctx.injury.is_none());
    }

    #[test]
    fn new_context_uses_defender_apo_mode() {
        use ffb_model::enums::ApothecaryMode;
        let t = InjuryTypeBlockProne::new();
        assert_eq!(t.injury_context().apothecary_mode, ApothecaryMode::Defender);
    }

    /// Java: `InjuryTypeBlockProne.injuryRoll()` never touches niggling injuries — unlike plain
    /// `Block`, it doesn't call `factory.getNigglingInjuryModifier`. A niggling-injured defender
    /// must not get a niggling injury modifier here.
    #[test]
    fn niggling_injured_defender_gets_no_niggling_injury_modifier() {
        use std::collections::HashSet;
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(Player { id: "p1".into(), name: "p1".into(), nr: 1,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour: 2, starting_skills: vec![], extra_skills: vec![],
            temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 3, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default() });
        let game = Game::new(home, crate::step::framework::test_team("away", 0), Rules::Bb2016);
        let mut t = InjuryTypeBlockProne::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game, &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(!t.ctx.injury_modifiers.iter().any(|m| m.name.contains("Niggling")));
    }
}
