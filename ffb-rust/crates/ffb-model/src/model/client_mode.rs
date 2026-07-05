use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.ClientMode.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClientMode {
    PLAYER,
    SPECTATOR,
    REPLAY,
}

impl ClientMode {
    pub fn get_name(self) -> &'static str {
        match self {
            ClientMode::PLAYER => "player",
            ClientMode::SPECTATOR => "spectator",
            ClientMode::REPLAY => "replay",
        }
    }

    pub fn get_argument(self) -> &'static str {
        match self {
            ClientMode::PLAYER => "-player",
            ClientMode::SPECTATOR => "-spectator",
            ClientMode::REPLAY => "-replay",
        }
    }

    pub fn for_name(name: &str) -> Option<Self> {
        match name {
            "player" => Some(ClientMode::PLAYER),
            "spectator" => Some(ClientMode::SPECTATOR),
            "replay" => Some(ClientMode::REPLAY),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_player() {
        assert_eq!(ClientMode::for_name("player"), Some(ClientMode::PLAYER));
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(ClientMode::for_name("admin"), None);
    }

    #[test]
    fn get_name_round_trip() {
        for &mode in &[ClientMode::PLAYER, ClientMode::SPECTATOR, ClientMode::REPLAY] {
            assert_eq!(ClientMode::for_name(mode.get_name()), Some(mode));
        }
    }
}
