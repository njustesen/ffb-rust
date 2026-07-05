/// 1:1 translation of `com.fumbbl.ffb.server.marking.ApplyTo`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyTo {
    Own,
    Opponent,
    Both,
}

impl ApplyTo {
    pub fn applies_to_own(&self) -> bool {
        matches!(self, ApplyTo::Own | ApplyTo::Both)
    }

    pub fn applies_to_opponent(&self) -> bool {
        matches!(self, ApplyTo::Opponent | ApplyTo::Both)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn own_applies_to_own_only() {
        assert!(ApplyTo::Own.applies_to_own());
        assert!(!ApplyTo::Own.applies_to_opponent());
    }

    #[test]
    fn opponent_applies_to_opponent_only() {
        assert!(!ApplyTo::Opponent.applies_to_own());
        assert!(ApplyTo::Opponent.applies_to_opponent());
    }

    #[test]
    fn both_applies_to_both() {
        assert!(ApplyTo::Both.applies_to_own());
        assert!(ApplyTo::Both.applies_to_opponent());
    }
}
