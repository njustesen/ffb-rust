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
        let sub_types = vec![
            InducementTypeEntry::new("bloodweiserBabes", vec![Usage::KNOCKOUT_RECOVERY]),
            InducementTypeEntry::new("card", vec![]),
            InducementTypeEntry::new("bribes", vec![Usage::AVOID_BAN]),
            InducementTypeEntry::new("igor", vec![Usage::REGENERATION]),
            InducementTypeEntry::new("halflingMasterChef", vec![Usage::STEAL_REROLL]),
            InducementTypeEntry::new("riotousRookies", vec![Usage::ADD_LINEMEN]),
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
