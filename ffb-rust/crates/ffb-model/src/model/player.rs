use std::any::Any;
use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use crate::enums::{PlayerType, PlayerGender, SeriousInjuryKind};
use crate::factory::player_gender_factory::PlayerGenderFactory;
use crate::factory::player_type_factory::PlayerTypeFactory;
use crate::factory::serious_injury_factory::SeriousInjuryFactory;
use crate::factory::skill_factory::SkillFactory;
use crate::model::game::Game;
use crate::model::player_status::PlayerStatus;
use crate::model::property::named_properties::NamedProperties;
use crate::model::skill_def::{SkillId, SkillWithValue};
use crate::model::roster_position::RosterPosition;
use crate::xml::{IXmlReadable, XmlAttributes};
use crate::xml::util_xml::{get_string_attribute, get_int_attribute, get_int_attribute_or, get_boolean_attribute};

/// Java: `Player.XML_TAG` (defined on the concrete `RosterPlayer` subclass).
pub(crate) const XML_TAG: &str = "player";
const XML_ATTRIBUTE_ID: &str = "id";
const XML_ATTRIBUTE_NR: &str = "nr";
const XML_ATTRIBUTE_STATUS: &str = "status";
const XML_ATTRIBUTE_VALUE: &str = "value";

const XML_TAG_NAME: &str = "name";
const XML_TAG_TYPE: &str = "type";
const XML_TAG_GENDER: &str = "gender";
const XML_TAG_POSITION_ID: &str = "positionId";

const XML_TAG_SKILL_LIST: &str = "skillList";
const XML_TAG_SKILL: &str = "skill";

const XML_TAG_INJURY_LIST: &str = "injuryList";
const XML_TAG_INJURY: &str = "injury";
const XML_ATTRIBUTE_RECOVERING: &str = "recovering";

const XML_TAG_PLAYER_STATISTICS: &str = "playerStatistics";
const XML_ATTRIBUTE_CURRENT_SPPS: &str = "currentSpps";

const XML_TAG_MOVEMENT: &str = "movement";
const XML_TAG_STRENGTH: &str = "strength";
const XML_TAG_AGILITY: &str = "agility";
const XML_TAG_PASSING: &str = "passing";
const XML_TAG_ARMOUR: &str = "armour";

/// Unique player identifier (string id as in the Java model).
pub type PlayerId = String;

/// Stat code constants matching Java's PlayerStatKey ordinal.
pub const STAT_MA: u8 = 0;
pub const STAT_ST: u8 = 1;
pub const STAT_AG: u8 = 2;
pub const STAT_PA: u8 = 3;
pub const STAT_AV: u8 = 4;

