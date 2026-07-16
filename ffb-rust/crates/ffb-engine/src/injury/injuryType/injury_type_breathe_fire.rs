/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeBreatheFiree.
/// ModificationAware: standard armor roll + injury roll. savedByArmour -> PRONE (default).
use ffb_model::enums::{ApothecaryMode, PlayerState, PS_PRONE};
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use ffb_mechanics::modifiers::injury_modifier_factory::InjuryModifierFactory;
use crate::injury::{InjuryContext, InjuryTypeServer, do_armor_roll, do_injury_roll_for_player};
use crate::injury::injuryType::modification_aware_injury_type_server::{ModificationAwareInjuryType, modification_aware_handle_injury, leak_injury_modifier};

pub struct InjuryTypeBreatheFire { ctx: InjuryContext }
impl InjuryTypeBreatheFire { pub fn new() -> Self { Self { ctx: InjuryContext::new(ApothecaryMode::Defender) } } }
impl Default for InjuryTypeBreatheFire { fn default() -> Self { Self::new() } }

impl InjuryTypeServer for InjuryTypeBreatheFire {
    fn handle_injury(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, from_coord: Option<FieldCoordinate>, old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
        modification_aware_handle_injury(self, game, rng, attacker_id, defender_id, coord, from_coord, old_ctx, apo_mode);
    }
    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    fn falling_down_causes_turnover(&self) -> bool { false }
    /// Java: `BreatheFire.isCausedByOpponent()` — true.
    fn is_caused_by_opponent(&self) -> bool { true }
}
impl ModificationAwareInjuryType for InjuryTypeBreatheFire {
    fn armour_roll(&mut self, game: &Game, rng: &mut GameRng, _attacker_id: Option<&str>, defender_id: &str, _roll: bool) {
        do_armor_roll(game, rng, &mut self.ctx, defender_id);
    }
    fn injury_roll(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str) {
        // Java: `factory.findInjuryModifiers(game, injuryContext, pAttacker, pDefender,
        // isStab(), isFoul(), isVomitLike())` — BreatheFire.isVomitLike() is true,
        // isStab()/isFoul() are false (inherited InjuryType defaults).
        if let Some(defender) = game.player(defender_id) {
            let attacker = attacker_id.and_then(|aid| game.player(aid));
            let factory = InjuryModifierFactory::new(game.rules);
            for m in factory.find_injury_modifiers(game, attacker, defender, false, false, true) {
                self.ctx.add_injury_modifier(leak_injury_modifier(m.as_ref(), attacker, defender, game.rules));
            }
        }
        do_injury_roll_for_player(rng, &mut self.ctx, game, defender_id);
    }
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
        let mut t = InjuryTypeBreatheFire::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(13), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert_eq!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test]
    fn armor_break_results_in_injury_roll() {
        let mut t = InjuryTypeBreatheFire::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(2), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken); assert_ne!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test]
    fn does_not_cause_turnover() { assert!(!InjuryTypeBreatheFire::new().falling_down_causes_turnover()); }
    #[test]
    fn context_stores_defender_and_coordinate() {
        let mut t = InjuryTypeBreatheFire::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(13), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert_eq!(t.ctx.defender_id.as_deref(), Some("p1"));
        assert_eq!(t.ctx.defender_coordinate, Some(coord()));
    }
    #[test]
    fn default_equivalent_to_new() {
        let t1 = InjuryTypeBreatheFire::new();
        let t2 = InjuryTypeBreatheFire::default();
        assert_eq!(t1.ctx.armor_broken, t2.ctx.armor_broken);
        assert!(t1.ctx.injury.is_none() && t2.ctx.injury.is_none());
    }

    /// isVomitLike=true blocks Mighty Blow, and DirtyPlayer needs isFoul=true, so no
    /// attacker skill applies for BreatheFire. Niggling Injuries (defender-side, BB2016-only
    /// since BB2025 has no niggling modifiers) proves the factory is now actually reached.
    fn game_with_niggling_defender(rules: Rules, niggling_injuries: i32) -> Game {
        use std::collections::HashSet;
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(Player { id: "p1".into(), name: "p1".into(), nr: 1,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour: 8, starting_skills: vec![], extra_skills: vec![],
            temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default() });
        Game::new(home, crate::step::framework::test_team("away", 0), rules)
    }

    #[test]
    fn niggling_injury_adds_injury_modifier() {
        use ffb_mechanics::modifiers::Modifier;
        let game = game_with_niggling_defender(Rules::Bb2016, 1);
        let mut t = InjuryTypeBreatheFire::new();
        let mut rng = GameRng::new(1);
        t.injury_roll(&game, &mut rng, None, "p1");
        assert!(t.ctx.injury_modifiers.contains(&Modifier::new("1 Niggling Injury", 1, game.rules)));
    }

    #[test]
    fn no_niggling_injury_no_injury_modifier() {
        use ffb_mechanics::modifiers::Modifier;
        let game = game_with_niggling_defender(Rules::Bb2016, 0);
        let mut t = InjuryTypeBreatheFire::new();
        let mut rng = GameRng::new(1);
        t.injury_roll(&game, &mut rng, None, "p1");
        assert!(!t.ctx.injury_modifiers.contains(&Modifier::new("1 Niggling Injury", 1, game.rules)));
    }
}
