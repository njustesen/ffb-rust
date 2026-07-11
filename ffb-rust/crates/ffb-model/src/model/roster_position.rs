use std::any::Any;
use serde::{Deserialize, Serialize};
use crate::enums::{Keyword, PlayerType, PlayerGender, SkillCategory};
use crate::model::game::Game;
use crate::model::skill_def::SkillWithValue;
use crate::xml::{IXmlReadable, XmlAttributes};
use crate::xml::util_xml::{get_string_attribute, get_int_attribute};
use crate::factory::skill_factory::SkillFactory;
use crate::factory::skill_category_factory::SkillCategoryFactory;
use crate::factory::player_type_factory::PlayerTypeFactory;
use crate::factory::player_gender_factory::PlayerGenderFactory;

const XML_TAG: &str = "position";
const XML_ATTRIBUTE_ID: &str = "id";
const XML_ATTRIBUTE_VALUE: &str = "value";

const XML_TAG_QUANTITY: &str = "quantity";
const XML_TAG_NAME: &str = "name";
const XML_TAG_DISPLAY_NAME: &str = "displayName";
const XML_TAG_TYPE: &str = "type";
const XML_TAG_GENDER: &str = "gender";
const XML_TAG_COST: &str = "cost";
const XML_TAG_MOVEMENT: &str = "movement";
const XML_TAG_STRENGTH: &str = "strength";
const XML_TAG_AGILITY: &str = "agility";
const XML_TAG_PASSING: &str = "passing";
const XML_TAG_ARMOUR: &str = "armour";
const XML_TAG_SHORTHAND: &str = "shorthand";
const XML_TAG_RACE: &str = "race";
const XML_TAG_UNDEAD: &str = "undead";
const XML_TAG_THRALL: &str = "thrall";
const XML_TAG_TEAM_WITH_POSITION_ID: &str = "teamWithPositionId";

const XML_TAG_SKILL_LIST: &str = "skillList";
const XML_TAG_SKILL: &str = "skill";

const XML_TAG_SKILLCATEGORY_LIST: &str = "skillCategoryList";
const XML_TAG_NORMAL: &str = "normal";
const XML_TAG_DOUBLE: &str = "double";

const XML_TAG_ICON_SET: &str = "iconSet";
const XML_ATTRIBUTE_SIZE: &str = "size";

const XML_TAG_REPLACES_POSITION: &str = "replacesPosition";
const XML_TAG_KEYWORD: &str = "keyword";

/// A position template within a team roster.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RosterPosition {
    pub id: String,
    pub name: String,
    pub display_name: Option<String>,
    pub shorthand: Option<String>,
    pub player_type: PlayerType,
    pub gender: PlayerGender,
    /// Maximum number of this position allowed in a team.
    pub quantity: i32,
    pub cost: i32,

    pub movement: i32,
    pub strength: i32,
    pub agility: i32,
    pub passing: i32,
    pub armour: i32,

    /// Starting skills.
    pub skills: Vec<SkillWithValue>,
    /// Skill categories available on a normal (single) roll.
    pub skill_categories_normal: Vec<SkillCategory>,
    /// Skill categories available on a double roll.
    pub skill_categories_double: Vec<SkillCategory>,

    pub keywords: Vec<String>,

    pub is_big_guy: bool,
    pub is_undead: bool,
    pub is_thrall: bool,
    pub race: Option<String>,
    /// ID of the position this replaces (star player slot, etc.).
    pub replaces_position: Option<String>,

    /// Java: `RosterPosition.fInsideSkillListTag` (transient) — true while inside `<skillList>`.
    #[serde(skip)]
    pub inside_skill_list_tag: bool,
    /// Java: `RosterPosition.fInsideSkillCategoryListTag` (transient).
    #[serde(skip)]
    pub inside_skill_category_list_tag: bool,
    /// Java: `RosterPosition.fCurrentSkillValue` (transient) — value= attribute of the
    /// `<skill>` tag currently being parsed.
    #[serde(skip)]
    pub current_skill_value: Option<String>,
}

