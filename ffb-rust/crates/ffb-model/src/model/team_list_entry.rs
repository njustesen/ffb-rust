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

    /// Java: `TeamListEntry.toJsonValue()`.
    /// Note: only `teamId`/`teamName`/`race` are currently modeled on this
    /// struct; Java's `teamStatus`/`division`/`teamValue`/`treasury` fields
    /// are not yet translated onto `TeamListEntry` (pre-existing gap, out of
    /// scope for this pass).
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "teamId": self.team_id,
            "teamName": self.team_name,
            "race": self.race,
        })
    }

    /// Java: `TeamListEntry.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            team_id: json.get("teamId").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            team_name: json.get("teamName").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            coach: String::new(),
            race: json.get("race").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        }
    }
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
