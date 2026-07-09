use serde::{Deserialize, Serialize};
use crate::enums::Rules;

/// 1:1 translation of com.fumbbl.ffb.model.RulesCollection.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RulesCollection {
    pub rules: Vec<Rules>,
}

impl RulesCollection {
    pub fn add(&mut self, r: Rules) { self.rules.push(r); }
    pub fn contains(&self, r: Rules) -> bool { self.rules.contains(&r) }
    pub fn len(&self) -> usize { self.rules.len() }
    pub fn is_empty(&self) -> bool { self.rules.is_empty() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::Rules;

    #[test]
    fn empty_by_default() {
        assert!(RulesCollection::default().is_empty());
    }

    #[test]
    fn add_and_contains() {
        let mut rc = RulesCollection::default();
        rc.add(Rules::Bb2025);
        assert!(rc.contains(Rules::Bb2025));
        assert_eq!(rc.len(), 1);
    }
}
