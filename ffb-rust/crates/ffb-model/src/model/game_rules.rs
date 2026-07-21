use serde::{Deserialize, Serialize};
use crate::enums::Rules;

/// 1:1 translation of com.fumbbl.ffb.model.GameRules.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameRules {
    pub rules: Option<Rules>,
}

impl GameRules {
    pub fn new(rules: Rules) -> Self { Self { rules: Some(rules) } }
    pub fn get_rules(&self) -> Option<Rules> { self.rules }
}
