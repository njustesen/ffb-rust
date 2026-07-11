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

    /// Java: `forArgument(String pArgument)`.
    pub fn for_argument(&self, argument: &str) -> Option<ClientMode> {
        [ClientMode::PLAYER, ClientMode::SPECTATOR, ClientMode::REPLAY]
            .into_iter()
            .find(|mode| mode.get_argument().eq_ignore_ascii_case(argument))
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

    #[test]
    fn initialize_does_not_panic() {
        let mut f = ClientModeFactory::default();
        f.initialize();
    }

    #[test]
    fn for_name_a_second_known_variant() {
        assert_eq!(ClientModeFactory::default().for_name("replay"), Some(ClientMode::REPLAY));
    }

    #[test]
    fn for_name_empty_string_returns_none() {
        assert_eq!(ClientModeFactory::default().for_name(""), None);
    }

    #[test]
    fn for_argument_returns_known_mode() {
        assert_eq!(ClientModeFactory::default().for_argument("-player"), Some(ClientMode::PLAYER));
        assert_eq!(ClientModeFactory::default().for_argument("-spectator"), Some(ClientMode::SPECTATOR));
        assert_eq!(ClientModeFactory::default().for_argument("-replay"), Some(ClientMode::REPLAY));
    }

    #[test]
    fn for_argument_is_case_insensitive() {
        assert_eq!(ClientModeFactory::default().for_argument("-PLAYER"), Some(ClientMode::PLAYER));
    }

    #[test]
    fn for_argument_unknown_returns_none() {
        assert_eq!(ClientModeFactory::default().for_argument("-bogus"), None);
    }
}
