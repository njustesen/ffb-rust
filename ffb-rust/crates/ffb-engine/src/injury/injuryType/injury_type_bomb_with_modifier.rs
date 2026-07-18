/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeBombWithModifier, via the
/// shared com.fumbbl.ffb.server.injury.injuryType.AbstractInjuryTypeBombWithModifier logic:
///   1. `placedProneCausesInjuryRoll` (Ball-and-Chain) skips the armor roll entirely — armor is
///      treated as already broken.
///   2. Otherwise roll armor; if not broken, add the dynamic `SpecialEffect.BOMB` armor
///      modifier(s) from `ArmorModifierFactory` (only non-empty when the `bombUsesMb` game
///      option is enabled — and never in BB2025, which has no such modifier at all) and
///      recompute.
///   3. If broken: roll injury with the normal per-skill injury modifiers, and — only if no
///      special-effect armor modifier was added in step 2 — also add the dynamic
///      `SpecialEffect.BOMB` injury modifier(s) from `InjuryModifierFactory` (same `bombUsesMb`
///      gating).
///   4. Else: PRONE.
use ffb_model::enums::{ApothecaryMode, PlayerState, PS_PRONE};
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::SpecialEffect;
use ffb_model::option::game_option_id::BOMB_USES_MB;
use ffb_model::option::util_game_option::is_option_enabled;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use ffb_mechanics::modifiers::armor_modifier::ArmorModifier;
use ffb_mechanics::modifiers::armor_modifier_factory::ArmorModifierFactory;
use ffb_mechanics::modifiers::injury_modifier_factory::InjuryModifierFactory;
use ffb_mechanics::modifiers::Modifier;
use ffb_model::model::player::Player;
use crate::injury::{InjuryContext, InjuryTypeServer, do_armor_roll, recalc_armor_broken, do_injury_roll_for_player};
use crate::injury::injuryType::modification_aware_injury_type_server::leak_injury_modifier;

/// Java: `ArmorModifier` instances are transient (owned by the special-effect factory); bridging
/// into the `'static`-name `Modifier` used by `InjuryContext` needs a leak, same convention as
/// `injury_type_block.rs`'s `leak_modifier` / `modification_aware_injury_type_server::leak_injury_modifier`.
fn leak_armor_modifier(m: &dyn ArmorModifier, attacker: Option<&Player>, defender: &Player, rules: ffb_model::enums::Rules) -> Modifier {
    let name: &'static str = Box::leak(m.get_name().to_owned().into_boxed_str());
    Modifier::new(name, m.get_modifier(attacker, defender), rules)
}

pub struct InjuryTypeBombWithModifier { ctx: InjuryContext }
impl InjuryTypeBombWithModifier { pub fn new() -> Self { Self { ctx: InjuryContext::new(ApothecaryMode::Defender) } } }
impl Default for InjuryTypeBombWithModifier { fn default() -> Self { Self::new() } }

