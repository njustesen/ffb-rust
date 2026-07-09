use std::collections::HashMap;
use crate::option::i_game_option::IGameOption;
use crate::option::game_option_abstract::GameOptionAbstract;

/// 1:1 translation of `com.fumbbl.ffb.option.GameOptionString`.
///
/// Overtime kickoff options.
pub const OVERTIME_KICK_OFF_ALL: &str = "all";
pub const OVERTIME_KICK_OFF_BLITZ: &str = "blitz";
pub const OVERTIME_KICK_OFF_SOLID_DEFENCE: &str = "solidDefence";
pub const OVERTIME_KICK_OFF_BLITZ_OR_SOLID_DEFENCE: &str = "blitzOrSolidDefence";
pub const OVERTIME_KICK_OFF_RANDOM_BLITZ_OR_SOLID_DEFENCE: &str = "randomBlitzOrSolidDefence";

/// Chainsaw turnover options.
pub const CHAINSAW_TURNOVER_NEVER: &str = "never";
pub const CHAINSAW_TURNOVER_KICKBACK: &str = "kickback";
pub const CHAINSAW_TURNOVER_KICKBACK_AV_BREAK_ONLY: &str = "kickbackAvBreak";
pub const CHAINSAW_TURNOVER_ALL_AV_BREAKS: &str = "allAvBreaks";
pub const CHAINSAW_TURNOVER_KICKBACK_ONLY: &str = "kickbackOnly";
pub const CHAINSAW_TURNOVER_ALWAYS: &str = "always";

#[derive(Debug, Clone)]
pub struct GameOptionString {
    base: GameOptionAbstract,
    /// Java: fDefault
    default: Option<String>,
    /// Java: fValue
    value: Option<String>,
    /// Java: fMessage — display message template.
    message: Option<String>,
    /// Java: messages — per-value display messages.
    messages: HashMap<String, String>,
}

impl GameOptionString {
    /// Java: `GameOptionString(GameOptionId pId)`
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            base: GameOptionAbstract::new(id),
            default: None,
            value: None,
            message: None,
            messages: HashMap::new(),
        }
    }

    /// Java: `setDefault(String pDefault)` — also sets value to default.
    pub fn set_default(&mut self, default: impl Into<String>) -> &mut Self {
        let s = default.into();
        self.default = Some(s.clone());
        self.value = Some(s);
        self
    }

    /// Java: `getValue()`
    pub fn get_value(&self) -> Option<&str> {
        self.value.as_deref()
    }

    /// Java: `setMessage(String pMessage)`
    pub fn set_message(&mut self, message: impl Into<String>) -> &mut Self {
        self.message = Some(message.into());
        self
    }

    /// Java: `addValueMessage(String value, String pMessage)`
    pub fn add_value_message(&mut self, value: impl Into<String>, msg: impl Into<String>) -> &mut Self {
        self.messages.insert(value.into(), msg.into());
        self
    }
}

impl IGameOption for GameOptionString {
    fn get_id(&self) -> &str {
        self.base.get_id()
    }

    fn get_value_as_string(&self) -> String {
        self.value.clone().unwrap_or_default()
    }

    fn set_value(&mut self, value: &str) {
        self.value = Some(value.to_string());
    }

    fn is_changed(&self) -> bool {
        let default = self.default.as_deref().unwrap_or("");
        GameOptionAbstract::is_changed(&self.get_value_as_string(), default)
    }

    fn get_display_message(&self) -> String {
        // Java: StringTool.bind(fMessage, param) where param comes from messages map
        let val = self.get_value_as_string();
        let param = self.messages.get(&val).map(|s| s.as_str()).unwrap_or(val.as_str());
        if let Some(msg) = &self.message {
            msg.replace("{0}", param)
        } else {
            param.to_string()
        }
    }
}

impl Default for GameOptionString {
    fn default() -> Self {
        Self::new("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::option::i_game_option::IGameOption;

    #[test]
    fn new_has_none_value() {
        let opt = GameOptionString::new("chainsawTurnover");
        assert!(opt.get_value().is_none());
    }

    #[test]
    fn set_value_from_string() {
        let mut opt = GameOptionString::new("chainsawTurnover");
        opt.set_value(CHAINSAW_TURNOVER_NEVER);
        assert_eq!(opt.get_value(), Some(CHAINSAW_TURNOVER_NEVER));
    }

    #[test]
    fn set_default_sets_value() {
        let mut opt = GameOptionString::new("overtime");
        opt.set_default(OVERTIME_KICK_OFF_ALL);
        assert_eq!(opt.get_value(), Some(OVERTIME_KICK_OFF_ALL));
        assert!(!opt.is_changed());
    }

    #[test]
    fn is_changed_after_update() {
        let mut opt = GameOptionString::new("overtime");
        opt.set_default(OVERTIME_KICK_OFF_ALL);
        opt.set_value(OVERTIME_KICK_OFF_BLITZ);
        assert!(opt.is_changed());
    }

    #[test]
    fn display_message_with_value_map() {
        let mut opt = GameOptionString::new("overtime");
        opt.set_message("Mode: {0}");
        opt.add_value_message(OVERTIME_KICK_OFF_ALL, "All events");
        opt.set_value(OVERTIME_KICK_OFF_ALL);
        assert_eq!(opt.get_display_message(), "Mode: All events");
    }
}
