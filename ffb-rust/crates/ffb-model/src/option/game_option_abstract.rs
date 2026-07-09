/// 1:1 translation of `com.fumbbl.ffb.option.GameOptionAbstract`.
///
/// Base struct shared by all concrete game option types.
/// In Java this is an abstract class; in Rust we provide a plain struct
/// holding the common fields, plus a helper for `isChanged()`.
#[derive(Debug, Clone)]
pub struct GameOptionAbstract {
    /// Java: fId — the option's GameOptionId key (name string).
    pub id: String,
}

impl GameOptionAbstract {
    /// Java: `GameOptionAbstract(GameOptionId pId)`
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }

    /// Java: `getId()`
    pub fn get_id(&self) -> &str {
        &self.id
    }

    /// Java: `isChanged()` — compares value string to default string.
    pub fn is_changed(value_as_string: &str, default_as_string: &str) -> bool {
        // Java: !StringTool.print(getDefaultAsString()).equals(getValueAsString())
        // StringTool.print() returns "" for null, otherwise the string itself.
        let default = if default_as_string.is_empty() { "" } else { default_as_string };
        value_as_string != default
    }
}

impl Default for GameOptionAbstract {
    fn default() -> Self {
        Self::new("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_has_correct_id() {
        let opt = GameOptionAbstract::new("testMode");
        assert_eq!(opt.get_id(), "testMode");
    }

    #[test]
    fn is_changed_false_when_same() {
        assert!(!GameOptionAbstract::is_changed("false", "false"));
    }

    #[test]
    fn is_changed_true_when_different() {
        assert!(GameOptionAbstract::is_changed("true", "false"));
    }

    #[test]
    fn default_has_empty_id() {
        let opt = GameOptionAbstract::default();
        assert_eq!(opt.get_id(), "");
    }
}
