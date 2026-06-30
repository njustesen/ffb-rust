use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.InjuryAttribute.
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InjuryAttribute {
    MA,
    ST,
    AG,
    AV,
    NI,
    PA,
}

impl InjuryAttribute {
    pub fn get_id(self) -> i32 {
        match self {
            InjuryAttribute::MA => 1,
            InjuryAttribute::ST => 2,
            InjuryAttribute::AG => 3,
            InjuryAttribute::AV => 4,
            InjuryAttribute::NI => 5,
            InjuryAttribute::PA => 6,
        }
    }

    pub fn get_name(self) -> &'static str {
        match self {
            InjuryAttribute::MA => "MA",
            InjuryAttribute::ST => "ST",
            InjuryAttribute::AG => "AG",
            InjuryAttribute::AV => "AV",
            InjuryAttribute::NI => "NI",
            InjuryAttribute::PA => "PA",
        }
    }

    pub fn for_name(raw_name: &str) -> Option<InjuryAttribute> {
        let name = raw_name.trim_start_matches(['+', '-']);
        match name.to_ascii_uppercase().as_str() {
            "MA" => Some(InjuryAttribute::MA),
            "ST" => Some(InjuryAttribute::ST),
            "AG" => Some(InjuryAttribute::AG),
            "AV" => Some(InjuryAttribute::AV),
            "NI" => Some(InjuryAttribute::NI),
            "PA" => Some(InjuryAttribute::PA),
            _ => None,
        }
    }
}
