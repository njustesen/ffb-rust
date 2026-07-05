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
            race: None, replaces_position: None,
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
            race: None, replaces_position: None,
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
            race: None, replaces_position: None,
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
}
