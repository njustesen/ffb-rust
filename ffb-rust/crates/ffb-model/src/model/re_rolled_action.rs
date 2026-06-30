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
