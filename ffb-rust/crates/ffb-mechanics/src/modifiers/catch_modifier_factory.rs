use ffb_model::enums::{Rules, SkillId};
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::{CatchScatterThrowInMode, Player};
use ffb_model::util::util_disturbing_presence::UtilDisturbingPresence;
use ffb_model::util::util_player::UtilPlayer;
use crate::modifiers::bb2025::catch_modifier_collection::CatchModifierCollection as Bb2025Collection;
use crate::modifiers::catch_context::CatchContext;
use crate::modifiers::catch_modifier::CatchModifier;
use crate::modifiers::catch_modifier_collection::CatchModifierCollection;
use crate::modifiers::modifier_type::ModifierType;

/// Edition-agnostic trait for a catch modifier collection.
///
/// Only `get_modifiers` is needed: `find_applicable` on `CatchModifierFactory` implements the
/// full GenerifiedModifierFactory.findModifiers selection logic itself (REGULAR predicate filter
/// + count-based TACKLEZONE/DISTURBING_PRESENCE selection) rather than delegating to a blanket
/// per-collection `find_applicable`, which would incorrectly include every modifier regardless
/// of the actual tacklezone/disturbing-presence count.
trait CatchCollection: Send + Sync {
    fn get_modifiers(&self) -> &[CatchModifier];
}

impl CatchCollection for CatchModifierCollection {
    fn get_modifiers(&self) -> &[CatchModifier] { self.get_modifiers() }
}

impl CatchCollection for Bb2025Collection {
    fn get_modifiers(&self) -> &[CatchModifier] { self.get_modifiers() }
}

/// 1:1 translation of com.fumbbl.ffb.factory.CatchModifierFactory.
///
/// Holds the edition-specific CatchModifierCollection and provides modifier
/// lookup. Unlike Java's generified factory, this Rust version holds the
/// collection directly rather than via a Scanner / DI framework.
pub struct CatchModifierFactory {
    collection: Box<dyn CatchCollection>,
}

impl CatchModifierFactory {
    /// Construct a factory for the given rules edition.
    pub fn for_rules(rules: Rules) -> Self {
        let collection: Box<dyn CatchCollection> = match rules {
            Rules::Bb2025 | Rules::Common => Box::new(Bb2025Collection::new()),
            _ => Box::new(CatchModifierCollection::new()),
        };
        Self { collection }
    }

    /// Returns the modifiers applicable to the given context.
    /// 1:1 translation of GenerifiedModifierFactory.findModifiers + CatchModifierFactory overrides.
    ///
    /// REGULAR modifiers are filtered by predicate (e.g. Pouring Rain weather check).
    /// TACKLEZONE: one modifier selected by counting opposing players with tacklezones
    /// adjacent to the catcher, unless the catcher has ignoreTacklezonesWhenCatching.
    /// DISTURBING_PRESENCE: one modifier selected by counting opposing Disturbing Presence
    /// players within range (isAffectedByDisturbingPresence is always true for catch).
    pub fn find_applicable<'a>(&'a self, context: &CatchContext<'_>) -> Vec<&'a CatchModifier> {
        let mut result: Vec<&'a CatchModifier> = self.collection.get_modifiers().iter()
            .filter(|m| m.get_type() == ModifierType::REGULAR && m.applies_to_context(context))
            .collect();

        let Some(player) = context.player else { return result; };

        // Java: isAffectedByTackleZones → !player.hasSkillProperty(ignoreTacklezonesWhenCatching)
        // Note: CatchModifier does not override RollModifier.getMultiplier(), so Java's default
        // (getMultiplier() == getModifier()) applies — selection is by get_modifier(), not a
        // separate multiplier field.
        if !player.has_skill_property(NamedProperties::IGNORE_TACKLEZONES_WHEN_CATCHING) {
            let count = UtilPlayer::find_tacklezones(context.game, &player.id) as i32;
            if let Some(tz_mod) = self.collection.get_modifiers().iter()
                .find(|m| m.get_type() == ModifierType::TACKLEZONE && m.get_modifier() == count)
            {
                result.push(tz_mod);
            }
        }

        // Java: isAffectedByDisturbingPresence is always true for CatchModifierFactory.
        let dp_count = UtilDisturbingPresence::find_opposing_disturbing_presences(context.game, &player.id);
        if let Some(dp_mod) = self.collection.get_modifiers().iter()
            .find(|m| m.get_type() == ModifierType::DISTURBING_PRESENCE && m.get_modifier() == dp_count)
        {
            result.push(dp_mod);
        }

        result
    }

