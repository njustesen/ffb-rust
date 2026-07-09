//! Static data loaders using `include_str!` — zero runtime I/O, compiled into the binary.

use once_cell::sync::Lazy;
use crate::data::roster_json::{RosterJson, PositionJson, StarPlayersJson, SkillsJson, InducementsJson, PrayersJson, SkillEntry};
use crate::enums::{PlayerGender, PlayerType, Rules, SkillCategory};
use crate::model::{Roster, RosterPosition};
use crate::model::skill_def::{SkillId, SkillWithValue};

// ── Roster bundles ────────────────────────────────────────────────────────────

macro_rules! include_roster {
    ($edition:literal, $race:literal) => {
        include_str!(concat!(
            "../../../../data/rosters/",
            $edition,
            "/roster_",
            $race,
            ".json"
        ))
    };
}

#[allow(dead_code)]
const ROSTER_NAMES: &[&str] = &[
    "amazon", "chaos", "chaos_dwarf", "chaos_pact", "dark_elf", "dark_elf_league_fumbbl",
    "dwarf", "elf", "goblin", "halfling", "high_elf", "human", "khemri", "khemri_fumbbl",
    "lizardman", "necromantic", "nippon", "norse", "nurgle", "ogre", "orc", "renegades",
    "skaven", "slann", "slann_fumbbl", "undead", "underworld", "vampire", "wood_elf",
];

fn parse_roster(json: &str) -> RosterJson {
    serde_json::from_str(json).expect("roster JSON parse failed")
}

pub static BB2020_ROSTERS_JSON: Lazy<Vec<&'static str>> = Lazy::new(|| {
    vec![
        include_roster!("bb2020", "amazon"),
        include_roster!("bb2020", "chaos"),
        include_roster!("bb2020", "chaos_dwarf"),
        include_roster!("bb2020", "chaos_pact"),
        include_roster!("bb2020", "dark_elf"),
        include_roster!("bb2020", "dark_elf_league_fumbbl"),
        include_roster!("bb2020", "dwarf"),
        include_roster!("bb2020", "elf"),
        include_roster!("bb2020", "goblin"),
        include_roster!("bb2020", "halfling"),
        include_roster!("bb2020", "high_elf"),
        include_roster!("bb2020", "human"),
        include_roster!("bb2020", "khemri"),
        include_roster!("bb2020", "khemri_fumbbl"),
        include_roster!("bb2020", "lizardman"),
        include_roster!("bb2020", "necromantic"),
        include_roster!("bb2020", "nippon"),
        include_roster!("bb2020", "norse"),
        include_roster!("bb2020", "nurgle"),
        include_roster!("bb2020", "ogre"),
        include_roster!("bb2020", "orc"),
        include_roster!("bb2020", "renegades"),
        include_roster!("bb2020", "skaven"),
        include_roster!("bb2020", "slann"),
        include_roster!("bb2020", "slann_fumbbl"),
        include_roster!("bb2020", "undead"),
        include_roster!("bb2020", "underworld"),
        include_roster!("bb2020", "vampire"),
        include_roster!("bb2020", "wood_elf"),
    ]
});

pub static BB2016_ROSTERS_JSON: Lazy<Vec<&'static str>> = Lazy::new(|| {
    vec![
        include_roster!("bb2016", "amazon"),
        include_roster!("bb2016", "chaos"),
        include_roster!("bb2016", "chaos_dwarf"),
        include_roster!("bb2016", "chaos_pact"),
        include_roster!("bb2016", "dark_elf"),
        include_roster!("bb2016", "dark_elf_league_fumbbl"),
        include_roster!("bb2016", "dwarf"),
        include_roster!("bb2016", "elf"),
        include_roster!("bb2016", "goblin"),
        include_roster!("bb2016", "halfling"),
        include_roster!("bb2016", "high_elf"),
        include_roster!("bb2016", "human"),
        include_roster!("bb2016", "khemri"),
        include_roster!("bb2016", "khemri_fumbbl"),
        include_roster!("bb2016", "lizardman"),
        include_roster!("bb2016", "necromantic"),
        include_roster!("bb2016", "nippon"),
        include_roster!("bb2016", "norse"),
        include_roster!("bb2016", "nurgle"),
        include_roster!("bb2016", "ogre"),
        include_roster!("bb2016", "orc"),
        include_roster!("bb2016", "renegades"),
        include_roster!("bb2016", "skaven"),
        include_roster!("bb2016", "slann"),
        include_roster!("bb2016", "slann_fumbbl"),
        include_roster!("bb2016", "undead"),
        include_roster!("bb2016", "underworld"),
        include_roster!("bb2016", "vampire"),
        include_roster!("bb2016", "wood_elf"),
    ]
});

