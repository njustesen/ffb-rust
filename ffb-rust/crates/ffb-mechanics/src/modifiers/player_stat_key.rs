use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.modifiers.PlayerStatKey.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlayerStatKey {
    MA,
    ST,
    AG,
    PA,
    AV,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variants_distinct() {
        assert_ne!(PlayerStatKey::AG, PlayerStatKey::AV);
        assert_ne!(PlayerStatKey::MA, PlayerStatKey::ST);
    }

    #[test]
    fn serde_round_trip() {
        let v = PlayerStatKey::AG;
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(serde_json::from_str::<PlayerStatKey>(&json).unwrap(), v);
    }
}
