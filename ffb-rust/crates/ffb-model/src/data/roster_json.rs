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
    pub positions: Vec<PositionJson>,
    #[serde(default)]
    pub special_rules: Vec<String>,
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
