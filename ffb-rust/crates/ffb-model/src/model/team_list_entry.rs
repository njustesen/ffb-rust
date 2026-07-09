use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.model.TeamListEntry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamListEntry {
    pub team_id: String,
    pub team_name: String,
    pub coach: String,
    pub race: String,
}

impl TeamListEntry {
    pub fn get_team_id(&self) -> &str { &self.team_id }
    pub fn get_team_name(&self) -> &str { &self.team_name }
    pub fn get_coach(&self) -> &str { &self.coach }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_empty() {
        assert!(TeamListEntry::default().team_id.is_empty());
    }

    #[test]
    fn get_team_name_returns_name() {
        let e = TeamListEntry { team_id: "1".to_string(), team_name: "Orcs".to_string(), coach: "Bob".to_string(), race: "Orc".to_string() };
        assert_eq!(e.get_team_name(), "Orcs");
    }
}
