use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.model.KickTeamMateRange.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KickTeamMateRange {
    LONG,
    MEDIUM,
    SHORT,
}

impl KickTeamMateRange {
    pub fn get_name(self) -> &'static str {
        match self {
            KickTeamMateRange::LONG => "long",
            KickTeamMateRange::MEDIUM => "medium",
            KickTeamMateRange::SHORT => "short",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_name_returns_lowercase() {
        assert_eq!(KickTeamMateRange::LONG.get_name(), "long");
        assert_eq!(KickTeamMateRange::MEDIUM.get_name(), "medium");
        assert_eq!(KickTeamMateRange::SHORT.get_name(), "short");
    }

    #[test]
    fn variants_are_distinct() {
        assert_ne!(KickTeamMateRange::LONG, KickTeamMateRange::MEDIUM);
        assert_ne!(KickTeamMateRange::MEDIUM, KickTeamMateRange::SHORT);
        assert_ne!(KickTeamMateRange::LONG, KickTeamMateRange::SHORT);
    }

    #[test]
    fn serde_round_trip() {
        for v in [KickTeamMateRange::LONG, KickTeamMateRange::MEDIUM, KickTeamMateRange::SHORT] {
            let json = serde_json::to_string(&v).unwrap();
            let back: KickTeamMateRange = serde_json::from_str(&json).unwrap();
            assert_eq!(v, back);
        }
    }

    #[test]
    fn copy_semantics() {
        let a = KickTeamMateRange::LONG;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn debug_contains_variant_name() {
        assert!(format!("{:?}", KickTeamMateRange::MEDIUM).contains("MEDIUM"));
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", KickTeamMateRange::LONG).is_empty());
    }

}
