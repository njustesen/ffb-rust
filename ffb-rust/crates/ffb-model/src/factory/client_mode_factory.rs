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