impl IXmlReadable for RosterPosition {
    /// Java: `RosterPosition.startXmlElement(Game, String, Attributes)`.
    fn start_xml_element(&mut self, _game: Option<&Game>, tag: &str, atts: &XmlAttributes) -> Option<Box<dyn IXmlReadable>> {
        if self.inside_skill_list_tag {
            if tag == XML_TAG_SKILL {
                self.current_skill_value = get_string_attribute(atts, XML_ATTRIBUTE_VALUE).filter(|v| !v.is_empty());
                // Java also tracks `currentDisplayValue` (displayValueAs=) here — cosmetic
                // client-rendering data with no mechanically-relevant field on this struct,
                // parsed and discarded (same treatment as portrait/iconSet elsewhere).
            }
        } else {
            if tag == XML_TAG {
                if let Some(id) = get_string_attribute(atts, XML_ATTRIBUTE_ID) {
                    self.id = id;
                }
            }
            if tag == XML_TAG_SKILLCATEGORY_LIST {
                self.inside_skill_category_list_tag = true;
            }
            // Java: `<iconSet size=...>` sets fNrOfIcons — cosmetic, no field here; discarded.
            if tag == XML_TAG_SKILL_LIST {
                self.inside_skill_list_tag = true;
            }
        }
        None
    }

    /// Java: `RosterPosition.endXmlElement(Game, String, String)`.
    fn end_xml_element(&mut self, game: Option<&Game>, tag: &str, value: &str) -> bool {
        let complete = tag == XML_TAG;
        if complete {
            // Java: set a default shorthand if it is missing.
            if self.shorthand.as_deref().unwrap_or("").is_empty() && !self.name.is_empty() {
                self.shorthand = Some(self.name.chars().next().unwrap().to_string());
            }
            // Rust-only derived field (mirrors data/loader.rs's position_json_to_roster_position):
            // Java computes "is this a Big Guy" at each call site instead of storing it.
            self.is_big_guy = self.player_type == PlayerType::BigGuy
                || self.keywords.iter().any(|k| k.eq_ignore_ascii_case("Big Guy"));
        } else if self.inside_skill_list_tag {
            if tag == XML_TAG_SKILL_LIST {
                self.inside_skill_list_tag = false;
            }
            if tag == XML_TAG_SKILL {
                if let Some(skill_id) = SkillFactory::new().for_name(value) {
                    let sw = match self.current_skill_value.take() {
                        Some(v) => SkillWithValue::with_value(skill_id, v),
                        None => SkillWithValue::new(skill_id),
                    };
                    self.skills.push(sw);
                }
            }
        } else if self.inside_skill_category_list_tag {
            if tag == XML_TAG_SKILLCATEGORY_LIST {
                self.inside_skill_category_list_tag = false;
            }
            if tag == XML_TAG_NORMAL {
                if let Some(cat) = SkillCategoryFactory::default().for_name(value) {
                    self.skill_categories_normal.push(cat);
                }
            }
            if tag == XML_TAG_DOUBLE {
                if let Some(cat) = SkillCategoryFactory::default().for_name(value) {
                    self.skill_categories_double.push(cat);
                }
            }
        } else {
            if tag == XML_TAG_QUANTITY {
                self.quantity = value.parse().unwrap_or(0);
            }
            if tag == XML_TAG_NAME {
                self.name = value.to_string();
            }
            if tag == XML_TAG_DISPLAY_NAME {
                self.display_name = Some(value.to_string());
            }
            if tag == XML_TAG_SHORTHAND {
                self.shorthand = Some(value.to_string());
            }
            if tag == XML_TAG_TYPE {
                if let Some(t) = PlayerTypeFactory::default().for_name(value) {
                    self.player_type = t;
                }
            }
            if tag == XML_TAG_GENDER {
                if let Some(g) = PlayerGenderFactory::default().for_name(value) {
                    self.gender = g;
                }
            }
            if tag == XML_TAG_COST {
                self.cost = value.parse().unwrap_or(0);
            }
            if tag == XML_TAG_MOVEMENT {
                self.movement = value.parse().unwrap_or(0);
            }
            if tag == XML_TAG_STRENGTH {
                self.strength = value.parse().unwrap_or(0);
            }
            if tag == XML_TAG_AGILITY {
                self.agility = value.parse().unwrap_or(0);
            }
            if tag == XML_TAG_PASSING {
                self.passing = if !value.is_empty() { value.parse().unwrap_or(0) } else { 0 };
            }
            if tag == XML_TAG_ARMOUR {
                self.armour = value.parse().unwrap_or(0);
            }
            if tag == XML_TAG_RACE {
                self.race = Some(value.to_string());
            }
            if tag == XML_TAG_UNDEAD {
                self.is_undead = value.eq_ignore_ascii_case("true");
            }
            if tag == XML_TAG_THRALL {
                self.is_thrall = value.eq_ignore_ascii_case("true");
            }
            // Java: `<teamWithPositionId>`/`<nameGenerator>` — no field on this struct;
            // discarded, same treatment as other cosmetic/administrative-only tags.
            let _ = XML_TAG_TEAM_WITH_POSITION_ID;
            if tag == XML_TAG_REPLACES_POSITION {
                self.replaces_position = Some(value.to_string());
            }
            if tag.eq_ignore_ascii_case(XML_TAG_KEYWORD) {
                self.keywords.push(value.to_string());
            }
        }
        let _ = game;
        let _ = XML_TAG_ICON_SET;
        let _ = XML_ATTRIBUTE_SIZE;
        let _ = get_int_attribute;
        complete
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
}

impl RosterPosition {
    /// Java: `RosterPosition.XML_TAG`.
    pub const XML_TAG: &'static str = "position";

