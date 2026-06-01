use serde::{Deserialize, Serialize};
use crate::model::player::Player;

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
        });
        assert_eq!(t.player_by_nr(5).map(|p| p.id.as_str()), Some("p1"));
        assert!(t.player_by_nr(99).is_none());
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
        });
        assert_eq!(t.players.len(), 1);
    }
}
