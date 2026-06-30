use crate::model::ConcedeGameStatus;

/// 1:1 translation of com.fumbbl.ffb.factory.ConcedeGameStatusFactory.
pub struct ConcedeGameStatusFactory;

impl Default for ConcedeGameStatusFactory {
    fn default() -> Self { ConcedeGameStatusFactory }
}

impl ConcedeGameStatusFactory {
    pub fn for_name(&self, name: &str) -> Option<ConcedeGameStatus> {
        ConcedeGameStatus::from_name(name)
    }

    pub fn initialize(&mut self) {}
}
