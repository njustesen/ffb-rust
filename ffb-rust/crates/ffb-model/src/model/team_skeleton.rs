use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.model.TeamSkeleton.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamSkeleton {
    pub team_id: String,
    pub team_name: String,
    pub roster_id: String,
}

impl TeamSkeleton {
    pub fn get_team_id(&self) -> &str { &self.team_id }
    pub fn get_team_name(&self) -> &str { &self.team_name }
    pub fn get_roster_id(&self) -> &str { &self.roster_id }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_empty() {
        assert!(TeamSkeleton::default().team_id.is_empty());
    }

    #[test]
    fn get_team_name_returns_name() {
        let t = TeamSkeleton { team_id: "42".to_string(), team_name: "Chaos".to_string(), roster_id: "c".to_string() };
        assert_eq!(t.get_team_name(), "Chaos");
    }
}
