use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ApothecaryMode {
    Attacker,
    Away,
    CrowdPush,
    Defender,
    Feeding,
    Home,
    SpecialEffect,
    ThrownPlayer,
    KickedPlayer,
    HitPlayer,
    Catcher,
    TrapDoor,
    AnimalSavagery,
    DroppedByOwnPlayer,
    QuickBite,
}

impl ApothecaryMode {
    pub fn from_name(name: &str) -> Option<Self> {
        [
            ApothecaryMode::Attacker, ApothecaryMode::Away, ApothecaryMode::CrowdPush,
            ApothecaryMode::Defender, ApothecaryMode::Feeding, ApothecaryMode::Home,
            ApothecaryMode::SpecialEffect, ApothecaryMode::ThrownPlayer, ApothecaryMode::KickedPlayer,
            ApothecaryMode::HitPlayer, ApothecaryMode::Catcher, ApothecaryMode::TrapDoor,
            ApothecaryMode::AnimalSavagery, ApothecaryMode::DroppedByOwnPlayer, ApothecaryMode::QuickBite,
        ]
        .iter().copied().find(|v| v.name().eq_ignore_ascii_case(name))
    }

    pub fn name(self) -> &'static str {
        match self {
            ApothecaryMode::Attacker => "attacker",
            ApothecaryMode::Away => "away",
            ApothecaryMode::CrowdPush => "crowdPush",
            ApothecaryMode::Defender => "defender",
            ApothecaryMode::Feeding => "feeding",
            ApothecaryMode::Home => "home",
            ApothecaryMode::SpecialEffect => "specialEffect",
            ApothecaryMode::ThrownPlayer => "thrownPlayer",
            ApothecaryMode::KickedPlayer => "kickedPlayer",
            ApothecaryMode::HitPlayer => "hitPlayer",
            ApothecaryMode::Catcher => "catcher",
            ApothecaryMode::TrapDoor => "trapDoor",
            ApothecaryMode::AnimalSavagery => "animalSavagery",
            ApothecaryMode::DroppedByOwnPlayer => "droppedByOwnPlayer",
            ApothecaryMode::QuickBite => "quickBite",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ApothecaryStatus {
    NoApothecary,
    DoRequest,
    WaitForApothecaryUse,
    WaitForApothecaryChoice,
    UseApothecary,
    DoNotUseApothecary,
    ResultChoice,
    WaitForIgorUse,
    UseIgor,
    DoNotUseIgor,
    WaitForGettingEven,
}

impl ApothecaryStatus {
    pub fn from_name(name: &str) -> Option<Self> {
        [
            ApothecaryStatus::NoApothecary, ApothecaryStatus::DoRequest,
            ApothecaryStatus::WaitForApothecaryUse, ApothecaryStatus::WaitForApothecaryChoice,
            ApothecaryStatus::UseApothecary, ApothecaryStatus::DoNotUseApothecary,
            ApothecaryStatus::ResultChoice, ApothecaryStatus::WaitForIgorUse,
            ApothecaryStatus::UseIgor, ApothecaryStatus::DoNotUseIgor,
            ApothecaryStatus::WaitForGettingEven,
        ]
        .iter().copied().find(|v| v.name().eq_ignore_ascii_case(name))
    }

    pub fn name(self) -> &'static str {
        match self {
            ApothecaryStatus::NoApothecary => "noApothecary",
            ApothecaryStatus::DoRequest => "doRequest",
            ApothecaryStatus::WaitForApothecaryUse => "waitForApothecaryUse",
            ApothecaryStatus::WaitForApothecaryChoice => "waitForApothecaryChoice",
            ApothecaryStatus::UseApothecary => "useApothecary",
            ApothecaryStatus::DoNotUseApothecary => "doNotUseApothecary",
            ApothecaryStatus::ResultChoice => "resultChoice",
            ApothecaryStatus::WaitForIgorUse => "waitForIgorUse",
            ApothecaryStatus::UseIgor => "useIgor",
            ApothecaryStatus::DoNotUseIgor => "doNotUseIgor",
            ApothecaryStatus::WaitForGettingEven => "waitForGettingEven",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ApothecaryType {
    Team,
    Wandering,
    Plague,
}

impl ApothecaryType {
    pub fn display_name(self) -> &'static str {
        match self {
            ApothecaryType::Team => "Team Apothecary",
            ApothecaryType::Wandering => "Wandering Apothecary",
            ApothecaryType::Plague => "Plague Doctor",
        }
    }

    /// Java: `ApothecaryType.name()` (enum constant name).
    pub fn name(self) -> &'static str {
        match self {
            ApothecaryType::Team => "TEAM",
            ApothecaryType::Wandering => "WANDERING",
            ApothecaryType::Plague => "PLAGUE",
        }
    }

    /// Java: `ApothecaryType.valueOf(name)`.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "TEAM" => Some(ApothecaryType::Team),
            "WANDERING" => Some(ApothecaryType::Wandering),
            "PLAGUE" => Some(ApothecaryType::Plague),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_apothecary_mode() {
        let m = ApothecaryMode::CrowdPush;
        let json = serde_json::to_string(&m).unwrap();
        let back: ApothecaryMode = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }

    #[test]
    fn serde_apothecary_status() {
        let s = ApothecaryStatus::UseApothecary;
        let json = serde_json::to_string(&s).unwrap();
        let back: ApothecaryStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    #[test]
    fn apothecary_mode_home_name() {
        assert_eq!(ApothecaryMode::Home.name(), "home");
    }

    #[test]
    fn apothecary_mode_away_name() {
        assert_eq!(ApothecaryMode::Away.name(), "away");
    }

    #[test]
    fn apothecary_mode_attacker_name() {
        assert_eq!(ApothecaryMode::Attacker.name(), "attacker");
    }

    #[test]
    fn apothecary_mode_defender_name() {
        assert_eq!(ApothecaryMode::Defender.name(), "defender");
    }

    #[test]
    fn apothecary_mode_all_have_non_empty_names() {
        for m in [
            ApothecaryMode::Attacker, ApothecaryMode::Away, ApothecaryMode::CrowdPush,
            ApothecaryMode::Defender, ApothecaryMode::Feeding, ApothecaryMode::Home,
            ApothecaryMode::SpecialEffect, ApothecaryMode::ThrownPlayer, ApothecaryMode::KickedPlayer,
            ApothecaryMode::HitPlayer, ApothecaryMode::Catcher, ApothecaryMode::TrapDoor,
            ApothecaryMode::AnimalSavagery, ApothecaryMode::DroppedByOwnPlayer, ApothecaryMode::QuickBite,
        ] {
            assert!(!m.name().is_empty());
        }
    }

    #[test]
    fn apothecary_status_use_apothecary_name() {
        assert_eq!(ApothecaryStatus::UseApothecary.name(), "useApothecary");
    }

    #[test]
    fn apothecary_status_no_apothecary_name() {
        assert_eq!(ApothecaryStatus::NoApothecary.name(), "noApothecary");
    }

    #[test]
    fn apothecary_status_all_have_non_empty_names() {
        for s in [
            ApothecaryStatus::NoApothecary, ApothecaryStatus::DoRequest,
            ApothecaryStatus::WaitForApothecaryUse, ApothecaryStatus::WaitForApothecaryChoice,
            ApothecaryStatus::UseApothecary, ApothecaryStatus::DoNotUseApothecary,
            ApothecaryStatus::ResultChoice, ApothecaryStatus::WaitForIgorUse,
            ApothecaryStatus::UseIgor, ApothecaryStatus::DoNotUseIgor,
            ApothecaryStatus::WaitForGettingEven,
        ] {
            assert!(!s.name().is_empty());
        }
    }

    #[test]
    fn apothecary_type_team_display_name() {
        assert_eq!(ApothecaryType::Team.display_name(), "Team Apothecary");
    }

    #[test]
    fn apothecary_type_plague_display_name() {
        assert_eq!(ApothecaryType::Plague.display_name(), "Plague Doctor");
    }

    #[test]
    fn apothecary_type_wandering_display_name() {
        assert_eq!(ApothecaryType::Wandering.display_name(), "Wandering Apothecary");
    }

    #[test]
    fn apothecary_type_count_is_three() {
        let all = [ApothecaryType::Team, ApothecaryType::Wandering, ApothecaryType::Plague];
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn apothecary_type_all_have_non_empty_display_names() {
        for t in [ApothecaryType::Team, ApothecaryType::Wandering, ApothecaryType::Plague] {
            assert!(!t.display_name().is_empty());
        }
    }
}
