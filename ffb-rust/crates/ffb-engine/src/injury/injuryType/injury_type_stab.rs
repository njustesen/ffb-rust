/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeStab.
/// ModificationAware: stab armor roll + injury roll. savedByArmour -> None.
use ffb_model::enums::{ApothecaryMode, SendToBoxReason};
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use ffb_mechanics::modifiers::injury_modifier_factory::InjuryModifierFactory;
use ffb_mechanics::modifiers::Modifier;
use crate::injury::{InjuryContext, InjuryTypeServer, do_armor_roll, do_injury_roll_for_player, recalc_armor_broken};
use crate::injury::injuryType::modification_aware_injury_type_server::{ModificationAwareInjuryType, modification_aware_handle_injury, leak_injury_modifier, leak_armor_modifier};
use ffb_mechanics::modifiers::armor_modifier_factory::ArmorModifierFactory;

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
    fn java_type_name(&self) -> &'static str { "stab" }
    fn java_class_name(&self) -> &'static str { "Stab" }
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
    /// Java `injuryRoll` ends with the context-less `setInjury(pDefender, gameState, diceRoller)`.
    fn interprets_member_injury(&self) -> bool { true }
    fn armour_roll(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str, roll: bool) {
        stab_armour_roll(&mut self.ctx, self.add_defender_chainsaw, game, rng, attacker_id, defender_id, roll);
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
        if self.use_injury_modifiers {
            do_injury_roll_for_player(rng, &mut self.ctx, game, defender_id);
        } else {
            // Java `RollMechanic.interpretInjuryRoll` derives isStunty from a Stunty injury
            // MODIFIER in the context; bb2016's `new InjuryTypeStab(false)` gathers none, so a
            // stabbed Stunty goblin's 9 is KO, not Badly Hurt (star tier: bb2016 dark_elf vs
            // goblin seed 2).
            crate::injury::do_injury_roll_for_player_no_stunty(rng, &mut self.ctx, game, defender_id);
        }
    }
    fn saved_by_armour(&mut self) {
        // Stab: no injury when armor holds
        self.ctx.injury = None;
    }
}

