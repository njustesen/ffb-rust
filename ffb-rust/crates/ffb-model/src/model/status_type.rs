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

    #[test]
    fn serde_round_trip_ref() {
        let s = serde_json::to_string(&StatusType::REF).unwrap();
        let d: StatusType = serde_json::from_str(&s).unwrap();
        assert_eq!(d, StatusType::REF);
    }

    #[test]
    fn copy_semantics() {
        let a = StatusType::WAITING;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn equality_is_reflexive() {
        assert_eq!(StatusType::WAITING, StatusType::WAITING);
        assert_eq!(StatusType::REF, StatusType::REF);
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", StatusType::WAITING).is_empty());
    }

}
