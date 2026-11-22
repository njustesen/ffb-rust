use std::any::Any;
use serde::{Deserialize, Serialize};
use crate::enums::{Keyword, PlayerType, PlayerGender, SkillCategory};
use crate::model::game::Game;
use crate::model::skill_def::{SkillId, SkillWithValue};
use crate::enums::Rules;
use crate::util::java_json::{self, JsonObject};
use serde_json::Value;

fn is_zero(v: &i32) -> bool {
    *v == 0
}
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
    /// Whether this position's keyword list contains `Keyword.LINEMAN`
    /// (Java: `position.getKeywords().contains(Keyword.LINEMAN)`).
    /// Stored here to avoid roster lookup at mechanic time, mirroring `is_big_guy`.
    pub is_lineman: bool,
    pub is_undead: bool,
    pub is_thrall: bool,
    pub race: Option<String>,
    /// ID of the position this replaces (star player slot, etc.).
    pub replaces_position: Option<String>,
    /// Java: `RosterPosition.fTeamWithPositionId` -- the partner position of a star pair, read by
    /// `UtilServerGame.checkForMissingPartners` (bb2020 Two for One).
    #[serde(default)]
    pub team_with_position_id: Option<String>,

    /// Java: `RosterPosition.fUrlPortrait` (client cosmetics; carried for the Java JSON, BACKLOG §O).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url_portrait: Option<String>,
    /// Java: `RosterPosition.fUrlIconSet`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url_icon_set: Option<String>,
    /// Java: `RosterPosition.fNrOfIcons`.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub nr_of_icons: i32,
    /// Java: `RosterPosition.fGender` is null (the JSON's `playerGender` null, absent or unknown --
    /// 21,152 corpus positions); `gender` keeps its default for the engine and the JSON writes null.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub gender_null: bool,
    /// Java: `RosterPosition.nameGenerator`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name_generator: Option<String>,
    /// Java: `RosterPosition.displayValues` (`LinkedHashMap<Skill, String>`, keyed like
    /// `fSkillValues` = `skills`): the NON-null entries only -- Java's `get` of an absent key and of
    /// a key mapped to null both read null, the only way the map is ever read.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skill_display_values: Vec<(SkillId, String)>,
    /// Java: `RosterPosition.fCurrentIconSetIndex` (transient; -1 from the Java constructor, set by
    /// `init_from`), advanced by `find_next_icon_set_index`.
    #[serde(skip)]
    pub current_icon_set_index: i32,

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
            // Rust-only derived field (mirrors data/loader.rs's position_json_to_roster_position):
            // Java computes "does this position have the LINEMAN keyword" at each call site
            // (`SwarmingLogicModule.squareHasSwarmingPlayer`) instead of storing it.
            self.is_lineman = self.keywords.iter().any(|k| k.eq_ignore_ascii_case("Lineman"));
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
            if tag == XML_TAG_TEAM_WITH_POSITION_ID {
                self.team_with_position_id = Some(value.to_string());
            }
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

    /// Java: `RosterPosition.findNextIconSetIndex()`.
    pub fn find_next_icon_set_index(&mut self) -> i32 {
        if self.nr_of_icons > 0 {
            self.current_icon_set_index += 1;
            if self.current_icon_set_index >= self.nr_of_icons {
                self.current_icon_set_index = 0;
            }
        }
        self.current_icon_set_index
    }

    /// Java: `fSkillValues.put(skill, value)` on the `LinkedHashMap`: a present key keeps its
    /// place and takes the new value.
    fn put_skill_value(&mut self, skill_id: SkillId, value: Option<String>) {
        match self.skills.iter_mut().find(|sw| sw.skill_id == skill_id) {
            Some(sw) => sw.value = value,
            None => self.skills.push(SkillWithValue { skill_id, value }),
        }
    }

    /// Java: `displayValues.put(skill, value)` (only non-null values are stored, see the field).
    fn put_display_value(&mut self, skill_id: SkillId, value: Option<String>) {
        self.skill_display_values.retain(|(s, _)| *s != skill_id);
        if let Some(v) = value {
            self.skill_display_values.push((skill_id, v));
        }
    }

    /// Java: `displayValues.get(skill)`.
    pub fn display_value(&self, skill_id: SkillId) -> Option<&str> {
        self.skill_display_values.iter().find(|(s, _)| *s == skill_id).map(|(_, v)| v.as_str())
    }

    /// Java: `RosterPosition.initFrom(IFactorySource, JsonValue)` (BACKLOG §O replay JSON), on a
    /// freshly constructed position. `Err` where Java throws, and where the Rust struct cannot hold
    /// the value Java would hold (a null id/name/type/gender, a null or unknown skill category).
    pub fn init_from(&mut self, rules: Rules, json_value: &Value) -> Result<(), String> {
        let o = java_json::to_json_object(json_value)?;
        // Java constructor.
        self.current_icon_set_index = -1;

        self.id = java_json::get_string(o, "positionId")?.ok_or("RosterPosition: null positionId")?;
        self.name = java_json::get_string(o, "positionName")?.ok_or("RosterPosition: null positionName")?;
        self.shorthand = java_json::get_string(o, "shorthand")?;
        self.display_name = java_json::get_string(o, "displayName")?;
        self.player_type = PlayerTypeFactory::default()
            .for_name(java_json::get_enum_name(o, "playerType")?.unwrap_or(""))
            .ok_or("RosterPosition: null playerType")?;
        let gender = PlayerGenderFactory::default().for_name(java_json::get_enum_name(o, "playerGender")?.unwrap_or(""));
        self.gender = gender.unwrap_or_default();
        self.gender_null = gender.is_none();
        self.quantity = java_json::get_int(o, "quantity")?;
        self.movement = java_json::get_int(o, "movement")?;
        self.strength = java_json::get_int(o, "strength")?;
        self.agility = java_json::get_int(o, "agility")?;
        self.passing = java_json::get_int(o, "passing")?;
        self.armour = java_json::get_int(o, "armour")?;
        self.cost = java_json::get_int(o, "cost")?;
        self.race = java_json::get_string(o, "race")?;
        self.is_undead = java_json::get_boolean(o, "undead")?;
        self.is_thrall = java_json::get_boolean(o, "thrall")?;
        self.team_with_position_id = java_json::get_string(o, "teamWithPositionId")?;
        self.name_generator = java_json::get_string(o, "nameGenerator")?;
        self.replaces_position = java_json::get_string(o, "replacesPosition")?;

        self.url_portrait = java_json::get_string(o, "urlPortrait")?;
        self.url_icon_set = java_json::get_string(o, "urlIconSet")?;
        self.nr_of_icons = java_json::get_int(o, "nrOfIcons")?;

        self.skill_categories_normal = Self::skill_categories_from(o, "skillCategoriesNormal")?;
        self.skill_categories_double = Self::skill_categories_from(o, "skillCategoriesDouble")?;

        self.skills.clear();
        self.skill_display_values.clear();
        let skill_array = java_json::get_array(o, "skillArray")?;
        let skill_values = java_json::get_string_array(o, "skillValues")?;
        let display_values = java_json::get_string_array(o, "skillDisplayValues")?;
        if let (Some(skill_array), Some(skill_values)) = (skill_array, skill_values.as_ref()) {
            // Java: `ArrayTool.isProvided(skillValues)` = non-null and non-empty.
            if !skill_array.is_empty() && !skill_values.is_empty() {
                for (i, element) in skill_array.iter().enumerate() {
                    let skill = match element {
                        Value::Null => None,
                        Value::String(name) => SkillFactory::for_name_in(rules, name),
                        other => return Err(format!("RosterPosition skillArray: asString on {other}")),
                    };
                    if let Some(skill) = skill {
                        let value = skill_values
                            .get(i)
                            .ok_or_else(|| format!("RosterPosition skillValues: index {i} out of bounds"))?
                            .clone();
                        self.put_skill_value(skill, value);
                        if let Some(display_values) = display_values.as_ref() {
                            if display_values.len() >= i + 1 {
                                self.put_display_value(skill, display_values[i].clone());
                            }
                        }
                    }
                }
            }
        }

        if java_json::is_defined_in(o, "keywords") {
            let names = java_json::get_string_array(o, "keywords")?
                .ok_or("RosterPosition: keywords is null (Java NPE)")?;
            for name in names {
                // Java: `Keyword.forName(name)` -- `equalsIgnoreCase(null)` is false -> UNKNOWN.
                let keyword = match name {
                    Some(n) => Keyword::for_name(&n),
                    None => Keyword::UNKNOWN,
                };
                self.keywords.push(keyword.get_name().to_string());
            }
        }

        // Rust-only caches the engine reads instead of the Java call-site computations (the same
        // derivation as the XML `end_xml_element`).
        self.is_big_guy = self.player_type == PlayerType::BigGuy
            || self.keywords.iter().any(|k| k.eq_ignore_ascii_case("Big Guy"));
        self.is_lineman = self.keywords.iter().any(|k| k.eq_ignore_ascii_case("Lineman"));
        Ok(())
    }

    /// Java: the `fSkillCategoriesOn{Normal,Double}Roll` loop of `initFrom` -- a `HashSet` of
    /// `UtilJson.toEnumWithName(SkillCategoryFactory, ..)`; a null array throws (`size()`).
    fn skill_categories_from(o: &JsonObject, key: &str) -> Result<Vec<SkillCategory>, String> {
        let array = java_json::get_array(o, key)?.ok_or_else(|| format!("RosterPosition: {key} is null (Java NPE)"))?;
        let mut categories = Vec::new();
        for element in array {
            let category = match element {
                Value::String(name) => SkillCategoryFactory::default().for_name(name),
                Value::Null => None,
                other => return Err(format!("RosterPosition {key}: asString on {other}")),
            };
            let category = category.ok_or_else(|| format!("RosterPosition {key}: null skill category {element}"))?;
            if !categories.contains(&category) {
                categories.push(category);
            }
        }
        Ok(categories)
    }

    /// Java: `RosterPosition.toJsonValue()` (BACKLOG §O). The skill categories are a Java
    /// `HashSet` (order is not part of the state: the canonical form sorts them).
    pub fn to_json_value(&self, rules: Rules) -> Result<Value, String> {
        let mut o = JsonObject::new();
        let s = |v: &Option<String>| java_json::string_or_null(v.as_deref());
        o.insert("positionId".into(), Value::from(self.id.clone()));
        o.insert("positionName".into(), Value::from(self.name.clone()));
        o.insert("shorthand".into(), s(&self.shorthand));
        o.insert("displayName".into(), s(&self.display_name));
        o.insert("playerType".into(), Value::from(self.player_type.name()));
        o.insert(
            "playerGender".into(),
            if self.gender_null { Value::Null } else { Value::from(self.gender.name()) },
        );
        o.insert("quantity".into(), Value::from(self.quantity));
        o.insert("movement".into(), Value::from(self.movement));
        o.insert("strength".into(), Value::from(self.strength));
        o.insert("agility".into(), Value::from(self.agility));
        o.insert("passing".into(), Value::from(self.passing));
        o.insert("armour".into(), Value::from(self.armour));
        o.insert("cost".into(), Value::from(self.cost));
        o.insert("race".into(), s(&self.race));
        o.insert("undead".into(), Value::from(self.is_undead));
        o.insert("thrall".into(), Value::from(self.is_thrall));
        o.insert("teamWithPositionId".into(), s(&self.team_with_position_id));
        o.insert("nameGenerator".into(), s(&self.name_generator));
        o.insert("replacesPosition".into(), s(&self.replaces_position));

        o.insert("urlPortrait".into(), s(&self.url_portrait));
        o.insert("urlIconSet".into(), s(&self.url_icon_set));
        o.insert("nrOfIcons".into(), Value::from(self.nr_of_icons));

        let categories = |v: &Vec<SkillCategory>| Value::Array(v.iter().map(|c| Value::from(c.name())).collect());
        o.insert("skillCategoriesNormal".into(), categories(&self.skill_categories_normal));
        o.insert("skillCategoriesDouble".into(), categories(&self.skill_categories_double));

        if !self.skills.is_empty() {
            let mut skill_array = Vec::new();
            let mut skill_values = Vec::new();
            let mut display_values = Vec::new();
            for sw in &self.skills {
                let name = SkillFactory::name_in(rules, sw.skill_id)
                    .ok_or_else(|| format!("RosterPosition: {:?} is not a {} skill", sw.skill_id, rules.name()))?;
                skill_array.push(Value::from(name));
                skill_values.push(s(&sw.value));
                display_values.push(java_json::string_or_null(self.display_value(sw.skill_id)));
            }
            o.insert("skillArray".into(), Value::Array(skill_array));
            o.insert("skillValues".into(), Value::Array(skill_values));
            o.insert("skillDisplayValues".into(), Value::Array(display_values));
        }

        o.insert("keywords".into(), Value::Array(self.keywords.iter().map(|k| Value::from(k.clone())).collect()));
        Ok(Value::Object(o))
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
            is_lineman: false,
            is_undead: false,
            is_thrall: false,
            race: None,
            replaces_position: None, team_with_position_id: None,
            inside_skill_list_tag: false,
            inside_skill_category_list_tag: false,
            current_skill_value: None, ..Default::default()
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
