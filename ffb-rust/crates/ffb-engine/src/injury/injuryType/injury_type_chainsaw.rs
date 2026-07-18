/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeChainsaw.
/// ModificationAware: chainsaw armor roll (complex modifier stub) + injury roll.
/// savedByArmour -> None (chainsaw always skips PRONE; attacker may go to reserves).
use ffb_model::enums::ApothecaryMode;
use ffb_model::model::property::NamedProperties;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use ffb_mechanics::modifiers::ARMOR_CHAINSAW_3;
use ffb_mechanics::modifiers::injury_modifier_factory::InjuryModifierFactory;
use crate::injury::{InjuryContext, InjuryTypeServer, do_armor_roll, do_injury_roll_for_player};
use crate::injury::injuryType::modification_aware_injury_type_server::{ModificationAwareInjuryType, modification_aware_handle_injury, leak_injury_modifier};

pub struct InjuryTypeChainsaw { ctx: InjuryContext }
impl InjuryTypeChainsaw { pub fn new() -> Self { Self { ctx: InjuryContext::new(ApothecaryMode::Defender) } } }
impl Default for InjuryTypeChainsaw { fn default() -> Self { Self::new() } }

impl InjuryTypeServer for InjuryTypeChainsaw {
    fn handle_injury(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, from_coord: Option<FieldCoordinate>, old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
        modification_aware_handle_injury(self, game, rng, attacker_id, defender_id, coord, from_coord, old_ctx, apo_mode);
    }
    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    // Java: `Chainsaw` does not override `fallingDownCausesTurnover()`, so the `InjuryType` base
    // default (`true`) applies — no override needed here (trait default is already `true`).
    /// Java: `InjuryTypeChainsaw()` constructor calls `setFailedArmourPlacesProne(false)`.
    fn failed_armour_places_prone(&self) -> bool { false }
    /// Java: `Chainsaw.isCausedByOpponent()` — true.
    fn is_caused_by_opponent(&self) -> bool { true }
    /// Java: `Chainsaw` constructor passes `SendToBoxReason.CHAINSAW`.
    fn send_to_box_reason(&self) -> Option<ffb_model::enums::SendToBoxReason> {
        Some(ffb_model::enums::SendToBoxReason::Chainsaw)
    }
}
impl ModificationAwareInjuryType for InjuryTypeChainsaw {
    fn armour_roll(&mut self, game: &Game, rng: &mut GameRng, _attacker_id: Option<&str>, defender_id: &str, _roll: bool) {
        // Java: `UtilCards.hasUnusedSkillWithProperty(pDefender, ignoresArmourModifiersFromSkills)`
        // (e.g. Iron Hard Skin) suppresses the Chainsaw +3 armor modifier.
        let defender_ignores = game.player(defender_id)
            .map(|p| p.has_unused_skill_with_property(NamedProperties::IGNORES_ARMOUR_MODIFIERS_FROM_SKILLS))
            .unwrap_or(false);
        // Java: add chainsaw +3 armor modifier unless one is already present (blocksLikeChainsaw skills)
        if !defender_ignores && !self.ctx.armor_modifiers.iter().any(|m| m.name == "Chainsaw") {
            self.ctx.add_armor_modifier(ARMOR_CHAINSAW_3);
        }
        do_armor_roll(game, rng, &mut self.ctx, defender_id);
    }
    fn injury_roll(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str) {
        // Java: `factory.findInjuryModifiers(game, injuryContext, pAttacker, pDefender, isStab(),
        // isFoul(), isVomitLike(), isChainsaw())` — includes niggling internally. Chainsaw is
        // never stab/foul/vomit-like, isChainsaw=true (skips Mighty Blow in the factory).
        if let Some(defender) = game.player(defender_id) {
            let attacker = attacker_id.and_then(|aid| game.player(aid));
            let factory = InjuryModifierFactory::new(game.rules);
            for m in factory.find_injury_modifiers_chainsaw(game, attacker, defender, false, false, false, true) {
                self.ctx.add_injury_modifier(leak_injury_modifier(m.as_ref(), attacker, defender, game.rules));
            }
        }
        do_injury_roll_for_player(rng, &mut self.ctx, game, defender_id);
    }
    fn saved_by_armour(&mut self) {
        // Chainsaw: armor save means no injury (attacker goes to reserves separately)
        self.ctx.injury = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, SkillId};
    use ffb_mechanics::modifiers::ARMOR_CHAINSAW_3;

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

    fn game_with_player_skills(armour: i32, skills: Vec<SkillId>) -> Game {
        use std::collections::HashSet;
        use ffb_model::model::player::Player;
        use ffb_model::model::SkillWithValue;
        use ffb_model::enums::{PlayerType, PlayerGender};
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(Player { id: "p1".into(), name: "p1".into(), nr: 1,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour,
            starting_skills: skills.into_iter().map(SkillWithValue::new).collect(),
            extra_skills: vec![],
            temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
    ..Default::default() });
        Game::new(home, crate::step::framework::test_team("away", 0), Rules::Bb2025)
    }

    fn coord() -> FieldCoordinate { FieldCoordinate::new(5, 5) }

