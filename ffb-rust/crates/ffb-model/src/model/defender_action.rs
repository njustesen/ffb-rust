use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.DefenderAction.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DefenderAction {
    DUMP_OFF,
}

impl DefenderAction {
    pub fn get_id(self) -> i32 {
        match self { DefenderAction::DUMP_OFF => 1 }
    }

    pub fn get_name(self) -> &'static str {
        match self { DefenderAction::DUMP_OFF => "dumpOff" }
    }

    pub fn get_title(self) -> &'static str {
        match self { DefenderAction::DUMP_OFF => "Dump Off" }
    }

    pub fn get_description(self) -> &'static str {
        match self { DefenderAction::DUMP_OFF => "dump off the ball" }
    }

    pub fn from_id(id: i32) -> Option<Self> {
        match id { 1 => Some(DefenderAction::DUMP_OFF), _ => None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dump_off_get_id_is_1() {
        assert_eq!(DefenderAction::DUMP_OFF.get_id(), 1);
        assert_eq!(DefenderAction::DUMP_OFF.get_name(), "dumpOff");
    }

    #[test]
    fn from_id_round_trips() {
        assert_eq!(DefenderAction::from_id(1), Some(DefenderAction::DUMP_OFF));
        assert_eq!(DefenderAction::from_id(0), None);
    }

    #[test]
    fn get_title_and_description() {
        assert_eq!(DefenderAction::DUMP_OFF.get_title(), "Dump Off");
        assert!(!DefenderAction::DUMP_OFF.get_description().is_empty());
    }

    #[test]
    fn serde_round_trip() {
        let s = serde_json::to_string(&DefenderAction::DUMP_OFF).unwrap();
        let back: DefenderAction = serde_json::from_str(&s).unwrap();
        assert_eq!(back, DefenderAction::DUMP_OFF);
    }

    #[test]
    fn from_id_negative_returns_none() {
        assert_eq!(DefenderAction::from_id(-1), None);
        assert_eq!(DefenderAction::from_id(99), None);
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", DefenderAction::DUMP_OFF).is_empty());
    }

}
