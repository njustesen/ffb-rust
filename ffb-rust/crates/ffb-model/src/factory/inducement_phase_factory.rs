use crate::enums::InducementPhase;

/// 1:1 translation of com.fumbbl.ffb.factory.InducementPhaseFactory.
pub struct InducementPhaseFactory;

impl Default for InducementPhaseFactory {
    fn default() -> Self { InducementPhaseFactory }
}

impl InducementPhaseFactory {
    pub fn for_name(&self, name: &str) -> Option<InducementPhase> {
        InducementPhase::from_name(name)
    }

    pub fn initialize(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_returns_known_phase() {
        assert_eq!(
            InducementPhaseFactory::default().for_name("endOfOpponentTurn"),
            Some(InducementPhase::EndOfOpponentTurn)
        );
        assert_eq!(
            InducementPhaseFactory::default().for_name("startOfOwnTurn"),
            Some(InducementPhase::StartOfOwnTurn)
        );
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(InducementPhaseFactory::default().for_name("invalid"), None);
    }

    #[test]
    fn initialize_does_not_panic() {
        let mut f = InducementPhaseFactory::default();
        f.initialize();
    }

    #[test]
    fn for_name_a_second_known_variant() {
        assert_eq!(
            InducementPhaseFactory::default().for_name("endOfOwnTurn"),
            Some(InducementPhase::EndOfOwnTurn)
        );
    }

    #[test]
    fn for_name_empty_string_returns_none() {
        assert_eq!(InducementPhaseFactory::default().for_name(""), None);
    }
}