/// A concrete player instance on a team.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub nr: i32,
    pub position_id: String,
    pub player_type: PlayerType,
    pub gender: PlayerGender,

    // Current stats (may include advancements — Java: player.getXxx())
    pub movement: i32,
    pub strength: i32,
    pub agility: i32,
    pub passing: i32,
    pub armour: i32,

    // Position base stats — never modified after creation (Java: player.getPosition().getXxx())
    #[serde(default)]
    pub position_movement: i32,
    #[serde(default)]
    pub position_strength: i32,
    #[serde(default)]
    pub position_agility: i32,
    #[serde(default)]
    pub position_passing: i32,
    #[serde(default)]
    pub position_armour: i32,

    /// Skills the position starts with (defined on the roster position).
    #[serde(default)]
    pub starting_skills: Vec<SkillWithValue>,
    /// Skills gained via levelling (on top of position starting skills).
    pub extra_skills: Vec<SkillWithValue>,
    /// Skills granted temporarily (cards, prayers, etc.).
    pub temporary_skills: Vec<SkillWithValue>,
    /// Skills used this turn (reset at turn start).
    pub used_skills: HashSet<SkillId>,

    /// Permanent serious injuries reducing stats.
    pub niggling_injuries: i32,
    pub stat_injuries: Vec<SeriousInjuryKind>,

    pub current_spps: i32,
    pub career_spps: i32,

    /// Whether this player's position is a thrall (Java: position.isThrall()).
    #[serde(default)]
    pub is_thrall: bool,

    /// Whether this player's position has the BIG_GUY keyword (Java: position.getKeywords().contains(BIG_GUY)).
    /// Stored here to avoid roster lookup at mechanic time.
    #[serde(default)]
    pub is_big_guy: bool,

    /// Race identifier for Animosity checks (e.g. "Hobgoblin", "Bull Centaur").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub race: Option<String>,

    /// Temporary stat modifications from prayers/cards, keyed by source name for removal.
    /// Java: Player.addTemporaryModifiers(sourceName, modifiers) / removeTemporaryModifiers(sourceName).
    /// Each entry: (source_name, stat_code, delta). stat_code uses STAT_MA..STAT_AV constants.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub temporary_stat_mods: Vec<(String, u8, i32)>,

    /// Source tracking for prayer/card skill grants, keyed by source name for removal.
    /// Java: Player.addTemporarySkills(sourceName, skills) / removeTemporarySkills(sourceName).
    /// Paired with `temporary_skills` — when a skill is added via prayer, it appears in both.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub temporary_skill_sources: Vec<(String, SkillId)>,

    /// Java: RosterPlayer.fRecoveringInjury — the serious injury the player is recovering from (MNG).
    /// None means the player has no current MNG injury.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovering_injury: Option<SeriousInjuryKind>,

    /// Java: RosterPlayer.playerStatus — ACTIVE for registered players, JOURNEYMAN for hired-for-game players.
    #[serde(default)]
    pub player_status: PlayerStatus,

    /// True when this player has been ZAP-ped by a card; stats replaced by ZappedPosition values.
    /// Java: player instanceof ZappedPlayer check — tracked via GameState.isZapped().
    #[serde(default)]
    pub zapped: bool,

    /// Java: `RosterPlayer.fInsideSkillList` (transient) — true while inside `<skillList>`.
    #[serde(skip)]
    pub inside_skill_list: bool,
    /// Java: `RosterPlayer.fInsideInjuryList` (transient).
    #[serde(skip)]
    pub inside_injury_list: bool,
    /// Java: `RosterPlayer.fInjuryCurrent` (transient) — recovering= attribute of `<injury>`.
    #[serde(skip)]
    pub injury_current: bool,
    /// Java: `RosterPlayer.fInsidePlayerStatistics` (transient).
    #[serde(skip)]
    pub inside_player_statistics: bool,
    /// Java: `RosterPlayer.fCurrentSkillValue` (transient) — value= attribute of `<skill>`.
    #[serde(skip)]
    pub current_skill_value: Option<String>,
}

impl Player {
    /// Java: Player.getMovementWithModifiers() — base movement plus all temporary stat deltas.
    pub fn movement_with_modifiers(&self) -> i32 {
        self.movement
            + self.temporary_stat_mods.iter()
                .filter(|(_, s, _)| *s == STAT_MA)
                .map(|(_, _, d)| *d)
                .sum::<i32>()
    }

    pub fn strength_with_modifiers(&self) -> i32 {
        self.strength
            + self.temporary_stat_mods.iter()
                .filter(|(_, s, _)| *s == STAT_ST)
                .map(|(_, _, d)| *d)
                .sum::<i32>()
    }

    pub fn agility_with_modifiers(&self) -> i32 {
        self.agility
            + self.temporary_stat_mods.iter()
                .filter(|(_, s, _)| *s == STAT_AG)
                .map(|(_, _, d)| *d)
                .sum::<i32>()
    }

    pub fn passing_with_modifiers(&self) -> i32 {
        self.passing
            + self.temporary_stat_mods.iter()
                .filter(|(_, s, _)| *s == STAT_PA)
                .map(|(_, _, d)| *d)
                .sum::<i32>()
    }

    pub fn armour_with_modifiers(&self) -> i32 {
        self.armour
            + self.temporary_stat_mods.iter()
                .filter(|(_, s, _)| *s == STAT_AV)
                .map(|(_, _, d)| *d)
                .sum::<i32>()
    }

