use std::any::Any;
use serde::{Deserialize, Serialize};
use crate::model::game::Game;
use crate::model::roster_position::RosterPosition;
use crate::xml::{IXmlReadable, XmlAttributes};
use crate::xml::util_xml::get_string_attribute;
use crate::enums::{Keyword, Rules};
use crate::util::java_json;
use serde_json::Value;

const XML_TAG: &str = "roster";
const XML_ATTRIBUTE_ID: &str = "id";
const XML_ATTRIBUTE_TEAM: &str = "team";
const XML_TAG_NAME: &str = "name";
const XML_TAG_RE_ROLL_COST: &str = "reRollCost";
const XML_TAG_MAX_RE_ROLLS: &str = "maxReRolls";
const XML_TAG_KEYWORD: &str = "keyword";

/// A team roster definition (one per race per edition).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Roster {
    pub id: String,
    pub name: String,
    pub race: String,
    pub reroll_cost: i32,
    pub max_rerolls: i32,
    pub positions: Vec<RosterPosition>,
    pub special_rules: Vec<String>,
    #[serde(default)]
    pub necromancer: bool,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub raised_position_id: Option<String>,

    /// Java: `Roster.fBaseIconPath` (client cosmetics; carried for the Java JSON, BACKLOG §O).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_icon_path: Option<String>,
    /// Java: `Roster.fLogoUrl`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logo_url: Option<String>,
    /// Java: `Roster.fApothecary` (`true` from the Java constructor; the JSON always sets it).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub apothecary: bool,
    /// Java: `Roster.fUndead`.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub undead: bool,
    /// Java: `Roster.riotousPositionId`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub riotous_position_id: Option<String>,
    /// Java: `Roster.nameGenerator`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name_generator: Option<String>,
    /// Java: `Roster.maxBigGuys`.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub max_big_guys: i32,
}

fn is_zero(v: &i32) -> bool {
    *v == 0
}

impl Roster {
    pub fn position(&self, id: &str) -> Option<&RosterPosition> {
        self.positions.iter().find(|p| p.id == id)
    }

    /// 1:1 translation of `Roster.getRaisedRosterPosition`.
    pub fn raised_roster_position(&self) -> Option<&RosterPosition> {
        self.raised_position_id.as_deref().and_then(|id| self.position(id))
    }

    pub fn non_star_positions(&self) -> impl Iterator<Item = &RosterPosition> {
        self.positions.iter().filter(|p| !p.is_star_player())
    }

    /// 1:1 translation of hasNecromancer.
    pub fn has_necromancer(&self) -> bool {
        self.necromancer
    }

    /// 1:1 translation of hasVampireLord.
    pub fn has_vampire_lord(&self) -> bool {
        self.keywords.iter().any(|k| k.eq_ignore_ascii_case("vampire lord"))
    }

    fn add_position(&mut self, position: RosterPosition) {
        self.positions.push(position);
    }

    /// Java: `Roster.addPosition` into `fRosterPositionById` (a `HashMap`): a position whose id
    /// is already present REPLACES that entry (the JSON path; order is not part of the state, the
    /// canonical form sorts `positionArray`).
    fn add_position_by_id(&mut self, position: RosterPosition) {
        match self.positions.iter_mut().find(|p| p.id == position.id) {
            Some(existing) => *existing = position,
            None => self.positions.push(position),
        }
    }

    /// Mutable `getPositionById` (the JSON `updatePosition` advances the position's icon index).
    pub fn position_mut(&mut self, id: &str) -> Option<&mut RosterPosition> {
        self.positions.iter_mut().find(|p| p.id == id)
    }

