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
}
