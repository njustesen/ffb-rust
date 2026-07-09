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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::Rules;

    #[test]
    fn default_has_no_rules() {
        assert!(GameRules::default().rules.is_none());
    }

    #[test]
    fn new_sets_rules() {
        let gr = GameRules::new(Rules::Bb2020);
        assert_eq!(gr.get_rules(), Some(Rules::Bb2020));
    }
}
