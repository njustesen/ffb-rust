use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.model.RosterSkeleton.
///
/// Java fields: `fId`, `fTeamId` (parsed from the `id`/`team` XML attributes of the
/// root `<roster>` element). There is no `name` field in the Java source — an earlier
/// version of this file invented one; corrected here to match Java exactly.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RosterSkeleton {
    pub id: String,
    pub team_id: String,
}

impl RosterSkeleton {
    pub fn get_id(&self) -> &str { &self.id }
    pub fn set_id(&mut self, id: impl Into<String>) { self.id = id.into(); }
    pub fn get_team_id(&self) -> &str { &self.team_id }
    pub fn set_team_id(&mut self, team_id: impl Into<String>) { self.team_id = team_id.into(); }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_empty() {
        assert!(RosterSkeleton::default().id.is_empty());
        assert!(RosterSkeleton::default().team_id.is_empty());
    }

    #[test]
    fn get_team_id_returns_team_id() {
        let r = RosterSkeleton { id: "1".to_string(), team_id: "284314".to_string() };
        assert_eq!(r.get_team_id(), "284314");
    }

    #[test]
    fn setters_update_fields() {
        let mut r = RosterSkeleton::default();
        r.set_id("undead");
        r.set_team_id("42");
        assert_eq!(r.get_id(), "undead");
        assert_eq!(r.get_team_id(), "42");
    }
}
