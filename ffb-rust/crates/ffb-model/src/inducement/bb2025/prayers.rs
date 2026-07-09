use std::collections::HashMap;

use crate::inducement::bb2025::prayer::Prayer;

/// BB2025 prayer roll table — 1:1 translation of Java bb2025/Prayers.
pub struct Prayers {
    all_prayers: HashMap<i32, Prayer>,
}

impl Prayers {
    pub fn new() -> Self {
        let mut all = HashMap::new();
        all.insert(1, Prayer::TREACHEROUS_TRAPDOOR);
        all.insert(2, Prayer::FRIENDS_WITH_THE_REF);
        all.insert(3, Prayer::STILETTO);
        all.insert(4, Prayer::IRON_MAN);
        all.insert(5, Prayer::KNUCKLE_DUSTERS);
        all.insert(6, Prayer::BAD_HABITS);
        all.insert(7, Prayer::GREASY_CLEATS);
        all.insert(8, Prayer::BLESSED_STATUE_OF_NUFFLE);
        all.insert(9, Prayer::MOLES_UNDER_THE_PITCH);
        all.insert(10, Prayer::PERFECT_PASSING);
        all.insert(11, Prayer::DAZZLING_CATCHING);
        all.insert(12, Prayer::FAN_INTERACTION);
        all.insert(13, Prayer::FOULING_FRENZY);
        all.insert(14, Prayer::THROW_A_ROCK);
        all.insert(15, Prayer::UNDER_SCRUTINY);
        all.insert(16, Prayer::INTENSIVE_TRAINING);

        Self { all_prayers: all }
    }

    pub fn get_all_prayers(&self) -> &HashMap<i32, Prayer> {
        &self.all_prayers
    }

    pub fn get_prayer(&self, roll: i32) -> Option<Prayer> {
        self.all_prayers.get(&roll).copied()
    }
}

impl Default for Prayers {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_prayers_has_16_entries() {
        let prayers = Prayers::new();
        assert_eq!(prayers.get_all_prayers().len(), 16);
    }

    #[test]
    fn test_roll_8_is_blessing_of_nuffle() {
        let prayers = Prayers::new();
        assert_eq!(prayers.get_prayer(8), Some(Prayer::BLESSED_STATUE_OF_NUFFLE));
    }
}
