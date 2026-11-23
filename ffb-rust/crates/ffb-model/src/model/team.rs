use std::any::Any;
use serde::{Deserialize, Serialize};
use crate::model::game::Game;
use crate::model::player::Player;
use crate::model::roster::Roster;
use crate::xml::{IXmlReadable, XmlAttributes};
use crate::xml::util_xml::get_string_attribute;
use crate::enums::Rules;
use crate::model::special_rule::SpecialRule;
use crate::util::java_json;
use serde_json::Value;

const XML_TAG: &str = "team";
const XML_ATTRIBUTE_ID: &str = "id";
const XML_TAG_NAME: &str = "name";
const XML_TAG_RACE: &str = "race";
const XML_TAG_ROSTER_ID: &str = "rosterId";
const XML_TAG_RE_ROLLS: &str = "reRolls";
const XML_TAG_APOTHECARIES: &str = "apothecaries";
const XML_TAG_CHEERLEADERS: &str = "cheerleaders";
const XML_TAG_ASSISTANT_COACHES: &str = "assistantCoaches";
const XML_TAG_COACH: &str = "coach";
const XML_TAG_FAN_FACTOR: &str = "fanFactor";
const XML_TAG_TEAM_VALUE: &str = "currentTeamValue";
const XML_TAG_TREASURY: &str = "treasury";
const XML_TAG_DEDICATED_FANS: &str = "dedicatedFans";
const XML_TAG_RULE: &str = "rule";

/// One of the two sides in a game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: String,
    pub name: String,
    pub race: String,
    pub roster_id: String,
    pub coach: String,

    pub rerolls: i32,
    pub apothecaries: i32,
    pub bribes: i32,
    #[serde(default)]
    pub master_chefs: i32,
    #[serde(default)]
    pub prayers_to_nuffle: i32,
    #[serde(default)]
    pub bloodweiser_kegs: i32,
    #[serde(default)]
    pub riotous_rookies: i32,
    pub cheerleaders: i32,
    pub assistant_coaches: i32,
    pub fan_factor: i32,
    pub dedicated_fans: i32,
    pub team_value: i32,
    pub treasury: i32,

    pub special_rules: Vec<String>,
    pub players: Vec<Player>,

    /// True when this team's roster has the VAMPIRE_LORD keyword (e.g. Blood Bowl vampire teams).
    /// Java: team.getRoster().hasVampireLord() — stored here to avoid roster lookup at mechanic time.
    #[serde(default)]
    pub vampire_lord: bool,

    /// True when this team's roster has `necromancer: true` (Necromantic Horror, Undead in BB2016).
    /// Java: team.getRoster().hasNecromancer() — stored here to avoid roster lookup at mechanic time.
    #[serde(default)]
    pub necromancer: bool,

    /// Java: `Team.fRoster` -- the roster the team JSON carries (BACKLOG §O replay state). The
    /// engine's own games take their positions from the roster cache instead and leave it `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roster: Option<Box<Roster>>,
    /// Java: `Team.fBaseIconPath` (client cosmetics; carried for the Java JSON).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_icon_path: Option<String>,
    /// Java: `Team.fLogoUrl`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logo_url: Option<String>,
}

