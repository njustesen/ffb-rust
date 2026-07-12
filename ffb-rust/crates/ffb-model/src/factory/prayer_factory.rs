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
