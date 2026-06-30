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
