use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.ReRolledAction.
/// Java uses a Class<? extends Skill> reference; Rust stores the skill name string.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReRolledAction {
    pub name: String,
}

impl ReRolledAction {
    pub fn new(name: impl Into<String>) -> Self {
        ReRolledAction { name: name.into() }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
}

impl Default for ReRolledAction {
    fn default() -> Self { ReRolledAction { name: String::new() } }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_stores_name() {
        let a = ReRolledAction::new("goForIt");
        assert_eq!(a.get_name(), "goForIt");
    }
    #[test]
    fn default_has_empty_name() {
        assert_eq!(ReRolledAction::default().get_name(), "");
    }
    #[test]
    fn equality_based_on_name() {
        assert_eq!(ReRolledAction::new("a"), ReRolledAction::new("a"));
        assert_ne!(ReRolledAction::new("a"), ReRolledAction::new("b"));
    }

    #[test]
    fn new_accepts_string_and_str() {
        // Verify Into<String> works for both owned String and &str
        let from_str = ReRolledAction::new("Dodge");
        let from_string = ReRolledAction::new(String::from("Dodge"));
        assert_eq!(from_str, from_string);
        assert_eq!(from_str.get_name(), "Dodge");
    }

    #[test]
    fn clone_is_independent() {
        let original = ReRolledAction::new("Block");
        let mut cloned = original.clone();
        cloned.name = "Dodge".into();
        // Original must not be affected by mutation of the clone
        assert_eq!(original.get_name(), "Block");
        assert_eq!(cloned.get_name(), "Dodge");
    }
}
