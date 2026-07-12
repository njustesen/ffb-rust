use std::collections::HashMap;

use crate::factory::prayer_factory::PrayerFactory as PrayerFactoryTrait;
use crate::inducement::bb2025::prayer::Prayer;
use crate::inducement::bb2025::prayers::Prayers;

/// 1:1 translation of com.fumbbl.ffb.factory.bb2025.PrayerFactory.
pub struct PrayerFactory {
    prayers: HashMap<i32, Prayer>,
}

impl PrayerFactory {
    pub fn new() -> Self {
        Self { prayers: HashMap::new() }
    }

    /// Java: initialize(Game) — BB2025 always uses the full 16-prayer table (no
    /// league-table game option branch, unlike BB2020).
    pub fn initialize(&mut self) {
        self.prayers = Prayers::new().get_all_prayers().clone();
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
            Prayer::DAZZLING_CATCHING,
            Prayer::FAN_INTERACTION,
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

    fn initialized() -> PrayerFactory {
        let mut f = PrayerFactory::new();
        f.initialize();
        f
    }

    #[test]
    fn initialize_has_16_prayers() {
        let f = initialized();
        assert_eq!(f.all_prayer_rolls().len(), 16);
        assert_eq!(f.for_roll(11), Some(Prayer::DAZZLING_CATCHING));
    }

    #[test]
    fn for_name_is_case_insensitive() {
        let f = initialized();
        assert_eq!(f.for_name("dazzling catching"), Some(Prayer::DAZZLING_CATCHING));
    }

    #[test]
    fn intensive_prayer_is_intensive_training() {
        let f = initialized();
        assert_eq!(f.intensive_prayer(), Prayer::INTENSIVE_TRAINING);
    }

    #[test]
    fn value_of_matches_enum_constant_name() {
        let f = initialized();
        assert_eq!(f.value_of("DAZZLING_CATCHING"), Some(Prayer::DAZZLING_CATCHING));
        assert_eq!(f.value_of("NOT_A_PRAYER"), None);
    }

    #[test]
    fn available_prayer_rolls_excludes_already_held_prayer() {
        let f = initialized();
        let mut team = InducementSet::new();
        team.add_prayer("Iron Man");
        let opponent = InducementSet::new();
        let available = f.available_prayer_rolls(&team, &opponent);
        assert!(!available.contains(&4));
        assert!(available.contains(&1));
    }

    #[test]
    fn available_prayer_rolls_excludes_both_team_affecting_prayer_held_by_opponent() {
        let f = initialized();
        let team = InducementSet::new();
        let mut opponent = InducementSet::new();
        opponent.add_prayer("Treacherous Trapdoor");
        let available = f.available_prayer_rolls(&team, &opponent);
        assert!(!available.contains(&1));
    }

    #[test]
    fn sort_orders_by_roll_and_filters_to_given_set() {
        let f = initialized();
        let unsorted = [Prayer::INTENSIVE_TRAINING, Prayer::TREACHEROUS_TRAPDOOR];
        let sorted = f.sort(&unsorted);
        assert_eq!(sorted, vec![Prayer::TREACHEROUS_TRAPDOOR, Prayer::INTENSIVE_TRAINING]);
    }
}
