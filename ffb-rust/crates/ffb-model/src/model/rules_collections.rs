use serde::{Deserialize, Serialize};
use super::rules_collection::RulesCollection;

/// 1:1 translation of com.fumbbl.ffb.model.RulesCollections.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RulesCollections {
    pub collections: Vec<RulesCollection>,
}

impl RulesCollections {
    pub fn add(&mut self, c: RulesCollection) { self.collections.push(c); }
    pub fn len(&self) -> usize { self.collections.len() }
    pub fn is_empty(&self) -> bool { self.collections.is_empty() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_by_default() {
        assert!(RulesCollections::default().is_empty());
    }

    #[test]
    fn add_increases_len() {
        let mut rcs = RulesCollections::default();
        rcs.add(RulesCollection::default());
        assert_eq!(rcs.len(), 1);
    }
}
