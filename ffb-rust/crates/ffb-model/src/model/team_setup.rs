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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_empty() {
        let ts = TeamSetup::default();
        assert!(ts.name.is_empty());
        assert!(ts.positions.is_empty());
    }

    #[test]
    fn new_sets_name_and_coach() {
        let ts = TeamSetup::new("MyTeam".to_string(), "Coach".to_string());
        assert_eq!(ts.name, "MyTeam");
        assert_eq!(ts.coach, "Coach");
    }
}
