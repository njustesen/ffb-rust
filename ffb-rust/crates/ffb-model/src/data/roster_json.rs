//! JSON deserialization structs that match the data/ directory JSON format.
//! These are separate from the model types to allow independent evolution.

use serde::Deserialize;

/// A skill entry in a position or star player's skills array.
/// Skills like Loner(4+) or Animosity(all) are serialized as objects.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum SkillEntry {
    Simple(String),
    WithValue { name: String, value: serde_json::Value },
}

impl SkillEntry {
    pub fn name(&self) -> &str {
        match self {
            SkillEntry::Simple(s) => s,
            SkillEntry::WithValue { name, .. } => name,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RosterJson {
    pub id: String,
    pub name: String,
    pub reroll_cost: i32,
    pub max_rerolls: i32,
    #[serde(default)]
    pub apothecary: bool,
    #[serde(default)]
    pub undead: bool,
    #[serde(default)]
    pub necromancer: bool,
    #[serde(default)]
    pub keywords: Vec<String>,
    pub positions: Vec<PositionJson>,
    #[serde(default)]
    pub special_rules: Vec<String>,
    #[serde(default)]
    pub raised_position_id: Option<String>,
}

impl RosterJson {
    /// Java: Roster.hasVampireLord() — checks roster-level keywords for "Vampire Lord".
    pub fn has_vampire_lord(&self) -> bool {
        self.keywords.iter().any(|k| k.eq_ignore_ascii_case("vampire lord"))
    }

    /// Java: Roster.hasNecromancer() — true for Necromantic Horror and Undead rosters.
    pub fn has_necromancer(&self) -> bool {
        self.necromancer
    }
}

#[derive(Debug, Deserialize)]
pub struct PositionJson {
    pub id: String,
    pub name: String,
    pub display_name: Option<String>,
    #[serde(rename = "type", default = "default_type")]
    pub player_type: String,
    pub quantity: i32,
    pub cost: i32,
    pub ma: i32,
    pub st: i32,
    pub ag: i32,
    pub pa: i32,
    pub av: i32,
    #[serde(default)]
    pub skills: Vec<SkillEntry>,
    #[serde(default)]
    pub skill_categories: SkillCategoriesJson,
    #[serde(default)]
    pub keywords: Vec<String>,
}

fn default_type() -> String {
    "Regular".to_string()
}

#[derive(Debug, Default, Deserialize)]
pub struct SkillCategoriesJson {
    #[serde(default)]
    pub normal: Vec<String>,
    #[serde(default)]
    pub double: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct StarPlayerJson {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(rename = "type", default = "default_type")]
    pub player_type: String,
    pub cost: i32,
    pub ma: i32,
    pub st: i32,
    pub ag: i32,
    pub pa: i32,
    pub av: i32,
    #[serde(default)]
    pub skills: Vec<SkillEntry>,
    #[serde(default)]
    pub special_rules: Vec<String>,
    #[serde(default)]
    pub available_for: Vec<String>,
    #[serde(default)]
    pub editions: Vec<String>,
    #[serde(default)]
    pub shorthand: Option<String>,
    #[serde(default)]
    pub quantity: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct StarPlayersJson {
    pub star_players: Vec<StarPlayerJson>,
}

/// A skill entry from the extracted skills JSON (just the class name).
#[derive(Debug, Deserialize)]
pub struct SkillJson {
    pub class_name: String,
}

#[derive(Debug, Deserialize)]
pub struct SkillsJson {
    #[serde(default)]
    pub edition: String,
    pub skills: Vec<SkillJson>,
}

#[derive(Debug, Deserialize)]
pub struct InducementJson {
    pub id: String,
    pub name: String,
    pub cost: i32,
    #[serde(default)]
    pub max_count: i32,
    #[serde(default)]
    pub usage: String,
    #[serde(default)]
    pub availability: String,
}

#[derive(Debug, Deserialize)]
pub struct InducementsJson {
    #[serde(default)]
    pub edition: String,
    pub inducements: Vec<InducementJson>,
}

#[derive(Debug, Deserialize)]
pub struct PrayerJson {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub roll: i32,
    #[serde(default)]
    pub duration: String,
}

#[derive(Debug, Deserialize)]
pub struct PrayersJson {
    #[serde(default)]
    pub edition: String,
    pub prayers: Vec<PrayerJson>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_roster(keywords: Vec<String>, necromancer: bool) -> RosterJson {
        RosterJson {
            id: "test".into(),
            name: "Test".into(),
            reroll_cost: 50000,
            max_rerolls: 8,
            apothecary: false,
            undead: false,
            necromancer,
            keywords,
            positions: vec![],
            special_rules: vec![],
            raised_position_id: None,
        }
    }

    #[test]
    fn has_vampire_lord_true_for_keyword() {
        let r = make_roster(vec!["Vampire Lord".into()], false);
        assert!(r.has_vampire_lord());
    }

    #[test]
    fn has_vampire_lord_case_insensitive() {
        let r = make_roster(vec!["vampire lord".into()], false);
        assert!(r.has_vampire_lord());
    }

    #[test]
    fn has_vampire_lord_false_without_keyword() {
        let r = make_roster(vec!["Undead".into()], false);
        assert!(!r.has_vampire_lord());
    }

    #[test]
    fn has_necromancer_true_when_set() {
        let r = make_roster(vec![], true);
        assert!(r.has_necromancer());
    }

    #[test]
    fn has_necromancer_false_when_unset() {
        let r = make_roster(vec![], false);
        assert!(!r.has_necromancer());
    }

    #[test]
    fn skill_entry_name_simple() {
        let e = SkillEntry::Simple("Dodge".into());
        assert_eq!(e.name(), "Dodge");
    }

    #[test]
    fn skill_entry_name_with_value() {
        let e = SkillEntry::WithValue { name: "Loner".into(), value: serde_json::Value::String("4+".into()) };
        assert_eq!(e.name(), "Loner");
    }
}
