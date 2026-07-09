use std::collections::HashMap;
use crate::kickoff::bb2016::kickoff_result::KickoffResult;
use crate::kickoff::kickoff_result_mapping::KickoffResultMapping as IKickoffResultMapping;
use crate::kickoff::KickoffEventKind;

/// 1:1 translation of `com.fumbbl.ffb.kickoff.bb2016.KickoffResultMapping`.
///
/// Maps 2d6 rolls to BB2016 kickoff results.
pub struct KickoffResultMapping {
    results: HashMap<i32, KickoffResult>,
}

impl Default for KickoffResultMapping {
    fn default() -> Self {
        Self::new()
    }
}

impl KickoffResultMapping {
    pub fn new() -> Self {
        let mut results = HashMap::new();
        results.insert(2, KickoffResult::GET_THE_REF);
        results.insert(3, KickoffResult::RIOT);
        results.insert(4, KickoffResult::PERFECT_DEFENCE);
        results.insert(5, KickoffResult::HIGH_KICK);
        results.insert(6, KickoffResult::CHEERING_FANS);
        results.insert(7, KickoffResult::WEATHER_CHANGE);
        results.insert(8, KickoffResult::BRILLIANT_COACHING);
        results.insert(9, KickoffResult::QUICK_SNAP);
        results.insert(10, KickoffResult::BLITZ);
        results.insert(11, KickoffResult::THROW_A_ROCK);
        results.insert(12, KickoffResult::PITCH_INVASION);
        Self { results }
    }

    /// Get the BB2016 KickoffResult for a roll.
    pub fn get_bb2016_result(&self, roll: i32) -> Option<KickoffResult> {
        self.results.get(&roll).copied()
    }
}

impl IKickoffResultMapping for KickoffResultMapping {
    fn get_key(&self) -> &str {
        "KickoffResultMapping"
    }

    fn get_result(&self, roll: i32) -> Option<KickoffEventKind> {
        crate::kickoff::kickoff_event_bb2016(roll)
    }

    fn get_values(&self) -> Vec<KickoffEventKind> {
        (2..=12).filter_map(|r| crate::kickoff::kickoff_event_bb2016(r)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roll_2_gives_get_the_ref() {
        let m = KickoffResultMapping::new();
        assert_eq!(m.get_bb2016_result(2), Some(KickoffResult::GET_THE_REF));
    }

    #[test]
    fn roll_3_gives_riot() {
        let m = KickoffResultMapping::new();
        assert_eq!(m.get_bb2016_result(3), Some(KickoffResult::RIOT));
    }

    #[test]
    fn roll_12_gives_pitch_invasion() {
        let m = KickoffResultMapping::new();
        assert_eq!(m.get_bb2016_result(12), Some(KickoffResult::PITCH_INVASION));
    }

    #[test]
    fn invalid_roll_gives_none() {
        let m = KickoffResultMapping::new();
        assert!(m.get_bb2016_result(1).is_none());
    }

    #[test]
    fn all_11_rolls_present() {
        let m = KickoffResultMapping::new();
        let count = (2..=12).filter(|&r| m.get_bb2016_result(r).is_some()).count();
        assert_eq!(count, 11);
    }
}
