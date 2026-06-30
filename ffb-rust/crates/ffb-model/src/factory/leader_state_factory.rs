use crate::enums::LeaderState;

/// 1:1 translation of com.fumbbl.ffb.factory.LeaderStateFactory.
pub struct LeaderStateFactory;

impl Default for LeaderStateFactory {
    fn default() -> Self { Self }
}

impl LeaderStateFactory {
    pub fn for_name(&self, name: &str) -> Option<LeaderState> {
        LeaderState::from_name(name)
    }

    pub fn initialize(&mut self) {}
}
