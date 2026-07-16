/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeTTMLanding.
use ffb_model::enums::{ApothecaryMode, PlayerState, PS_PRONE};
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use ffb_model::model::property::NamedProperties;
use ffb_mechanics::modifiers::ARMOR_CHAINSAW_3;
use ffb_mechanics::modifiers::injury_modifier_factory::InjuryModifierFactory;
use crate::injury::{InjuryContext, InjuryTypeServer, do_armor_roll, do_injury_roll_for_player};
use crate::injury::injuryType::modification_aware_injury_type_server::leak_injury_modifier;

pub struct InjuryTypeTTMLanding { ctx: InjuryContext }
impl InjuryTypeTTMLanding { pub fn new() -> Self { Self { ctx: InjuryContext::new(ApothecaryMode::ThrownPlayer) } } }
impl Default for InjuryTypeTTMLanding { fn default() -> Self { Self::new() } }

impl InjuryTypeServer for InjuryTypeTTMLanding {
    fn handle_injury(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, _from_coord: Option<FieldCoordinate>, _old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
        self.ctx.defender_id = Some(defender_id.to_owned());
        self.ctx.attacker_id = attacker_id.map(str::to_owned);
        self.ctx.defender_coordinate = Some(coord);
        self.ctx.apothecary_mode = apo_mode;
        if !self.ctx.armor_broken {
            let defender_ignores = game.player(defender_id)
                .map(|p| p.has_unused_skill_with_property(NamedProperties::IGNORES_ARMOUR_MODIFIERS_FROM_SKILLS))
                .unwrap_or(false);
            if !defender_ignores {
                if game.player(defender_id)
                    .map(|p| p.has_skill_property(NamedProperties::BLOCKS_LIKE_CHAINSAW))
                    .unwrap_or(false)
                {
                    self.ctx.add_armor_modifier(ARMOR_CHAINSAW_3);
                }
            }
            do_armor_roll(game, rng, &mut self.ctx, defender_id);
        }
        if self.ctx.armor_broken {
            // Java: `factory.findInjuryModifiers(game, injuryContext, null, pDefender, isStab(),
            // isFoul(), isVomitLike())` — TTMLanding passes attacker as `null` explicitly (a
            // player injures itself on landing; there is no attacker), unlike TTMHitPlayer which
            // passes pAttacker. isStab/isFoul/isVomitLike all default false (not overridden).
            if let Some(defender) = game.player(defender_id) {
                let factory = InjuryModifierFactory::new(game.rules);
                for m in factory.find_injury_modifiers(game, None, defender, false, false, false) {
                    self.ctx.add_injury_modifier(leak_injury_modifier(m.as_ref(), None, defender, game.rules));
                }
            }
            do_injury_roll_for_player(rng, &mut self.ctx, game, defender_id);
        }
        else { self.ctx.injury = Some(PlayerState::new(PS_PRONE)); }
    }
    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    fn falling_down_causes_turnover(&self) -> bool { false }
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
        let mut t = InjuryTypeTTMLanding::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(13), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::ThrownPlayer);
        assert_eq!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test]
    fn armor_break_results_in_injury_roll() {
        let mut t = InjuryTypeTTMLanding::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(2), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::ThrownPlayer);
        assert!(t.ctx.armor_broken); assert_ne!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test] fn no_turnover() { assert!(!InjuryTypeTTMLanding::new().falling_down_causes_turnover()); }
    #[test]
    fn new_creates_instance_with_thrown_player_apo_mode() {
        let t = InjuryTypeTTMLanding::new();
        assert_eq!(t.ctx.apothecary_mode, ApothecaryMode::ThrownPlayer);
    }
    #[test]
    fn sets_attacker_and_defender_ids() {
        let mut t = InjuryTypeTTMLanding::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(13), &mut rng, Some("atk1"), "p1", coord(), None, None, ApothecaryMode::ThrownPlayer);
        assert_eq!(t.ctx.defender_id.as_deref(), Some("p1"));
        assert_eq!(t.ctx.attacker_id.as_deref(), Some("atk1"));
    }

    /// TTMLanding passes `null` explicitly as attacker to `findInjuryModifiers` (Java) — a player
    /// falls and injures itself, there is no attacker — so attacker-only modifiers like MightyBlow
    /// can never apply here. Use the defender-side niggling-injury modifier (BB2016 only) instead
    /// to prove the factory is reachable and wired in.
    fn game_with_defender_nigglings(armour: i32, nigglings: i32) -> Game {
        use std::collections::HashSet;
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender, Rules};
        let home = crate::step::framework::test_team("home", 0);
        let mut away = crate::step::framework::test_team("away", 0);
        away.players.push(Player { id: "p1".into(), name: "p1".into(), nr: 1,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour, starting_skills: vec![], extra_skills: vec![],
            temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: nigglings, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default() });
        Game::new(home, away, Rules::Bb2016)
    }
    #[test]
    fn niggling_injury_adds_injury_modifier() {
        let game = game_with_defender_nigglings(2, 1);
        let mut t = InjuryTypeTTMLanding::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game, &mut rng, Some("atk1"), "p1", coord(), None, None, ApothecaryMode::ThrownPlayer);
        assert!(t.ctx.armor_broken);
        assert!(t.ctx.injury_modifiers.iter().any(|m| m.name.contains("Niggling")));
    }
    #[test]
    fn no_nigglings_no_niggling_modifier() {
        let game = game_with_defender_nigglings(2, 0);
        let mut t = InjuryTypeTTMLanding::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game, &mut rng, Some("atk1"), "p1", coord(), None, None, ApothecaryMode::ThrownPlayer);
        assert!(t.ctx.armor_broken);
        assert!(!t.ctx.injury_modifiers.iter().any(|m| m.name.contains("Niggling")));
    }
}
