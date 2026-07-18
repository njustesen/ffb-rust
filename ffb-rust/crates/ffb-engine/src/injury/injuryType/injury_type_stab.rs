/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeStab.
/// ModificationAware: stab armor roll + injury roll. savedByArmour -> None.
/// Sneaky git pair armor modifier is TODO.
use ffb_model::enums::{ApothecaryMode, SendToBoxReason};
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use ffb_mechanics::modifiers::injury_modifier_factory::InjuryModifierFactory;
use ffb_mechanics::modifiers::Modifier;
use crate::injury::{InjuryContext, InjuryTypeServer, do_armor_roll, do_injury_roll_for_player, recalc_armor_broken};
use crate::injury::injuryType::modification_aware_injury_type_server::{ModificationAwareInjuryType, modification_aware_handle_injury, leak_injury_modifier};

pub struct InjuryTypeStab { ctx: InjuryContext, use_injury_modifiers: bool, add_defender_chainsaw: bool }
impl InjuryTypeStab {
    /// Java: `InjuryTypeStab(boolean useInjuryModifiers)`.
    pub fn new(use_injury_modifiers: bool) -> Self {
        Self::new_with_chainsaw(use_injury_modifiers, false)
    }
    /// Java: `InjuryTypeStab(boolean useInjuryModifiers, boolean addDefenderChainsaw)`.
    pub fn new_with_chainsaw(use_injury_modifiers: bool, add_defender_chainsaw: bool) -> Self {
        Self { ctx: InjuryContext::new(ApothecaryMode::Defender), use_injury_modifiers, add_defender_chainsaw }
    }
}
impl Default for InjuryTypeStab { fn default() -> Self { Self::new(true) } }

