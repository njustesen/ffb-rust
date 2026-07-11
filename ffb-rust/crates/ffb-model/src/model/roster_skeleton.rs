use std::any::Any;
use serde::{Deserialize, Serialize};
use crate::model::game::Game;
use crate::xml::{IXmlReadable, XmlAttributes};

/// XML tag/attribute names, Java: RosterSkeleton.XML_TAG / _XML_ATTRIBUTE_ID / _XML_ATTRIBUTE_TEAM.
const XML_TAG: &str = "roster";
const XML_ATTRIBUTE_ID: &str = "id";
const XML_ATTRIBUTE_TEAM: &str = "team";

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

impl IXmlReadable for RosterSkeleton {
    /// Java: `RosterSkeleton.startXmlElement(Game, String, Attributes)`.
    fn start_xml_element(&mut self, _game: Option<&Game>, tag: &str, atts: &XmlAttributes) -> Option<Box<dyn IXmlReadable>> {
        if tag == XML_TAG {
            if let Some(id) = crate::xml::util_xml::get_string_attribute(atts, XML_ATTRIBUTE_ID) {
                if !id.is_empty() {
                    self.id = id;
                }
            }
            if let Some(team_id) = crate::xml::util_xml::get_string_attribute(atts, XML_ATTRIBUTE_TEAM) {
                if !team_id.is_empty() {
                    self.team_id = team_id;
                }
            }
        }
        None
    }

    /// Java: `RosterSkeleton.endXmlElement(Game, String, String)`.
    fn end_xml_element(&mut self, _game: Option<&Game>, tag: &str, _value: &str) -> bool {
        tag == XML_TAG
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

    #[test]
    fn parses_id_and_team_attributes_from_root_tag() {
        let xml = r#"<roster id="undead" team="42"/>"#;
        let parsed = crate::xml::XmlHandler::parse(None, xml, Box::new(RosterSkeleton::default()));
        let r = parsed.as_any().downcast_ref::<RosterSkeleton>().unwrap();
        assert_eq!(r.get_id(), "undead");
        assert_eq!(r.get_team_id(), "42");
    }
}
