use crate::inducement::inducement_collection::{InducementCollection as BaseCollection, InducementTypeEntry};
use crate::inducement::usage::Usage;

/// 1:1 translation of `com.fumbbl.ffb.inducement.bb2020.InducementCollection`.
///
/// BB2020-specific inducement types (extends base collection).
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
            InducementTypeEntry::new("bloodweiserBabes", vec![Usage::KNOCKOUT_RECOVERY]),
            InducementTypeEntry::new("card", vec![]),
            InducementTypeEntry::new("bribes", vec![Usage::AVOID_BAN]),
            InducementTypeEntry::new("prayers", vec![Usage::GAME_MODIFICATION]),
            InducementTypeEntry::new("briberyAndCorruption", vec![Usage::REROLL_ARGUE]),
            InducementTypeEntry::new("halflingMasterChef", vec![Usage::STEAL_REROLL]),
            InducementTypeEntry::new("mortuaryAssistant", vec![Usage::REGENERATION]),
            InducementTypeEntry::new("plagueDoctor", vec![Usage::REGENERATION, Usage::APOTHECARY_JOURNEYMEN]),
            InducementTypeEntry::new("riotousRookies", vec![Usage::ADD_LINEMEN]),
            InducementTypeEntry::new("tempCheerleader", vec![Usage::ADD_CHEERLEADER]),
            InducementTypeEntry::new("partTimeCoach", vec![Usage::ADD_COACH]),
            InducementTypeEntry::new("biasedRef", vec![Usage::ADD_TO_ARGUE_ROLL, Usage::SPOT_FOUL]),
            InducementTypeEntry::new("weatherMage", vec![Usage::CHANGE_WEATHER]),
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
    fn bb2020_collection_has_prayers() {
        let c = InducementCollection::new();
        assert!(c.get_all_types().iter().any(|t| t.id == "prayers"));
    }

    #[test]
    fn has_biased_ref() {
        let c = InducementCollection::new();
        assert!(c.get_all_types().iter().any(|t| t.id == "biasedRef"));
    }

    #[test]
    fn total_types_count_is_at_least_15() {
        let c = InducementCollection::new();
        assert!(c.get_all_types().len() >= 15);
    }
}
