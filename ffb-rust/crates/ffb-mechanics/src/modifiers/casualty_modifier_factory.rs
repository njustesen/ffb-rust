use ffb_model::enums::Rules;
use ffb_model::model::Player;
use crate::modifiers::modifiers::Modifier;

/// 1:1 translation of com.fumbbl.ffb.factory.mixed.CasualtyModifierFactory.
///
/// **Correction (Phase AD):** this struct's prior doc comment claimed no real skill subclass
/// ever overrides `Skill.getCasualtyModifiers()` — that was stale/wrong. `skill/mixed/Decay.java`
/// (bb2020/bb2025) registers `new CasualtyModifier("Decay", 1)`. `find_registered_modifiers`
/// below (backing `for_name`/the `ModifierAggregator.getCasualtyModifiers()` lookup path) now
/// reflects that. **Not fixed here:** whether `find_modifiers(Player)` — the live per-roll path
/// consumed by `roll_mechanic.rs`'s casualty-modifier-sum computation — should also add a Decay
/// contribution is a separate, real question. `roll_mechanic.rs`'s
/// `interpret_casualty_roll_and_add_modifiers` already carries an `_is_decay_roll` parameter
/// (currently unused) suggesting Decay's "roll twice on the Casualty table" effect is meant to be
/// a reroll trigger, not a numeric modifier-sum addition; wiring `find_registered_modifiers` into
/// `find_modifiers` without first tracing that existing mechanism risks double-counting Decay's
/// effect. Sized for its own phase, not attempted here.
pub struct CasualtyModifierFactory;

impl CasualtyModifierFactory {
    pub fn new() -> Self {
        Self
    }

    /// Java: findModifiers(Player) — skill scan + niggling count. The skill scan here only
    /// covers the niggling-injury count; see the struct doc for the real, deferred Decay gap.
    pub fn find_modifiers(&self, player: &Player) -> Vec<Modifier> {
        self.for_number(player.niggling_injuries).into_iter().collect()
    }

    /// Java: `ModifierAggregator.getCasualtyModifiers()`'s skill half. This factory (per its
    /// Java `@RulesCollection(BB2020)`/`@RulesCollection(BB2025)`) only ever exists for
    /// bb2020/bb2025 games, so `Decay` applies unconditionally here (no further rules gating).
    pub fn find_registered_modifiers() -> Vec<Modifier> {
        vec![Modifier::new("Decay", 1, Rules::Common)]
    }

    /// Java: forNumber(int) — "<n> Niggling Injury"/"Injuries" modifier, or None if n <= 0.
    pub fn for_number(&self, number: i32) -> Option<Modifier> {
        if number > 0 {
            Some(Modifier::new(casualty_niggling_name(number), number, Rules::Common))
        } else {
            None
        }
    }

    /// Java: fromName(String) — parses the leading integer off a modifier's display name.
    pub fn from_name(&self, name: &str) -> Option<Modifier> {
        let count: i32 = name.split(' ').next()?.parse().ok()?;
        self.for_number(count)
    }

    /// Java: forName(String) — checks `ModifierAggregator.getCasualtyModifiers()` first, then
    /// falls back to `fromName`.
    pub fn for_name(&self, name: &str) -> Option<Modifier> {
        Self::find_registered_modifiers()
            .into_iter()
            .find(|m| m.name == name)
            .or_else(|| self.from_name(name))
    }
}

impl Default for CasualtyModifierFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// "<n> Niggling Injury" (n == 1) or "<n> Niggling Injuries" (n != 1), matching Java's
/// `number + " Niggling Injur" + (number == 1 ? "y" : "ies")`.
fn casualty_niggling_name(count: i32) -> &'static str {
    match count {
        1 => "1 Niggling Injury",
        2 => "2 Niggling Injuries",
        3 => "3 Niggling Injuries",
        4 => "4 Niggling Injuries",
        5 => "5 Niggling Injuries",
        6 => "6 Niggling Injuries",
        7 => "7 Niggling Injuries",
        8 => "8 Niggling Injuries",
        // Beyond typical play, Java still builds this dynamically with no upper bound;
        // Modifier::name requires `&'static str`, so this rare tail leaks a bounded amount
        // of memory once per distinct count value ever observed by this process.
        _ => Box::leak(format!("{count} Niggling Injuries").into_boxed_str()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerType};

    fn player_with_nigglings(count: i32) -> Player {
        Player {
            id: "p".into(), name: "p".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: count, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        }
    }

    #[test]
    fn find_modifiers_empty_when_no_nigglings() {
        let f = CasualtyModifierFactory::new();
        let p = player_with_nigglings(0);
        assert!(f.find_modifiers(&p).is_empty());
    }

    #[test]
    fn find_modifiers_single_niggling() {
        let f = CasualtyModifierFactory::new();
        let p = player_with_nigglings(1);
        let mods = f.find_modifiers(&p);
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].name, "1 Niggling Injury");
        assert_eq!(mods[0].value, 1);
    }

    #[test]
    fn find_modifiers_pluralizes_multiple_nigglings() {
        let f = CasualtyModifierFactory::new();
        let p = player_with_nigglings(3);
        let mods = f.find_modifiers(&p);
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].name, "3 Niggling Injuries");
        assert_eq!(mods[0].value, 3);
    }

    #[test]
    fn for_number_zero_or_negative_is_none() {
        let f = CasualtyModifierFactory::new();
        assert!(f.for_number(0).is_none());
        assert!(f.for_number(-1).is_none());
    }

    #[test]
    fn from_name_parses_leading_count() {
        let f = CasualtyModifierFactory::new();
        let m = f.from_name("2 Niggling Injuries").unwrap();
        assert_eq!(m.value, 2);
        assert_eq!(m.name, "2 Niggling Injuries");
    }

    #[test]
    fn from_name_invalid_returns_none() {
        let f = CasualtyModifierFactory::new();
        assert!(f.from_name("not a number").is_none());
        assert!(f.from_name("").is_none());
    }

    #[test]
    fn for_name_matches_from_name() {
        let f = CasualtyModifierFactory::new();
        assert_eq!(f.for_name("1 Niggling Injury"), f.from_name("1 Niggling Injury"));
    }

    #[test]
    fn find_registered_modifiers_includes_decay() {
        let mods = CasualtyModifierFactory::find_registered_modifiers();
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].name, "Decay");
        assert_eq!(mods[0].value, 1);
    }

    #[test]
    fn for_name_finds_decay_via_registered_modifiers() {
        let f = CasualtyModifierFactory::new();
        let m = f.for_name("Decay").unwrap();
        assert_eq!(m.value, 1);
    }

    #[test]
    fn beyond_static_table_still_pluralizes_correctly() {
        let f = CasualtyModifierFactory::new();
        let m = f.for_number(12).unwrap();
        assert_eq!(m.name, "12 Niggling Injuries");
        assert_eq!(m.value, 12);
    }
}
