use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::factory::game_option_factory::GameOptionFactory;
use crate::option::game_option_id::GameOptionId as GameOptionEnum;
use crate::option::i_game_option::IGameOption;

/// Game option key (maps to Java's GameOptionId).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GameOptionId(pub String);

impl GameOptionId {
    pub fn new(s: impl Into<String>) -> Self {
        GameOptionId(s.into())
    }
}

/// All configuration options for a game (edition, house rules, etc.).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameOptions {
    options: HashMap<String, String>,
}

impl GameOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.options.insert(key.into(), value.into());
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.options.get(key).map(|s| s.as_str())
    }

    pub fn is_enabled(&self, key: &str) -> bool {
        matches!(self.get(key), Some("true") | Some("1") | Some("yes"))
    }

    pub fn get_int(&self, key: &str) -> Option<i32> {
        self.get(key)?.parse().ok()
    }

    /// Java: `GameOptions.getOptionWithDefault(GameOptionId pOptionId)`.
    ///
    /// Java stores fully materialized `IGameOption` instances in `fOptionById` and
    /// falls back to `fGameOptionFactory.createGameOption(pOptionId)` when the id was
    /// never explicitly added. This `GameOptions` stores raw key/value strings instead
    /// of `IGameOption` objects, so the equivalent here is: build the id's default
    /// `IGameOption` via `GameOptionFactory`, then overlay the stored string value (if
    /// any) on top of it — same net behaviour (stored value wins, default otherwise).
    pub fn get_option_with_default(&self, option_id: GameOptionEnum) -> Box<dyn IGameOption> {
        let factory = GameOptionFactory::new();
        let mut option = factory
            .create_game_option(option_id)
            .expect("every GameOptionId variant has a GameOptionFactory case");
        if let Some(stored) = self.get(option_id.get_name()) {
            option.set_value(stored);
        }
        option
    }

    /// Java: `GameOptions.addOption(IGameOption pOption)` -- store the option, then switch off
    /// the option it is mutually exclusive with (piling on armour/injury only, foul bonus
    /// in/outside a tacklezone). This struct stores each option as its `getValueAsString()`.
    pub fn add_option(&mut self, option: Option<Box<dyn IGameOption>>) {
        let Some(option) = option else { return };
        self.add_option_internal(option.as_ref());
        let exclusive = match GameOptionEnum::for_name(option.get_id()) {
            Some(GameOptionEnum::PILING_ON_ARMOR_ONLY) => Some(GameOptionEnum::PILING_ON_INJURY_ONLY),
            Some(GameOptionEnum::PILING_ON_INJURY_ONLY) => Some(GameOptionEnum::PILING_ON_ARMOR_ONLY),
            Some(GameOptionEnum::FOUL_BONUS) => Some(GameOptionEnum::FOUL_BONUS_OUTSIDE_TACKLEZONE),
            Some(GameOptionEnum::FOUL_BONUS_OUTSIDE_TACKLEZONE) => Some(GameOptionEnum::FOUL_BONUS),
            _ => None,
        };
        // ((GameOptionBoolean) pOption).isEnabled()
        if let Some(other_id) = exclusive {
            if option.get_value_as_string() == "true" {
                let mut other = self.get_option_with_default(other_id);
                if other.get_value_as_string() == "true" {
                    other.set_value("false");
                    self.add_option_internal(other.as_ref());
                }
            }
        }
    }

    /// Java: `addOptionInternal` -- `fOptionById.put(pOption.getId(), pOption)` (the model-change
    /// notification has no observer on this side).
    fn add_option_internal(&mut self, option: &dyn IGameOption) {
        self.options.insert(option.get_id().to_string(), option.get_value_as_string());
    }

    /// Java: `GameOptions.toJsonValue()` -- `{"gameOptionArray":[{"gameOptionId","gameOptionValue"}...]}`
    /// (`GameOptionAbstract.toJsonValue`). Java iterates a HashMap, so the array order is not
    /// model state; this one is sorted by id.
    pub fn to_json_value(&self) -> serde_json::Value {
        let mut keys: Vec<&String> = self.options.keys().collect();
        keys.sort();
        let option_array: Vec<serde_json::Value> = keys
            .into_iter()
            .map(|k| serde_json::json!({ "gameOptionId": k, "gameOptionValue": self.options[k] }))
            .collect();
        serde_json::json!({ "gameOptionArray": option_array })
    }

    /// Java: `GameOptions.initFrom(IFactorySource, JsonValue)` + `GameOptionFactory.fromJsonValue`:
    /// an id this engine does not know creates no option (`createGameOption(null)` = null, and
    /// `addOption(null)` is a no-op); a missing `rulesVersion` becomes BB2016.
    /// `Err` where Java would throw.
    pub fn init_from(&mut self, json: &serde_json::Value) -> Result<(), String> {
        let option_array = json
            .get("gameOptionArray")
            .and_then(|v| v.as_array())
            .ok_or("GameOptions.initFrom: gameOptionArray missing")?;
        let factory = GameOptionFactory::new();
        for element in option_array {
            let id = element.get("gameOptionId").and_then(|v| v.as_str()).and_then(GameOptionEnum::for_name);
            let Some(id) = id else { continue };
            let mut option = factory.create_game_option(id);
            if let Some(opt) = option.as_mut() {
                match element.get("gameOptionValue") {
                    Some(serde_json::Value::String(s)) => opt.set_value(s),
                    None | Some(serde_json::Value::Null) => {
                        return Err(format!("GameOptions.initFrom: null value for {} (not supported)", id.get_name()))
                    }
                    Some(other) => return Err(format!("GameOptions.initFrom: non-string value {other}")),
                }
            }
            self.add_option(option);
        }
        if !self.options.contains_key(GameOptionEnum::RULESVERSION.get_name()) {
            let mut rules = factory.create_game_option(GameOptionEnum::RULESVERSION);
            if let Some(r) = rules.as_mut() {
                r.set_value("BB2016");
            }
            self.add_option(rules);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_and_get() {
        let mut opts = GameOptions::new();
        opts.set("bribes", "true");
        assert_eq!(opts.get("bribes"), Some("true"));
        assert!(opts.is_enabled("bribes"));
        assert!(!opts.is_enabled("missing"));
    }

    #[test]
    fn get_option_with_default_returns_factory_default_when_unset() {
        let opts = GameOptions::new();
        let opt = opts.get_option_with_default(GameOptionEnum::TURNTIME);
        assert_eq!(opt.get_value_as_string(), "240");
    }

    #[test]
    fn get_option_with_default_returns_stored_value_when_set() {
        let mut opts = GameOptions::new();
        opts.set(GameOptionEnum::TURNTIME.get_name(), "120");
        let opt = opts.get_option_with_default(GameOptionEnum::TURNTIME);
        assert_eq!(opt.get_value_as_string(), "120");
    }

    #[test]
    fn get_option_with_default_boolean_uses_default_when_unset() {
        let opts = GameOptions::new();
        let opt = opts.get_option_with_default(GameOptionEnum::TEST_MODE);
        assert_eq!(opt.get_value_as_string(), "false");
    }

    #[test]
    fn serde_round_trip() {
        let mut opts = GameOptions::new();
        opts.set("maxRerolls", "4");
        let json = serde_json::to_string(&opts).unwrap();
        let back: GameOptions = serde_json::from_str(&json).unwrap();
        assert_eq!(opts.get("maxRerolls"), back.get("maxRerolls"));
    }

    #[test]
    fn get_int_parses_valid_integer() {
        let mut opts = GameOptions::new();
        opts.set("maxRerolls", "6");
        assert_eq!(opts.get_int("maxRerolls"), Some(6));
    }

    #[test]
    fn get_int_returns_none_for_missing_key() {
        let opts = GameOptions::new();
        assert_eq!(opts.get_int("nonexistent"), None);
    }

    #[test]
    fn get_int_returns_none_for_non_numeric_value() {
        let mut opts = GameOptions::new();
        opts.set("rulesVersion", "BB2020");
        assert_eq!(opts.get_int("rulesVersion"), None);
    }

    #[test]
    fn is_enabled_false_for_explicit_false_string() {
        let mut opts = GameOptions::new();
        opts.set("extraTime", "false");
        assert!(!opts.is_enabled("extraTime"));
    }

    #[test]
    fn init_from_normalises_values_drops_unknown_ids_and_defaults_rules_version() {
        // Java: GameOptionBoolean.setValue("1") -> true, GameOptionInt "007" -> 7, an unknown id
        // creates no option, no rulesVersion -> BB2016.
        let json = serde_json::json!({ "gameOptionArray": [
            { "gameOptionId": "testMode", "gameOptionValue": "1" },
            { "gameOptionId": "turntime", "gameOptionValue": "007" },
            { "gameOptionId": "someNewerServerOption", "gameOptionValue": "x" },
        ]});
        let mut opts = GameOptions::new();
        opts.init_from(&json).unwrap();
        assert_eq!(opts.get("testMode"), Some("true"));
        assert_eq!(opts.get("turntime"), Some("7"));
        assert_eq!(opts.get("someNewerServerOption"), None);
        assert_eq!(opts.get("rulesVersion"), Some("BB2016"));
        let back = opts.to_json_value();
        assert_eq!(back["gameOptionArray"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn add_option_switches_off_the_mutually_exclusive_option() {
        // Java GameOptions.addOption: enabling foulBonus disables foulBonusOutsideTacklezone.
        let json = serde_json::json!({ "gameOptionArray": [
            { "gameOptionId": "foulBonusOutsideTacklezone", "gameOptionValue": "true" },
            { "gameOptionId": "foulBonus", "gameOptionValue": "true" },
            { "gameOptionId": "rulesVersion", "gameOptionValue": "BB2025" },
        ]});
        let mut opts = GameOptions::new();
        opts.init_from(&json).unwrap();
        assert_eq!(opts.get("foulBonus"), Some("true"));
        assert_eq!(opts.get("foulBonusOutsideTacklezone"), Some("false"));
        assert_eq!(opts.get("rulesVersion"), Some("BB2025"));
    }

    #[test]
    fn overwrite_key_returns_new_value() {
        let mut opts = GameOptions::new();
        opts.set("rerolls", "3");
        opts.set("rerolls", "5");
        assert_eq!(opts.get("rerolls"), Some("5"));
    }
}
