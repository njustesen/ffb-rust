use crate::enums::ServerStatus;

/// 1:1 translation of com.fumbbl.ffb.factory.ServerStatusFactory.
pub struct ServerStatusFactory;

impl Default for ServerStatusFactory {
    fn default() -> Self { ServerStatusFactory }
}

impl ServerStatusFactory {
    pub fn for_name(&self, name: &str) -> Option<ServerStatus> {
        ServerStatus::from_name(name)
    }

    pub fn initialize(&mut self) {}
}