pub static BB2025_ROSTERS_JSON: Lazy<Vec<&'static str>> = Lazy::new(|| {
    vec![
        include_roster!("bb2025", "amazon"),
        include_roster!("bb2025", "chaos"),
        include_roster!("bb2025", "chaos_dwarf"),
        include_roster!("bb2025", "chaos_pact"),
        include_roster!("bb2025", "dark_elf"),
        include_roster!("bb2025", "dark_elf_league_fumbbl"),
        include_roster!("bb2025", "dwarf"),
        include_roster!("bb2025", "elf"),
        include_roster!("bb2025", "goblin"),
        include_roster!("bb2025", "halfling"),
        include_roster!("bb2025", "high_elf"),
        include_roster!("bb2025", "human"),
        include_roster!("bb2025", "khemri"),
        include_roster!("bb2025", "khemri_fumbbl"),
        include_roster!("bb2025", "lizardman"),
        include_roster!("bb2025", "necromantic"),
        include_roster!("bb2025", "nippon"),
        include_roster!("bb2025", "norse"),
        include_roster!("bb2025", "nurgle"),
        include_roster!("bb2025", "ogre"),
        include_roster!("bb2025", "orc"),
        include_roster!("bb2025", "renegades"),
        include_roster!("bb2025", "skaven"),
        include_roster!("bb2025", "slann"),
        include_roster!("bb2025", "slann_fumbbl"),
        include_roster!("bb2025", "undead"),
        include_roster!("bb2025", "underworld"),
        include_roster!("bb2025", "vampire"),
        include_roster!("bb2025", "wood_elf"),
    ]
});

// ── Star players ──────────────────────────────────────────────────────────────

const STAR_PLAYERS_JSON: &str =
    include_str!("../../../../data/star_players/all_editions.json");

pub static STAR_PLAYERS: Lazy<StarPlayersJson> = Lazy::new(|| {
    serde_json::from_str(STAR_PLAYERS_JSON).expect("star_players JSON parse failed")
});

// ── Skills ────────────────────────────────────────────────────────────────────

pub static BB2020_SKILLS: Lazy<SkillsJson> = Lazy::new(|| {
    serde_json::from_str(include_str!("../../../../data/skills/bb2020_skills.json"))
        .expect("bb2020_skills JSON parse failed")
});

pub static BB2016_SKILLS: Lazy<SkillsJson> = Lazy::new(|| {
    serde_json::from_str(include_str!("../../../../data/skills/bb2016_skills.json"))
        .expect("bb2016_skills JSON parse failed")
});

pub static BB2025_SKILLS: Lazy<SkillsJson> = Lazy::new(|| {
    serde_json::from_str(include_str!("../../../../data/skills/bb2025_skills.json"))
        .expect("bb2025_skills JSON parse failed")
});

pub static COMMON_SKILLS: Lazy<SkillsJson> = Lazy::new(|| {
    serde_json::from_str(include_str!("../../../../data/skills/common_skills.json"))
        .expect("common_skills JSON parse failed")
});

// ── Inducements ───────────────────────────────────────────────────────────────

pub static BB2020_INDUCEMENTS: Lazy<InducementsJson> = Lazy::new(|| {
    serde_json::from_str(include_str!("../../../../data/inducements/bb2020_inducements.json"))
        .expect("bb2020_inducements JSON parse failed")
});

pub static BB2016_INDUCEMENTS: Lazy<InducementsJson> = Lazy::new(|| {
    serde_json::from_str(include_str!("../../../../data/inducements/bb2016_inducements.json"))
        .expect("bb2016_inducements JSON parse failed")
});