    pub fn is_star_player(&self) -> bool {
        self.player_type == PlayerType::Star
    }

    /// Java: position.getKeywords().contains(keyword) — case-insensitive check.
    /// Keywords are stored as strings in JSON (e.g. "Lineman", "Big Guy").
    pub fn has_keyword(&self, kw: Keyword) -> bool {
        let target = kw.get_name().to_lowercase();
        self.keywords.iter().any(|k| k.to_lowercase() == target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::{PlayerType, PlayerGender, SkillCategory};

    fn minimal() -> RosterPosition {
        RosterPosition {
            id: "lineman".into(),
            name: "Lineman".into(),
            display_name: None,
            shorthand: None,
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            quantity: 12,
            cost: 50_000,
            movement: 6,
            strength: 3,
            agility: 3,
            passing: 4,
            armour: 8,
            skills: vec![],
            skill_categories_normal: vec![SkillCategory::General],
            skill_categories_double: vec![
                SkillCategory::Agility,
                SkillCategory::Strength,
                SkillCategory::Passing,
            ],
            keywords: vec![],
            is_big_guy: false,
            is_undead: false,
            is_thrall: false,
            race: None,
            replaces_position: None,
            inside_skill_list_tag: false,
            inside_skill_category_list_tag: false,
            current_skill_value: None,
        }
    }

    #[test]
    fn serde_round_trip() {
        let pos = minimal();
        let json = serde_json::to_string(&pos).unwrap();
        let back: RosterPosition = serde_json::from_str(&json).unwrap();
        assert_eq!(pos.id, back.id);
        assert_eq!(pos.movement, back.movement);
    }

    #[test]
    fn is_star_player_false_for_regular() {
        assert!(!minimal().is_star_player());
    }

    #[test]
    fn is_star_player_true_for_star_type() {
        let mut pos = minimal();
        pos.player_type = PlayerType::Star;
        assert!(pos.is_star_player());
    }

    #[test]
    fn has_keyword_case_insensitive() {
        let mut pos = minimal();
        pos.keywords = vec!["Big Guy".into()];
        // The stored keyword uses title-case; has_keyword must match regardless of case
        assert!(pos.has_keyword(Keyword::BIG_GUY), "should match 'Big Guy' stored as title-case");
    }

    #[test]
    fn has_keyword_returns_false_when_absent() {
        let pos = minimal(); // keywords vec is empty
        assert!(
            !pos.has_keyword(Keyword::LINEMAN),
            "empty keywords list should never match any keyword"
        );
    }
}