impl InjuryTypeServer for InjuryTypeStab {
    fn handle_injury(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, from_coord: Option<FieldCoordinate>, old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
        modification_aware_handle_injury(self, game, rng, attacker_id, defender_id, coord, from_coord, old_ctx, apo_mode);
    }
    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    // Java: `Stab` does not override `fallingDownCausesTurnover()`, so the `InjuryType` base
    // default (`true`) applies — no override needed here (trait default is already `true`).
    /// Java: `Stab.isCausedByOpponent()` → true.
    fn is_caused_by_opponent(&self) -> bool { true }
    /// Java: `Stab` constructed with `worthSpps=false`.
    fn is_worth_spps(&self) -> bool { false }
    /// Java: `Stab` constructed with `super("stab", false, SendToBoxReason.STABBED)`. Was
    /// previously missing (defaulted to `None`), so a KO'd/casualtied stab victim never got a
    /// "Stabbed" send-to-box reason.
    fn send_to_box_reason(&self) -> Option<SendToBoxReason> { Some(SendToBoxReason::Stabbed) }
    /// Java: `InjuryTypeStab`'s constructor calls `super.setFailedArmourPlacesProne(false)`.
    /// Was previously missing (defaulted to the trait's `true`), which meant
    /// `UtilServerInjury.handleInjury`'s "ball-and-chain always breaks armor" special case
    /// (`failedArmourPlacesProne() && defender.hasSkillProperty(placedProneCausesInjuryRoll)`)
    /// would incorrectly force `armor_broken = true` when stabbing a Ball & Chain player.
    fn failed_armour_places_prone(&self) -> bool { false }
}
impl ModificationAwareInjuryType for InjuryTypeStab {
    fn armour_roll(&mut self, game: &Game, rng: &mut GameRng, _attacker_id: Option<&str>, defender_id: &str, _roll: bool) {
        // TODO: add sneaky git pair armor modifier when ArmorModifierFactory is ported
        do_armor_roll(game, rng, &mut self.ctx, defender_id);
        // Java (lines 64-71): unless the defender has an unused
        // `ignoresArmourModifiersFromSkills` skill (not modeled here — see TODO above), a
        // defender-side Chainsaw skill contributes its own armor modifier when
        // `addDefenderChainsaw` is set (used by e.g. `StepTreacherous`'s `new InjuryTypeStab(true,
        // true)`). Chainsaw's armor bonus is a flat +3 (see InjuryTypeBlock's `chainsaw_modifier`).
        if self.add_defender_chainsaw {
            if let Some(defender) = game.player(defender_id) {
                if defender.has_skill_property(NamedProperties::BLOCKS_LIKE_CHAINSAW) {
                    self.ctx.add_armor_modifier(Modifier::new("Chainsaw", 3, game.rules));
                    recalc_armor_broken(game, &mut self.ctx, defender_id);
                }
            }
        }
    }
    fn injury_roll(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str) {
        // Java: `factory.findInjuryModifiers(game, injuryContext, pAttacker, pDefender, isStab(),
        // isFoul(), isVomitLike())` — Stab has isStab=true, isFoul=false, isVomitLike=false. The
        // factory's `findInjuryModifiers` includes niggling internally, so no separate call is needed.
        if self.use_injury_modifiers {
            if let Some(defender) = game.player(defender_id) {
                let attacker = attacker_id.and_then(|aid| game.player(aid));
                let factory = InjuryModifierFactory::new(game.rules);
                for m in factory.find_injury_modifiers(game, attacker, defender, true, false, false) {
                    self.ctx.add_injury_modifier(leak_injury_modifier(m.as_ref(), attacker, defender, game.rules));
                }
            }
        }
        do_injury_roll_for_player(rng, &mut self.ctx, game, defender_id);
    }
    fn saved_by_armour(&mut self) {
        // Stab: no injury when armor holds
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
        let mut t = InjuryTypeStab::new(true); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(13), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(!t.ctx.armor_broken); assert!(t.ctx.injury.is_none());
    }
    #[test]
    fn armor_break_results_in_injury_roll() {
        let mut t = InjuryTypeStab::new(true); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(2), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken); assert!(t.ctx.injury.is_some());
    }

    #[test]
    fn initial_context_has_no_injury() {
        let t = InjuryTypeStab::new(true);
        assert!(!t.ctx.armor_broken);
        assert!(t.ctx.injury.is_none());
    }
    #[test]
    fn default_equivalent_to_new_true() {
        let t1 = InjuryTypeStab::new(true);
        let t2 = InjuryTypeStab::default();
        assert_eq!(t1.ctx.armor_broken, t2.ctx.armor_broken);
        assert!(t1.ctx.injury.is_none() && t2.ctx.injury.is_none());
    }

    #[test]
    fn new_context_uses_defender_apo_mode() {
        use ffb_model::enums::ApothecaryMode;
        let t = InjuryTypeStab::new(true);
        assert_eq!(t.injury_context().apothecary_mode, ApothecaryMode::Defender);
    }

    #[test]
    fn niggling_injury_modifier_applied_when_armor_breaks() {
        // Proves InjuryModifierFactory is now reached from injury_roll (fixes the bug where
        // Stab silently skipped skill/niggling injury modifiers).
        let mut t = InjuryTypeStab::new(true);
        let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor_and_niggling(2, 1), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken);
        assert!(t.ctx.injury_modifiers.iter().any(|m| m.name.contains("Niggling")),
            "expected a niggling injury modifier to be present, got {:?}", t.ctx.injury_modifiers);
    }
    #[test]
    fn no_niggling_injury_no_modifier() {
        let mut t = InjuryTypeStab::new(true);
        let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor_and_niggling(2, 0), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken);
        assert!(!t.ctx.injury_modifiers.iter().any(|m| m.name.contains("Niggling")));
    }

    #[test]
    fn use_injury_modifiers_false_skips_niggling_modifier_even_when_present() {
        // Java: bb2016 StabBehaviour calls `new InjuryTypeStab(false)` — no injury modifiers
        // (niggling, skill-based) are ever applied, unlike bb2020/bb2025's `new InjuryTypeStab(true)`.
        let mut t = InjuryTypeStab::new(false);
        let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor_and_niggling(2, 1), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken);
        assert!(t.ctx.injury_modifiers.is_empty(),
            "expected no injury modifiers with use_injury_modifiers=false, got {:?}", t.ctx.injury_modifiers);
    }

    fn game_with_chainsaw_defender(armour: i32) -> Game {
        use std::collections::HashSet;
        use ffb_model::model::player::Player;
        use ffb_model::model::SkillWithValue;
        use ffb_model::enums::{PlayerType, PlayerGender, SkillId};
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(Player { id: "p1".into(), name: "p1".into(), nr: 1,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour, starting_skills: vec![SkillWithValue::new(SkillId::Chainsaw)], extra_skills: vec![],
            temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
    ..Default::default() });
        Game::new(home, crate::step::framework::test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn add_defender_chainsaw_true_applies_chainsaw_armor_modifier() {
        // Java: `new InjuryTypeStab(true, true)` (used by StepTreacherous) adds the defender's
        // Chainsaw armor modifier (+3) in armourRoll (lines 66-71). Regression test for a
        // previously-dropped constructor argument: the Rust constructor only took
        // `useInjuryModifiers` and silently defaulted `addDefenderChainsaw` to false/absent.
        let mut t = InjuryTypeStab::new_with_chainsaw(true, true);
        let mut rng = GameRng::new(1);
        // Armour 8: a 6 needs the +3 chainsaw modifier to break (6 < 8, but 6+3=9 >= 8).
        t.handle_injury(&game_with_chainsaw_defender(8), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_modifiers.iter().any(|m| m.name == "Chainsaw" && m.value == 3));
    }

    #[test]
    fn send_to_box_reason_is_stabbed() {
        // Java: `new Stab()` → `super("stab", false, SendToBoxReason.STABBED)`. Regression test
        // for a previously-missing override (defaulted to `None`).
        let t = InjuryTypeStab::new(true);
        assert_eq!(t.send_to_box_reason(), Some(ffb_model::enums::SendToBoxReason::Stabbed));
    }

    #[test]
    fn falling_down_causes_turnover_defaults_true() {
        // Java: `Stab` does not override `fallingDownCausesTurnover()`, so `InjuryType`'s base
        // default (`true`) applies. Regression test for a previously-inverted override
        // (`false`) that had no basis in the Java source.
        let t = InjuryTypeStab::new(true);
        assert!(t.falling_down_causes_turnover());
    }

    #[test]
    fn failed_armour_places_prone_is_false() {
        // Java: `InjuryTypeStab`'s constructor calls `super.setFailedArmourPlacesProne(false)`.
        // Regression test for a previously-missing override (defaulted to the trait's `true`).
        let t = InjuryTypeStab::new(true);
        assert!(!t.failed_armour_places_prone());
    }

    #[test]
    fn add_defender_chainsaw_false_does_not_apply_chainsaw_modifier() {
        // Java: `new InjuryTypeStab(true)` (or explicit `false`) must NOT add the defender's
        // chainsaw modifier even if the defender happens to have the skill.
        let mut t = InjuryTypeStab::new(true);
        let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_chainsaw_defender(8), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(!t.ctx.armor_modifiers.iter().any(|m| m.name == "Chainsaw"));
    }
}
