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
    /// Java: `Usage.valueOf(String)` — parses a SCREAMING_SNAKE_CASE name (as used in the
    /// inducement-catalog JSON `usage` field) back into a `Usage` variant.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "ADD_LINEMEN" => Some(Usage::ADD_LINEMEN),
            "ADD_CHEERLEADER" => Some(Usage::ADD_CHEERLEADER),
            "ADD_COACH" => Some(Usage::ADD_COACH),
            "ADD_TO_ARGUE_ROLL" => Some(Usage::ADD_TO_ARGUE_ROLL),
            "APOTHECARY" => Some(Usage::APOTHECARY),
            "APOTHECARY_JOURNEYMEN" => Some(Usage::APOTHECARY_JOURNEYMEN),
            "AVOID_BAN" => Some(Usage::AVOID_BAN),
            "CHANGE_WEATHER" => Some(Usage::CHANGE_WEATHER),
            "CONDITIONAL_REROLL" => Some(Usage::CONDITIONAL_REROLL),
            "GAME_MODIFICATION" => Some(Usage::GAME_MODIFICATION),
            "KNOCKOUT_RECOVERY" => Some(Usage::KNOCKOUT_RECOVERY),
            "LONER" => Some(Usage::LONER),
            "REGENERATION" => Some(Usage::REGENERATION),
            "REROLL" => Some(Usage::REROLL),
            "REROLL_ARGUE" => Some(Usage::REROLL_ARGUE),
            "REROLL_CHEERING_FANS" => Some(Usage::REROLL_CHEERING_FANS),
            "REROLL_ONES_ON_KOS" => Some(Usage::REROLL_ONES_ON_KOS),
            "SPELL" => Some(Usage::SPELL),
            "SPOT_FOUL" => Some(Usage::SPOT_FOUL),
            "STAFF" => Some(Usage::STAFF),
            "STAR" => Some(Usage::STAR),
            "STEAL_REROLL" => Some(Usage::STEAL_REROLL),
            "THROW_ROCK" => Some(Usage::THROW_ROCK),
            "UNSPECIFIC" => Some(Usage::UNSPECIFIC),
            _ => None,
        }
    }

    /// Parses a comma-separated `usage` field from the inducement-catalog JSON (e.g.
    /// `"REGENERATION,APOTHECARY_JOURNEYMEN"`) into the list of matching `Usage` variants.
    /// Unknown segments are skipped.
    pub fn parse_list(usage_field: &str) -> Vec<Usage> {
        usage_field.split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .filter_map(Usage::from_name)
            .collect()
    }

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

    #[test]
    fn exclude_from_result_contains_loner() {
        assert!(Usage::exclude_from_result().contains(&Usage::LONER));
    }

    #[test]
    fn from_name_round_trips_single() {
        assert_eq!(Usage::from_name("KNOCKOUT_RECOVERY"), Some(Usage::KNOCKOUT_RECOVERY));
        assert_eq!(Usage::from_name("unknown"), None);
    }

    #[test]
    fn parse_list_handles_comma_separated() {
        assert_eq!(
            Usage::parse_list("REGENERATION,APOTHECARY_JOURNEYMEN"),
            vec![Usage::REGENERATION, Usage::APOTHECARY_JOURNEYMEN]
        );
    }

    #[test]
    fn parse_list_skips_unknown_segments() {
        assert_eq!(Usage::parse_list("REROLL,bogus"), vec![Usage::REROLL]);
    }

    #[test]
    fn variants_are_distinct() {
        assert_ne!(Usage::REROLL, Usage::CONDITIONAL_REROLL);
        assert_ne!(Usage::STAR, Usage::STAFF);
    }
}
