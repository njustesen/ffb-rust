use serde::{Deserialize, Serialize};

/// 1:1 translation of `com.fumbbl.ffb.inducement.Usage`.
/// Categorises what an inducement is used for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum Usage {
    ADD_LINEMEN,
    ADD_CHEERLEADER,
    ADD_COACH,
    ADD_TO_ARGUE_ROLL,
    APOTHECARY,
    APOTHECARY_JOURNEYMEN,
    AVOID_BAN,
    CHANGE_WEATHER,
    CONDITIONAL_REROLL,
    GAME_MODIFICATION,
    KNOCKOUT_RECOVERY,
    LONER,
    REGENERATION,
    REROLL,
    REROLL_ARGUE,
    REROLL_CHEERING_FANS,
    REROLL_ONES_ON_KOS,
    SPELL,
    SPOT_FOUL,
    STAFF,
    STAR,
    STEAL_REROLL,
    THROW_ROCK,
    UNSPECIFIC,
}

impl Usage {
    /// Java: `Usage.EXCLUDE_FROM_RESULT` — usages excluded from match result reporting.
    pub fn exclude_from_result() -> &'static [Usage] {
        &[Usage::LONER, Usage::STAR, Usage::STAFF, Usage::REROLL_ARGUE, Usage::REROLL_ONES_ON_KOS]
    }

    /// Java: `Usage.EXCLUDE_FROM_COUNT` — usages excluded from total inducement count.
    pub fn exclude_from_count() -> &'static [Usage] {
        &[Usage::REROLL_ARGUE, Usage::REROLL_ONES_ON_KOS, Usage::THROW_ROCK]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exclude_from_result_contains_star() {
        assert!(Usage::exclude_from_result().contains(&Usage::STAR));
    }

    #[test]
    fn exclude_from_count_does_not_contain_reroll() {
        assert!(!Usage::exclude_from_count().contains(&Usage::REROLL));
    }

    #[test]
    fn exclude_from_count_contains_throw_rock() {
        assert!(Usage::exclude_from_count().contains(&Usage::THROW_ROCK));
    }

    #[test]
    fn serde_round_trip() {
        let u = Usage::ADD_CHEERLEADER;
        let s = serde_json::to_string(&u).unwrap();
        let back: Usage = serde_json::from_str(&s).unwrap();
        assert_eq!(u, back);
    }
}
