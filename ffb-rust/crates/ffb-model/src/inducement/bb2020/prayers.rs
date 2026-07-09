use std::collections::HashMap;

use crate::inducement::bb2020::prayer::Prayer;

/// BB2020 prayer roll table — 1:1 translation of Java bb2020/Prayers.
pub struct Prayers {
    exhibition_prayers: HashMap<i32, Prayer>,
    league_only_prayers: HashMap<i32, Prayer>,
}

impl Prayers {
    pub fn new() -> Self {
        let mut exhibition = HashMap::new();
        exhibition.insert(1, Prayer::TREACHEROUS_TRAPDOOR);
        exhibition.insert(2, Prayer::FRIENDS_WITH_THE_REF);
        exhibition.insert(3, Prayer::STILETTO);
        exhibition.insert(4, Prayer::IRON_MAN);
        exhibition.insert(5, Prayer::KNUCKLE_DUSTERS);
        exhibition.insert(6, Prayer::BAD_HABITS);
        exhibition.insert(7, Prayer::GREASY_CLEATS);
        exhibition.insert(8, Prayer::BLESSED_STATUE_OF_NUFFLE);

        let mut league_only = HashMap::new();
        league_only.insert(9, Prayer::MOLES_UNDER_THE_PITCH);
        league_only.insert(10, Prayer::PERFECT_PASSING);
        league_only.insert(11, Prayer::FAN_INTERACTION);
        league_only.insert(12, Prayer::NECESSARY_VIOLENCE);
        league_only.insert(13, Prayer::FOULING_FRENZY);
        league_only.insert(14, Prayer::THROW_A_ROCK);
        league_only.insert(15, Prayer::UNDER_SCRUTINY);
        league_only.insert(16, Prayer::INTENSIVE_TRAINING);

        Self {
            exhibition_prayers: exhibition,
            league_only_prayers: league_only,
        }
    }

    pub fn get_exhibition_prayers(&self) -> &HashMap<i32, Prayer> {
        &self.exhibition_prayers
    }

    pub fn get_league_only_prayers(&self) -> &HashMap<i32, Prayer> {
        &self.league_only_prayers
    }

    pub fn get_prayer(&self, roll: i32) -> Option<Prayer> {
        self.exhibition_prayers.get(&roll)
            .or_else(|| self.league_only_prayers.get(&roll))
            .copied()
    }
}

impl Default for Prayers {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exhibition_has_8_entries() {
        let prayers = Prayers::new();
        assert_eq!(prayers.get_exhibition_prayers().len(), 8);
    }

    #[test]
    fn test_roll_1_is_treacherous_trapdoor() {
        let prayers = Prayers::new();
        assert_eq!(prayers.get_prayer(1), Some(Prayer::TREACHEROUS_TRAPDOOR));
    }
}
