/// 1:1 translation of `com.fumbbl.ffb.option.IGameOption`.
///
/// Interface for a single configurable game option.
pub trait IGameOption {
    /// Java: `getId()` — returns the option's key (GameOptionId name).
    fn get_id(&self) -> &str;

    /// Java: `getValueAsString()` — serializes the current value to a string.
    fn get_value_as_string(&self) -> String;

    /// Java: `setValue(String)` — set value from a string representation.
    fn set_value(&mut self, value: &str);

    /// Java: `isChanged()` — true when the value differs from the default.
    fn is_changed(&self) -> bool;

    /// Java: `getDisplayMessage()` — human-readable description of the current value.
    fn get_display_message(&self) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SimpleOption {
        id: String,
        value: String,
        default: String,
    }

    impl SimpleOption {
        fn new(id: &str, default: &str) -> Self {
            Self { id: id.into(), value: default.into(), default: default.into() }
        }
    }

    impl IGameOption for SimpleOption {
        fn get_id(&self) -> &str { &self.id }
        fn get_value_as_string(&self) -> String { self.value.clone() }
        fn set_value(&mut self, v: &str) { self.value = v.into(); }
        fn is_changed(&self) -> bool { self.value != self.default }
        fn get_display_message(&self) -> String { format!("{}={}", self.id, self.value) }
    }

    #[test]
    fn new_option_is_not_changed() {
        let opt = SimpleOption::new("testMode", "false");
        assert!(!opt.is_changed());
    }

    #[test]
    fn set_value_marks_changed() {
        let mut opt = SimpleOption::new("testMode", "false");
        opt.set_value("true");
        assert!(opt.is_changed());
        assert_eq!(opt.get_value_as_string(), "true");
    }
}
