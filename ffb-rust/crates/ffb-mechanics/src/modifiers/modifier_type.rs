use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.modifiers.ModifierType.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModifierType {
    DEPENDS_ON_SUM_OF_OTHERS,
    DISTURBING_PRESENCE,
    DIVING_TACKLE,
    PREHENSILE_TAIL,
    REGULAR,
    TACKLEZONE,
    STAT_BASED,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variants_distinct() {
        assert_ne!(ModifierType::REGULAR, ModifierType::TACKLEZONE);
        assert_ne!(ModifierType::DISTURBING_PRESENCE, ModifierType::DIVING_TACKLE);
    }

    #[test]
    fn serde_round_trip() {
        let v = ModifierType::DISTURBING_PRESENCE;
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(serde_json::from_str::<ModifierType>(&json).unwrap(), v);
    }

    #[test]
    fn all_variants_serialize_and_deserialize() {
        let variants = [
            ModifierType::DEPENDS_ON_SUM_OF_OTHERS,
            ModifierType::DISTURBING_PRESENCE,
            ModifierType::DIVING_TACKLE,
            ModifierType::PREHENSILE_TAIL,
            ModifierType::REGULAR,
            ModifierType::TACKLEZONE,
            ModifierType::STAT_BASED,
        ];
        for v in variants {
            let json = serde_json::to_string(&v).unwrap();
            let back: ModifierType = serde_json::from_str(&json).unwrap();
            assert_eq!(back, v);
        }
    }

    #[test]
    fn equality_is_reflexive() {
        assert_eq!(ModifierType::REGULAR, ModifierType::REGULAR);
        assert_eq!(ModifierType::TACKLEZONE, ModifierType::TACKLEZONE);
        assert_eq!(ModifierType::STAT_BASED, ModifierType::STAT_BASED);
    }

    #[test]
    fn copy_semantics() {
        let a = ModifierType::TACKLEZONE;
        let b = a; // copy
        assert_eq!(a, b);
    }

    #[test]
    fn hash_works_in_hashmap() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert(ModifierType::REGULAR, 1);
        map.insert(ModifierType::TACKLEZONE, 2);
        assert_eq!(map[&ModifierType::REGULAR], 1);
        assert_eq!(map[&ModifierType::TACKLEZONE], 2);
    }
}
