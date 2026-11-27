use crate::inducement::inducement_collection::{InducementCollection as BaseCollection, InducementTypeEntry};
use crate::inducement::usage::Usage;

/// 1:1 translation of `com.fumbbl.ffb.inducement.bb2016.InducementCollection`.
///
/// BB2016-specific inducement types (extends base collection).
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
        use crate::inducement::inducement_collection::InducementTypeRule as Rule;
        use crate::option::game_option_id::*;
        // `with_ids(max, cost, reducedCost, reducedMax)` as each Java constructor call fills them.
        let sub_types = vec![
            InducementTypeEntry::new("bloodweiserBabes", vec![Usage::KNOCKOUT_RECOVERY])
                .with_ids(Some(INDUCEMENT_KEGS_MAX), Some(INDUCEMENT_KEGS_COST),
                    Some(INDUCEMENT_KEGS_COST), Some(INDUCEMENT_KEGS_MAX)),
            InducementTypeEntry::new("card", vec![]),
            InducementTypeEntry::new("bribes", vec![Usage::AVOID_BAN])
                .with_ids(Some(INDUCEMENT_BRIBES_MAX), Some(INDUCEMENT_BRIBES_COST),
                    Some(INDUCEMENT_BRIBES_REDUCED_COST), Some(INDUCEMENT_BRIBES_MAX))
                .with_rule(Rule::ReducedForRosterNames(&["Goblin", "Snotling"])),
            InducementTypeEntry::new("igor", vec![Usage::REGENERATION])
                .with_ids(Some(INDUCEMENT_IGORS_MAX), Some(INDUCEMENT_IGORS_COST),
                    Some(INDUCEMENT_IGORS_COST), Some(INDUCEMENT_IGORS_MAX))
                .with_rule(Rule::RequiresNoApothecary),
            InducementTypeEntry::new("halflingMasterChef", vec![Usage::STEAL_REROLL])
                .with_ids(Some(INDUCEMENT_CHEFS_MAX), Some(INDUCEMENT_CHEFS_COST),
                    Some(INDUCEMENT_CHEFS_REDUCED_COST), Some(INDUCEMENT_CHEFS_MAX))
                .with_rule(Rule::ReducedForRosterNames(&["Halfling"])),
            InducementTypeEntry::new("riotousRookies", vec![Usage::ADD_LINEMEN])
                .with_ids(Some(INDUCEMENT_RIOTOUS_ROOKIES_MAX), Some(INDUCEMENT_RIOTOUS_ROOKIES_COST),
                    Some(INDUCEMENT_RIOTOUS_ROOKIES_COST), Some(INDUCEMENT_RIOTOUS_ROOKIES_MAX))
                .with_rule(Rule::RequiresRiotousPosition),
        ];
        Self { base: BaseCollection::new(), sub_types }
    }

    /// Java: `getTypes()` — base types + sub-types.
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
    fn bb2016_collection_has_more_than_base() {
        let c = InducementCollection::new();
        assert!(c.get_all_types().len() > 5);
    }

    #[test]
    fn has_bribes_type() {
        let c = InducementCollection::new();
        assert!(c.get_all_types().iter().any(|t| t.id == "bribes"));
    }

    #[test]
    fn has_igor_type() {
        let c = InducementCollection::new();
        assert!(c.get_all_types().iter().any(|t| t.id == "igor"));
    }
}
