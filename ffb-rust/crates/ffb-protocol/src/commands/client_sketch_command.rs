use serde::{Deserialize, Serialize};

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientSketchCommand`.
/// Abstract base for sketch-related client commands (add, remove, modify, clear).
/// Java: extends `ClientCommand` and marks `requiresControl() = false`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClientSketchCommand {
    /// Java: `fEntropy` — optional entropy byte inherited from `ClientCommand`.
    pub entropy: Option<u8>,
}

impl ClientSketchCommand {
    pub fn new() -> Self {
        Self::default()
    }

    /// Java: `requiresControl()` — sketch commands never require board control.
    pub fn requires_control(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requires_control_is_false() {
        assert!(!ClientSketchCommand::new().requires_control());
    }

    #[test]
    fn entropy_defaults_to_none() {
        assert!(ClientSketchCommand::new().entropy.is_none());
    }

    #[test]
    fn serde_round_trip_with_entropy() {
        let cmd = ClientSketchCommand { entropy: Some(99) };
        let json = serde_json::to_string(&cmd).unwrap();
        let back: ClientSketchCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(back.entropy, Some(99));
    }
}
