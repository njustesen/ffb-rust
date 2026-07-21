use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.model.TeamSetup.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamSetup {
    pub name: String,
    pub coach: String,
    pub positions: Vec<String>,
}

impl TeamSetup {
    pub fn new(name: String, coach: String) -> Self {
        Self { name, coach, positions: Vec::new() }
    }
}
