use serde::{Deserialize, Serialize};
use crate::enums::PlayerStatKey;

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

    /// Java: `InjuryAttribute.forStatKey(PlayerStatKey)`.
    pub fn for_stat_key(key: PlayerStatKey) -> Option<InjuryAttribute> {
        match key {
            PlayerStatKey::Ma => Some(InjuryAttribute::MA),
            PlayerStatKey::St => Some(InjuryAttribute::ST),
            PlayerStatKey::Ag => Some(InjuryAttribute::AG),
            PlayerStatKey::Pa => Some(InjuryAttribute::PA),
            PlayerStatKey::Av => Some(InjuryAttribute::AV),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_round_trip() {
        assert_eq!(InjuryAttribute::for_name("MA"), Some(InjuryAttribute::MA));
        assert_eq!(InjuryAttribute::for_name("AG"), Some(InjuryAttribute::AG));
    }

    #[test]
    fn for_name_strips_prefix() {
        assert_eq!(InjuryAttribute::for_name("-MA"), Some(InjuryAttribute::MA));
        assert_eq!(InjuryAttribute::for_name("+AG"), Some(InjuryAttribute::AG));
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(InjuryAttribute::for_name("XX"), None);
    }

    #[test]
    fn get_id_is_unique_per_variant() {
        let ma_id = InjuryAttribute::MA.get_id();
        let ag_id = InjuryAttribute::AG.get_id();
        assert_ne!(ma_id, ag_id);
    }

    #[test]
    fn for_stat_key_ma_returns_ma() {
        assert_eq!(InjuryAttribute::for_stat_key(PlayerStatKey::Ma), Some(InjuryAttribute::MA));
    }

    #[test]
    fn get_name_av_is_av() {
        assert_eq!(InjuryAttribute::AV.get_name(), "AV");
    }

    #[test]
    fn for_name_case_insensitive() {
        assert_eq!(InjuryAttribute::for_name("ma"), Some(InjuryAttribute::MA));
    }
}
