use serde::{Deserialize, Serialize};
use crate::inducement::usage::Usage;

/// 1:1 translation of `com.fumbbl.ffb.inducement.Inducement`.
/// Represents one inducement entry: how many purchased and how many used.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inducement {
    /// Java: `fType.getName()` — name of the InducementType.
    pub type_id: String,
    /// Java: `fValue` — number purchased (slots available).
    pub value: i32,
    /// Java: `fUses` — number already used this game.
    pub uses: i32,
    /// Java: `InducementType.getUsages()` — which Usage categories this type belongs to.
    pub usages: Vec<Usage>,
}

impl Inducement {
    /// Java: `new Inducement(type, value)` — create with no uses yet.
    pub fn new(type_id: impl Into<String>, value: i32, usages: Vec<Usage>) -> Self {
        Inducement { type_id: type_id.into(), value, uses: 0, usages }
    }

    /// Java: `Inducement.getType().getName()`
    pub fn get_type_id(&self) -> &str {
        &self.type_id
    }

    /// Java: `Inducement.getValue()`
    pub fn get_value(&self) -> i32 {
        self.value
    }

    /// Java: `Inducement.getUses()`
    pub fn get_uses(&self) -> i32 {
        self.uses
    }

    /// Java: `Inducement.setUses(int)`
    pub fn set_uses(&mut self, uses: i32) {
        self.uses = uses;
    }

    /// Java: `Inducement.getUsesLeft()` — clamped to 0.
    pub fn get_uses_left(&self) -> i32 {
        (self.value - self.uses).max(0)
    }

    /// True if any of this inducement's usages match the given usage.
    pub fn has_usage(&self, usage: Usage) -> bool {
        self.usages.contains(&usage)
    }
}

impl Default for Inducement {
    fn default() -> Self {
        Inducement { type_id: String::new(), value: 0, uses: 0, usages: vec![] }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_with_zero_uses() {
        let i = Inducement::new("BRIBE", 2, vec![Usage::AVOID_BAN]);
        assert_eq!(i.get_uses(), 0);
        assert_eq!(i.get_uses_left(), 2);
    }

    #[test]
    fn uses_left_clamped() {
        let mut i = Inducement::new("BRIBE", 1, vec![Usage::AVOID_BAN]);
        i.set_uses(5);
        assert_eq!(i.get_uses_left(), 0);
    }

    #[test]
    fn has_usage_returns_true_for_matching() {
        let i = Inducement::new("BRIBE", 1, vec![Usage::AVOID_BAN, Usage::REROLL]);
        assert!(i.has_usage(Usage::AVOID_BAN));
        assert!(i.has_usage(Usage::REROLL));
        assert!(!i.has_usage(Usage::STAR));
    }

    #[test]
    fn serde_round_trip() {
        let i = Inducement::new("APO", 1, vec![Usage::APOTHECARY]);
        let s = serde_json::to_string(&i).unwrap();
        let back: Inducement = serde_json::from_str(&s).unwrap();
        assert_eq!(back.type_id, "APO");
        assert_eq!(back.value, 1);
        assert_eq!(back.usages, vec![Usage::APOTHECARY]);
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", Inducement::default()).is_empty());
    }

}
