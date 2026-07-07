/// 1:1 translation of `com.fumbbl.ffb.inducement.bb2020.CardHandlerKey`.
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
    fn force_shield_has_correct_name() {
        assert_eq!(CardHandlerKey::FORCE_SHIELD.get_name(), "FORCE_SHIELD");
    }

    #[test]
    fn equality_holds() {
        assert_eq!(CardHandlerKey::WITCH_BREW, CardHandlerKey::WITCH_BREW);
        assert_ne!(CardHandlerKey::WITCH_BREW, CardHandlerKey::DISTRACT);
    }

    #[test]
    fn specific_variant_names_are_correct() {
        assert_eq!(CardHandlerKey::CHOP_BLOCK.get_name(), "CHOP_BLOCK");
        assert_eq!(CardHandlerKey::RABBITS_FOOT.get_name(), "RABBITS_FOOT");
        assert_eq!(CardHandlerKey::ILLEGAL_SUBSTITUTION.get_name(), "ILLEGAL_SUBSTITUTION");
    }

    #[test]
    fn copy_produces_independent_value() {
        let original = CardHandlerKey::PIT_TRAP;
        let copied = original;
        assert_eq!(original, copied);
        assert_eq!(copied.get_name(), "PIT_TRAP");
    }
}
