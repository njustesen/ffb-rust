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
}