    /// Java: `Roster.initFrom(IFactorySource, JsonValue)` (BACKLOG §O replay JSON), on a freshly
    /// constructed roster. `Err` where Java throws or the Rust struct cannot hold the value.
    pub fn init_from(&mut self, rules: Rules, json_value: &Value) -> Result<(), String> {
        let o = java_json::to_json_object(json_value)?;
        self.id = java_json::get_string(o, "rosterId")?.ok_or("Roster: null rosterId")?;
        self.name = java_json::get_string(o, "rosterName")?.ok_or("Roster: null rosterName")?;
        self.reroll_cost = java_json::get_int(o, "reRollCost")?;
        self.max_rerolls = java_json::get_int(o, "maxReRolls")?;
        self.base_icon_path = java_json::get_string(o, "baseIconPath")?;
        self.logo_url = java_json::get_string(o, "logoUrl")?;
        self.raised_position_id = java_json::get_string(o, "raisedPositionId")?;
        self.apothecary = java_json::get_boolean(o, "apothecary")?;
        self.necromancer = java_json::get_boolean(o, "necromancer")?;
        self.undead = java_json::get_boolean(o, "undead")?;
        self.riotous_position_id = java_json::get_string(o, "riotousPositionId")?;
        self.name_generator = java_json::get_string(o, "nameGenerator")?;

        self.positions.clear();
        if let Some(position_array) = java_json::get_array(o, "positionArray")? {
            for element in position_array {
                let mut position = RosterPosition::default();
                position.init_from(rules, element)?;
                self.add_position_by_id(position);
            }
        }

        self.max_big_guys = java_json::get_int(o, "maxBigGuys")?;

        if java_json::is_defined_in(o, "keywords") {
            let names = java_json::get_string_array(o, "keywords")?.ok_or("Roster: keywords is null (Java NPE)")?;
            for name in names {
                let keyword = match name {
                    Some(n) => Keyword::for_name(&n),
                    None => Keyword::UNKNOWN,
                };
                self.keywords.push(keyword.get_name().to_string());
            }
        }
        Ok(())
    }

    /// Java: `Roster.toJsonValue()` (BACKLOG §O). `positionArray` iterates a `HashMap` in Java;
    /// the canonical form sorts it.
    pub fn to_json_value(&self, rules: Rules) -> Result<Value, String> {
        let mut o = java_json::JsonObject::new();
        let s = |v: &Option<String>| java_json::string_or_null(v.as_deref());
        o.insert("rosterId".into(), Value::from(self.id.clone()));
        o.insert("rosterName".into(), Value::from(self.name.clone()));
        o.insert("reRollCost".into(), Value::from(self.reroll_cost));
        o.insert("maxReRolls".into(), Value::from(self.max_rerolls));
        o.insert("baseIconPath".into(), s(&self.base_icon_path));
        o.insert("logoUrl".into(), s(&self.logo_url));
        o.insert("raisedPositionId".into(), s(&self.raised_position_id));
        o.insert("apothecary".into(), Value::from(self.apothecary));
        o.insert("necromancer".into(), Value::from(self.necromancer));
        o.insert("undead".into(), Value::from(self.undead));
        o.insert("riotousPositionId".into(), s(&self.riotous_position_id));
        o.insert("nameGenerator".into(), s(&self.name_generator));
        o.insert("maxBigGuys".into(), Value::from(self.max_big_guys));
        let positions = self.positions.iter().map(|p| p.to_json_value(rules)).collect::<Result<Vec<_>, _>>()?;
        o.insert("positionArray".into(), Value::Array(positions));
        o.insert("keywords".into(), Value::Array(self.keywords.iter().map(|k| Value::from(k.clone())).collect()));
        Ok(Value::Object(o))
    }
}

impl IXmlReadable for Roster {
    /// Java: `Roster.startXmlElement(Game, String, Attributes)`.
    fn start_xml_element(&mut self, game: Option<&Game>, tag: &str, atts: &XmlAttributes) -> Option<Box<dyn IXmlReadable>> {
        if tag == XML_TAG {
            if let Some(id) = get_string_attribute(atts, XML_ATTRIBUTE_ID).filter(|v| !v.is_empty()) {
                self.id = id;
            }
            // Java: the "team" attribute also overwrites fId here (not a separate field) —
            // reproduced faithfully even though it looks like a quirk in the original source.
            if let Some(team) = get_string_attribute(atts, XML_ATTRIBUTE_TEAM).filter(|v| !v.is_empty()) {
                self.id = team;
            }
        }
        if tag == RosterPosition::XML_TAG {
            let mut position = RosterPosition::default();
            position.start_xml_element(game, tag, atts);
            return Some(Box::new(position));
        }
        None
    }

