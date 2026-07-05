use crate::enums::ClientStateId;

/// 1:1 translation of com.fumbbl.ffb.factory.ClientStateIdFactory.
pub struct ClientStateIdFactory;

impl Default for ClientStateIdFactory {
    fn default() -> Self { ClientStateIdFactory }
}

impl ClientStateIdFactory {
    pub fn for_name(&self, name: &str) -> Option<ClientStateId> {
        ClientStateId::from_name(name)
    }

    pub fn initialize(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_returns_known_id() {
        assert_eq!(ClientStateIdFactory::default().for_name("login"), Some(ClientStateId::Login));
        assert_eq!(ClientStateIdFactory::default().for_name("move"), Some(ClientStateId::Move));
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(ClientStateIdFactory::default().for_name("invalid"), None);
    }
}
