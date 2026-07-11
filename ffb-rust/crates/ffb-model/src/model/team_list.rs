use serde::{Deserialize, Serialize};
use super::team_list_entry::TeamListEntry;

/// 1:1 translation of com.fumbbl.ffb.model.TeamList.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamList {
    /// Java: `fCoach`.
    pub coach: Option<String>,
    pub entries: Vec<TeamListEntry>,
}

impl TeamList {
    pub fn new() -> Self { Self::default() }
    pub fn add(&mut self, entry: TeamListEntry) { self.entries.push(entry); }
    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }

    /// Java: `TeamList.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let entries: Vec<serde_json::Value> = self.entries.iter().map(TeamListEntry::to_json_value).collect();
        serde_json::json!({
            "coach": self.coach,
            "teamListEntries": entries,
        })
    }

    /// Java: `TeamList.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let coach = json.get("coach").and_then(|v| v.as_str()).map(str::to_string);
        let entries = json
            .get("teamListEntries")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().map(TeamListEntry::from_json).collect())
            .unwrap_or_default();
        Self { coach, entries }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_by_default() {
        assert!(TeamList::new().is_empty());
    }

    #[test]
    fn add_increases_len() {
        let mut tl = TeamList::new();
        tl.add(TeamListEntry::default());
        assert_eq!(tl.len(), 1);
    }
}
