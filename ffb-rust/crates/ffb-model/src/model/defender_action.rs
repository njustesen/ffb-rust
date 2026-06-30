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