    /// Java: Player.addTemporaryModifiers(source, modifiers) — add a temporary stat delta.
    pub fn add_temporary_stat_mod(&mut self, source: &str, stat: u8, delta: i32) {
        self.temporary_stat_mods.push((source.to_string(), stat, delta));
    }

    /// Java: Player.removeTemporaryModifiers(source) — remove all stat mods for this source.
    pub fn remove_temporary_stat_mods(&mut self, source: &str) {
        self.temporary_stat_mods.retain(|(s, _, _)| s != source);
    }

    /// Java: Player.addTemporarySkills(source, skills) — add a skill grant tagged by source.
    pub fn add_prayer_skill(&mut self, source: &str, skill_id: SkillId, value: Option<String>) {
        self.temporary_skill_sources.push((source.to_string(), skill_id));
        self.temporary_skills.push(SkillWithValue { skill_id, value });
    }

    /// Java: Player.removeTemporarySkills(source) — remove all skills granted by this source.
    pub fn remove_prayer_skills(&mut self, source: &str) {
        let to_remove: Vec<SkillId> = self.temporary_skill_sources.iter()
            .filter(|(s, _)| s == source)
            .map(|(_, id)| *id)
            .collect();
        self.temporary_skill_sources.retain(|(s, _)| s != source);
        for skill_id in to_remove {
            if let Some(pos) = self.temporary_skills.iter().position(|sw| sw.skill_id == skill_id) {
                self.temporary_skills.remove(pos);
            }
        }
    }

    /// Java: Player.removeEnhancements(source) — remove all stat mods AND skill grants for source.
    pub fn remove_enhancements(&mut self, source: &str) {
        self.remove_temporary_stat_mods(source);
        self.remove_prayer_skills(source);
    }

