use std::any::Any;
use serde::{Deserialize, Serialize};
use crate::model::game::Game;
use crate::xml::{IXmlReadable, XmlAttributes};
use crate::xml::util_xml::get_string_attribute;

const XML_TAG: &str = "team";
const XML_ATTRIBUTE_ID: &str = "id";
const XML_TAG_NAME: &str = "name";
const XML_TAG_TEAM_VALUE: &str = "teamValue";
const XML_TAG_COACH: &str = "coach";
/// Java: `RosterPlayer.XML_TAG` — referenced by `TeamSkeleton` to suppress a nested
/// player's own `<name>` from overwriting the team name while `parsingPlayer` is set.
const PLAYER_XML_TAG: &str = "player";

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
    /// Java: `TeamSkeleton.parsingPlayer` (transient) — true while inside a `<player>`
    /// element, so that player's own `<name>` doesn't overwrite the team name.
    #[serde(skip)]
    pub parsing_player: bool,
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

impl IXmlReadable for TeamSkeleton {
    /// Java: `TeamSkeleton.startXmlElement(Game, String, Attributes)`.
    fn start_xml_element(&mut self, _game: Option<&Game>, tag: &str, atts: &XmlAttributes) -> Option<Box<dyn IXmlReadable>> {
        if tag == XML_TAG {
            if let Some(id) = get_string_attribute(atts, XML_ATTRIBUTE_ID) {
                self.id = id;
            }
        }
        if tag == PLAYER_XML_TAG {
            self.parsing_player = true;
        }
        None
    }

    /// Java: `TeamSkeleton.endXmlElement(Game, String, String)`.
    fn end_xml_element(&mut self, _game: Option<&Game>, tag: &str, value: &str) -> bool {
        let complete = tag == XML_TAG;
        if !complete {
            if tag == XML_TAG_NAME && !self.parsing_player {
                self.name = value.to_string();
            }
            if tag == XML_TAG_COACH {
                self.coach = value.to_string();
            }
            if tag == XML_TAG_TEAM_VALUE {
                self.team_value = value.parse().unwrap_or(0);
            }
            if tag == PLAYER_XML_TAG {
                self.parsing_player = false;
            }
        }
        complete
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
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
        let t = TeamSkeleton { id: "42".to_string(), name: "Chaos".to_string(), coach: "Kalimar".to_string(), team_value: 1_000_000, xml_content: String::new(), parsing_player: false };
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

    #[test]
    fn parses_id_name_coach_from_xml() {
        let xml = r#"<team id="42"><coach>Kalimar</coach><name>Chaos</name></team>"#;
        let parsed = crate::xml::XmlHandler::parse(None, xml, Box::new(TeamSkeleton::default()));
        let t = parsed.as_any().downcast_ref::<TeamSkeleton>().unwrap();
        assert_eq!(t.get_id(), "42");
        assert_eq!(t.get_coach(), "Kalimar");
        assert_eq!(t.get_name(), "Chaos");
    }

    #[test]
    fn parses_team_value_tag() {
        let xml = r#"<team id="1"><teamValue>1100000</teamValue></team>"#;
        let parsed = crate::xml::XmlHandler::parse(None, xml, Box::new(TeamSkeleton::default()));
        let t = parsed.as_any().downcast_ref::<TeamSkeleton>().unwrap();
        assert_eq!(t.get_team_value(), 1_100_000);
    }

    #[test]
    fn nested_player_name_does_not_overwrite_team_name() {
        let xml = r#"<team id="1"><name>Chaos</name><player id="p1"><name>Bob</name></player></team>"#;
        let parsed = crate::xml::XmlHandler::parse(None, xml, Box::new(TeamSkeleton::default()));
        let t = parsed.as_any().downcast_ref::<TeamSkeleton>().unwrap();
        assert_eq!(t.get_name(), "Chaos");
        assert!(!t.parsing_player);
    }
}
