use std::collections::HashMap;

use crate::model::inducement_set::InducementSet;

/// 1:1 translation of com.fumbbl.ffb.factory.PrayerFactory (abstract base).
///
/// Java's abstract base holds a `Map<Integer, Prayer>` field and shared lookup/sort logic,
/// with `intensivePrayer`/`valueOf`/`initialize` left abstract for the BB2020/BB2025
/// subclasses. Rust has no inheritance and `bb2020::Prayer`/`bb2025::Prayer` are distinct
/// concrete enum types (not a shared class hierarchy), so the shared logic below is a
/// default-method trait generic over the concrete prayer type instead of a base class;
/// `crate::factory::bb2020::prayer_factory` and `crate::factory::bb2025::prayer_factory`
/// implement it.
pub trait PrayerFactory {
    type PrayerT: Copy + PartialEq;

    fn prayers(&self) -> &HashMap<i32, Self::PrayerT>;
    fn prayer_name(prayer: Self::PrayerT) -> &'static str;
    fn prayer_affects_both_teams(prayer: Self::PrayerT) -> bool;

    /// Java: forName(String) — case-insensitive display-name lookup.
    fn for_name(&self, name: &str) -> Option<Self::PrayerT> {
        self.prayers()
            .values()
            .find(|p| Self::prayer_name(**p).eq_ignore_ascii_case(name))
            .copied()
    }

    /// Java: forRoll(int).
    fn for_roll(&self, roll: i32) -> Option<Self::PrayerT> {
        self.prayers().get(&roll).copied()
    }

    /// Java: allPrayerRolls().
    fn all_prayer_rolls(&self) -> Vec<i32> {
        self.prayers().keys().copied().collect()
    }

    /// Java: availablePrayerRolls(InducementSet, InducementSet).
    fn available_prayer_rolls(
        &self,
        team_inducements: &InducementSet,
        opponent_inducements: &InducementSet,
    ) -> Vec<i32> {
        self.prayers()
            .iter()
            .filter(|(_, prayer)| {
                let name = Self::prayer_name(**prayer);
                !team_inducements.has_prayer(name)
                    && !(Self::prayer_affects_both_teams(**prayer) && opponent_inducements.has_prayer(name))
            })
            .map(|(roll, _)| *roll)
            .collect()
    }

    /// Java: sort(Set<Prayer>) — prayers in roll order, filtered to the given set.
    fn sort(&self, unsorted: &[Self::PrayerT]) -> Vec<Self::PrayerT> {
        let mut entries: Vec<(i32, Self::PrayerT)> =
            self.prayers().iter().map(|(k, v)| (*k, *v)).collect();
        entries.sort_by_key(|(roll, _)| *roll);
        entries
            .into_iter()
            .filter(|(_, p)| unsorted.contains(p))
            .map(|(_, p)| p)
            .collect()
    }
}

/// Java: `game.getFactory(Factory.PRAYER).forName(name)` for a game of `rules` (BACKLOG §O replay
/// JSON). Returns the prayer as its ENUM-CONSTANT name, the form the Rust engine keeps in
/// `InducementSet.prayers` (the prayer handlers match on it). `Ok(None)` = Java `forName` null.
/// BB2020's factory holds the exhibition table plus, when `INDUCEMENT_PRAYERS_USE_LEAGUE_TABLE`
/// is enabled (default true), the league-only prayers -- so a league prayer is unknown in an
/// exhibition-table game, exactly as in Java. BB2016 has no prayer factory: `Err`.
pub fn for_name_in(
    rules: crate::enums::Rules,
    options: &crate::model::game_options::GameOptions,
    name: &str,
) -> Result<Option<&'static str>, String> {
    use crate::enums::Rules;
    use crate::util::java_json::java_equals_ignore_case;
    match rules {
        Rules::Bb2020 => {
            let use_league_table = options
                .get_option_with_default(crate::option::game_option_id::GameOptionId::INDUCEMENT_PRAYERS_USE_LEAGUE_TABLE)
                .get_value_as_string()
                == "true";
            let mut factory = crate::factory::bb2020::prayer_factory::PrayerFactory::new();
            factory.initialize(use_league_table);
            Ok(factory.prayers().values().find(|p| java_equals_ignore_case(p.get_name(), name)).map(|p| p.name()))
        }
        Rules::Bb2025 => {
            let mut factory = crate::factory::bb2025::prayer_factory::PrayerFactory::new();
            factory.initialize();
            Ok(factory.prayers().values().find(|p| java_equals_ignore_case(p.get_name(), name)).map(|p| p.name()))
        }
        other => Err(format!("no PrayerFactory for {other:?} (prayer {name:?})")),
    }
}

/// Java `Prayer.getName()` of a prayer the Rust engine holds as its enum-constant name, for the
/// ruleset's `Prayer` enum (`InducementSet.toJsonValue` writes `getName()`).
pub fn display_name_in(rules: crate::enums::Rules, enum_name: &str) -> Option<&'static str> {
    use crate::enums::Rules;
    match rules {
        Rules::Bb2020 => crate::inducement::bb2020::prayer::Prayer::for_enum_name(enum_name).map(|p| p.get_name()),
        Rules::Bb2025 => crate::inducement::bb2025::prayer::Prayer::for_enum_name(enum_name).map(|p| p.get_name()),
        _ => None,
    }
}

#[cfg(test)]
mod ruleset_tests {
    use super::*;
    use crate::enums::Rules;
    use crate::model::game_options::GameOptions;

    #[test]
    fn for_name_in_matches_java_forname_per_ruleset() {
        let options = GameOptions::new();
        assert_eq!(for_name_in(Rules::Bb2025, &options, "iron man"), Ok(Some("IRON_MAN")));
        assert_eq!(for_name_in(Rules::Bb2025, &options, "Dazzling Catching"), Ok(Some("DAZZLING_CATCHING")));
        // Dazzling Catching is BB2025-only; Necessary Violence is BB2020-only.
        assert_eq!(for_name_in(Rules::Bb2020, &options, "Dazzling Catching"), Ok(None));
        assert_eq!(for_name_in(Rules::Bb2025, &options, "Necessary Violence"), Ok(None));
        // newer-server prayer: unknown (the harness drops it before decoding)
        assert_eq!(for_name_in(Rules::Bb2025, &options, "Blessing of Nuffle"), Ok(None));
        assert!(for_name_in(Rules::Bb2016, &options, "Iron Man").is_err());
        assert_eq!(display_name_in(Rules::Bb2025, "MOLES_UNDER_THE_PITCH"), Some("Moles under the Pitch"));
    }
}
