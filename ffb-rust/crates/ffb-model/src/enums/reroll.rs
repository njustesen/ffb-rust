use serde::{Deserialize, Serialize};

/// Properties a re-roll source may have.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReRollProperty {
    /// Team re-roll token.
    Trr,
    BrilliantCoaching,
    /// Mascot (counts as actual re-roll).
    Mascot,
    /// Pro skill re-roll.
    Pro,
    /// Loner — must roll to use.
    Loner,
    PumpUpTheCrowd,
    ShowStar,
}

impl ReRollProperty {
    pub fn name(self) -> &'static str {
        match self {
            ReRollProperty::Trr => "TRR",
            ReRollProperty::BrilliantCoaching => "BRILLIANT_COACHING",
            ReRollProperty::Mascot => "MASCOT",
            ReRollProperty::Pro => "PRO",
            ReRollProperty::Loner => "LONER",
            ReRollProperty::PumpUpTheCrowd => "PUMP_UP_THE_CROWD",
            ReRollProperty::ShowStar => "SHOW_STAR",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        [
            ReRollProperty::Trr, ReRollProperty::BrilliantCoaching, ReRollProperty::Mascot,
            ReRollProperty::Pro, ReRollProperty::Loner, ReRollProperty::PumpUpTheCrowd,
            ReRollProperty::ShowStar,
        ]
        .iter().copied().find(|v| v.name().eq_ignore_ascii_case(name))
    }

    /// Whether this source consumes a physical re-roll token.
    pub fn is_actual_reroll(self) -> bool {
        matches!(self, ReRollProperty::Trr | ReRollProperty::Mascot | ReRollProperty::Pro)
    }
}

/// A re-roll source: named string + priority.
/// The Java class is not an enum but a plain data class; we model it the same way.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReRollSource {
    pub name: String,
    pub priority: i32,
    /// Optional superior source (if this skill is unavailable, fall back to superior's skill).
    pub superior: Option<Box<ReRollSource>>,
}

impl ReRollSource {
    pub fn new(name: impl Into<String>) -> Self {
        ReRollSource { name: name.into(), priority: 1, superior: None }
    }

    pub fn with_priority(name: impl Into<String>, priority: i32) -> Self {
        ReRollSource { name: name.into(), priority, superior: None }
    }

    pub fn with_superior(name: impl Into<String>, superior: ReRollSource) -> Self {
        ReRollSource {
            name: name.into(),
            priority: 1,
            superior: Some(Box::new(superior)),
        }
    }
}

/// Leader re-roll state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LeaderState {
    None,
    Available,
    Used,
}

impl LeaderState {
    pub fn name(self) -> &'static str {
        match self {
            LeaderState::None => "none",
            LeaderState::Available => "available",
            LeaderState::Used => "used",
        }
    }

    pub fn from_name(name: &str) -> Option<LeaderState> {
        match name {
            "none" => Some(LeaderState::None),
            "available" => Some(LeaderState::Available),
            "used" => Some(LeaderState::Used),
            _ => None,
        }
    }
}

/// Wrapper holding available re-roll properties for a given action offer.
/// Mirrors Java `ReRollOptions` — determines if an actual team re-roll is available.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReRollOptions {
    pub properties: Vec<ReRollProperty>,
}

impl ReRollOptions {
    pub fn new(properties: Vec<ReRollProperty>) -> Self {
        ReRollOptions { properties }
    }

    /// True if any property constitutes an actual team re-roll.
    pub fn can_actually_reroll(&self) -> bool {
        self.properties.iter().any(|p| p.is_actual_reroll())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reroll_property_actual_reroll() {
        assert!(ReRollProperty::Trr.is_actual_reroll());
        assert!(!ReRollProperty::Loner.is_actual_reroll());
        assert!(!ReRollProperty::BrilliantCoaching.is_actual_reroll());
    }

    #[test]
    fn leader_state_round_trip() {
        for s in &[LeaderState::None, LeaderState::Available, LeaderState::Used] {
            assert_eq!(LeaderState::from_name(s.name()), Some(*s));
        }
    }

    #[test]
    fn serde_leader_state() {
        let s = LeaderState::Available;
        let json = serde_json::to_string(&s).unwrap();
        let back: LeaderState = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    #[test]
    fn reroll_property_count_is_seven() {
        let all = [
            ReRollProperty::Trr, ReRollProperty::BrilliantCoaching, ReRollProperty::Mascot,
            ReRollProperty::Pro, ReRollProperty::Loner, ReRollProperty::PumpUpTheCrowd,
            ReRollProperty::ShowStar,
        ];
        assert_eq!(all.len(), 7);
    }

    #[test]
    fn reroll_property_all_have_non_empty_names() {
        for p in [
            ReRollProperty::Trr, ReRollProperty::BrilliantCoaching, ReRollProperty::Mascot,
            ReRollProperty::Pro, ReRollProperty::Loner, ReRollProperty::PumpUpTheCrowd,
            ReRollProperty::ShowStar,
        ] {
            assert!(!p.name().is_empty());
        }
    }

    #[test]
    fn reroll_property_trr_name() {
        assert_eq!(ReRollProperty::Trr.name(), "TRR");
    }

    #[test]
    fn leader_state_count_is_three() {
        let all = [LeaderState::None, LeaderState::Available, LeaderState::Used];
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn leader_state_none_name() {
        assert_eq!(LeaderState::None.name(), "none");
    }

    #[test]
    fn leader_state_available_name() {
        assert_eq!(LeaderState::Available.name(), "available");
    }

    #[test]
    fn reroll_options_can_actually_reroll_when_trr() {
        let opts = ReRollOptions::new(vec![ReRollProperty::Trr]);
        assert!(opts.can_actually_reroll());
    }

    #[test]
    fn reroll_options_cannot_reroll_when_empty() {
        let opts = ReRollOptions::new(vec![]);
        assert!(!opts.can_actually_reroll());
    }

    #[test]
    fn reroll_options_loner_alone_cannot_actually_reroll() {
        let opts = ReRollOptions::new(vec![ReRollProperty::Loner]);
        assert!(!opts.can_actually_reroll());
    }
}
