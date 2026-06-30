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