pub static BB2025_INDUCEMENTS: Lazy<InducementsJson> = Lazy::new(|| {
    serde_json::from_str(include_str!("../../../../data/inducements/bb2025_inducements.json"))
        .expect("bb2025_inducements JSON parse failed")
});

// ── Prayers ───────────────────────────────────────────────────────────────────

pub static BB2020_PRAYERS: Lazy<PrayersJson> = Lazy::new(|| {
    serde_json::from_str(include_str!("../../../../data/prayers/bb2020_prayers.json"))
        .expect("bb2020_prayers JSON parse failed")
});

pub static BB2025_PRAYERS: Lazy<PrayersJson> = Lazy::new(|| {
    serde_json::from_str(include_str!("../../../../data/prayers/bb2025_prayers.json"))
        .expect("bb2025_prayers JSON parse failed")
});

// ── Parsed roster accessors ───────────────────────────────────────────────────

/// Java: `Team.getRoster().getPositionById(positionId)` — combined lookup from static data.
///
/// Searches the edition-appropriate roster list for a roster matching `roster_id`, then
/// returns a clone of the position entry matching `position_id`.
pub fn find_position(roster_id: &str, position_id: &str, rules: Rules) -> Option<PositionJson> {
    let rosters = match rules {
        Rules::Bb2016 | Rules::Common => bb2016_rosters(),
        Rules::Bb2020 => bb2020_rosters(),
        Rules::Bb2025 => bb2025_rosters(),
    };
    rosters.into_iter()
        .find(|r| r.id == roster_id)
        .and_then(|r| r.positions.into_iter().find(|p| p.id == position_id))
}

/// Finds the roster matching `roster_id` for the given rules edition and converts it to a `Roster`.
///
/// Returns `None` if no roster with that id exists in the static data.
pub fn find_roster(roster_id: &str, rules: Rules) -> Option<Roster> {
    let rosters = match rules {
        Rules::Bb2016 | Rules::Common => bb2016_rosters(),
        Rules::Bb2020 => bb2020_rosters(),
        Rules::Bb2025 => bb2025_rosters(),
    };
    rosters.into_iter()
        .find(|r| r.id == roster_id)
        .map(|r| roster_json_to_roster(&r))
}

/// Converts a `RosterJson` (deserialized from data/) into a `Roster` model struct.
pub fn roster_json_to_roster(rj: &RosterJson) -> Roster {
    let positions = rj.positions.iter()
        .map(|p| position_json_to_roster_position(p, &rj.id, rj.undead))
        .collect();
    Roster {
        id: rj.id.clone(),
        name: rj.name.clone(),
        race: rj.name.clone(),
        reroll_cost: rj.reroll_cost,
        max_rerolls: rj.max_rerolls,
        positions,
        special_rules: rj.special_rules.clone(),
        necromancer: rj.necromancer,
        keywords: rj.keywords.clone(),
    }
}

/// Converts a `PositionJson` to a `RosterPosition`.
pub fn position_json_to_roster_position(pos: &PositionJson, roster_id: &str, is_undead: bool) -> RosterPosition {
    let skills: Vec<SkillWithValue> = pos.skills.iter()
        .filter_map(skill_entry_to_skill_with_value)
        .collect();
    let cats_normal: Vec<SkillCategory> = pos.skill_categories.normal.iter()
        .filter_map(|s| SkillCategory::from_name(s))
        .collect();
    let cats_double: Vec<SkillCategory> = pos.skill_categories.double.iter()
        .filter_map(|s| SkillCategory::from_name(s))
        .collect();
    let player_type = PlayerType::from_name(&pos.player_type).unwrap_or(PlayerType::Regular);
    let is_big_guy = player_type == PlayerType::BigGuy
        || pos.keywords.iter().any(|k| k == "Big Guy");
    RosterPosition {
        id: pos.id.clone(),
        name: pos.name.clone(),
        display_name: pos.display_name.clone(),
        shorthand: None,
        player_type,
        gender: PlayerGender::Male,
        quantity: pos.quantity,
        cost: pos.cost,
        movement: pos.ma,
        strength: pos.st,
        agility: pos.ag,
        passing: pos.pa,
        armour: pos.av,
        skills,
        skill_categories_normal: cats_normal,
        skill_categories_double: cats_double,
        keywords: pos.keywords.clone(),
        is_big_guy,
        is_undead,
        is_thrall: false,
        race: Some(roster_id.to_string()),
        replaces_position: None,
    }
}

