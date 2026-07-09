use std::collections::HashMap;
use crate::kickoff::bb2020::kickoff_result::KickoffResult;
use crate::kickoff::kickoff_result_mapping::KickoffResultMapping as IKickoffResultMapping;
use crate::kickoff::KickoffEventKind;

/// 1:1 translation of `com.fumbbl.ffb.kickoff.bb2020.KickoffResultMapping`.
///
/// Maps 2d6 rolls to BB2020 kickoff results.
pub struct KickoffResultMapping {
    results: HashMap<i32, KickoffResult>,
}

impl Default for KickoffResultMapping {
    fn default() -> Self { Self::new() }
}

impl KickoffResultMapping {
    pub fn new() -> Self {
        let mut results = HashMap::new();
        results.insert(2, KickoffResult::GET_THE_REF);
        results.insert(3, KickoffResult::TIME_OUT);
        results.insert(4, KickoffResult::SOLID_DEFENCE);
        results.insert(5, KickoffResult::HIGH_KICK);
        results.insert(6, KickoffResult::CHEERING_FANS);
        results.insert(7, KickoffResult::BRILLIANT_COACHING);
        results.insert(8, KickoffResult::WEATHER_CHANGE);
        results.insert(9, KickoffResult::QUICK_SNAP);
        results.insert(10, KickoffResult::BLITZ);
        results.insert(11, KickoffResult::OFFICIOUS_REF);
        results.insert(12, KickoffResult::PITCH_INVASION);
        Self { results }
    }

    pub fn get_bb2020_result(&self, roll: i32) -> Option<KickoffResult> {
        self.results.get(&roll).copied()
    }
}

impl IKickoffResultMapping for KickoffResultMapping {
    fn get_key(&self) -> &str { "KickoffResultMapping" }

    fn get_result(&self, roll: i32) -> Option<KickoffEventKind> {
        crate::kickoff::kickoff_event_bb2020(roll)
    }

    fn get_values(&self) -> Vec<KickoffEventKind> {
        (2..=12).filter_map(|r| crate::kickoff::kickoff_event_bb2020(r)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roll_2_gives_get_the_ref() {
        let m = KickoffResultMapping::new();
        assert_eq!(m.get_bb2020_result(2), Some(KickoffResult::GET_THE_REF));
    }

    #[test]
    fn roll_7_gives_brilliant_coaching() {
        let m = KickoffResultMapping::new();
        assert_eq!(m.get_bb2020_result(7), Some(KickoffResult::BRILLIANT_COACHING));
    }

    #[test]
    fn roll_12_gives_pitch_invasion() {
        let m = KickoffResultMapping::new();
        assert_eq!(m.get_bb2020_result(12), Some(KickoffResult::PITCH_INVASION));
    }

    #[test]
    fn all_11_present() {
        let m = KickoffResultMapping::new();
        let count = (2..=12).filter(|&r| m.get_bb2020_result(r).is_some()).count();
        assert_eq!(count, 11);
    }
}