    /// Returns skill-based catch modifiers for the catcher.
    /// 1:1 translation of GenerifiedModifierFactory skill iteration for CatchModifierFactory.
    ///
    /// Java: common.ExtraArms registers CatchModifier("Extra Arms", -1, REGULAR).
    ///       common.DivingCatch registers CatchModifier("Diving Catch", -1, REGULAR) only on accurate pass/bomb.
    pub fn find_skill_modifiers(&self, context: &CatchContext<'_>) -> Vec<CatchModifier> {
        let Some(player) = context.player else { return vec![]; };
        let mut result = Vec::new();
        for skill_id in player.all_skill_ids() {
            match skill_id {
                SkillId::ExtraArms => {
                    result.push(CatchModifier::new("Extra Arms", -1, crate::modifiers::modifier_type::ModifierType::REGULAR));
                }
                SkillId::DivingCatch => {
                    // Only applies to accurate pass/bomb.
                    if matches!(context.catch_mode, CatchScatterThrowInMode::CatchAccuratePass | CatchScatterThrowInMode::CatchAccurateBomb) {
                        result.push(CatchModifier::new("Diving Catch", -1, crate::modifiers::modifier_type::ModifierType::REGULAR));
                    }
                }
                _ => {}
            }
        }
        result
    }

    /// Java: `ModifierAggregator.getCatchModifiers()`'s skill half —
    /// `skillFactory.getSkills().stream().flatMap(skill -> skill.getCatchModifiers().stream())`.
    /// Every skill's statically-registered modifier objects, unfiltered by any predicate
    /// (Java's `Skill.getCatchModifiers()` returns the raw registered list; predicates are only
    /// evaluated later by callers like `GenerifiedModifierFactory.findModifiers`, not here).
    /// Only `common.ExtraArms`/`common.DivingCatch` register a `CatchModifier` in the Java source.
    pub fn find_registered_modifiers(rules: Rules) -> Vec<CatchModifier> {
        let _ = rules; // Both registrants are edition-agnostic (no @RulesCollection restriction).
        let mut result = Vec::new();
        for skill_id in ffb_model::factory::skill_factory::SkillFactory::new().get_skills() {
            match skill_id {
                SkillId::ExtraArms => {
                    result.push(CatchModifier::new("Extra Arms", -1, crate::modifiers::modifier_type::ModifierType::REGULAR));
                }
                SkillId::DivingCatch => {
                    result.push(CatchModifier::new("Diving Catch", -1, crate::modifiers::modifier_type::ModifierType::REGULAR));
                }
                _ => {}
            }
        }
        result
    }

    /// Compute the catch minimum roll from the catcher and applicable modifiers.
    /// 1:1 translation of AgilityMechanic.minimumRollCatch (BB2025: max(2, agility + sum)).
    pub fn minimum_roll_catch(player: &Player, modifiers: &[&CatchModifier]) -> i32 {
        let total: i32 = modifiers.iter().map(|m| m.get_modifier()).sum();
        2_i32.max(player.agility_with_modifiers() + total)
    }
}

