use crate::option::i_game_option::IGameOption;
use crate::option::game_option_abstract::GameOptionAbstract;

/// 1:1 translation of `com.fumbbl.ffb.option.GameOptionInt`.
#[derive(Debug, Clone)]
pub struct GameOptionInt {
    base: GameOptionAbstract,
    /// Java: fDefault
    default: i32,
    /// Java: fValue
    value: i32,
    /// Java: fMessage — display message template (may use {0} placeholder).
    message: Option<String>,
}

impl GameOptionInt {
    /// Java: `GameOptionInt(GameOptionId pId)`
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            base: GameOptionAbstract::new(id),
            default: 0,
            value: 0,
            message: None,
        }
    }

    /// Java: `setDefault(int pDefault)` — also sets value to default.
    pub fn set_default(&mut self, default: i32) -> &mut Self {
        self.default = default;
        self.value = default;
        self
    }

    /// Java: `getValue()`
    pub fn get_value(&self) -> i32 {
        self.value
    }

    /// Java: `setValue(int pValue)`
    pub fn set_int_value(&mut self, value: i32) -> &mut Self {
        self.value = value;
        self
    }

    /// Java: `setMessage(String pMessage)`
    pub fn set_message(&mut self, message: impl Into<String>) -> &mut Self {
        self.message = Some(message.into());
        self
    }
}

impl IGameOption for GameOptionInt {
    fn get_id(&self) -> &str {
        self.base.get_id()
    }

    fn get_value_as_string(&self) -> String {
        self.value.to_string()
    }

    fn set_value(&mut self, value: &str) {
        // Java: if provided parse as integer; else 0
        self.value = if value.is_empty() {
            0
        } else {
            value.parse().unwrap_or(0)
        };
    }

    fn is_changed(&self) -> bool {
        GameOptionAbstract::is_changed(&self.get_value_as_string(), &self.default.to_string())
    }

    fn get_display_message(&self) -> String {
        // Java: StringTool.bind(fMessage, StringTool.formatThousands(getValue()))
        // Simplified: replace {0} with the value.
        if let Some(msg) = &self.message {
            msg.replace("{0}", &self.value.to_string())
        } else {
            self.value.to_string()
        }
    }
}

impl Default for GameOptionInt {
    fn default() -> Self {
        Self::new("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::option::i_game_option::IGameOption;

    #[test]
    fn new_has_zero_value() {
        let opt = GameOptionInt::new("turntime");
        assert_eq!(opt.get_value(), 0);
    }

    #[test]
    fn set_value_from_string() {
        let mut opt = GameOptionInt::new("turntime");
        opt.set_value("120");
        assert_eq!(opt.get_value(), 120);
    }

    #[test]
    fn set_default_sets_value_too() {
        let mut opt = GameOptionInt::new("turntime");
        opt.set_default(60);
        assert_eq!(opt.get_value(), 60);
        assert!(!opt.is_changed());
    }

    #[test]
    fn is_changed_after_setting_non_default() {
        let mut opt = GameOptionInt::new("turntime");
        opt.set_default(60);
        opt.set_int_value(120);
        assert!(opt.is_changed());
    }

    #[test]
    fn display_message_with_template() {
        let mut opt = GameOptionInt::new("turntime");
        opt.set_message("Turn time: {0}s");
        opt.set_int_value(90);
        assert_eq!(opt.get_display_message(), "Turn time: 90s");
    }
}