    pub fn all_skill_ids(&self) -> impl Iterator<Item = SkillId> + '_ {
        self.starting_skills
            .iter()
            .chain(self.extra_skills.iter())
            .chain(self.temporary_skills.iter())
            .map(|sw| sw.skill_id)
    }

    pub fn has_skill(&self, id: SkillId) -> bool {
        self.all_skill_ids().any(|s| s == id)
    }

    /// 1:1 translation of hasSkillProperty — checks if any of the player's skills has the given property.
    pub fn has_skill_property(&self, property: &str) -> bool {
        self.all_skill_ids().any(|id| id.properties().contains(&property))
    }

    /// Java: getSkillWithProperty — returns the first SkillId that has the given property.
    pub fn skill_id_with_property(&self, property: &str) -> Option<SkillId> {
        self.all_skill_ids().find(|id| id.properties().contains(&property))
    }

    /// Java: UtilCards.hasUnusedSkillWithProperty — true if player has a skill with the given property
    /// AND that skill has not been used this drive (not in used_skills).
    pub fn has_unused_skill_with_property(&self, property: &str) -> bool {
        self.all_skill_ids()
            .filter(|id| id.properties().contains(&property))
            .any(|id| !self.used_skills.contains(&id))
    }

    /// 1:1 translation of getSkillIntValue — returns the integer value for a skill with a numeric property.
    /// TODO: requires full Skill property lookup to be implemented.
    pub fn get_skill_int_value(&self, _property: &str) -> i32 {
        0
    }

    /// 1:1 translation of canBeThrown — true if player has canBeThrown property, or canBeThrownIfStrengthIs3orLess and ST<=3.
    pub fn can_be_thrown(&self) -> bool {
        self.has_skill_property(NamedProperties::CAN_BE_THROWN)
            || (self.has_skill_property(NamedProperties::CAN_BE_THROWN_IF_STRENGTH_IS_3_OR_LESS) && self.strength_with_modifiers() <= 3)
    }

    /// 1:1 translation of isJourneyman — true if the player has journeyman status (borrowed for the drive).
    pub fn is_journeyman(&self) -> bool { self.player_status == PlayerStatus::JOURNEYMAN }

    /// Java: player instanceof ZappedPlayer — true when this player was ZAP-ped this drive.
    pub fn is_zapped(&self) -> bool { self.zapped }

    /// Java: RosterPlayer.resetUsedSkills — removes from used_skills all entries with the given usage type.
    pub fn reset_used_skills(&mut self, usage_type: crate::enums::SkillUsageType) {
        self.used_skills.retain(|id| id.usage_type() != usage_type);
    }

    /// Java: RosterPlayer.setPlayerStatus
    pub fn set_player_status(&mut self, status: PlayerStatus) { self.player_status = status; }

    /// Java: RosterPlayer.getPlayerStatus
    pub fn get_player_status(&self) -> PlayerStatus { self.player_status }

    /// Java: RosterPlayer.addSkill — adds to extra_skills if not already present.
    pub fn add_skill(&mut self, skill_id: SkillId) {
        if !self.has_skill(skill_id) {
            self.extra_skills.push(SkillWithValue { skill_id, value: None });
        }
    }

    /// Java: RosterPlayer.removeSkill — removes from extra_skills.
    pub fn remove_skill(&mut self, skill_id: SkillId) {
        if let Some(pos) = self.extra_skills.iter().position(|sw| sw.skill_id == skill_id) {
            self.extra_skills.remove(pos);
        }
    }

    /// Java: RosterPlayer.getSkills — all skills (starting + extra).
    pub fn get_skills(&self) -> Vec<SkillId> {
        self.starting_skills.iter().chain(self.extra_skills.iter()).map(|sw| sw.skill_id).collect()
    }

    /// Construct a new player instance from a roster position template.
    pub fn from_position(id: impl Into<String>, name: impl Into<String>, nr: i32, pos: &RosterPosition) -> Self {
        Player {
            id: id.into(),
            name: name.into(),
            nr,
            position_id: pos.id.clone(),
            player_type: pos.player_type,
            gender: pos.gender,
            movement: pos.movement,
            strength: pos.strength,
            agility: pos.agility,
            passing: pos.passing,
            armour: pos.armour,
            position_movement: pos.movement,
            position_strength: pos.strength,
            position_agility: pos.agility,
            position_passing: pos.passing,
            position_armour: pos.armour,
            starting_skills: pos.skills.clone(),
            extra_skills: vec![],
            temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0,
            stat_injuries: vec![],
            current_spps: 0,
            career_spps: 0,
            is_thrall: pos.is_thrall,
            is_big_guy: pos.is_big_guy,
            race: pos.race.clone(),
            temporary_stat_mods: vec![],
            temporary_skill_sources: vec![],
            recovering_injury: None,
            player_status: PlayerStatus::ACTIVE,
            zapped: false,
            inside_skill_list: false,
            inside_injury_list: false,
            injury_current: false,
            inside_player_statistics: false,
            current_skill_value: None,
        }
    }

    /// Java: `RosterPlayer.updatePosition(RosterPosition, IFactorySource, long)`, called from
    /// `Team.updateRoster` once a team's roster has been resolved (e.g. after XML-loading a
    /// standalone-mode team). Bounded scope: applies stat/skill resolution from the roster
    /// position; does not replay `PlayerModifier`/skill-behaviour effects (those apply once
    /// the player enters an active game, at a different layer, not at roster-load time).
    pub fn update_position(&mut self, position: Option<&RosterPosition>) {
        let Some(position) = position else { return };
        self.position_id = position.id.clone();
        self.movement = position.movement;
        self.strength = position.strength;
        self.agility = position.agility;
        self.passing = position.passing;
        self.armour = position.armour;
        self.position_movement = position.movement;
        self.position_strength = position.strength;
        self.position_agility = position.agility;
        self.position_passing = position.passing;
        self.position_armour = position.armour;
        self.is_thrall = position.is_thrall;
        self.is_big_guy = position.is_big_guy;
        self.race = position.race.clone();
        for sw in &position.skills {
            self.add_skill(sw.skill_id);
        }
    }
}

