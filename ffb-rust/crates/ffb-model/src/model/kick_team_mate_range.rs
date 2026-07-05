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
}
