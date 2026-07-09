use crate::option::i_game_option::IGameOption;
use crate::option::game_option_abstract::GameOptionAbstract;

/// 1:1 translation of `com.fumbbl.ffb.option.GameOptionBoolean`.
#[derive(Debug, Clone)]
pub struct GameOptionBoolean {
    base: GameOptionAbstract,
    /// Java: fDefault
    default: bool,
    /// Java: fValue
    value: bool,
    /// Java: fMessageTrue
    message_true: Option<String>,
    /// Java: fMessageFalse
    message_false: Option<String>,
}

impl GameOptionBoolean {
    /// Java: `GameOptionBoolean(GameOptionId pId)`
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            base: GameOptionAbstract::new(id),
            default: false,
            value: false,
            message_true: None,
            message_false: None,
        }
    }

    /// Java: `setDefault(boolean pDefault)` — also sets value to default.
    pub fn set_default(&mut self, default: bool) -> &mut Self {
        self.default = default;
        self.value = default;
        self
    }

    /// Java: `isEnabled()`
    pub fn is_enabled(&self) -> bool {
        self.value
    }

    /// Java: `setValue(boolean pValue)`
    pub fn set_bool_value(&mut self, value: bool) -> &mut Self {
        self.value = value;
        self
    }

    /// Java: `setMessageTrue(String pMessage)`
    pub fn set_message_true(&mut self, message: impl Into<String>) -> &mut Self {
        self.message_true = Some(message.into());
        self
    }

    /// Java: `setMessageFalse(String pMessage)`
    pub fn set_message_false(&mut self, message: impl Into<String>) -> &mut Self {
        self.message_false = Some(message.into());
        self
    }
}

impl IGameOption for GameOptionBoolean {
    fn get_id(&self) -> &str {
        self.base.get_id()
    }

    fn get_value_as_string(&self) -> String {
        self.value.to_string()
    }

    fn set_value(&mut self, value: &str) {
        // Java: if not provided or "0" → false; if "1" → true; else parseBoolean
        self.value = match value {
            "" | "0" => false,
            "1" => true,
            _ => value.eq_ignore_ascii_case("true"),
        };
    }

    fn is_changed(&self) -> bool {
        GameOptionAbstract::is_changed(&self.get_value_as_string(), &self.default.to_string())
    }

    fn get_display_message(&self) -> String {
        if self.value {
            self.message_true.clone().unwrap_or_default()
        } else {
            self.message_false.clone().unwrap_or_default()
        }
    }
}

impl Default for GameOptionBoolean {
    fn default() -> Self {
        Self::new("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::option::i_game_option::IGameOption;

    #[test]
    fn new_is_disabled() {
        let opt = GameOptionBoolean::new("testMode");
        assert!(!opt.is_enabled());
        assert_eq!(opt.get_value_as_string(), "false");
    }

    #[test]
    fn set_value_from_string() {
        let mut opt = GameOptionBoolean::new("testMode");
        opt.set_value("true");
        assert!(opt.is_enabled());
    }

    #[test]
    fn set_value_from_one() {
        let mut opt = GameOptionBoolean::new("testMode");
        opt.set_value("1");
        assert!(opt.is_enabled());
    }

    #[test]
    fn is_changed_after_set() {
        let mut opt = GameOptionBoolean::new("testMode");
        opt.set_default(false);
        opt.set_value("true");
        assert!(opt.is_changed());
    }

    #[test]
    fn display_message_true() {
        let mut opt = GameOptionBoolean::new("testMode");
        opt.set_message_true("Enabled");
        opt.set_message_false("Disabled");
        opt.set_bool_value(true);
        assert_eq!(opt.get_display_message(), "Enabled");
    }
}
