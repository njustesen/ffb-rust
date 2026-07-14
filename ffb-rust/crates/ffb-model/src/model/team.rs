use std::any::Any;
use serde::{Deserialize, Serialize};
use crate::model::game::Game;
use crate::model::player::Player;
use crate::model::roster::Roster;
use crate::xml::{IXmlReadable, XmlAttributes};
use crate::xml::util_xml::get_string_attribute;

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
}

impl Team {
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
                self.special_rules.push(value.to_string());
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
            vampire_lord: false,
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
    fn resource_fields_are_accessible() {
        let t = empty_team();
        assert_eq!(t.rerolls, 3);
        assert_eq!(t.apothecaries, 1);
        assert_eq!(t.fan_factor, 5);
        assert_eq!(t.team_value, 1_000_000);
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
            vampire_lord: false, necromancer: false,
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
            raised_position_id: None,
        };

        team.update_roster(&roster);

        assert_eq!(team.roster_id, "amazon.lrb6");
        assert_eq!(team.race, "Amazon");
        let blitzer = team.player("teamAmazonKalimar1").unwrap();
        assert_eq!(blitzer.movement, 6);
        assert_eq!(blitzer.armour, 7);
    }
}
