use serde::{Deserialize, Serialize};
use crate::model::roster_position::RosterPosition;

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
}

impl Roster {
    pub fn position(&self, id: &str) -> Option<&RosterPosition> {
        self.positions.iter().find(|p| p.id == id)
    }

    pub fn non_star_positions(&self) -> impl Iterator<Item = &RosterPosition> {
        self.positions.iter().filter(|p| !p.is_star_player())
    }
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
        });
        assert!(r.position("lineman").is_some());
        assert!(r.position("blitzer").is_none());
    }
}
