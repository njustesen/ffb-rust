use crate::model::ClientMode;

/// 1:1 translation of com.fumbbl.ffb.factory.ClientModeFactory.
pub struct ClientModeFactory;

impl Default for ClientModeFactory {
    fn default() -> Self { ClientModeFactory }
}

impl ClientModeFactory {
    pub fn for_name(&self, name: &str) -> Option<ClientMode> {
        ClientMode::for_name(name)
    }

    pub fn initialize(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_returns_known_mode() {
        assert_eq!(ClientModeFactory::default().for_name("player"), Some(ClientMode::PLAYER));
        assert_eq!(ClientModeFactory::default().for_name("spectator"), Some(ClientMode::SPECTATOR));
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(ClientModeFactory::default().for_name("invalid"), None);
    }
}
