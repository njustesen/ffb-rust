/// 1:1 translation of `com.fumbbl.ffb.kickoff.KickoffResult`.
///
/// Interface for a single kickoff event — provides display name, description,
/// and flags for the two special re-roll types.
pub trait KickoffResult {
    /// Java: `getName()`
    fn get_name(&self) -> &str;

    /// Java: `getDescription()`
    fn get_description(&self) -> &str;

    /// Java: `isFanReRoll()` — true only for CheeringFans (BB2016/BB2020).
    fn is_fan_reroll(&self) -> bool {
        false
    }

    /// Java: `isCoachReRoll()` — true only for BrilliantCoaching.
    fn is_coach_reroll(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeResult;
    impl KickoffResult for FakeResult {
        fn get_name(&self) -> &str { "Fake" }
        fn get_description(&self) -> &str { "A fake result." }
    }

    #[test]
    fn default_not_fan_reroll() {
        let r = FakeResult;
        assert!(!r.is_fan_reroll());
    }

    #[test]
    fn default_not_coach_reroll() {
        let r = FakeResult;
        assert!(!r.is_coach_reroll());
    }
}
