use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.model.SpecialRule.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpecialRule {
    BADLANDS_BRAWL,
    ELVEN_KINGDOMS_LEAGUE,
    HALFLING_THIMBLE_CUP,
    LUSTRIAN_SUPERLEAGUE,
    OLD_WORLD_CLASSIC,
    SYLVANIAN_SPOTLIGHT,
    UNDERWORLD_CHALLENGE,
    WORLDS_EDGE_SUPERLEAGUE,
    BRIBERY_AND_CORRUPTION,
    FAVOURED_OF_UNDIVIDED,
    FAVOURED_OF_KHORNE,
    FAVOURED_OF_NURGLE,
    FAVOURED_OF_TZEENTCH,
    FAVOURED_OF_SLAANESH,
    LOW_COST_LINEMEN,
    SWARMING,
    MASTERS_OF_UNDEATH,
    BRAWLIN_BRUTES,
}

impl SpecialRule {
    pub fn get_rule_name(self) -> &'static str {
        match self {
            SpecialRule::BADLANDS_BRAWL => "Badlands Brawl",
            SpecialRule::ELVEN_KINGDOMS_LEAGUE => "Elven Kingdoms League",
            SpecialRule::HALFLING_THIMBLE_CUP => "Halfling Thimble Cup",
            SpecialRule::LUSTRIAN_SUPERLEAGUE => "Lustrian Superleague",
            SpecialRule::OLD_WORLD_CLASSIC => "Old World Classic",
            SpecialRule::SYLVANIAN_SPOTLIGHT => "Sylvanian Spotlight",
            SpecialRule::UNDERWORLD_CHALLENGE => "Underworld Challenge",
            SpecialRule::WORLDS_EDGE_SUPERLEAGUE => "Worlds Edge Superleague",
            SpecialRule::BRIBERY_AND_CORRUPTION => "Bribery and Corruption",
            SpecialRule::FAVOURED_OF_UNDIVIDED => "Favoured of Chaos Undivided",
            SpecialRule::FAVOURED_OF_KHORNE => "Favoured of Khorne",
            SpecialRule::FAVOURED_OF_NURGLE => "Favoured of Nurgle",
            SpecialRule::FAVOURED_OF_TZEENTCH => "Favoured of Tzeentch",
            SpecialRule::FAVOURED_OF_SLAANESH => "Favoured of Slaanesh",
            SpecialRule::LOW_COST_LINEMEN => "Low Cost Linemen",
            SpecialRule::SWARMING => "Swarming",
            SpecialRule::MASTERS_OF_UNDEATH => "Masters of Undeath",
            SpecialRule::BRAWLIN_BRUTES => "Brawlin' Brutes",
        }
    }

    pub fn from(name: &str) -> Option<Self> {
        [
            SpecialRule::BADLANDS_BRAWL, SpecialRule::ELVEN_KINGDOMS_LEAGUE,
            SpecialRule::HALFLING_THIMBLE_CUP, SpecialRule::LUSTRIAN_SUPERLEAGUE,
            SpecialRule::OLD_WORLD_CLASSIC, SpecialRule::SYLVANIAN_SPOTLIGHT,
            SpecialRule::UNDERWORLD_CHALLENGE, SpecialRule::WORLDS_EDGE_SUPERLEAGUE,
            SpecialRule::BRIBERY_AND_CORRUPTION, SpecialRule::FAVOURED_OF_UNDIVIDED,
            SpecialRule::FAVOURED_OF_KHORNE, SpecialRule::FAVOURED_OF_NURGLE,
            SpecialRule::FAVOURED_OF_TZEENTCH, SpecialRule::FAVOURED_OF_SLAANESH,
            SpecialRule::LOW_COST_LINEMEN, SpecialRule::SWARMING,
            SpecialRule::MASTERS_OF_UNDEATH, SpecialRule::BRAWLIN_BRUTES,
        ]
        .iter().copied().find(|r| r.get_rule_name().eq_ignore_ascii_case(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swarming_rule_name() {
        assert_eq!(SpecialRule::SWARMING.get_rule_name(), "Swarming");
    }

    #[test]
    fn from_case_insensitive() {
        assert_eq!(SpecialRule::from("swarming"), Some(SpecialRule::SWARMING));
        assert_eq!(SpecialRule::from("SWARMING"), Some(SpecialRule::SWARMING));
    }

    #[test]
    fn from_unknown() {
        assert_eq!(SpecialRule::from("unknown"), None);
    }
}