/// Java: `new Team(source)` -- the blank team a fresh server `Game` holds before a coach joins.
impl Default for Team {
    fn default() -> Self {
        Team {
            id: String::new(), name: String::new(), race: String::new(),
            roster_id: String::new(), coach: String::new(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0,
            team_value: 0, treasury: 0, special_rules: vec![], players: vec![],
            vampire_lord: false, necromancer: false, roster: None, base_icon_path: None, logo_url: None,
        }
    }
}

impl Team {
    /// Java: `new Team(IFactorySource)` -- an empty team (no players, no special rules).
    pub fn new() -> Self {
        Team {
            id: String::new(), name: String::new(), race: String::new(), roster_id: String::new(),
            coach: String::new(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: Vec::new(), players: Vec::new(), vampire_lord: false, necromancer: false,
            roster: None, base_icon_path: None, logo_url: None,
        }
    }

    pub fn player(&self, id: &str) -> Option<&Player> {
        self.players.iter().find(|p| p.id == id)
    }

    pub fn player_mut(&mut self, id: &str) -> Option<&mut Player> {
        self.players.iter_mut().find(|p| p.id == id)
    }

    pub fn has_player(&self, id: &str) -> bool {
        self.player(id).is_some()
    }

    pub fn player_by_nr(&self, nr: i32) -> Option<&Player> {
        self.players.iter().find(|p| p.nr == nr)
    }

    /// Java: `Team.updateRoster(Roster, IFactorySource)` — `updateStats` defaults to `true`.
    pub fn update_roster(&mut self, roster: &Roster) {
        self.roster_id = roster.id.clone();
        self.race = roster.name.clone();
        for player in &mut self.players {
            let position = roster.position(&player.position_id);
            player.update_position(position);
        }
    }

    /// Java: `Team.addPlayer` into `fPlayerByNr` (a `HashMap`): a player whose nr is already
    /// present REPLACES that player (`getPlayers()` -- and so the JSON -- only ever sees the last
    /// one; Java's `fPlayerById` would still find the replaced one by id, Rust's lookups do not).
    fn add_player_by_nr(&mut self, player: Player) {
        match self.players.iter_mut().find(|p| p.nr == player.nr) {
            Some(existing) => *existing = player,
            None => self.players.push(player),
        }
    }

    /// Java: `Team.addPlayer(player)` for a player whose id the team does not hold yet (BACKLOG §O,
    /// `ClientCommandHandlerAddPlayer`). `Err` when another player already has that nr: Java's
    /// `fPlayerByNr` would drop it from `getPlayers()` while `fPlayerById` still finds it, which the
    /// Rust team (one list) cannot hold.
    pub fn add_player(&mut self, player: Player) -> Result<(), String> {
        if let Some(existing) = self.players.iter().find(|p| p.nr == player.nr && p.id != player.id) {
            return Err(format!(
                "Team.addPlayer: nr {} of {} replaces {} in fPlayerByNr only (not representable)",
                player.nr, player.id, existing.id
            ));
        }
        self.add_player_by_nr(player);
        Ok(())
    }

    /// Java: `Team.initFrom(IFactorySource, JsonValue)` (BACKLOG §O replay JSON), on a freshly
    /// constructed team. `Err` where Java throws (an unknown special rule: `SpecialRule.valueOf`)
    /// or where the Rust struct cannot hold the Java value.
    pub fn init_from(&mut self, rules: Rules, json_value: &Value) -> Result<(), String> {
        let o = java_json::to_json_object(json_value)?;
        self.id = java_json::get_string(o, "teamId")?.ok_or("Team: null teamId")?;
        self.name = java_json::get_string(o, "teamName")?.ok_or("Team: null teamName")?;
        self.coach = java_json::get_string(o, "coach")?.ok_or("Team: null coach")?;
        self.race = java_json::get_string(o, "race")?.ok_or("Team: null race")?;
        self.rerolls = java_json::get_int(o, "reRolls")?;
        self.apothecaries = java_json::get_int(o, "apothecaries")?;
        self.cheerleaders = java_json::get_int(o, "cheerleaders")?;
        self.assistant_coaches = java_json::get_int(o, "assistantCoaches")?;
        self.fan_factor = java_json::get_int(o, "fanFactor")?;
        self.team_value = java_json::get_int(o, "teamValue")?;
        self.treasury = java_json::get_int(o, "treasury")?;
        self.base_icon_path = java_json::get_string(o, "baseIconPath")?;
        self.logo_url = java_json::get_string(o, "logoUrl")?;
        self.dedicated_fans = java_json::get_int(o, "dedicatedFans")?;
        if let Some(rules_array) = java_json::get_string_array(o, "specialRules")? {
            // ArrayTool.isProvided + filter(nonNull) + SpecialRule::valueOf (throws on unknown),
            // into a Set. The JSON carries the enum CONSTANT names; the engine keeps every special
            // rule by its `getRuleName()` (the XML/roster form every engine check compares with).
            for name in rules_array.into_iter().flatten() {
                let Some(rule) = SpecialRule::value_of(&name) else {
                    return Err(format!("Team: SpecialRule.valueOf({name:?}) throws"));
                };
                let rule_name = rule.get_rule_name().to_string();
                if !self.special_rules.contains(&rule_name) {
                    self.special_rules.push(rule_name);
                }
            }
        }

        self.players.clear();
        let player_array = java_json::get_array(o, "playerArray")?.ok_or("Team: playerArray is null (Java NPE)")?;
        for element in player_array {
            let mut player = Player::default();
            player.init_from(rules, element)?;
            self.add_player_by_nr(player);
        }

        let roster = match o.get("roster") {
            Some(Value::Object(_)) => {
                let mut roster = Roster::default();
                roster.init_from(rules, &o["roster"])?;
                Some(Box::new(roster))
            }
            None | Some(Value::Null) => None,
            Some(other) => return Err(format!("Team roster: asObject on {other}")),
        };
        self.update_roster_json(roster);
        Ok(())
    }

    /// Java: `Team.updateRoster(roster, updateStats = false, ..)`: keeps the roster, takes its id
    /// and NAME as the team's roster id and race, and updates every player's position in
    /// `getPlayers()` order (by nr -- the order the positions' icon counters advance in). Plus the
    /// Rust-only roster caches the engine reads (`vampire_lord`, `necromancer`).
    pub fn update_roster_json(&mut self, roster: Option<Box<Roster>>) {
        self.roster = roster;
        let Some(roster) = self.roster.as_mut() else { return };
        self.roster_id = roster.id.clone();
        self.race = roster.name.clone();
        self.vampire_lord = roster.has_vampire_lord();
        self.necromancer = roster.has_necromancer();
        let mut order: Vec<usize> = (0..self.players.len()).collect();
        order.sort_by_key(|&i| self.players[i].nr);
        for i in order {
            let position_id = self.players[i].position_id.clone();
            self.players[i].update_position_json(roster.position_mut(&position_id));
        }
    }

    /// Java: `Team.toJsonValue()` (BACKLOG §O). `playerArray` = `getPlayers()`, sorted by nr.
    pub fn to_json_value(&self, rules: Rules) -> Result<Value, String> {
        self.to_json_value_with(rules, &[])
    }

    /// `Team.toJsonValue()` with the game's zapped players: a zapped player is a Java
    /// `ZappedPlayer` in the team, whose `toJsonValue` is `{"playerKind": "zappedPlayer", "player":
    /// <originalPlayer.toJsonValue()>}` (the Rust team holds the frog-stat copy, the original lives
    /// in `Game.zapped_players`).
    pub fn to_json_value_with(&self, rules: Rules, zapped: &[crate::model::zapped_player::ZappedPlayer]) -> Result<Value, String> {
        let s = |v: &Option<String>| java_json::string_or_null(v.as_deref());
        let mut o = java_json::JsonObject::new();
        o.insert("teamId".into(), Value::from(self.id.clone()));
        o.insert("teamName".into(), Value::from(self.name.clone()));
        o.insert("coach".into(), Value::from(self.coach.clone()));
        o.insert("race".into(), Value::from(self.race.clone()));
        o.insert("reRolls".into(), Value::from(self.rerolls));
        o.insert("apothecaries".into(), Value::from(self.apothecaries));
        o.insert("cheerleaders".into(), Value::from(self.cheerleaders));
        o.insert("assistantCoaches".into(), Value::from(self.assistant_coaches));
        o.insert("fanFactor".into(), Value::from(self.fan_factor));
        o.insert("teamValue".into(), Value::from(self.team_value));
        o.insert("treasury".into(), Value::from(self.treasury));
        o.insert("baseIconPath".into(), s(&self.base_icon_path));
        o.insert("logoUrl".into(), s(&self.logo_url));
        o.insert("dedicatedFans".into(), Value::from(self.dedicated_fans));
        // Java `specialRules.stream().map(Enum::name)`: the constant name of each rule.
        let special_rules = self
            .special_rules
            .iter()
            .map(|r| {
                SpecialRule::from(r)
                    .map(|rule| Value::from(format!("{rule:?}")))
                    .ok_or_else(|| format!("Team {}: special rule {r:?} is not a SpecialRule", self.id))
            })
            .collect::<Result<Vec<_>, String>>()?;
        o.insert("specialRules".into(), Value::Array(special_rules));
        let mut players: Vec<&Player> = self.players.iter().collect();
        players.sort_by_key(|p| p.nr);
        let player_array = players
            .into_iter()
            .map(|p| match zapped.iter().find(|z| z.original_player.id == p.id) {
                Some(z) => {
                    let mut o = java_json::JsonObject::new();
                    o.insert("playerKind".into(), Value::from("zappedPlayer"));
                    o.insert("player".into(), z.original_player.to_json_value(rules)?);
                    Ok(Value::Object(o))
                }
                None => p.to_json_value(rules),
            })
            .collect::<Result<Vec<_>, String>>()?;
        o.insert("playerArray".into(), Value::Array(player_array));
        if let Some(roster) = &self.roster {
            o.insert("roster".into(), roster.to_json_value(rules)?);
        }
        Ok(Value::Object(o))
    }
}

impl IXmlReadable for Team {
    /// Java: `Team.startXmlElement(Game, String, Attributes)`.
    fn start_xml_element(&mut self, game: Option<&Game>, tag: &str, atts: &XmlAttributes) -> Option<Box<dyn IXmlReadable>> {
        if tag == XML_TAG {
            if let Some(id) = get_string_attribute(atts, XML_ATTRIBUTE_ID) {
                self.id = id;
            }
        }
        if tag == crate::model::player::XML_TAG {
            let mut player = Player::default();
            player.start_xml_element(game, tag, atts);
            return Some(Box::new(player));
        }
        // Java also handles `ZappedPlayer.XML_TAG`/`InducementSet.XML_TAG` here — both are
        // mid-game serialized-snapshot concerns (travel as JSON via DbGamesSerialized) that
        // never appear in the standalone disk `teams/*.xml` this parser targets; documented
        // no-op, matching the project's stated scope for this XML pipeline.
        None
    }

