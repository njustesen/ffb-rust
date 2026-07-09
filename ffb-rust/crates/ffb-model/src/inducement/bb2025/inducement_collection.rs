use crate::inducement::inducement_collection::{InducementCollection as BaseCollection, InducementTypeEntry};
use crate::inducement::usage::Usage;

/// 1:1 translation of `com.fumbbl.ffb.inducement.bb2025.InducementCollection`.
///
/// BB2025-specific inducement types (extends base collection).
pub struct InducementCollection {
    base: BaseCollection,
    sub_types: Vec<InducementTypeEntry>,
}

impl Default for InducementCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl InducementCollection {
    pub fn new() -> Self {
        let sub_types = vec![
            InducementTypeEntry::new("teamMascot", vec![Usage::CONDITIONAL_REROLL, Usage::REROLL_CHEERING_FANS]),
            InducementTypeEntry::new("prayers", vec![Usage::GAME_MODIFICATION]),
            InducementTypeEntry::new("partTimeCoach", vec![Usage::ADD_COACH]),
            InducementTypeEntry::new("tempCheerleader", vec![Usage::ADD_CHEERLEADER]),
            InducementTypeEntry::new("weatherMage", vec![Usage::CHANGE_WEATHER]),
            InducementTypeEntry::new("bloodweiserBabes", vec![Usage::KNOCKOUT_RECOVERY]),
            InducementTypeEntry::new("bribes", vec![Usage::AVOID_BAN]),
            InducementTypeEntry::new("mortuaryAssistant", vec![Usage::REGENERATION]),
            InducementTypeEntry::new("briberyAndCorruption", vec![Usage::REROLL_ARGUE]),
            InducementTypeEntry::new("plagueDoctor", vec![Usage::REGENERATION, Usage::APOTHECARY_JOURNEYMEN]),
            InducementTypeEntry::new("riotousRookies", vec![Usage::ADD_LINEMEN]),
            InducementTypeEntry::new("throwARock", vec![Usage::THROW_ROCK]),
            InducementTypeEntry::new("halflingMasterChef", vec![Usage::STEAL_REROLL]),
            InducementTypeEntry::new("biasedRef", vec![Usage::ADD_TO_ARGUE_ROLL, Usage::SPOT_FOUL]),
            InducementTypeEntry::new("infamousStaff", vec![Usage::STAFF]),
            InducementTypeEntry::new("bugmansXXXXXX", vec![Usage::REROLL_ONES_ON_KOS]),
        ];
        Self { base: BaseCollection::new(), sub_types }
    }

    pub fn get_all_types(&self) -> Vec<&InducementTypeEntry> {
        self.base.get_types().iter().chain(self.sub_types.iter()).collect()
    }

    pub fn get_key(&self) -> &str {
        "InducementCollection"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bb2025_has_team_mascot() {
        let c = InducementCollection::new();
        assert!(c.get_all_types().iter().any(|t| t.id == "teamMascot"));
    }

    #[test]
    fn bb2025_has_throw_a_rock() {
        let c = InducementCollection::new();
        assert!(c.get_all_types().iter().any(|t| t.id == "throwARock"));
    }

    #[test]
    fn total_count_is_at_least_16() {
        let c = InducementCollection::new();
        assert!(c.get_all_types().len() >= 16);
    }
}