    #[test]
    fn armor_save_leaves_no_injury() {
        let mut t = InjuryTypeChainsaw::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(13), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(!t.ctx.armor_broken); assert!(t.ctx.injury.is_none());
    }

    #[test]
    fn armor_break_results_in_injury_roll() {
        let mut t = InjuryTypeChainsaw::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(2), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken); assert!(t.ctx.injury.is_some());
    }

    #[test]
    fn chainsaw_armor_modifier_added_to_roll() {
        let game = game_with_armor(7);
        let mut t = InjuryTypeChainsaw::new();
        let mut rng = GameRng::new(1);
        t.armour_roll(&game, &mut rng, None, "p1", true);
        assert!(t.ctx.armor_modifiers.contains(&ARMOR_CHAINSAW_3));
    }

    #[test]
    fn chainsaw_modifier_not_duplicated_if_already_present() {
        let game = game_with_armor(7);
        let mut t = InjuryTypeChainsaw::new();
        t.ctx.add_armor_modifier(ARMOR_CHAINSAW_3);
        let mut rng = GameRng::new(1);
        t.armour_roll(&game, &mut rng, None, "p1", true);
        assert_eq!(t.ctx.armor_modifiers.iter().filter(|m| m.name == "Chainsaw").count(), 1);
    }

    #[test]
    fn stunty_defender_uses_stunty_injury_table() {
        use ffb_model::enums::{PS_KNOCKED_OUT, PS_STUNNED};
        let game = game_with_player_skills(2, vec![SkillId::Stunty]);
        let mut t = InjuryTypeChainsaw::new();
        let mut rng = GameRng::new(42);
        t.ctx.armor_broken = true;
        t.injury_roll(&game, &mut rng, None, "p1");
        if t.ctx.injury_roll == Some([3, 4]) || t.ctx.injury_roll == Some([4, 3]) {
            assert_eq!(t.ctx.injury.map(|s| s.base()), Some(PS_KNOCKED_OUT),
                "Stunty at total 7 must be KO in BB2020");
        }
        let game2 = game_with_armor(2);
        let mut t2 = InjuryTypeChainsaw::new();
        let mut rng2 = GameRng::new(42);
        t2.ctx.armor_broken = true;
        t2.injury_roll(&game2, &mut rng2, None, "p1");
        if t2.ctx.injury_roll == Some([3, 4]) || t2.ctx.injury_roll == Some([4, 3]) {
            assert_eq!(t2.ctx.injury.map(|s| s.base()), Some(PS_STUNNED),
                "non-Stunty at total 7 must be Stunned");
        }
    }

    #[test]
    fn is_caused_by_opponent_is_true() {
        assert!(InjuryTypeChainsaw::new().is_caused_by_opponent());
    }

    #[test]
    fn falling_down_causes_turnover_defaults_true() {
        // Java: `Chainsaw` does not override `fallingDownCausesTurnover()`, so `InjuryType`'s
        // base default (`true`) applies. Regression test for a previously-inverted override
        // (`false`) that had no basis in the Java source.
        assert!(InjuryTypeChainsaw::new().falling_down_causes_turnover());
    }

    #[test]
    fn is_worth_spps_is_false_for_base_chainsaw() {
        assert!(!InjuryTypeChainsaw::new().is_worth_spps());
    }

    #[test]
    fn failed_armour_places_prone_is_false() {
        assert!(!InjuryTypeChainsaw::new().failed_armour_places_prone());
    }

    #[test]
    fn send_to_box_reason_is_chainsaw() {
        use ffb_model::enums::SendToBoxReason;
        assert_eq!(InjuryTypeChainsaw::new().send_to_box_reason(), Some(SendToBoxReason::Chainsaw));
    }

    #[test]
    fn niggling_injury_modifier_now_reaches_injury_roll() {
        // Before Phase ABJ, injury_roll never called InjuryModifierFactory at all, so a
        // niggling-injured defender got no modifier on a chainsaw hit. Niggling injury
        // modifiers only exist in BB2016's InjuryModifiers collection (bb2020/bb2025 have none).
        let mut game = game_with_armor(7);
        game.rules = Rules::Bb2016;
        game.team_home.players[0].niggling_injuries = 1;
        let mut t = InjuryTypeChainsaw::new();
        let mut rng = GameRng::new(1);
        t.ctx.armor_broken = true;
        t.injury_roll(&game, &mut rng, None, "p1");
        assert!(t.ctx.injury_modifiers.iter().any(|m| m.name == "1 Niggling Injury"));
    }

    #[test]
    fn iron_hard_skin_defender_suppresses_chainsaw_modifier() {
        use ffb_model::model::SkillWithValue;
        let mut game = game_with_armor(7);
        game.team_home.players[0].extra_skills.push(SkillWithValue::new(SkillId::IronHardSkin));
        let mut t = InjuryTypeChainsaw::new();
        let mut rng = GameRng::new(1);
        t.armour_roll(&game, &mut rng, None, "p1", true);
        assert!(!t.ctx.armor_modifiers.contains(&ARMOR_CHAINSAW_3));
    }
}
