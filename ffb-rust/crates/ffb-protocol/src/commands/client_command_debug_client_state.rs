/// 1:1 translation of com.fumbbl.ffb.net.commands.ClientCommandDebugClientState.
use ffb_model::enums::ClientStateId;

#[derive(Debug, Clone, Default)]
pub struct ClientCommandDebugClientState {
    /// Java: `fClientStateId`
    pub client_state_id: Option<ClientStateId>,
}

impl ClientCommandDebugClientState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Java: `getClientStateId()`
    pub fn get_client_state_id(&self) -> Option<ClientStateId> {
        self.client_state_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_no_state_id() {
        let cmd = ClientCommandDebugClientState::new();
        assert!(cmd.get_client_state_id().is_none());
    }

    #[test]
    fn stores_client_state_id() {
        let cmd = ClientCommandDebugClientState {
            client_state_id: Some(ClientStateId::Login),
        };
        assert_eq!(cmd.get_client_state_id(), Some(ClientStateId::Login));
    }

    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandDebugClientState::new()).is_empty());
    }
}