impl InjuryTypeServer for InjuryTypeBombWithModifier {
    fn handle_injury(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, _from_coord: Option<FieldCoordinate>, _old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
        self.ctx.defender_id = Some(defender_id.to_owned());
        self.ctx.attacker_id = attacker_id.map(str::to_owned);
        self.ctx.defender_coordinate = Some(coord);
        self.ctx.apothecary_mode = apo_mode;

        // Java: `boolean skipArmourRoll = pDefender.hasSkillProperty(placedProneCausesInjuryRoll);`
        let skip_armour_roll = game.player(defender_id)
            .map(|d| d.has_skill_property(NamedProperties::PLACED_PRONE_CAUSES_INJURY_ROLL))
            .unwrap_or(false);
        if skip_armour_roll {
            self.ctx.armor_broken = true;
        } else {
            do_armor_roll(game, rng, &mut self.ctx, defender_id);
        }

        let mut added_special_armor_modifier = false;
        if !self.ctx.armor_broken {
            if let Some(defender) = game.player(defender_id) {
                let mut factory = ArmorModifierFactory::new(game.rules);
                factory.set_use_all(is_option_enabled(game, BOMB_USES_MB));
                let attacker = attacker_id.and_then(|aid| game.player(aid));
                let mods = factory.special_effect_armour_modifiers(SpecialEffect::BOMB, defender);
                added_special_armor_modifier = !mods.is_empty();
                for m in mods {
                    self.ctx.add_armor_modifier(leak_armor_modifier(m.as_ref(), attacker, defender, game.rules));
                }
                recalc_armor_broken(game, &mut self.ctx, defender_id);
            }
        }

        if self.ctx.armor_broken {
            // Java: `((InjuryModifierFactory) game.getFactory(...)).findInjuryModifiers(game, injuryContext,
            // pAttacker, pDefender, isStab(), isFoul(), isVomitLike())` — Bomb never overrides
            // isStab/isFoul/isVomitLike, all false.
            if let Some(defender) = game.player(defender_id) {
                let attacker = attacker_id.and_then(|aid| game.player(aid));
                let mut factory = InjuryModifierFactory::new(game.rules);
                for m in factory.find_injury_modifiers(game, attacker, defender, false, false, false) {
                    self.ctx.add_injury_modifier(leak_injury_modifier(m.as_ref(), attacker, defender, game.rules));
                }
                if !added_special_armor_modifier {
                    factory.set_use_all(is_option_enabled(game, BOMB_USES_MB));
                    for m in factory.special_effect_injury_modifiers(SpecialEffect::BOMB) {
                        self.ctx.add_injury_modifier(leak_injury_modifier(m.as_ref(), attacker, defender, game.rules));
                    }
                }
            }
            do_injury_roll_for_player(rng, &mut self.ctx, game, defender_id);
        } else {
            self.ctx.injury = Some(PlayerState::new(PS_PRONE));
        }
    }
    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    fn falling_down_causes_turnover(&self) -> bool { false }
    /// Java: `new Bomb()` constructor passes `SendToBoxReason.BOMB` to the `InjuryType` base class.
    fn send_to_box_reason(&self) -> Option<ffb_model::enums::SendToBoxReason> {
        Some(ffb_model::enums::SendToBoxReason::Bomb)
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
        let mut t = InjuryTypeBombWithModifier::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(13), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert_eq!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test]
    fn armor_break_results_in_injury_roll() {
        let mut t = InjuryTypeBombWithModifier::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(2), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken); assert_ne!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test]
    fn does_not_cause_turnover() { assert!(!InjuryTypeBombWithModifier::new().falling_down_causes_turnover()); }
    #[test]
    fn send_to_box_reason_is_bomb() {
        use ffb_model::enums::SendToBoxReason;
        assert_eq!(InjuryTypeBombWithModifier::new().send_to_box_reason(), Some(SendToBoxReason::Bomb));
    }
    /// Java: `AbstractInjuryTypeBombWithModifier` only adds the "Bomb" special-effect armor
    /// modifier when `ArmorModifierFactory.specialEffectArmourModifiers(BOMB, defender)` returns
    /// something non-empty, which requires the legacy `bombUsesMb` game option to be enabled —
    /// and BB2025's `ArmorModifiers` catalog has no "Bomb" entry at all (see
    /// `ffb_mechanics::modifiers::bb2025::armor_modifiers`), so under BB2025 with default
    /// options no modifier is ever added, regardless of the option.
    #[test]
    fn no_bomb_armor_modifier_under_bb2025_default_options() {
        let mut t = InjuryTypeBombWithModifier::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(13), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_modifiers.is_empty());
    }
    #[test]
    fn no_bomb_armor_modifier_under_bb2020_when_option_disabled() {
        let mut game = game_with_armor(13);
        game.rules = Rules::Bb2020;
        let mut t = InjuryTypeBombWithModifier::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game, &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_modifiers.is_empty());
    }
    #[test]
    fn bomb_armor_modifier_is_added_under_bb2020_when_bomb_uses_mb_enabled() {
        let mut game = game_with_armor(13);
        game.rules = Rules::Bb2020;
        game.options.set(ffb_model::option::game_option_id::BOMB_USES_MB, "true");
        let mut t = InjuryTypeBombWithModifier::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game, &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_modifiers.iter().any(|m| m.name == "Bomb"));
    }
    // Note: Java's `AbstractInjuryTypeBombWithModifier` also skips the armor roll entirely for
    // any defender with the `placedProneCausesInjuryRoll` property (Ball-and-Chain in all three
    // editions). The `handle_injury` logic above implements that check via
    // `NamedProperties::PLACED_PRONE_CAUSES_INJURY_ROLL`, but `SkillId::BallAndChain::properties()`
    // (crates/ffb-model/src/enums/skill_id.rs) does not currently register that property name —
    // a pre-existing gap outside this file's scope — so no skill-driven regression test for the
    // skip path is added here; the property-name lookup itself is covered indirectly by
    // `armor_save_results_in_prone`/`armor_break_results_in_injury_roll` exercising the
    // non-skip branch.
    #[test]
    fn default_equivalent_to_new() {
        let t1 = InjuryTypeBombWithModifier::new();
        let t2 = InjuryTypeBombWithModifier::default();
        assert_eq!(t1.ctx.armor_broken, t2.ctx.armor_broken);
        assert!(t1.ctx.injury.is_none() && t2.ctx.injury.is_none());
    }

    fn game_with_attacker_and_defender(attacker_skills: Vec<ffb_model::enums::SkillId>, defender_armour: i32) -> Game {
        use std::collections::HashSet;
        use ffb_model::model::player::Player;
        use ffb_model::model::SkillWithValue;
        use ffb_model::enums::{PlayerType, PlayerGender};
        let make_player = |id: &str, armour: i32, skills: Vec<ffb_model::enums::SkillId>| Player {
            id: id.into(), name: id.into(), nr: 1,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour, starting_skills: skills.into_iter().map(SkillWithValue::new).collect(), extra_skills: vec![],
            temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(make_player("attacker", 7, attacker_skills));
        let mut away = crate::step::framework::test_team("away", 0);
        away.players.push(make_player("defender", defender_armour, vec![]));
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn mighty_blow_adds_injury_modifier() {
        use ffb_model::enums::SkillId;
        let game = game_with_attacker_and_defender(vec![SkillId::MightyBlow], 2);
        let mut t = InjuryTypeBombWithModifier::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game, &mut rng, Some("attacker"), "defender", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.injury_modifiers.iter().any(|m| m.name == "Mighty Blow"));
    }
    #[test]
    fn no_mighty_blow_no_injury_modifier() {
        let game = game_with_attacker_and_defender(vec![], 2);
        let mut t = InjuryTypeBombWithModifier::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game, &mut rng, Some("attacker"), "defender", coord(), None, None, ApothecaryMode::Defender);
        assert!(!t.ctx.injury_modifiers.iter().any(|m| m.name == "Mighty Blow"));
    }
}