impl Default for CatchModifierFactory {
    fn default() -> Self {
        Self::for_rules(Rules::Bb2025)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modifiers::catch_context::CatchContext;
    use ffb_model::enums::{Rules, Weather};
    use ffb_model::model::{Game, Team, Player};
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::model::CatchScatterThrowInMode;

    fn empty_team(id: &str) -> Team {
        Team {
            id: id.into(), name: id.into(), race: "Human".into(),
            roster_id: "human".into(), coach: "c".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0,
            dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![],
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game(weather: Weather) -> Game {
        let mut game = Game::new(empty_team("home"), empty_team("away"), Rules::Bb2025);
        game.field_model.weather = weather;
        game
    }

    fn minimal_player(agility: i32) -> Player {
        Player {
            id: "p1".into(), name: "Joe".into(), nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        }
    }

    #[test]
    fn nice_weather_no_modifiers() {
        let game = make_game(Weather::Nice);
        let player = minimal_player(3);
        let factory = CatchModifierFactory::for_rules(Rules::Bb2025);
        let ctx = CatchContext::new(&game, Some(&player), CatchScatterThrowInMode::CatchScatter, None);
        let mods = factory.find_applicable(&ctx);
        // Tacklezone and disturbing presence modifiers need game state; without any on-pitch
        // opponents the only potential trigger is weather. Nice → no weather modifier.
        // Inaccurate-pass modifier fires for CATCH_SCATTER — that is expected.
        // Only check that the pouring-rain modifier is absent.
        assert!(!mods.iter().any(|m| m.get_name() == "Pouring Rain"));
    }

    #[test]
    fn pouring_rain_adds_modifier() {
        let game = make_game(Weather::PouringRain);
        let player = minimal_player(3);
        let factory = CatchModifierFactory::for_rules(Rules::Bb2025);
        let ctx = CatchContext::new(&game, Some(&player), CatchScatterThrowInMode::CatchHandOff, None);
        let mods = factory.find_applicable(&ctx);
        assert!(mods.iter().any(|m| m.get_name() == "Pouring Rain"),
            "Pouring rain modifier should be present");
    }

    /// Regression test: Java's GenerifiedModifierFactory.findModifiers selects exactly one
    /// TACKLEZONE modifier matching the actual adjacent-opponent count, and exactly one
    /// DISTURBING_PRESENCE modifier matching the actual count — not every registered modifier
    /// in the collection. Previously find_applicable delegated to a blanket predicate filter
    /// that returned all 8 tacklezone + 11 disturbing-presence modifiers unconditionally.
    #[test]
    fn find_applicable_selects_single_tacklezone_modifier_by_count() {
        use ffb_model::model::Team;
        use ffb_model::enums::{PS_STANDING, PlayerState};
        use ffb_model::types::FieldCoordinate;

        let catcher = minimal_player(3);
        let mut opponent = minimal_player(3);
        opponent.id = "opp1".into();

        let home = Team {
            id: "home".into(), name: "home".into(), race: "Human".into(),
            roster_id: "human".into(), coach: "c".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0,
            bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0, assistant_coaches: 0,
            fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![catcher.clone()],
            vampire_lord: false, necromancer: false,
        };
        let away = Team {
            id: "away".into(), name: "away".into(), race: "Human".into(),
            roster_id: "human".into(), coach: "c".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0,
            bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0, assistant_coaches: 0,
            fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![opponent.clone()],
            vampire_lord: false, necromancer: false,
        };
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.field_model.weather = Weather::Nice;
        game.field_model.set_player_coordinate(&catcher.id, FieldCoordinate::new(5, 5));
        game.field_model.set_player_state(&catcher.id, PlayerState(PS_STANDING));
        game.field_model.set_player_coordinate(&opponent.id, FieldCoordinate::new(6, 5));
        game.field_model.set_player_state(&opponent.id, PlayerState(PS_STANDING));

        let factory = CatchModifierFactory::for_rules(Rules::Bb2025);
        let ctx = CatchContext::new(&game, Some(&catcher), CatchScatterThrowInMode::CatchHandOff, None);
        let mods = factory.find_applicable(&ctx);

        let tz_mods: Vec<_> = mods.iter().filter(|m| m.get_type() == crate::modifiers::modifier_type::ModifierType::TACKLEZONE).collect();
        assert_eq!(tz_mods.len(), 1, "exactly one tacklezone modifier should be selected, got: {:?}",
            mods.iter().map(|m| m.get_name()).collect::<Vec<_>>());
        assert_eq!(tz_mods[0].get_name(), "1 Tacklezone");

        // No disturbing-presence opponents in range → no DP modifier selected.
        assert!(!mods.iter().any(|m| m.get_type() == crate::modifiers::modifier_type::ModifierType::DISTURBING_PRESENCE));
    }

    #[test]
    fn minimum_roll_catch_base() {
        let player = minimal_player(3);
        let min = CatchModifierFactory::minimum_roll_catch(&player, &[]);
        assert_eq!(min, 3);
    }

    #[test]
    fn minimum_roll_catch_never_below_two() {
        let player = minimal_player(1);
        let modifier = crate::modifiers::catch_modifier::CatchModifier::new(
            "Bonus", -5,
            crate::modifiers::modifier_type::ModifierType::REGULAR,
        );
        let min = CatchModifierFactory::minimum_roll_catch(&player, &[&modifier]);
        assert_eq!(min, 2);
    }

    #[test]
    fn minimum_roll_catch_with_positive_modifier_raises_minimum() {
        let player = minimal_player(3);
        let modifier = crate::modifiers::catch_modifier::CatchModifier::new(
            "Gloves", 1,
            crate::modifiers::modifier_type::ModifierType::REGULAR,
        );
        let min = CatchModifierFactory::minimum_roll_catch(&player, &[&modifier]);
        assert_eq!(min, 4); // 3 + 1 = 4
    }

    #[test]
    fn find_registered_modifiers_includes_extra_arms_and_diving_catch() {
        let mods = CatchModifierFactory::find_registered_modifiers(Rules::Bb2025);
        assert!(mods.iter().any(|m| m.get_name() == "Extra Arms"));
        assert!(mods.iter().any(|m| m.get_name() == "Diving Catch"));
        assert_eq!(mods.len(), 2);
    }

    fn player_with_skill(agility: i32, skill_id: ffb_model::enums::SkillId) -> Player {
        use ffb_model::model::SkillWithValue;
        let mut p = minimal_player(agility);
        p.starting_skills.push(SkillWithValue::new(skill_id));
        p
    }

    #[test]
    fn find_skill_modifiers_extra_arms_always_applies() {
        let game = make_game(Weather::Nice);
        let player = player_with_skill(3, ffb_model::enums::SkillId::ExtraArms);
        let factory = CatchModifierFactory::for_rules(Rules::Bb2025);
        let ctx = CatchContext::new(&game, Some(&player), CatchScatterThrowInMode::CatchHandOff, None);
        let mods = factory.find_skill_modifiers(&ctx);
        assert!(mods.iter().any(|m| m.get_name() == "Extra Arms"));
        assert_eq!(mods.iter().find(|m| m.get_name() == "Extra Arms").unwrap().get_modifier(), -1);
    }

    #[test]
    fn find_skill_modifiers_diving_catch_on_accurate_pass() {
        let game = make_game(Weather::Nice);
        let player = player_with_skill(3, ffb_model::enums::SkillId::DivingCatch);
        let factory = CatchModifierFactory::for_rules(Rules::Bb2025);
        let ctx = CatchContext::new(&game, Some(&player), CatchScatterThrowInMode::CatchAccuratePass, None);
        let mods = factory.find_skill_modifiers(&ctx);
        assert!(mods.iter().any(|m| m.get_name() == "Diving Catch"), "Diving Catch should apply on accurate pass");
    }

    #[test]
    fn find_skill_modifiers_diving_catch_not_on_inaccurate() {
        let game = make_game(Weather::Nice);
        let player = player_with_skill(3, ffb_model::enums::SkillId::DivingCatch);
        let factory = CatchModifierFactory::for_rules(Rules::Bb2025);
        let ctx = CatchContext::new(&game, Some(&player), CatchScatterThrowInMode::CatchScatter, None);
        let mods = factory.find_skill_modifiers(&ctx);
        assert!(!mods.iter().any(|m| m.get_name() == "Diving Catch"), "Diving Catch should NOT apply on scatter");
    }

    #[test]
    fn find_skill_modifiers_no_player_returns_empty() {
        let game = make_game(Weather::Nice);
        let factory = CatchModifierFactory::for_rules(Rules::Bb2025);
        let ctx = CatchContext::new(&game, None, CatchScatterThrowInMode::CatchAccuratePass, None);
        let mods = factory.find_skill_modifiers(&ctx);
        assert!(mods.is_empty());
    }
}
