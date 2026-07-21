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
