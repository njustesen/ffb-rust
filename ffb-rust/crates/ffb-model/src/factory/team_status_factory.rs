use crate::enums::TeamStatus;

/// 1:1 translation of com.fumbbl.ffb.factory.TeamStatusFactory.
pub struct TeamStatusFactory;

impl Default for TeamStatusFactory {
    fn default() -> Self { TeamStatusFactory }
}

impl TeamStatusFactory {
    pub fn for_name(&self, name: &str) -> Option<TeamStatus> {
        TeamStatus::from_name(name)
    }

    pub fn initialize(&mut self) {}
}
