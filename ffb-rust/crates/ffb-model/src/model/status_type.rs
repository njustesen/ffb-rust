use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.StatusType.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatusType {
    WAITING,
    REF,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variants_are_distinct() {
        assert_ne!(StatusType::WAITING, StatusType::REF);
    }

    #[test]
    fn serde_round_trip() {
        let s = serde_json::to_string(&StatusType::WAITING).unwrap();
        let d: StatusType = serde_json::from_str(&s).unwrap();
        assert_eq!(d, StatusType::WAITING);
    }
}
