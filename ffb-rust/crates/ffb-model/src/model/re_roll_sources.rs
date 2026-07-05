use std::collections::HashMap;
use crate::enums::ReRollSource;

/// 1:1 translation of com.fumbbl.ffb.ReRollSources.
/// Java uses reflection to populate the map; Rust enumerates all constants directly.
pub struct ReRollSources {
    values: HashMap<String, ReRollSource>,
}

impl ReRollSources {
    pub fn new() -> Self {
        let all = Self::all_sources();
        let mut values = HashMap::new();
        for source in all {
            values.insert(source.name.to_lowercase(), source);
        }
        ReRollSources { values }
    }

    pub fn values(&self) -> &HashMap<String, ReRollSource> {
        &self.values
    }

    pub fn for_name(&self, name: &str) -> Option<&ReRollSource> {
        self.values.get(&name.to_lowercase())
    }

    fn all_sources() -> Vec<ReRollSource> {
        vec![
            ReRollSource::new("Team ReRoll"),
            ReRollSource::new("Brilliant Coaching ReRoll"),
            ReRollSource::new("Dodge"),
            ReRollSource::new("Pro"),
            ReRollSource::new("Sure Feet"),
            ReRollSource::new("Sure Hands"),
            ReRollSource::new("Catch"),
            ReRollSource::new("Pass"),
            ReRollSource::new("Winnings"),
            ReRollSource::new("Loner"),
            ReRollSource::new("Leader"),
            ReRollSource::new("Monstrous Mouth"),
            ReRollSource::new("Brawler"),
            ReRollSource::new("Bribery and Corruption"),
            ReRollSource::new("Blind Rage"),
            ReRollSource::with_priority("The Ballista", 2),
            ReRollSource::new("Mesmerising Dance"),
            ReRollSource::new("Mesmerizing Dance"),
            ReRollSource::new("Lord of Chaos"),
            ReRollSource::new("Consummate Professional"),
            ReRollSource::new("Pump up the Crowd"),
            ReRollSource::new("Star of the Show"),
            ReRollSource::new("Whirling Dervish"),
            ReRollSource::new("Thinking Man's Troll"),
            ReRollSource::new("Halfling Luck"),
            ReRollSource::new("Bounding Leap"),
            ReRollSource::new("Unstoppable Momentum"),
            ReRollSource::new("Savage Blow"),
            ReRollSource::new("Team Mascot"),
            ReRollSource::new("Mascot TRR"),
            ReRollSource::with_superior("Pro Mascot", ReRollSource::new("Pro")),
            ReRollSource::with_superior("Pro Mascot TRR", ReRollSource::new("Pro")),
            ReRollSource::with_superior("Pro TRR", ReRollSource::new("Pro")),
            ReRollSource::new("Swoop"),
            ReRollSource::new("Hatred"),
            ReRollSource::new("Working in Tandem"),
            ReRollSource::new("Woodland Fury"),
            ReRollSource::new("Kick"),
        ]
    }
}

impl Default for ReRollSources {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn for_name_finds_known_source() {
        let r = ReRollSources::new();
        assert!(r.for_name("Pro").is_some());
        assert_eq!(r.for_name("pro").unwrap().name, "Pro");
    }
    #[test]
    fn for_name_unknown_returns_none() {
        assert!(ReRollSources::new().for_name("NOT_VALID").is_none());
    }
    #[test]
    fn values_is_non_empty() {
        assert!(!ReRollSources::new().values().is_empty());
    }
}
