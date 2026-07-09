use serde::{Deserialize, Serialize};

/// 1:1 translation of `com.fumbbl.ffb.net.NetCommand`.
/// Abstract base for all network commands in the FFB protocol.
/// In Java this is an abstract class; in Rust we model it as an enum
/// over the two main categories (client / server) plus a catch-all.
///
/// For the low-level message framing layer only the `id` string is
/// needed; structured variants live in `client_commands` / `server_commands`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "netCommandId", rename_all = "camelCase")]
pub enum NetCommand {
    /// Sentinel for commands that could not be identified.
    #[serde(other)]
    Unknown,
}

impl Default for NetCommand {
    fn default() -> Self {
        Self::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_unknown() {
        assert!(matches!(NetCommand::default(), NetCommand::Unknown));
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = NetCommand::Unknown.clone();
    }
}
