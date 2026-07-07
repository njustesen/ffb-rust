use serde::{Deserialize, Serialize};

/// Edition of the Blood Bowl rules in effect for a game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Rules {
    Common,
    Bb2016,
    Bb2020,
    Bb2025,
}

impl Rules {
    pub fn name(self) -> &'static str {
        match self {
            Rules::Common => "COMMON",
            Rules::Bb2016 => "BB2016",
            Rules::Bb2020 => "BB2020",
            Rules::Bb2025 => "BB2025",
        }
    }

    fn parent(self) -> Option<Rules> {
        match self {
            Rules::Common => None,
            Rules::Bb2016 | Rules::Bb2020 | Rules::Bb2025 => Some(Rules::Common),
        }
    }

    /// True if `self` is `other` or extends `other` (walks the parent chain).
    pub fn is_or_extends(self, other: Rules) -> bool {
        if self == other {
            return true;
        }
        let mut current = self.parent();
        while let Some(p) = current {
            if p == other {
                return true;
            }
            current = p.parent();
        }
        false
    }

    pub fn hierarchy_level(self) -> u32 {
        let mut level = 0;
        let mut current = self.parent();
        while let Some(p) = current {
            level += 1;
            current = p.parent();
        }
        level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bb2020_extends_common() {
        assert!(Rules::Bb2020.is_or_extends(Rules::Common));
        assert!(Rules::Bb2020.is_or_extends(Rules::Bb2020));
        assert!(!Rules::Bb2020.is_or_extends(Rules::Bb2016));
    }

    #[test]
    fn common_does_not_extend_edition() {
        assert!(!Rules::Common.is_or_extends(Rules::Bb2020));
    }

    #[test]
    fn hierarchy_levels() {
        assert_eq!(Rules::Common.hierarchy_level(), 0);
        assert_eq!(Rules::Bb2016.hierarchy_level(), 1);
    }

    #[test]
    fn serde_round_trip() {
        let r = Rules::Bb2025;
        let json = serde_json::to_string(&r).unwrap();
        let back: Rules = serde_json::from_str(&json).unwrap();
        assert_eq!(r, back);
    }

    #[test]
    fn bb2025_extends_common() {
        assert!(Rules::Bb2025.is_or_extends(Rules::Common));
        assert!(!Rules::Bb2025.is_or_extends(Rules::Bb2016));
    }

    #[test]
    fn name_values() {
        assert_eq!(Rules::Bb2016.name(), "BB2016");
        assert_eq!(Rules::Bb2025.name(), "BB2025");
        assert_eq!(Rules::Common.name(), "COMMON");
    }
}
