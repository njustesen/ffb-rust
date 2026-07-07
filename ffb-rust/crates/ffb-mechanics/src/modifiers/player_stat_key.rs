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

    #[test]
    fn all_variants_round_trip() {
        let variants = [PlayerStatKey::MA, PlayerStatKey::ST, PlayerStatKey::AG, PlayerStatKey::PA, PlayerStatKey::AV];
        for v in variants {
            let json = serde_json::to_string(&v).unwrap();
            let back: PlayerStatKey = serde_json::from_str(&json).unwrap();
            assert_eq!(back, v);
        }
    }

    #[test]
    fn equality_is_reflexive() {
        assert_eq!(PlayerStatKey::MA, PlayerStatKey::MA);
        assert_eq!(PlayerStatKey::PA, PlayerStatKey::PA);
    }

    #[test]
    fn hash_works_in_hashmap() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert(PlayerStatKey::MA, "movement");
        map.insert(PlayerStatKey::AG, "agility");
        assert_eq!(map[&PlayerStatKey::MA], "movement");
        assert_eq!(map[&PlayerStatKey::AG], "agility");
    }

    #[test]
    fn copy_semantics() {
        let a = PlayerStatKey::ST;
        let b = a; // copy
        assert_eq!(a, b);
    }
}