impl IXmlReadable for Player {
    /// Java: `RosterPlayer.startXmlElement(Game, String, Attributes)`.
    fn start_xml_element(&mut self, _game: Option<&Game>, tag: &str, atts: &XmlAttributes) -> Option<Box<dyn IXmlReadable>> {
        if self.inside_skill_list {
            if tag == XML_TAG_SKILL {
                self.current_skill_value = get_string_attribute(atts, XML_ATTRIBUTE_VALUE).filter(|v| !v.is_empty());
                // Java also tracks `currentDisplayValue` (displayValueAs=) — cosmetic, discarded.
            }
        } else if self.inside_injury_list {
            if tag == XML_TAG_INJURY {
                self.injury_current = get_boolean_attribute(atts, XML_ATTRIBUTE_RECOVERING);
            }
        } else {
            if tag == XML_TAG {
                if let Some(id) = get_string_attribute(atts, XML_ATTRIBUTE_ID) {
                    self.id = id;
                }
                self.nr = get_int_attribute(atts, XML_ATTRIBUTE_NR);
                if let Some(status) = get_string_attribute(atts, XML_ATTRIBUTE_STATUS).and_then(|s| PlayerStatus::for_name(&s)) {
                    self.player_status = status;
                }
                // Java: `iconSetIndex=` attribute — cosmetic client-rendering data, discarded.
            }
            if tag == XML_TAG_INJURY_LIST {
                self.inside_injury_list = true;
            }
            // Java: `<iconSet size=...>` — cosmetic, no field here; discarded.
            if tag == XML_TAG_SKILL_LIST {
                self.inside_skill_list = true;
            }
            if tag == XML_TAG_PLAYER_STATISTICS {
                self.current_spps = get_int_attribute_or(atts, XML_ATTRIBUTE_CURRENT_SPPS, 0);
                self.inside_player_statistics = true;
            }
        }
        None
    }

