use std::collections::HashMap;

use crate::factory::prayer_factory::PrayerFactory as PrayerFactoryTrait;
use crate::inducement::bb2020::prayer::Prayer;
use crate::inducement::bb2020::prayers::Prayers;

/// 1:1 translation of com.fumbbl.ffb.factory.bb2020.PrayerFactory.
pub struct PrayerFactory {
    prayers: HashMap<i32, Prayer>,
}

impl PrayerFactory {
    pub fn new() -> Self {
        Self { prayers: HashMap::new() }
    }

    /// Java: initialize(Game) — reads the INDUCEMENT_PRAYERS_USE_LEAGUE_TABLE game option;
    /// the caller resolves that option and passes the resulting bool in.
    pub fn initialize(&mut self, use_league_table: bool) {
        let all = Prayers::new();
        let mut prayers = all.get_exhibition_prayers().clone();
        if use_league_table {
            prayers.extend(all.get_league_only_prayers().iter());
        }
        self.prayers = prayers;
    }

    /// Java: intensivePrayer().
    pub fn intensive_prayer(&self) -> Prayer {
        Prayer::INTENSIVE_TRAINING
    }

    /// Java: valueOf(String) — Java enum valueOf against the constant name.
    pub fn value_of(&self, enum_name: &str) -> Option<Prayer> {
        [
            Prayer::TREACHEROUS_TRAPDOOR,
            Prayer::FRIENDS_WITH_THE_REF,
            Prayer::STILETTO,
            Prayer::IRON_MAN,
            Prayer::KNUCKLE_DUSTERS,
            Prayer::BAD_HABITS,
            Prayer::GREASY_CLEATS,
            Prayer::BLESSED_STATUE_OF_NUFFLE,
            Prayer::MOLES_UNDER_THE_PITCH,
            Prayer::PERFECT_PASSING,
            Prayer::FAN_INTERACTION,
            Prayer::NECESSARY_VIOLENCE,
            Prayer::FOULING_FRENZY,
            Prayer::THROW_A_ROCK,
            Prayer::UNDER_SCRUTINY,
            Prayer::INTENSIVE_TRAINING,
        ]
        .into_iter()
        .find(|p| p.name() == enum_name)
    }
}

impl Default for PrayerFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl PrayerFactoryTrait for PrayerFactory {
    type PrayerT = Prayer;

    fn prayers(&self) -> &HashMap<i32, Prayer> {
        &self.prayers
    }

    fn prayer_name(prayer: Prayer) -> &'static str {
        prayer.get_name()
    }

    fn prayer_affects_both_teams(prayer: Prayer) -> bool {
        prayer.affects_both_teams()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::inducement_set::InducementSet;

    fn initialized(use_league_table: bool) -> PrayerFactory {
        let mut f = PrayerFactory::new();
        f.initialize(use_league_table);
        f
    }

    #[test]
    fn initialize_without_league_table_has_8_exhibition_prayers() {
        let f = initialized(false);
        assert_eq!(f.all_prayer_rolls().len(), 8);
        assert!(f.for_roll(9).is_none());
    }

    #[test]
    fn initialize_with_league_table_has_16_prayers() {
        let f = initialized(true);
        assert_eq!(f.all_prayer_rolls().len(), 16);
        assert_eq!(f.for_roll(16), Some(Prayer::INTENSIVE_TRAINING));
    }

    #[test]
    fn for_name_is_case_insensitive() {
        let f = initialized(false);
        assert_eq!(f.for_name("iron man"), Some(Prayer::IRON_MAN));
        assert_eq!(f.for_name("Iron Man"), Some(Prayer::IRON_MAN));
    }

    #[test]
    fn intensive_prayer_is_intensive_training() {
        let f = initialized(true);
        assert_eq!(f.intensive_prayer(), Prayer::INTENSIVE_TRAINING);
    }

    #[test]
    fn value_of_matches_enum_constant_name() {
        let f = initialized(false);
        assert_eq!(f.value_of("STILETTO"), Some(Prayer::STILETTO));
        assert_eq!(f.value_of("NOT_A_PRAYER"), None);
    }

    #[test]
    fn available_prayer_rolls_excludes_already_held_prayer() {
        let f = initialized(false);
        let mut team = InducementSet::new();
        team.add_prayer("Iron Man");
        let opponent = InducementSet::new();
        let available = f.available_prayer_rolls(&team, &opponent);
        assert!(!available.contains(&4)); // roll 4 == IRON_MAN
        assert!(available.contains(&1)); // roll 1 == TREACHEROUS_TRAPDOOR, still available
    }

    #[test]
    fn available_prayer_rolls_excludes_both_team_affecting_prayer_held_by_opponent() {
        let f = initialized(false);
        let team = InducementSet::new();
        let mut opponent = InducementSet::new();
        opponent.add_prayer("Treacherous Trapdoor");
        let available = f.available_prayer_rolls(&team, &opponent);
        // roll 1 == TREACHEROUS_TRAPDOOR, affects both teams, held by opponent → excluded.
        assert!(!available.contains(&1));
    }

    #[test]
    fn sort_orders_by_roll_and_filters_to_given_set() {
        let f = initialized(false);
        let unsorted = [Prayer::BLESSED_STATUE_OF_NUFFLE, Prayer::TREACHEROUS_TRAPDOOR];
        let sorted = f.sort(&unsorted);
        assert_eq!(sorted, vec![Prayer::TREACHEROUS_TRAPDOOR, Prayer::BLESSED_STATUE_OF_NUFFLE]);
    }
}
