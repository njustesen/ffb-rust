/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypePilingOnArmour.
/// Piling On armor roll: standard armor roll + ARMOR_PILING_ON (+2) modifier.
/// If broken: injury roll. Else: PRONE. turnover=false, no apo, stun treated as KO = false.
use ffb_model::enums::{ApothecaryMode, PlayerState, PS_PRONE};
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use ffb_mechanics::modifiers::ARMOR_PILING_ON;
use ffb_mechanics::modifiers::injury_modifier_factory::InjuryModifierFactory;
use ffb_model::option::game_option_id;
use crate::injury::{InjuryContext, InjuryTypeServer, do_armor_roll, do_injury_roll_for_player};
use crate::injury::injuryType::modification_aware_injury_type_server::leak_injury_modifier;

pub struct InjuryTypePilingOnArmour { ctx: InjuryContext }
impl InjuryTypePilingOnArmour { pub fn new() -> Self { Self { ctx: InjuryContext::new(ApothecaryMode::Defender) } } }
impl Default for InjuryTypePilingOnArmour { fn default() -> Self { Self::new() } }

impl InjuryTypeServer for InjuryTypePilingOnArmour {
    fn handle_injury(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, _from_coord: Option<FieldCoordinate>, _old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
        self.ctx.defender_id = Some(defender_id.to_owned());
        self.ctx.attacker_id = attacker_id.map(str::to_owned);
        self.ctx.defender_coordinate = Some(coord);
        self.ctx.apothecary_mode = apo_mode;
        if !self.ctx.armor_broken {
            self.ctx.add_armor_modifier(ARMOR_PILING_ON);
            do_armor_roll(game, rng, &mut self.ctx, defender_id);
        }
        if self.ctx.armor_broken {
            // Java: `factory.getNigglingInjuryModifier(pDefender).ifPresent(...)` — always added.
            // Then, if PILING_ON_DOES_NOT_STACK is not enabled: `factory.findInjuryModifiersWithoutNiggling(
            // game, injuryContext, pAttacker, pDefender, isStab(), isFoul(), isVomitLike(), isChainsaw())`.
            // PilingOnArmour isStab/isFoul/isVomitLike/isChainsaw all default false (InjuryType base).
            if let Some(defender) = game.player(defender_id) {
                let factory = InjuryModifierFactory::new(game.rules);
                if let Some(niggling) = factory.get_niggling_injury_modifier(defender) {
                    self.ctx.add_injury_modifier(leak_injury_modifier(niggling.as_ref(), None, defender, game.rules));
                }
                if !game.options.is_enabled(game_option_id::PILING_ON_DOES_NOT_STACK) {
                    let attacker = attacker_id.and_then(|aid| game.player(aid));
                    for m in factory.find_injury_modifiers_without_niggling(game, attacker, defender, false, false, false, false) {
                        self.ctx.add_injury_modifier(leak_injury_modifier(m.as_ref(), attacker, defender, game.rules));
                    }
                }
            }
            do_injury_roll_for_player(rng, &mut self.ctx, game, defender_id);
        }
        else { self.ctx.injury = Some(PlayerState::new(PS_PRONE)); }
    }
    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    fn falling_down_causes_turnover(&self) -> bool { false }
    fn can_use_apo(&self) -> bool { false }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, SkillId};
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
    fn make_skilled_player(id: &str, armour: i32, skills: Vec<SkillId>) -> ffb_model::model::player::Player {
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
            is_big_guy: false,
            ..Default::default() }
    }
    fn game_with_attacker_and_defender(attacker_skills: Vec<SkillId>, defender_armour: i32) -> Game {
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(make_skilled_player("attacker", 7, attacker_skills));
        let mut away = crate::step::framework::test_team("away", 0);
        away.players.push(make_skilled_player("defender", defender_armour, vec![]));
        Game::new(home, away, Rules::Bb2025)
    }
    fn coord() -> FieldCoordinate { FieldCoordinate::new(5, 5) }
    #[test]
    fn armor_save_results_in_prone() {
        let mut t = InjuryTypePilingOnArmour::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(13), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert_eq!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test]
    fn armor_break_results_in_injury_roll() {
        let mut t = InjuryTypePilingOnArmour::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(2), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken); assert_ne!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test]
    fn no_apo() { assert!(!InjuryTypePilingOnArmour::new().can_use_apo()); }

    #[test]
    fn piling_on_armor_modifier_applied() {
        let mut t = InjuryTypePilingOnArmour::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(7), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_modifiers.contains(&ARMOR_PILING_ON),
            "ARMOR_PILING_ON (+2) must be in armor_modifiers");
    }
    #[test]
    fn default_equivalent_to_new() {
        let t1 = InjuryTypePilingOnArmour::new();
        let t2 = InjuryTypePilingOnArmour::default();
        assert_eq!(t1.ctx.armor_broken, t2.ctx.armor_broken);
        assert!(t1.ctx.injury.is_none() && t2.ctx.injury.is_none());
    }

    #[test]
    fn mighty_blow_adds_injury_modifier() {
        // Proves InjuryModifierFactory is now reached from handle_injury (Phase ABJ bug fix):
        // Mighty Blow applies since isStab/isFoul/isVomitLike are all false for PilingOnArmour,
        // and PILING_ON_DOES_NOT_STACK is not enabled by default.
        let game = game_with_attacker_and_defender(vec![SkillId::MightyBlow], 2);
        let mut t = InjuryTypePilingOnArmour::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game, &mut rng, Some("attacker"), "defender", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken);
        assert!(t.ctx.injury_modifiers.iter().any(|m| m.name == "Mighty Blow"),
            "expected Mighty Blow injury modifier, got {:?}", t.ctx.injury_modifiers);
    }

    #[test]
    fn no_mighty_blow_no_injury_modifier() {
        let game = game_with_attacker_and_defender(vec![], 2);
        let mut t = InjuryTypePilingOnArmour::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game, &mut rng, Some("attacker"), "defender", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken);
        assert!(!t.ctx.injury_modifiers.iter().any(|m| m.name == "Mighty Blow"));
    }

    #[test]
    fn piling_on_does_not_stack_suppresses_mighty_blow_but_not_niggling() {
        use ffb_model::option::game_option_id;
        let mut game = game_with_attacker_and_defender(vec![SkillId::MightyBlow], 2);
        game.options.set(game_option_id::PILING_ON_DOES_NOT_STACK, "true");
        game.team_away.players[0].niggling_injuries = 1;
        // Bb2016 has a niggling injury modifier; Bb2025's factory has none (see
        // injury_modifier_factory.rs test `special_effect_bomb_not_in_bb2025` and friends).
        game.rules = Rules::Bb2016;
        let mut t = InjuryTypePilingOnArmour::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game, &mut rng, Some("attacker"), "defender", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken);
        assert!(!t.ctx.injury_modifiers.iter().any(|m| m.name == "Mighty Blow"),
            "PILING_ON_DOES_NOT_STACK must suppress findInjuryModifiersWithoutNiggling, got {:?}", t.ctx.injury_modifiers);
        assert!(t.ctx.injury_modifiers.iter().any(|m| m.name.contains("Niggling")),
            "niggling modifier is always added regardless of PILING_ON_DOES_NOT_STACK");
    }
}
