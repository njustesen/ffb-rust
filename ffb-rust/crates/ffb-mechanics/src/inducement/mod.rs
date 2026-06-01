use serde::{Deserialize, Serialize};
use ffb_model::enums::Rules;

/// A parsed inducement record from the JSON data files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InducementDef {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub name_singular: String,
    pub cost: u32,
    pub max_count: u32,
    #[serde(default)]
    pub usage: String,
    #[serde(default)]
    pub availability: Option<String>,
}

/// A team's purchased inducement (type + remaining uses).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inducement {
    pub id: String,
    pub uses_remaining: u32,
}

impl Inducement {
    pub fn new(id: impl Into<String>, uses: u32) -> Self {
        Inducement { id: id.into(), uses_remaining: uses }
    }

    pub fn is_used_up(&self) -> bool {
        self.uses_remaining == 0
    }

    pub fn use_one(&mut self) {
        if self.uses_remaining > 0 {
            self.uses_remaining -= 1;
        }
    }
}

/// The team's full set of purchased inducements for a game.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InducementSet {
    pub items: Vec<Inducement>,
}

impl InducementSet {
    pub fn add(&mut self, id: impl Into<String>, uses: u32) {
        self.items.push(Inducement::new(id, uses));
    }

    pub fn find_mut(&mut self, id: &str) -> Option<&mut Inducement> {
        self.items.iter_mut().find(|i| i.id == id)
    }

    pub fn has_available(&self, id: &str) -> bool {
        self.items.iter().any(|i| i.id == id && !i.is_used_up())
    }

    pub fn count_available(&self, id: &str) -> u32 {
        self.items.iter()
            .filter(|i| i.id == id)
            .map(|i| i.uses_remaining)
            .sum()
    }
}

/// Whether a bribe inducement can be used at the point of a foul referee check.
pub fn can_use_bribe(set: &InducementSet, _rules: Rules) -> bool {
    set.has_available("bribes")
}

/// Whether the Halfling Master Chef event has been purchased.
pub fn has_master_chef(set: &InducementSet) -> bool {
    set.has_available("halflingMasterChef")
}

/// Bloodweiser Keg: bonus to KO recovery roll.
/// Returns +1 per keg purchased (stacking up to 3).
pub fn bloodweiser_keg_bonus(set: &InducementSet) -> i32 {
    set.count_available("bloodweiserKegs") as i32
}

/// Brawler's Kegs (BB2025 rename of Bloodweiser Kegs).
pub fn brawlers_kegs_bonus(set: &InducementSet) -> i32 {
    (set.count_available("bloodweiserKegs") + set.count_available("brawlersKegs")) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inducement_use_tracks_remaining() {
        let mut ind = Inducement::new("bribes", 3);
        assert!(!ind.is_used_up());
        ind.use_one();
        assert_eq!(ind.uses_remaining, 2);
        ind.use_one();
        ind.use_one();
        assert!(ind.is_used_up());
        ind.use_one(); // no underflow
        assert_eq!(ind.uses_remaining, 0);
    }

    #[test]
    fn inducement_set_has_available() {
        let mut set = InducementSet::default();
        assert!(!set.has_available("bribes"));
        set.add("bribes", 2);
        assert!(set.has_available("bribes"));
    }

    #[test]
    fn bloodweiser_bonus_stacks() {
        let mut set = InducementSet::default();
        set.add("bloodweiserKegs", 3);
        assert_eq!(bloodweiser_keg_bonus(&set), 3);
    }

    #[test]
    fn can_use_bribe_requires_remaining() {
        let mut set = InducementSet::default();
        assert!(!can_use_bribe(&set, Rules::Bb2020));
        set.add("bribes", 1);
        assert!(can_use_bribe(&set, Rules::Bb2020));
        set.find_mut("bribes").unwrap().use_one();
        assert!(!can_use_bribe(&set, Rules::Bb2020));
    }
}
