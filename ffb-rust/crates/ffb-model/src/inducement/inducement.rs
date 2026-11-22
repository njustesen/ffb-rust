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

    /// Java: `Inducement.toJsonValue()` -- `inducementType` (`getName()`), `value`, `uses`.
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "inducementType": self.type_id,
            "value": self.value,
            "uses": self.uses,
        })
    }

    /// Java: `Inducement.initFrom(IFactorySource, JsonValue)` in a game of `rules`: the type
    /// through the ruleset's `InducementTypeFactory.forName`, `value`/`uses` as `JsonIntOption`
    /// (absent -> 0). A null type (absent, JSON null or unknown) is `Err`: Java would key the
    /// inducement by null, which the Rust map cannot hold (the harness drops unknown types from
    /// the replay JSON first, so this does not occur on the corpus).
    pub fn init_from(&mut self, rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::util::java_json::{get_enum_name, get_int, to_json_object};
        let obj = to_json_object(json)?;
        let name = get_enum_name(obj, "inducementType")?;
        let entry = name
            .and_then(|n| crate::factory::inducement_type_factory::for_name_in(rules, n))
            .ok_or_else(|| format!("Inducement.initFrom: inducementType {name:?} is null in {rules:?}"))?;
        self.type_id = entry.id;
        self.usages = entry.usages;
        self.value = get_int(obj, "value")?;
        self.uses = get_int(obj, "uses")?;
        Ok(())
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
    fn java_json_round_trip_uses_the_rulesets_type() {
        use crate::enums::Rules;
        let mut i = Inducement::default();
        i.init_from(Rules::Bb2025, &serde_json::json!({"inducementType": "TEAMMASCOT", "value": 1})).unwrap();
        assert_eq!(i.type_id, "teamMascot");
        assert!(i.has_usage(Usage::CONDITIONAL_REROLL));
        assert_eq!(
            i.to_json_value(),
            serde_json::json!({"inducementType": "teamMascot", "value": 1, "uses": 0})
        );
        // teamMascot is not a BB2020 type -> Java null type
        assert!(Inducement::default().init_from(Rules::Bb2020, &serde_json::json!({"inducementType": "teamMascot"})).is_err());
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

}
