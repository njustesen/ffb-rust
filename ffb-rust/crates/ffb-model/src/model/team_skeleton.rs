use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.model.TeamSkeleton.
///
/// Java `TeamSkeleton extends Team` purely to reuse `Team`'s `IFactorySource`-based
/// constructor; the skeleton itself only tracks `fId`/`fName`/`fTeamValue`/`fCoach`/
/// `xmlContent`, parsed from the `id` XML attribute and the `name`/`teamValue`/`coach`
/// child elements of the team XML. An earlier version of this file had a mismatched
/// shape (`team_id`/`team_name`/`roster_id`, none of which exist on the Java class);
/// corrected here to match Java exactly.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamSkeleton {
    pub id: String,
    pub name: String,
    pub team_value: i32,
    pub coach: String,
    pub xml_content: String,
}

impl TeamSkeleton {
    pub fn get_id(&self) -> &str { &self.id }
    pub fn set_id(&mut self, id: impl Into<String>) { self.id = id.into(); }
    pub fn get_name(&self) -> &str { &self.name }
    pub fn set_name(&mut self, name: impl Into<String>) { self.name = name.into(); }
    pub fn get_team_value(&self) -> i32 { self.team_value }
    pub fn set_team_value(&mut self, team_value: i32) { self.team_value = team_value; }
    pub fn get_coach(&self) -> &str { &self.coach }
    pub fn set_coach(&mut self, coach: impl Into<String>) { self.coach = coach.into(); }
    pub fn get_xml_content(&self) -> &str { &self.xml_content }
    pub fn set_xml_content(&mut self, xml_content: impl Into<String>) { self.xml_content = xml_content.into(); }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_empty() {
        assert!(TeamSkeleton::default().id.is_empty());
    }

    #[test]
    fn get_name_returns_name() {
        let t = TeamSkeleton { id: "42".to_string(), name: "Chaos".to_string(), coach: "Kalimar".to_string(), team_value: 1_000_000, xml_content: String::new() };
        assert_eq!(t.get_name(), "Chaos");
        assert_eq!(t.get_coach(), "Kalimar");
        assert_eq!(t.get_team_value(), 1_000_000);
    }

    #[test]
    fn setters_update_fields() {
        let mut t = TeamSkeleton::default();
        t.set_id("1");
        t.set_name("Amazon");
        t.set_coach("Coach");
        t.set_team_value(1_100_000);
        t.set_xml_content("<team/>");
        assert_eq!(t.get_id(), "1");
        assert_eq!(t.get_name(), "Amazon");
        assert_eq!(t.get_coach(), "Coach");
        assert_eq!(t.get_team_value(), 1_100_000);
        assert_eq!(t.get_xml_content(), "<team/>");
    }
}