fn skill_entry_to_skill_with_value(entry: &SkillEntry) -> Option<SkillWithValue> {
    let skill_id = SkillId::from_class_name(entry.name())?;
    let value = match entry {
        SkillEntry::WithValue { value, .. } => Some(value.to_string()),
        SkillEntry::Simple(_) => None,
    };
    Some(SkillWithValue { skill_id, value })
}

pub fn bb2020_rosters() -> Vec<RosterJson> {
    BB2020_ROSTERS_JSON.iter().map(|s| parse_roster(s)).collect()
}

pub fn bb2016_rosters() -> Vec<RosterJson> {
    BB2016_ROSTERS_JSON.iter().map(|s| parse_roster(s)).collect()
}

pub fn bb2025_rosters() -> Vec<RosterJson> {
    BB2025_ROSTERS_JSON.iter().map(|s| parse_roster(s)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bb2020_rosters_load() {
        let rosters = bb2020_rosters();
        assert_eq!(rosters.len(), 29, "expected 29 BB2020 rosters");
        let human = rosters.iter().find(|r| r.name == "Human").unwrap();
        assert!(human.reroll_cost > 0);
        assert!(!human.positions.is_empty());
    }

    #[test]
    fn star_players_load() {
        let _ = &*STAR_PLAYERS;
    }

    #[test]
    fn skills_load() {
        let _ = &*BB2020_SKILLS;
        let _ = &*COMMON_SKILLS;
    }

    #[test]
    fn bb2016_rosters_load_all() {
        let rosters = bb2016_rosters();
        assert_eq!(rosters.len(), 29, "expected 29 BB2016 rosters");
        // Every roster must have at least one position defined.
        for r in &rosters {
            assert!(!r.positions.is_empty(), "roster '{}' has no positions", r.name);
        }
    }

    #[test]
    fn bb2025_rosters_load_all() {
        let rosters = bb2025_rosters();
        assert_eq!(rosters.len(), 29, "expected 29 BB2025 rosters");
        // Every roster must have a positive reroll cost.
        for r in &rosters {
            assert!(r.reroll_cost > 0, "roster '{}' has zero reroll cost", r.name);
        }
    }

    #[test]
    fn find_roster_returns_human_for_bb2025() {
        use crate::enums::Rules;
        let roster = find_roster("human.lrb6", Rules::Bb2025).expect("human.lrb6 roster should exist");
        assert_eq!(roster.id, "human.lrb6");
        assert!(!roster.positions.is_empty());
        assert!(roster.reroll_cost > 0);
    }

    #[test]
    fn find_roster_returns_none_for_unknown_id() {
        use crate::enums::Rules;
        assert!(find_roster("not_a_real_race", Rules::Bb2025).is_none());
    }

    #[test]
    fn find_roster_positions_have_stats() {
        use crate::enums::Rules;
        let roster = find_roster("human.lrb6", Rules::Bb2025).unwrap();
        for pos in &roster.positions {
            assert!(pos.movement > 0, "position '{}' has 0 movement", pos.name);
            assert!(pos.armour > 0, "position '{}' has 0 armour", pos.name);
        }
    }

    #[test]
    fn roster_json_to_roster_converts_player_type() {
        use crate::enums::{PlayerType, Rules};
        let roster = find_roster("human.lrb6", Rules::Bb2016).unwrap();
        // All human positions should be Regular or BigGuy, never Star/Irregular
        for pos in &roster.positions {
            assert!(
                pos.player_type == PlayerType::Regular || pos.player_type == PlayerType::BigGuy,
                "position '{}' has unexpected type {:?}",
                pos.name, pos.player_type
            );
        }
    }
}
