use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.PositionChoiceMode.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PositionChoiceMode {
    RAISE_DEAD,
}

impl PositionChoiceMode {
    pub fn get_name(self) -> &'static str {
        match self { PositionChoiceMode::RAISE_DEAD => "raiseDead" }
    }

    pub fn get_dialog_header(self) -> &'static str {
        match self { PositionChoiceMode::RAISE_DEAD => "Select position for raised player" }
    }

    pub fn get_status_title(self) -> &'static str {
        match self { PositionChoiceMode::RAISE_DEAD => "Raise Dead" }
    }

    pub fn get_status_message(self) -> &'static str {
        match self { PositionChoiceMode::RAISE_DEAD => "Waiting for coach to choose position." }
    }

    pub fn for_name(name: &str) -> Option<Self> {
        match name { "raiseDead" => Some(PositionChoiceMode::RAISE_DEAD), _ => None }
    }

    /// Java `Enum.name()`: the constant name.
    pub fn name(self) -> &'static str {
        match self { PositionChoiceMode::RAISE_DEAD => "RAISE_DEAD" }
    }

    /// Java `PositionChoiceMode.valueOf(String)`: the EXACT constant name; null -> NullPointerException,
    /// anything else -> IllegalArgumentException (both `Err`).
    pub fn value_of(name: Option<&str>) -> Result<Self, String> {
        match name {
            Some("RAISE_DEAD") => Ok(PositionChoiceMode::RAISE_DEAD),
            Some(other) => Err(format!("No enum constant com.fumbbl.ffb.PositionChoiceMode.{other}")),
            None => Err("PositionChoiceMode.valueOf: Name is null".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_round_trips() {
        assert_eq!(PositionChoiceMode::for_name("raiseDead"), Some(PositionChoiceMode::RAISE_DEAD));
        assert_eq!(PositionChoiceMode::for_name("invalid"), None);
    }

    #[test]
    fn serde_round_trip() {
        let s = serde_json::to_string(&PositionChoiceMode::RAISE_DEAD).unwrap();
        let back: PositionChoiceMode = serde_json::from_str(&s).unwrap();
        assert_eq!(back, PositionChoiceMode::RAISE_DEAD);
    }

}
