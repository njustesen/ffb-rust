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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_returns_known_status() {
        assert_eq!(ServerStatusFactory::default().for_name("Game In Use"), Some(ServerStatus::ErrorGameInUse));
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(ServerStatusFactory::default().for_name("invalid"), None);
    }
}
