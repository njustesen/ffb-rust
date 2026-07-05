/// 1:1 translation of `com.fumbbl.ffb.inducement.bb2016.CardHandlerKey`.
/// Identifies which CardHandler is responsible for a given card.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardHandlerKey {
    CHOP_BLOCK,
    CUSTARD_PIE,
    DISTRACT,
    FORCE_SHIELD,
    ILLEGAL_SUBSTITUTION,
    PIT_TRAP,
    RABBITS_FOOT,
    WITCH_BREW,
}

impl CardHandlerKey {
    pub fn get_name(self) -> &'static str {
        match self {
            CardHandlerKey::CHOP_BLOCK => "CHOP_BLOCK",
            CardHandlerKey::CUSTARD_PIE => "CUSTARD_PIE",
            CardHandlerKey::DISTRACT => "DISTRACT",
            CardHandlerKey::FORCE_SHIELD => "FORCE_SHIELD",
            CardHandlerKey::ILLEGAL_SUBSTITUTION => "ILLEGAL_SUBSTITUTION",
            CardHandlerKey::PIT_TRAP => "PIT_TRAP",
            CardHandlerKey::RABBITS_FOOT => "RABBITS_FOOT",
            CardHandlerKey::WITCH_BREW => "WITCH_BREW",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_variants_have_names() {
        let variants = [
            CardHandlerKey::CHOP_BLOCK,
            CardHandlerKey::CUSTARD_PIE,
            CardHandlerKey::DISTRACT,
            CardHandlerKey::FORCE_SHIELD,
            CardHandlerKey::ILLEGAL_SUBSTITUTION,
            CardHandlerKey::PIT_TRAP,
            CardHandlerKey::RABBITS_FOOT,
            CardHandlerKey::WITCH_BREW,
        ];
        for v in &variants {
            assert!(!v.get_name().is_empty());
        }
    }

    #[test]
    fn chop_block_has_correct_name() {
        assert_eq!(CardHandlerKey::CHOP_BLOCK.get_name(), "CHOP_BLOCK");
    }

    #[test]
    fn equality_holds() {
        assert_eq!(CardHandlerKey::RABBITS_FOOT, CardHandlerKey::RABBITS_FOOT);
        assert_ne!(CardHandlerKey::RABBITS_FOOT, CardHandlerKey::PIT_TRAP);
    }
}