    /// Java: `Team.endXmlElement(Game, String, String)`.
    fn end_xml_element(&mut self, _game: Option<&Game>, tag: &str, value: &str) -> bool {
        let complete = tag == XML_TAG;
        if !complete {
            if tag == XML_TAG_NAME {
                self.name = value.to_string();
            }
            if tag == XML_TAG_COACH {
                self.coach = value.to_string();
            }
            if tag == XML_TAG_RACE {
                self.race = value.to_string();
            }
            if tag == XML_TAG_ROSTER_ID {
                self.roster_id = value.to_string();
            }
            if tag == XML_TAG_RE_ROLLS {
                self.rerolls = value.parse().unwrap_or(0);
            }
            if tag == XML_TAG_FAN_FACTOR {
                self.fan_factor = value.parse().unwrap_or(0);
            }
            if tag == XML_TAG_APOTHECARIES {
                self.apothecaries = value.parse().unwrap_or(0);
            }
            if tag == XML_TAG_CHEERLEADERS {
                self.cheerleaders = value.parse().unwrap_or(0);
            }
            if tag == XML_TAG_ASSISTANT_COACHES {
                self.assistant_coaches = value.parse().unwrap_or(0);
            }
            if tag == XML_TAG_TEAM_VALUE {
                self.team_value = value.parse().unwrap_or(0);
            }
            if tag == XML_TAG_TREASURY {
                self.treasury = value.parse().unwrap_or(0);
            }
            if tag == XML_TAG_DEDICATED_FANS {
                self.dedicated_fans = value.parse().unwrap_or(0);
            }
            // Java: `<division>`/`<baseIconPath>`/`<logo>` — administrative/cosmetic-only
            // fields with no field on this struct; discarded, same treatment as elsewhere.
            if tag == XML_TAG_RULE {
                // Java: `SpecialRule.from(pValue)`; an unknown rule is logged and dropped.
                if SpecialRule::from(value).is_some() {
                    self.special_rules.push(value.to_string());
                }
            }
        }
        complete
    }