    /// Java: `RosterPlayer.endXmlElement(Game, String, String)`.
    fn end_xml_element(&mut self, game: Option<&Game>, tag: &str, value: &str) -> bool {
        let complete = tag == XML_TAG;
        if !complete {
            if self.inside_skill_list {
                if tag == XML_TAG_SKILL_LIST {
                    self.inside_skill_list = false;
                }
                if tag == XML_TAG_SKILL {
                    if let Some(skill_id) = SkillFactory::new().for_name(value) {
                        let sw = match self.current_skill_value.take() {
                            Some(v) => SkillWithValue::with_value(skill_id, v),
                            None => SkillWithValue::new(skill_id),
                        };
                        self.extra_skills.push(sw);
                    }
                }
            } else if self.inside_injury_list {
                if tag == XML_TAG_INJURY_LIST {
                    self.inside_injury_list = false;
                }
                if tag == XML_TAG_INJURY {
                    // Java: `((SeriousInjuryFactory) game.getFactory(SERIOUS_INJURY)).forName(pValue)`
                    // — requires a real `Game` (for `rules`); skipped when parsing without one
                    // (e.g. standalone roster/team caching before a game exists).
                    if let Some(game) = game {
                        let mut factory = SeriousInjuryFactory::new();
                        factory.initialize(game);
                        if let Some(injury) = factory.for_name(value) {
                            let kind = injury.to_kind();
                            self.stat_injuries.push(kind);
                            if self.injury_current {
                                self.recovering_injury = Some(kind);
                            }
                        }
                    }
                }
            } else if self.inside_player_statistics {
                if tag == XML_TAG_PLAYER_STATISTICS {
                    self.inside_player_statistics = false;
                }
            } else {
                // Java: `<portrait>`/`<iconSet>` — cosmetic client-rendering data, discarded.
                if tag == XML_TAG_NAME {
                    self.name = value.to_string();
                }
                if tag == XML_TAG_GENDER {
                    self.gender = PlayerGenderFactory::default().for_name(value).unwrap_or(PlayerGender::Male);
                }
                if tag == XML_TAG_POSITION_ID {
                    self.position_id = value.to_string();
                }
                if tag == XML_TAG_TYPE {
                    if let Some(t) = PlayerTypeFactory::default().for_name(value) {
                        self.player_type = t;
                    }
                }
                // Java: special "player without rosterPosition" fields — set stats on this
                // player directly, matching the RosterPlayer fallback path.
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
                // Java: `<race>`/`<shorthand>` write into `getPosition()` (the resolved
                // RosterPosition) — this parse layer has no roster reference to mutate;
                // discarded, same treatment as the other position-lookup-dependent tags.
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
    use crate::enums::{PlayerType, PlayerGender};
    use crate::model::player_status::PlayerStatus;

    fn test_player() -> Player {
        Player {
            id: "p1".into(),
            name: "Joe".into(),
            nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6,
            strength: 3,
            agility: 3,
            passing: 4,
            armour: 8,
            position_movement: 6,
            position_strength: 3,
            position_agility: 3,
            position_passing: 4,
            position_armour: 8,
            starting_skills: vec![],
            extra_skills: vec![],
            temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0,
            stat_injuries: vec![],
            current_spps: 0,
            career_spps: 0,
            is_thrall: false,
            is_big_guy: false,
            race: None,
            temporary_stat_mods: vec![],
            temporary_skill_sources: vec![],
            recovering_injury: None,
            player_status: PlayerStatus::ACTIVE,
            zapped: false,
            inside_skill_list: false,
            inside_injury_list: false,
            injury_current: false,
            inside_player_statistics: false,
            current_skill_value: None,
        }
    }

    #[test]
    fn serde_round_trip() {
        let p = test_player();
        let json = serde_json::to_string(&p).unwrap();
        let back: Player = serde_json::from_str(&json).unwrap();
        assert_eq!(p.id, back.id);
        assert_eq!(p.movement, back.movement);
    }

    #[test]
    fn has_skill_false_when_empty() {
        let p = test_player();
        assert!(!p.has_skill(SkillId::Block));
    }

    #[test]
    fn has_skill_true_for_starting_skill() {
        use crate::model::skill_def::SkillWithValue;
        let mut p = test_player();
        p.starting_skills.push(SkillWithValue { skill_id: SkillId::Block, value: None });
        assert!(p.has_skill(SkillId::Block));
        assert!(!p.has_skill(SkillId::Tackle));
    }

    #[test]
    fn has_skill_true_for_extra_skill() {
        use crate::model::skill_def::SkillWithValue;
        let mut p = test_player();
        p.extra_skills.push(SkillWithValue { skill_id: SkillId::Dodge, value: None });
        assert!(p.has_skill(SkillId::Dodge));
    }

    #[test]
    fn movement_with_modifiers_returns_base() {
        let p = test_player();
        assert_eq!(p.movement_with_modifiers(), 6);
    }

    #[test]
    fn strength_with_modifiers_returns_base() {
        let p = test_player();
        assert_eq!(p.strength_with_modifiers(), 3);
    }

    #[test]
    fn agility_with_modifiers_returns_base() {
        let p = test_player();
        assert_eq!(p.agility_with_modifiers(), 3);
    }

    #[test]
    fn armour_with_modifiers_returns_base() {
        let p = test_player();
        assert_eq!(p.armour_with_modifiers(), 8);
    }

    #[test]
    fn passing_with_modifiers_returns_base() {
        let p = test_player();
        assert_eq!(p.passing_with_modifiers(), 4);
    }

    #[test]
    fn has_skill_true_for_temporary_skill() {
        use crate::model::skill_def::SkillWithValue;
        let mut p = test_player();
        p.temporary_skills.push(SkillWithValue { skill_id: SkillId::Sprint, value: None });
        assert!(p.has_skill(SkillId::Sprint));
        assert!(!p.has_skill(SkillId::Block));
    }

    #[test]
    fn all_skill_ids_iterates_all_three_skill_lists() {
        use crate::model::skill_def::SkillWithValue;
        let mut p = test_player();
        p.starting_skills.push(SkillWithValue { skill_id: SkillId::Block, value: None });
        p.extra_skills.push(SkillWithValue { skill_id: SkillId::Dodge, value: None });
        p.temporary_skills.push(SkillWithValue { skill_id: SkillId::Sprint, value: None });
        let ids: Vec<SkillId> = p.all_skill_ids().collect();
        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&SkillId::Block));
        assert!(ids.contains(&SkillId::Dodge));
        assert!(ids.contains(&SkillId::Sprint));
    }

    #[test]
    fn niggling_injuries_default_zero_and_stat_injuries_empty() {
        let p = test_player();
        assert_eq!(p.niggling_injuries, 0);
        assert!(p.stat_injuries.is_empty());
        assert_eq!(p.current_spps, 0);
        assert_eq!(p.career_spps, 0);
    }

    #[test]
    fn from_position_copies_starting_skills() {
        use crate::model::skill_def::SkillWithValue;
        use crate::model::roster_position::RosterPosition;
        use crate::enums::{PlayerType, PlayerGender, SkillCategory};
        let pos = RosterPosition {
            id: "blitzer".into(),
            name: "Blitzer".into(),
            display_name: None,
            shorthand: None,
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            quantity: 4,
            cost: 80_000,
            movement: 7,
            strength: 3,
            agility: 3,
            passing: 4,
            armour: 9,
            skills: vec![SkillWithValue { skill_id: SkillId::Block, value: None }],
            skill_categories_normal: vec![SkillCategory::General],
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
        };
        let p = Player::from_position("p1", "Blitzer Joe", 3, &pos);
        assert_eq!(p.position_id, "blitzer");
        assert_eq!(p.movement, 7);
        assert!(p.has_skill(SkillId::Block));
        assert!(!p.has_skill(SkillId::Tackle));
    }

    #[test]
    fn has_skill_property_returns_true_for_matching_skill() {
        use crate::model::skill_def::SkillWithValue;
        let mut p = test_player();
        p.starting_skills.push(SkillWithValue { skill_id: SkillId::Block, value: None });
        assert!(p.has_skill_property("preventFallOnBothDown"));
        assert!(!p.has_skill_property("canLeap"));
    }

    #[test]
    fn has_skill_property_false_when_no_skills() {
        let p = test_player();
        assert!(!p.has_skill_property("preventFallOnBothDown"));
    }

    #[test]
    fn has_skill_property_checks_all_skill_lists() {
        use crate::model::skill_def::SkillWithValue;
        let mut p = test_player();
        p.extra_skills.push(SkillWithValue { skill_id: SkillId::Leap, value: None });
        assert!(p.has_skill_property("canLeap"));
    }

    #[test]
    fn reset_used_skills_clears_matching_usage_type() {
        use crate::enums::SkillUsageType;
        let mut p = test_player();
        p.used_skills.insert(SkillId::BeerBarrelBash); // OncePerDrive
        p.used_skills.insert(SkillId::Leader);         // OncePerHalf
        p.reset_used_skills(SkillUsageType::OncePerDrive);
        assert!(!p.used_skills.contains(&SkillId::BeerBarrelBash));
        assert!(p.used_skills.contains(&SkillId::Leader));
    }

    #[test]
    fn reset_used_skills_does_not_clear_wrong_type() {
        use crate::enums::SkillUsageType;
        let mut p = test_player();
        p.used_skills.insert(SkillId::GhostlyFlames); // OncePerHalf
        p.reset_used_skills(SkillUsageType::OncePerDrive);
        assert!(p.used_skills.contains(&SkillId::GhostlyFlames));
    }

    // ── temporary stat mod tests ─────────────────────────────────────────────

    #[test]
    fn movement_with_modifiers_includes_negative_delta() {
        let mut p = test_player(); // movement = 6
        p.add_temporary_stat_mod("GREASY_CLEATS", STAT_MA, -1);
        assert_eq!(p.movement_with_modifiers(), 5);
    }

    #[test]
    fn armour_with_modifiers_includes_positive_delta() {
        let mut p = test_player(); // armour = 8
        p.add_temporary_stat_mod("IRON_MAN", STAT_AV, 1);
        assert_eq!(p.armour_with_modifiers(), 9);
    }

    #[test]
    fn multiple_stat_mods_stack() {
        let mut p = test_player(); // movement = 6
        p.add_temporary_stat_mod("GREASY_CLEATS", STAT_MA, -1);
        p.add_temporary_stat_mod("OTHER", STAT_MA, -1);
        assert_eq!(p.movement_with_modifiers(), 4);
    }

    #[test]
    fn stat_mod_does_not_affect_other_stats() {
        let mut p = test_player();
        p.add_temporary_stat_mod("GREASY_CLEATS", STAT_MA, -1);
        // armour unaffected
        assert_eq!(p.armour_with_modifiers(), 8);
    }

    #[test]
    fn remove_temporary_stat_mods_clears_source() {
        let mut p = test_player();
        p.add_temporary_stat_mod("GREASY_CLEATS", STAT_MA, -1);
        p.remove_temporary_stat_mods("GREASY_CLEATS");
        assert_eq!(p.movement_with_modifiers(), 6);
        assert!(p.temporary_stat_mods.is_empty());
    }

    #[test]
    fn remove_temporary_stat_mods_only_removes_matching_source() {
        let mut p = test_player();
        p.add_temporary_stat_mod("GREASY_CLEATS", STAT_MA, -1);
        p.add_temporary_stat_mod("OTHER", STAT_MA, -1);
        p.remove_temporary_stat_mods("GREASY_CLEATS");
        assert_eq!(p.movement_with_modifiers(), 5); // OTHER still applies
    }

    // ── prayer skill grant tests ──────────────────────────────────────────────

    #[test]
    fn add_prayer_skill_adds_to_temporary_skills() {
        let mut p = test_player();
        p.add_prayer_skill("STILETTO", SkillId::Stab, None);
        assert!(p.has_skill(SkillId::Stab));
    }

    #[test]
    fn add_prayer_skill_with_value_stores_value() {
        let mut p = test_player();
        p.add_prayer_skill("BAD_HABITS", SkillId::Loner, Some("2".to_string()));
        assert!(p.has_skill(SkillId::Loner));
        let sw = p.temporary_skills.iter().find(|sw| sw.skill_id == SkillId::Loner).unwrap();
        assert_eq!(sw.value.as_deref(), Some("2"));
    }

    #[test]
    fn remove_prayer_skills_removes_from_temporary() {
        let mut p = test_player();
        p.add_prayer_skill("STILETTO", SkillId::Stab, None);
        assert!(p.has_skill(SkillId::Stab));
        p.remove_prayer_skills("STILETTO");
        assert!(!p.has_skill(SkillId::Stab));
        assert!(p.temporary_skill_sources.is_empty());
    }

    #[test]
    fn remove_prayer_skills_only_removes_matching_source() {
        let mut p = test_player();
        p.add_prayer_skill("STILETTO", SkillId::Stab, None);
        p.add_prayer_skill("BLESSING", SkillId::Block, None);
        p.remove_prayer_skills("STILETTO");
        assert!(!p.has_skill(SkillId::Stab));
        assert!(p.has_skill(SkillId::Block));
    }

    #[test]
    fn remove_enhancements_clears_both_mods_and_skills() {
        let mut p = test_player();
        p.add_temporary_stat_mod("GREASY_CLEATS", STAT_MA, -1);
        p.add_prayer_skill("GREASY_CLEATS", SkillId::Stab, None); // hypothetical
        p.remove_enhancements("GREASY_CLEATS");
        assert_eq!(p.movement_with_modifiers(), 6);
        assert!(!p.has_skill(SkillId::Stab));
    }

    #[test]
    fn player_status_defaults_to_active() {
        let p = Player::default();
        assert_eq!(p.player_status, PlayerStatus::ACTIVE);
    }

    #[test]
    fn is_journeyman_false_for_active() {
        let p = test_player();
        assert!(!p.is_journeyman());
    }

    #[test]
    fn is_journeyman_true_for_journeyman_status() {
        let mut p = test_player();
        p.set_player_status(PlayerStatus::JOURNEYMAN);
        assert!(p.is_journeyman());
    }

    #[test]
    fn add_skill_and_get_skills() {
        let mut p = test_player();
        p.add_skill(SkillId::Dodge);
        assert!(p.get_skills().contains(&SkillId::Dodge));
    }

    #[test]
    fn remove_skill_removes_from_extra() {
        let mut p = test_player();
        p.add_skill(SkillId::Dodge);
        p.remove_skill(SkillId::Dodge);
        assert!(!p.get_skills().contains(&SkillId::Dodge));
    }
}