    /// Java: `Roster.endXmlElement(Game, String, String)`.
    fn end_xml_element(&mut self, _game: Option<&Game>, tag: &str, value: &str) -> bool {
        let complete = tag == XML_TAG;
        if !complete {
            if tag == XML_TAG_NAME {
                self.name = value.to_string();
            }
            if tag == XML_TAG_RE_ROLL_COST {
                self.reroll_cost = value.parse().unwrap_or(0);
            }
            if tag == XML_TAG_MAX_RE_ROLLS {
                self.max_rerolls = value.parse().unwrap_or(0);
            }
            // Java: RosterPosition.XML_TAG completion calls `addPosition` here directly
            // (it already holds the child via `fCurrentlyParsedRosterPosition`) — the Rust
            // equivalent is `end_child`, called by the XML driver instead of from here.
            if tag.eq_ignore_ascii_case(XML_TAG_KEYWORD) {
                self.keywords.push(value.to_string());
            }
        }
        complete
    }

    /// Rust-only substitute for Java's `fCurrentlyParsedRosterPosition` field: called by the
    /// XML driver once a `<position>` element completes, handing back the fully-parsed child.
    fn end_child(&mut self, tag: &str, child: Box<dyn IXmlReadable>) {
        if tag == RosterPosition::XML_TAG {
            if let Ok(position) = child.into_any().downcast::<RosterPosition>() {
                self.add_position(*position);
            }
        }
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_roster() -> Roster {
        Roster {
            id: "human".into(),
            name: "Human".into(),
            race: "Human".into(),
            reroll_cost: 50_000,
            max_rerolls: 8,
            positions: vec![],
            special_rules: vec![],
            necromancer: false,
            keywords: vec![],
            raised_position_id: None, ..Default::default()
        }
    }

    #[test]
    fn serde_round_trip() {
        let r = empty_roster();
        let json = serde_json::to_string(&r).unwrap();
        let back: Roster = serde_json::from_str(&json).unwrap();
        assert_eq!(r.id, back.id);
        assert_eq!(r.reroll_cost, back.reroll_cost);
    }

    #[test]
    fn position_lookup() {
        let mut r = empty_roster();
        r.positions.push(crate::model::roster_position::RosterPosition {
            id: "lineman".into(),
            name: "Lineman".into(),
            display_name: None,
            shorthand: None,
            player_type: crate::enums::PlayerType::Regular,
            gender: crate::enums::PlayerGender::Male,
            quantity: 12,
            cost: 50_000,
            movement: 6,
            strength: 3,
            agility: 3,
            passing: 4,
            armour: 8,
            skills: vec![],
            skill_categories_normal: vec![],
            skill_categories_double: vec![],
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
        });
        assert!(r.position("lineman").is_some());
        assert!(r.position("blitzer").is_none());
    }

    #[test]
    fn non_star_positions_excludes_star_players() {
        let mut r = empty_roster();
        r.positions.push(crate::model::roster_position::RosterPosition {
            id: "lineman".into(),
            name: "Lineman".into(),
            display_name: None, shorthand: None,
            player_type: crate::enums::PlayerType::Regular,
            gender: crate::enums::PlayerGender::Male,
            quantity: 12, cost: 50_000,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            skills: vec![], skill_categories_normal: vec![],
            skill_categories_double: vec![], keywords: vec![],
            is_big_guy: false, is_lineman: false, is_undead: false, is_thrall: false,
            race: None, replaces_position: None, team_with_position_id: None, inside_skill_list_tag: false, inside_skill_category_list_tag: false, current_skill_value: None, ..Default::default()
        });
        r.positions.push(crate::model::roster_position::RosterPosition {
            id: "star_griff".into(),
            name: "Griff Oberwald".into(),
            display_name: None, shorthand: None,
            player_type: crate::enums::PlayerType::Star,
            gender: crate::enums::PlayerGender::Male,
            quantity: 1, cost: 280_000,
            movement: 8, strength: 3, agility: 2, passing: 2, armour: 8,
            skills: vec![], skill_categories_normal: vec![],
            skill_categories_double: vec![], keywords: vec![],
            is_big_guy: false, is_lineman: false, is_undead: false, is_thrall: false,
            race: None, replaces_position: None, team_with_position_id: None, inside_skill_list_tag: false, inside_skill_category_list_tag: false, current_skill_value: None, ..Default::default()
        });
        let non_stars: Vec<_> = r.non_star_positions().collect();
        assert_eq!(non_stars.len(), 1, "Should only return non-star positions");
        assert_eq!(non_stars[0].id, "lineman");
    }

    #[test]
    fn position_count_reflects_added_positions() {
        let mut r = empty_roster();
        assert_eq!(r.positions.len(), 0);
        r.positions.push(crate::model::roster_position::RosterPosition {
            id: "pos1".into(), name: "P1".into(),
            display_name: None, shorthand: None,
            player_type: crate::enums::PlayerType::Regular,
            gender: crate::enums::PlayerGender::Male,
            quantity: 4, cost: 60_000,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            skills: vec![], skill_categories_normal: vec![],
            skill_categories_double: vec![], keywords: vec![],
            is_big_guy: false, is_lineman: false, is_undead: false, is_thrall: false,
            race: None, replaces_position: None, team_with_position_id: None, inside_skill_list_tag: false, inside_skill_category_list_tag: false, current_skill_value: None, ..Default::default()
        });
        assert_eq!(r.positions.len(), 1);
    }

    fn zombie_position() -> crate::model::roster_position::RosterPosition {
        crate::model::roster_position::RosterPosition {
            id: "necromantic.zombie".into(), name: "Zombie".into(),
            display_name: None, shorthand: None,
            player_type: crate::enums::PlayerType::Regular,
            gender: crate::enums::PlayerGender::Male,
            quantity: 16, cost: 40_000,
            movement: 3, strength: 3, agility: 3, passing: 5, armour: 8,
            skills: vec![], skill_categories_normal: vec![],
            skill_categories_double: vec![], keywords: vec![],
            is_big_guy: false, is_lineman: false, is_undead: true, is_thrall: false,
            race: None, replaces_position: None, team_with_position_id: None, inside_skill_list_tag: false, inside_skill_category_list_tag: false, current_skill_value: None, ..Default::default()
        }
    }

    #[test]
    fn raised_roster_position_resolves_by_id() {
        let mut r = empty_roster();
        r.positions.push(zombie_position());
        r.raised_position_id = Some("necromantic.zombie".into());
        let pos = r.raised_roster_position().expect("should resolve");
        assert_eq!(pos.id, "necromantic.zombie");
    }

    #[test]
    fn raised_roster_position_none_when_id_not_found() {
        let mut r = empty_roster();
        r.raised_position_id = Some("missing.position".into());
        assert!(r.raised_roster_position().is_none());
    }

    #[test]
    fn has_vampire_lord_false_without_keyword() {
        let r = empty_roster();
        assert!(!r.has_vampire_lord());
    }

    #[test]
    fn has_vampire_lord_true_with_keyword() {
        let mut r = empty_roster();
        r.keywords.push("vampire lord".into());
        assert!(r.has_vampire_lord());
    }

    /// Excerpt of the real ffb-java `ffb-server/rosters/roster_amazon.xml` fixture
    /// (two regular positions + one star, trimmed to keep the test focused).
    const AMAZON_ROSTER_XML: &str = r#"
        <roster id="amazon.lrb6">
            <name>Amazon</name>
            <reRollCost>50000</reRollCost>
            <maxReRolls>8</maxReRolls>
            <position id="amazon.blitzer">
                <quantity>4</quantity>
                <name>Blitzer</name>
                <type>Regular</type>
                <cost>90000</cost>
                <movement>6</movement>
                <strength>3</strength>
                <agility>3</agility>
                <passing>5</passing>
                <armour>7</armour>
                <skillList>
                    <skill>Dodge</skill>
                    <skill>Block</skill>
                </skillList>
                <skillCategoryList>
                    <normal>General</normal>
                    <normal>Strength</normal>
                    <double>Agility</double>
                    <double>Passing</double>
                </skillCategoryList>
            </position>
            <position id="amazon.Helmut">
                <quantity>1</quantity>
                <name>Helmut Wulf</name>
                <shorthand>HW</shorthand>
                <type>Star</type>
                <cost>110000</cost>
                <movement>6</movement>
                <strength>3</strength>
                <agility>3</agility>
                <armour>8</armour>
                <skillList>
                    <skill>Stand Firm</skill>
                </skillList>
            </position>
        </roster>
    "#;

    #[test]
    fn parses_real_roster_fixture_root_fields() {
        let parsed = crate::xml::XmlHandler::parse(None, AMAZON_ROSTER_XML, Box::new(Roster {
            id: String::new(), name: String::new(), race: String::new(),
            reroll_cost: 0, max_rerolls: 0, positions: vec![], special_rules: vec![],
            necromancer: false, keywords: vec![], raised_position_id: None, ..Default::default()
        }));
        let roster = parsed.as_any().downcast_ref::<Roster>().unwrap();
        assert_eq!(roster.id, "amazon.lrb6");
        assert_eq!(roster.name, "Amazon");
        assert_eq!(roster.reroll_cost, 50_000);
        assert_eq!(roster.max_rerolls, 8);
    }

    #[test]
    fn parses_real_roster_fixture_positions() {
        let parsed = crate::xml::XmlHandler::parse(None, AMAZON_ROSTER_XML, Box::new(Roster {
            id: String::new(), name: String::new(), race: String::new(),
            reroll_cost: 0, max_rerolls: 0, positions: vec![], special_rules: vec![],
            necromancer: false, keywords: vec![], raised_position_id: None, ..Default::default()
        }));
        let roster = parsed.as_any().downcast_ref::<Roster>().unwrap();
        assert_eq!(roster.positions.len(), 2);

        let blitzer = roster.position("amazon.blitzer").unwrap();
        assert_eq!(blitzer.name, "Blitzer");
        assert_eq!(blitzer.quantity, 4);
        assert_eq!(blitzer.cost, 90_000);
        assert_eq!(blitzer.movement, 6);
        assert_eq!(blitzer.armour, 7);
        assert_eq!(blitzer.player_type, crate::enums::PlayerType::Regular);
        assert_eq!(blitzer.skills.len(), 2);
        assert!(blitzer.skills.iter().any(|s| s.skill_id == crate::enums::SkillId::Dodge));
        assert!(blitzer.skills.iter().any(|s| s.skill_id == crate::enums::SkillId::Block));
        assert_eq!(blitzer.skill_categories_normal.len(), 2);
        assert_eq!(blitzer.skill_categories_double.len(), 2);
        // Blitzer has no explicit <shorthand> tag, so the default-shorthand-from-first-letter
        // fallback (RosterPosition.endXmlElement's XML_TAG-complete branch) must kick in.
        assert_eq!(blitzer.shorthand.as_deref(), Some("B"));

        let helmut = roster.position("amazon.Helmut").unwrap();
        assert_eq!(helmut.name, "Helmut Wulf");
        assert_eq!(helmut.shorthand.as_deref(), Some("HW"));
        assert_eq!(helmut.player_type, crate::enums::PlayerType::Star);
        assert!(helmut.is_star_player());
    }
}
