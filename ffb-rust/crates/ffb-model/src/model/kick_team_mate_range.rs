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
    fn serde_round_trip() {
        for v in [KickTeamMateRange::LONG, KickTeamMateRange::MEDIUM, KickTeamMateRange::SHORT] {
            let json = serde_json::to_string(&v).unwrap();
            let back: KickTeamMateRange = serde_json::from_str(&json).unwrap();
            assert_eq!(v, back);
        }
    }

}