    /// Rust-only substitute for Java's "parent holds the child" ownership pattern (see
    /// `Roster::end_child`): appends the fully-parsed player once its `</player>` completes.
    fn end_child(&mut self, tag: &str, child: Box<dyn IXmlReadable>) {
        if tag == crate::model::player::XML_TAG {
            if let Ok(player) = child.into_any().downcast::<Player>() {
                self.players.push(*player);
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
    use std::collections::HashSet;
    use crate::enums::{PlayerType, PlayerGender};
    use crate::model::player::Player;

    fn empty_team() -> Team {
        Team {
            id: "t1".into(),
            name: "Humans".into(),
            race: "Human".into(),
            roster_id: "human".into(),
            coach: "Coach".into(),
            rerolls: 3,
            apothecaries: 1,
            bribes: 0,
            master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0,
            assistant_coaches: 0,
            fan_factor: 5,
            dedicated_fans: 5,
            team_value: 1_000_000,
            treasury: 0,
            special_rules: vec![],
            players: vec![],
            vampire_lord: false, roster: None, base_icon_path: None, logo_url: None,
            necromancer: false,
        }
    }

    #[test]
    fn serde_round_trip() {
        let t = empty_team();
        let json = serde_json::to_string(&t).unwrap();
        let back: Team = serde_json::from_str(&json).unwrap();
        assert_eq!(t.id, back.id);
        assert_eq!(t.rerolls, back.rerolls);
    }

    #[test]
    fn player_lookup() {
        let mut t = empty_team();
        t.players.push(Player {
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
            starting_skills: vec![],
            extra_skills: vec![],
            temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0,
            stat_injuries: vec![],
            current_spps: 0,
            career_spps: 0,
            race: None,
            is_big_guy: false,
            ..Default::default()
});
        assert!(t.player("p1").is_some());
        assert!(t.player("p2").is_none());
        assert!(t.has_player("p1"));
    }

    #[test]
    fn player_by_nr_finds_correct_player() {
        let mut t = empty_team();
        t.players.push(Player {
            id: "p1".into(),
            name: "Joe".into(),
            nr: 5,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6,
            strength: 3,
            agility: 3,
            passing: 4,
            armour: 8,
            starting_skills: vec![],
            extra_skills: vec![],
            temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0,
            stat_injuries: vec![],
            current_spps: 0,
            career_spps: 0,
            race: None,
            is_big_guy: false,
            ..Default::default()
});
        assert_eq!(t.player_by_nr(5).map(|p| p.id.as_str()), Some("p1"));
        assert!(t.player_by_nr(99).is_none());
    }

    #[test]
    fn player_mut_modifies_player_in_place() {
        let mut t = empty_team();
        t.players.push(Player {
            id: "p1".into(), name: "Joe".into(), nr: 1,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour: 8, starting_skills: vec![], extra_skills: vec![],
            temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0,
            career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
});
        t.player_mut("p1").unwrap().current_spps = 10;
        assert_eq!(t.player("p1").unwrap().current_spps, 10);
    }

    #[test]
    fn has_player_false_for_unknown_id() {
        let t = empty_team();
        assert!(!t.has_player("nobody"));
    }

    #[test]
    fn multiple_players_looked_up_independently() {
        let mut t = empty_team();
        for nr in 1..=3i32 {
            t.players.push(Player {
                id: format!("p{}", nr), name: format!("Player{}", nr), nr,
                position_id: "lineman".into(), player_type: PlayerType::Regular,
                gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
                passing: 4, armour: 8, starting_skills: vec![], extra_skills: vec![],
                temporary_skills: vec![], used_skills: HashSet::new(),
                niggling_injuries: 0, stat_injuries: vec![], current_spps: 0,
                career_spps: 0, race: None,
                is_big_guy: false,
                ..Default::default()
});
        }
        assert_eq!(t.players.len(), 3);
        assert_eq!(t.player("p2").map(|p| p.nr), Some(2));
        assert!(t.player("p4").is_none());
    }

    #[test]
    fn players_list_reflects_added_players() {
        let mut t = empty_team();
        assert_eq!(t.players.len(), 0);
        t.players.push(Player {
            id: "p1".into(),
            name: "Alice".into(),
            nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6,
            strength: 3,
            agility: 3,
            passing: 4,
            armour: 8,
            starting_skills: vec![],
            extra_skills: vec![],
            temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0,
            stat_injuries: vec![],
            current_spps: 0,
            career_spps: 0,
            race: None,
            is_big_guy: false,
            ..Default::default()
});
        assert_eq!(t.players.len(), 1);
    }

    /// Excerpt of the real ffb-java `ffb-server/teams/team_amazon_Kalimar_167.xml` fixture.
    const AMAZON_TEAM_XML: &str = r#"
        <team id="teamAmazonKalimar">
            <coach>Kalimar</coach>
            <name>Kalimar's Amazons</name>
            <race>Amazon</race>
            <rosterId>amazon.lrb6</rosterId>
            <reRolls>4</reRolls>
            <fanFactor>10</fanFactor>
            <apothecaries>2</apothecaries>
            <currentTeamValue>100</currentTeamValue>
            <player nr="1" id="teamAmazonKalimar1">
                <name>Blitzer1</name>
                <gender>female</gender>
                <positionId>amazon.blitzer</positionId>
                <skillList></skillList>
            </player>
            <player nr="9" id="teamAmazonKalimar9">
                <name>Linewoman1</name>
                <gender>female</gender>
                <positionId>amazon.linewoman</positionId>
                <skillList>
                    <skill>Fend</skill>
                </skillList>
            </player>
        </team>
    "#;

    fn default_team() -> Team {
        Team {
            id: String::new(), name: String::new(), race: String::new(),
            roster_id: String::new(), coach: String::new(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0,
            team_value: 0, treasury: 0, special_rules: vec![], players: vec![],
            vampire_lord: false, roster: None, base_icon_path: None, logo_url: None, necromancer: false,
        }
    }

    #[test]
    fn parses_real_team_fixture_root_fields() {
        let parsed = crate::xml::XmlHandler::parse(None, AMAZON_TEAM_XML, Box::new(default_team()));
        let team = parsed.as_any().downcast_ref::<Team>().unwrap();
        assert_eq!(team.id, "teamAmazonKalimar");
        assert_eq!(team.coach, "Kalimar");
        assert_eq!(team.name, "Kalimar's Amazons");
        assert_eq!(team.race, "Amazon");
        assert_eq!(team.roster_id, "amazon.lrb6");
        assert_eq!(team.rerolls, 4);
        assert_eq!(team.fan_factor, 10);
        assert_eq!(team.apothecaries, 2);
        assert_eq!(team.team_value, 100);
    }

    #[test]
    fn parses_real_team_fixture_players() {
        let parsed = crate::xml::XmlHandler::parse(None, AMAZON_TEAM_XML, Box::new(default_team()));
        let team = parsed.as_any().downcast_ref::<Team>().unwrap();
        assert_eq!(team.players.len(), 2);

        let blitzer = team.player("teamAmazonKalimar1").unwrap();
        assert_eq!(blitzer.nr, 1);
        assert_eq!(blitzer.name, "Blitzer1");
        assert_eq!(blitzer.position_id, "amazon.blitzer");
        assert_eq!(blitzer.gender, crate::enums::PlayerGender::Female);
        assert!(blitzer.extra_skills.is_empty());

        let linewoman = team.player("teamAmazonKalimar9").unwrap();
        assert_eq!(linewoman.nr, 9);
        assert_eq!(linewoman.position_id, "amazon.linewoman");
        assert_eq!(linewoman.extra_skills.len(), 1);
        assert_eq!(linewoman.extra_skills[0].skill_id, crate::model::skill_def::SkillId::Fend);
    }

    #[test]
    fn update_roster_resolves_player_stats_from_positions() {
        let parsed = crate::xml::XmlHandler::parse(None, AMAZON_TEAM_XML, Box::new(default_team()));
        let mut team = *parsed.into_any().downcast::<Team>().unwrap();

        let mut blitzer_pos = crate::model::roster_position::RosterPosition::default();
        blitzer_pos.id = "amazon.blitzer".into();
        blitzer_pos.movement = 6;
        blitzer_pos.strength = 3;
        blitzer_pos.agility = 3;
        blitzer_pos.passing = 5;
        blitzer_pos.armour = 7;

        let roster = Roster {
            id: "amazon.lrb6".into(), name: "Amazon".into(), race: String::new(),
            reroll_cost: 50_000, max_rerolls: 8, positions: vec![blitzer_pos],
            special_rules: vec![], necromancer: false, keywords: vec![],
            raised_position_id: None, ..Default::default()
        };

        team.update_roster(&roster);

        assert_eq!(team.roster_id, "amazon.lrb6");
        assert_eq!(team.race, "Amazon");
        let blitzer = team.player("teamAmazonKalimar1").unwrap();
        assert_eq!(blitzer.movement, 6);
        assert_eq!(blitzer.armour, 7);
    }

    // ── Java JSON (BACKLOG §O): expectations read off Team/Roster/RosterPosition/RosterPlayer.java ──

    fn json_position() -> serde_json::Value {
        serde_json::json!({
            "positionId": "p1", "positionName": "Lineman", "shorthand": "L", "displayName": null,
            "playerType": "regular", "playerGender": null, "quantity": 16, "movement": 6,
            "strength": 3, "agility": 3, "passing": 4, "armour": 9, "cost": 50000, "race": null,
            "undead": false, "thrall": false, "teamWithPositionId": null, "nameGenerator": null,
            "replacesPosition": "", "urlPortrait": null, "urlIconSet": "i/1.png", "nrOfIcons": 2,
            "skillCategoriesNormal": ["General", "general"], "skillCategoriesDouble": ["mutations"],
            "skillArray": ["Block", "I'll Carry You", "Mighty Blow", "Block"],
            "skillValues": [null, "x", "1", "2"], "skillDisplayValues": ["B"],
            "keywords": ["lineman", "No Such Keyword"]
        })
    }

    fn json_player(nr: i32, name: &str) -> serde_json::Value {
        serde_json::json!({
            "playerKind": "rosterPlayer", "playerId": format!("id{nr}"), "playerNr": nr,
            "positionId": "p1", "playerName": name, "playerGender": "female", "playerType": "Regular",
            "movement": 6, "strength": 3, "agility": 3, "passing": 4, "armour": 9,
            "lastingInjuries": ["Smashed Knee (-MA)", "Broken Shoulder (-ST)"],
            "recoveringInjury": "Smashed Knee (-MA)",
            "urlPortrait": null, "urlIconSet": null, "nrOfIcons": 0, "positionIconIndex": -1,
            "skillArray": ["Dodge", "I'll Carry You", "Block"],
            "temporarySkillsMap": { "Blessed Statue of Nuffle": ["Pro", "Mighty Blow_1"] },
            "temporaryModifiersMap": { "Iron Man": ["AV-com.fumbbl.ffb.modifiers.TemporaryStatIncrementer"] },
            "temporaryPropertiesMap": { "Iron Man": [] },
            "skillValuesMap": { "Block": null, "I'll Carry You": "3" },
            "skillDisplayValuesMap": {},
            "usedSkills": ["Dodge"]
        })
    }

    fn json_team() -> serde_json::Value {
        serde_json::json!({
            "teamId": "t1", "teamName": "T", "coach": "C", "race": "Raw Race", "reRolls": 3,
            "apothecaries": 1, "cheerleaders": 0, "assistantCoaches": 0, "fanFactor": 0,
            "teamValue": 1000000, "treasury": 0, "baseIconPath": null, "logoUrl": null,
            "dedicatedFans": 2, "specialRules": ["SWARMING", "SWARMING", null],
            "playerArray": [json_player(2, "first"), json_player(1, "one"), json_player(2, "second")],
            "roster": {
                "rosterId": "r1", "rosterName": "Roster Name", "reRollCost": 50000, "maxReRolls": 8,
                "baseIconPath": null, "logoUrl": null, "raisedPositionId": null, "apothecary": true,
                "necromancer": false, "undead": false, "riotousPositionId": null, "nameGenerator": null,
                "maxBigGuys": 1, "positionArray": [json_position()], "keywords": ["Vampire Lord"]
            }
        })
    }

    #[test]
    fn json_team_follows_the_java_decode() {
        let rules = crate::enums::Rules::Bb2025;
        let mut team = Team::new();
        team.init_from(rules, &json_team()).unwrap();
        // updateRoster: race = roster NAME, roster id taken over; roster caches filled.
        assert_eq!(team.race, "Roster Name");
        assert_eq!(team.roster_id, "r1");
        assert!(team.vampire_lord);
        // fPlayerByNr: the second nr-2 player replaced the first.
        assert_eq!(team.players.len(), 2);
        assert_eq!(team.player_by_nr(2).unwrap().name, "second");
        // Set<SpecialRule>, null filtered; kept as the rule NAME the engine compares with
        // (SppMechanic's Brawlin' Brutes check missed the JSON constant name, BACKLOG §O O3).
        assert_eq!(team.special_rules, vec!["Swarming".to_string()]);
        assert_eq!(team.to_json_value(rules).unwrap()["specialRules"], serde_json::json!(["SWARMING"]));
        // Icon indexes: -1 -> findNextIconSetIndex, in nr order (nr 1 gets 0, nr 2 gets 1).
        assert_eq!(team.player_by_nr(1).unwrap().icon_set_index, 0);
        assert_eq!(team.player_by_nr(2).unwrap().icon_set_index, 1);

        let out = team.to_json_value(rules).unwrap();
        assert_eq!(out["race"], "Roster Name");
        let players = out["playerArray"].as_array().unwrap();
        assert_eq!(players.iter().map(|p| p["playerNr"].as_i64().unwrap()).collect::<Vec<_>>(), vec![1, 2]);
        let p = &players[0];
        // Unknown skill stays a null IN PLACE; unknown injury a null element.
        assert_eq!(p["skillArray"], serde_json::json!(["Dodge", null, "Block"]));
        assert_eq!(p["lastingInjuries"], serde_json::json!(["Smashed Knee (-MA)", null]));
        assert_eq!(p["recoveringInjury"], "Smashed Knee (-MA)");
        // A null key (unknown skill) is dropped from the values map; a null value is kept.
        assert_eq!(p["skillValuesMap"], serde_json::json!({ "Block": null }));
        assert_eq!(p["temporarySkillsMap"], serde_json::json!({ "Blessed Statue of Nuffle": ["Pro", "Mighty Blow_1"] }));
        assert_eq!(p["temporaryPropertiesMap"], serde_json::json!({ "Iron Man": [] }));
        assert!(p.get("playerStatus").is_none(), "a null playerStatus is not written");
        assert_eq!(p["playerType"], "Regular");
        // The temporary modifier reaches the engine's effective stats (AV 9 + 1).
        assert_eq!(team.player_by_nr(1).unwrap().armour_with_modifiers(), 10);

        let pos = &out["roster"]["positionArray"][0];
        // LinkedHashMap: the duplicate Block keeps its place with the LAST value; the unknown
        // skill is left out; display values only up to the array's length.
        assert_eq!(pos["skillArray"], serde_json::json!(["Block", "Mighty Blow"]));
        assert_eq!(pos["skillValues"], serde_json::json!(["2", "1"]));
        assert_eq!(pos["skillDisplayValues"], serde_json::json!(["B", null]));
        assert_eq!(pos["playerGender"], serde_json::Value::Null);
        assert_eq!(pos["playerType"], "Regular");
        assert_eq!(pos["skillCategoriesNormal"], serde_json::json!(["General"]));
        assert_eq!(pos["skillCategoriesDouble"], serde_json::json!(["Mutation"]));
        assert_eq!(pos["keywords"], serde_json::json!(["Lineman", "Unknown"]));
    }

    #[test]
    fn json_team_errors_where_java_throws() {
        let rules = crate::enums::Rules::Bb2025;
        let mut bad_rule = json_team();
        bad_rule["specialRules"] = serde_json::json!(["NOT_A_RULE"]);
        assert!(Team::new().init_from(rules, &bad_rule).is_err(), "SpecialRule.valueOf throws");
        let mut bad_modifier = json_team();
        bad_modifier["playerArray"][0]["temporaryModifiersMap"] = serde_json::json!({ "x": ["AV-no.such.Class"] });
        assert!(Team::new().init_from(rules, &bad_modifier).is_err(), "Class.forName throws");
        let mut no_apo = json_team();
        no_apo["roster"].as_object_mut().unwrap().remove("apothecary");
        assert!(Team::new().init_from(rules, &no_apo).is_err(), "null Boolean unboxed");
    }
}
