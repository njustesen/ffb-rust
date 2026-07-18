/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeProjectileVomit.
/// ModificationAware: armor roll + injury roll. savedByArmour -> None (no injury set, like Stab/Chainsaw).
use ffb_model::enums::{ApothecaryMode, SendToBoxReason};
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use ffb_mechanics::modifiers::injury_modifier_factory::InjuryModifierFactory;
use crate::injury::{InjuryContext, InjuryTypeServer, do_armor_roll, do_injury_roll_for_player};
use crate::injury::injuryType::modification_aware_injury_type_server::{ModificationAwareInjuryType, modification_aware_handle_injury, leak_injury_modifier};

pub struct InjuryTypeProjectileVomit { ctx: InjuryContext }
impl InjuryTypeProjectileVomit { pub fn new() -> Self { Self { ctx: InjuryContext::new(ApothecaryMode::Defender) } } }
impl Default for InjuryTypeProjectileVomit { fn default() -> Self { Self::new() } }

impl InjuryTypeServer for InjuryTypeProjectileVomit {
    fn handle_injury(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, from_coord: Option<FieldCoordinate>, old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
        modification_aware_handle_injury(self, game, rng, attacker_id, defender_id, coord, from_coord, old_ctx, apo_mode);
    }
    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    fn falling_down_causes_turnover(&self) -> bool { false }
    /// Java: `ProjectileVomit()` constructor passes `SendToBoxReason.PROJECTILE_VOMIT`.
    fn send_to_box_reason(&self) -> Option<SendToBoxReason> { Some(SendToBoxReason::ProjectileVomit) }
    /// Java: `ProjectileVomit.isCausedByOpponent()` — true.
    fn is_caused_by_opponent(&self) -> bool { true }
    /// Java: `ProjectileVomit()` constructor calls `super.setFailedArmourPlacesProne(false)` —
    /// unlike the `InjuryType` base default of true (this was previously missing entirely, which
    /// would have incorrectly force-broken armor for a defender with `placedProneCausesInjuryRoll`
    /// via `UtilServerInjury.handleInjury`'s pre-roll check).
    fn failed_armour_places_prone(&self) -> bool { false }
}
impl ModificationAwareInjuryType for InjuryTypeProjectileVomit {
    fn armour_roll(&mut self, game: &Game, rng: &mut GameRng, _attacker_id: Option<&str>, defender_id: &str, _roll: bool) {
        do_armor_roll(game, rng, &mut self.ctx, defender_id);
    }
    fn injury_roll(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str) {
        // Java: `factory.findInjuryModifiers(game, injuryContext, pAttacker, pDefender, isStab(),
        // isFoul(), isVomitLike())` — ProjectileVomit has isVomitLike=true, isStab=false, isFoul=false.
        if let Some(defender) = game.player(defender_id) {
            let attacker = attacker_id.and_then(|aid| game.player(aid));
            let factory = InjuryModifierFactory::new(game.rules);
            for m in factory.find_injury_modifiers(game, attacker, defender, false, false, true) {
                self.ctx.add_injury_modifier(leak_injury_modifier(m.as_ref(), attacker, defender, game.rules));
            }
        }
        do_injury_roll_for_player(rng, &mut self.ctx, game, defender_id);
    }
    fn saved_by_armour(&mut self) {
        // No injury set when armor holds (Java: savedByArmour = null / no-op)
        self.ctx.injury = None;
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
    fn game_with_armor_and_niggling(armour: i32, niggling: i32) -> Game {
        use std::collections::HashSet;
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(Player { id: "p1".into(), name: "p1".into(), nr: 1,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour, starting_skills: vec![], extra_skills: vec![],
            temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: niggling, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
    ..Default::default() });
        // Bb2016 rules: InjuryModifierFactory has a niggling-injury modifier (Bb2025 has none),
        // used here to prove the factory is now reached from injury_roll.
        Game::new(home, crate::step::framework::test_team("away", 0), Rules::Bb2016)
    }
    fn coord() -> FieldCoordinate { FieldCoordinate::new(5, 5) }
    #[test]
    fn armor_save_leaves_no_injury() {
        let mut t = InjuryTypeProjectileVomit::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(13), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(!t.ctx.armor_broken); assert!(t.ctx.injury.is_none());
    }
    #[test]
    fn armor_break_results_in_injury_roll() {
        let mut t = InjuryTypeProjectileVomit::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(2), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken); assert!(t.ctx.injury.is_some());
    }
    #[test]
    fn does_not_cause_turnover() { assert!(!InjuryTypeProjectileVomit::new().falling_down_causes_turnover()); }
    #[test]
    fn send_to_box_reason_is_projectile_vomit() {
        assert_eq!(InjuryTypeProjectileVomit::new().send_to_box_reason(), Some(SendToBoxReason::ProjectileVomit));
    }
    #[test]
    fn is_caused_by_opponent() {
        assert!(InjuryTypeProjectileVomit::new().is_caused_by_opponent());
    }
    #[test]
    fn failed_armour_does_not_place_prone() {
        assert!(!InjuryTypeProjectileVomit::new().failed_armour_places_prone());
    }
    #[test]
    fn context_stores_defender_id() {
        let mut t = InjuryTypeProjectileVomit::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(13), &mut rng, None, "vomit_target", coord(), None, None, ApothecaryMode::Defender);
        assert_eq!(t.ctx.defender_id.as_deref(), Some("vomit_target"));
    }
    #[test]
    fn default_equivalent_to_new() {
        let t1 = InjuryTypeProjectileVomit::new();
        let t2 = InjuryTypeProjectileVomit::default();
        assert_eq!(t1.ctx.armor_broken, t2.ctx.armor_broken);
        assert!(t1.ctx.injury.is_none() && t2.ctx.injury.is_none());
    }
    #[test]
    fn niggling_injury_modifier_applied_when_armor_breaks() {
        // Proves InjuryModifierFactory is now reached from injury_roll (fixes the bug where
        // ProjectileVomit silently skipped skill/niggling injury modifiers).
        let mut t = InjuryTypeProjectileVomit::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor_and_niggling(2, 1), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken);
        assert!(t.ctx.injury_modifiers.iter().any(|m| m.name.contains("Niggling")),
            "expected a niggling injury modifier to be present, got {:?}", t.ctx.injury_modifiers);
    }
    #[test]
    fn no_niggling_injury_no_modifier() {
        let mut t = InjuryTypeProjectileVomit::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor_and_niggling(2, 0), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken);
        assert!(!t.ctx.injury_modifiers.iter().any(|m| m.name.contains("Niggling")));
    }
}