/// Java `InjuryTypeStab.armourRoll`, ported in full. `InjuryTypeStabForSpp` is the same Java code
/// line for line (only the injury type differs), so both Rust types share this body.
///
/// The old port only rolled the dice: it skipped the not-already-broken guard, the
/// `ArmorModifierFactory.findArmorModifiers(isStab=true)` pass (A Sneaky Pair, bb2016 Stakes, the
/// Iron Hard Skin marker), the ignoresArmourModifiersFromSkills branch, the `roll=false`
/// re-evaluation, and the sneaky-pair second pass.
pub(crate) fn stab_armour_roll(ctx: &mut InjuryContext, add_defender_chainsaw: bool, game: &Game,
    rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str, roll: bool) {
    // Java: if (!injuryContext.isArmorBroken()) { ... }
    if ctx.armor_broken {
        return;
    }
    let defender = match game.player(defender_id) {
        Some(d) => d,
        None => {
            if roll { do_armor_roll(game, rng, ctx, defender_id); }
            return;
        }
    };
    let attacker = attacker_id.and_then(|aid| game.player(aid));
    // Java: modifiers = armorModifierFactory.findArmorModifiers(game, pAttacker, pDefender,
    //   isStab(), isFoul()) -- Stab: isStab=true, isFoul=false.
    let factory = ArmorModifierFactory::new(game.rules);
    let mut modifiers = factory.find_armor_modifiers(game, attacker, defender, true, false);
    // Java: the first modifier registered to affectsEitherArmourOrInjuryWithPartner (A Sneaky
    // Pair) is held back for a second pass, spent only if the rest did not break armour.
    let sneaky_idx = modifiers.iter().position(|m| {
        m.is_registered_to_skill_with_property(NamedProperties::AFFECTS_EITHER_ARMOUR_OR_INJURY_WITH_PARTNER)
    });
    let sneaky = sneaky_idx.map(|i| modifiers.remove(i));
    for m in modifiers {
        // Java's armour modifiers are a Set of the skills' registered objects; a repeat collapses.
        if ctx.armor_modifiers.iter().any(|x| x.name == m.get_name()) {
            continue;
        }
        ctx.add_armor_modifier(leak_armor_modifier(m.as_ref(), attacker, defender, game.rules));
    }
    if defender.has_unused_skill_with_property(NamedProperties::IGNORES_ARMOUR_MODIFIERS_FROM_SKILLS) {
        // Java: addArmorModifiers(defender.getSkillWithProperty(ignoresArmourModifiersFromSkills)
        //   .getArmorModifiers()) -- Iron Hard Skin's value-0 marker, which names the skill in
        //   the report (the factory already returned the same object, so the Set collapses it).
        if !ctx.armor_modifiers.iter().any(|m| m.name == "Iron Hard Skin") {
            ctx.add_armor_modifier(Modifier::new("Iron Hard Skin", 0, game.rules));
        }
    } else if add_defender_chainsaw && defender.has_skill_property(NamedProperties::BLOCKS_LIKE_CHAINSAW) {
        // Java: addDefenderChainsaw ? defender's Chainsaw armour modifiers (+3) -- used by e.g.
        // StepTreacherous's `new InjuryTypeStab(true, true)`.
        ctx.add_armor_modifier(Modifier::new("Chainsaw", 3, game.rules));
    }
    // Java: if (roll) setArmorRoll(rollArmour()); setArmorBroken(isArmourBroken(...)).
    if roll {
        do_armor_roll(game, rng, ctx, defender_id);
    } else {
        recalc_armor_broken(game, ctx, defender_id);
    }
    if !ctx.armor_broken {
        if let Some(sneaky) = sneaky {
            ctx.add_armor_modifier(leak_armor_modifier(sneaky.as_ref(), attacker, defender, game.rules));
            recalc_armor_broken(game, ctx, defender_id);
        }
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
    // NOTE (test equalization): default_equivalent_to_new_true pruned — Rust `Default` impl
    // plumbing; Java has no default constructor. new_context_uses_defender_apo_mode pruned —
    // Rust stores the apothecary mode on the context at construction; Java's InjuryContext has
    // no such field (the mode travels through handleInjury/InjuryResult), so no faithful twin.

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

    /// Java: isStunty comes from a Stunty injury MODIFIER; bb2016 `new InjuryTypeStab(false)`
    /// gathers none, so a Stunty defender's injury total of 9 is KO (not Badly Hurt).
    #[test]
    fn stab_without_injury_modifiers_ignores_stunty() {
        use ffb_model::enums::{SkillId, PS_KNOCKED_OUT};
        use ffb_model::model::SkillWithValue;
        let mut checked = false;
        for seed in 0..500u64 {
            let mut game = game_with_armor_and_niggling(2, 0);
            game.team_home.players[0].extra_skills.push(SkillWithValue::new(SkillId::Stunty));
            let mut t = InjuryTypeStab::new(false);
            let mut rng = GameRng::new(seed);
            t.handle_injury(&game, &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
            if let Some([a, b]) = t.ctx.injury_roll {
                if a + b == 9 {
                    assert_eq!(t.ctx.injury.map(|s| s.base()), Some(PS_KNOCKED_OUT),
                        "a 9 without a Stunty modifier is KO");
                    checked = true;
                    break;
                }
            }
        }
        assert!(checked, "no seed produced an injury total of 9");
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

    fn stab_player(id: &str, armour: i32, skills: Vec<ffb_model::enums::SkillId>) -> ffb_model::model::player::Player {
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

    /// Stabber + an A-Sneaky-Pair partner both marking the victim (bb2025).
    fn sneaky_pair_game(victim_av: i32) -> Game {
        use ffb_model::enums::{SkillId, PlayerState, PS_STANDING};
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(stab_player("stabber", 7, vec![SkillId::Stab, SkillId::ASneakyPair]));
        home.players.push(stab_player("partner", 7, vec![SkillId::ASneakyPair]));
        let mut away = crate::step::framework::test_team("away", 0);
        away.players.push(stab_player("victim", victim_av, vec![]));
        let mut game = Game::new(home, away, Rules::Bb2025);
        for (id, x, y) in [("stabber", 5, 5), ("partner", 7, 5), ("victim", 6, 5)] {
            game.field_model.set_player_coordinate(id, FieldCoordinate::new(x, y));
            game.field_model.set_player_state(id, PlayerState::new(PS_STANDING));
        }
        game
    }

    fn seed_with_armour_total(total: i32) -> u64 {
        let mut seed = 0u64;
        loop {
            let mut probe = GameRng::new(seed);
            if probe.d6() + probe.d6() == total { return seed; }
            seed += 1;
            assert!(seed < 10_000, "no seed found");
        }
    }

    /// Java `InjuryTypeStab.armourRoll`: A Sneaky Pair is held back and only spent when the
    /// roll without it does not break armour. A 9 against AV10 breaks only with its +1.
    #[test]
    fn sneaky_pair_breaks_armour_only_on_the_second_pass() {
        let game = sneaky_pair_game(10);
        let mut t = InjuryTypeStab::new(true);
        let mut rng = GameRng::new(seed_with_armour_total(9));
        t.armour_roll(&game, &mut rng, Some("stabber"), "victim", true);
        assert!(t.ctx.armor_broken, "A Sneaky Pair +1 must break AV10 on a 9");
        assert!(t.ctx.armor_modifiers.iter().any(|m| m.name == "A Sneaky Pair"));
    }

    /// When the roll alone breaks armour, the held-back Sneaky Pair is never added.
    #[test]
    fn sneaky_pair_not_added_when_the_roll_already_breaks() {
        let game = sneaky_pair_game(7);
        let mut t = InjuryTypeStab::new(true);
        let mut rng = GameRng::new(seed_with_armour_total(10));
        t.armour_roll(&game, &mut rng, Some("stabber"), "victim", true);
        assert!(t.ctx.armor_broken);
        assert!(!t.ctx.armor_modifiers.iter().any(|m| m.name == "A Sneaky Pair"));
    }

    /// Java's whole armourRoll sits inside `if (!injuryContext.isArmorBroken())`.
    #[test]
    fn already_broken_armour_adds_nothing() {
        let game = sneaky_pair_game(10);
        let mut t = InjuryTypeStab::new(true);
        t.ctx.armor_broken = true;
        let mut rng = GameRng::new(1);
        t.armour_roll(&game, &mut rng, Some("stabber"), "victim", true);
        assert!(t.ctx.armor_roll.is_none());
        assert!(t.ctx.armor_modifiers.is_empty());
    }

    /// Java names Iron Hard Skin's value-0 marker on a stab against an IHS defender.
    #[test]
    fn iron_hard_skin_defender_is_named() {
        use ffb_model::enums::SkillId;
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(stab_player("stabber", 7, vec![SkillId::Stab]));
        let mut away = crate::step::framework::test_team("away", 0);
        away.players.push(stab_player("victim", 10, vec![SkillId::IronHardSkin]));
        let game = Game::new(home, away, Rules::Bb2025);
        let mut t = InjuryTypeStab::new(true);
        let mut rng = GameRng::new(1);
        t.armour_roll(&game, &mut rng, Some("stabber"), "victim", true);
        assert_eq!(t.ctx.armor_modifiers.iter().filter(|m| m.name == "Iron Hard Skin").count(), 1);
    }
}
