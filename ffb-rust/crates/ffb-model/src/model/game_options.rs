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
    fn overwrite_key_returns_new_value() {
        let mut opts = GameOptions::new();
        opts.set("rerolls", "3");
        opts.set("rerolls", "5");
        assert_eq!(opts.get("rerolls"), Some("5"));
    }
}
