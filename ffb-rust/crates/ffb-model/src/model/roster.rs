use std::any::Any;
use serde::{Deserialize, Serialize};
use crate::model::game::Game;
use crate::model::roster_position::RosterPosition;
use crate::xml::{IXmlReadable, XmlAttributes};
use crate::xml::util_xml::get_string_attribute;

const XML_TAG: &str = "roster";
const XML_ATTRIBUTE_ID: &str = "id";
const XML_ATTRIBUTE_TEAM: &str = "team";
const XML_TAG_NAME: &str = "name";
const XML_TAG_RE_ROLL_COST: &str = "reRollCost";
const XML_TAG_MAX_RE_ROLLS: &str = "maxReRolls";
const XML_TAG_KEYWORD: &str = "keyword";

/// A team roster definition (one per race per edition).
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

impl Roster {
    pub fn position(&self, id: &str) -> Option<&RosterPosition> {
        self.positions.iter().find(|p| p.id == id)
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
            is_undead: false,
            is_thrall: false,
            race: None,
            replaces_position: None,
            inside_skill_list_tag: false,
            inside_skill_category_list_tag: false,
            current_skill_value: None,
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
            is_big_guy: false, is_undead: false, is_thrall: false,
            race: None, replaces_position: None, inside_skill_list_tag: false, inside_skill_category_list_tag: false, current_skill_value: None,
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
            is_big_guy: false, is_undead: false, is_thrall: false,
            race: None, replaces_position: None, inside_skill_list_tag: false, inside_skill_category_list_tag: false, current_skill_value: None,
        });
        let non_stars: Vec<_> = r.non_star_positions().collect();
        assert_eq!(non_stars.len(), 1, "Should only return non-star positions");
        assert_eq!(non_stars[0].id, "lineman");
    }

    #[test]
    fn roster_fields_accessible() {
        let r = empty_roster();
        assert_eq!(r.id, "human");
        assert_eq!(r.reroll_cost, 50_000);
        assert_eq!(r.max_rerolls, 8);
        assert!(r.special_rules.is_empty());
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
            is_big_guy: false, is_undead: false, is_thrall: false,
            race: None, replaces_position: None, inside_skill_list_tag: false, inside_skill_category_list_tag: false, current_skill_value: None,
        });
        assert_eq!(r.positions.len(), 1);
    }

    #[test]
    fn has_necromancer_false_by_default() {
        let r = empty_roster();
        assert!(!r.has_necromancer());
    }

    #[test]
    fn has_necromancer_true_when_set() {
        let mut r = empty_roster();
        r.necromancer = true;
        assert!(r.has_necromancer());
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
            necromancer: false, keywords: vec![],
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
            necromancer: false, keywords: vec![],
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
