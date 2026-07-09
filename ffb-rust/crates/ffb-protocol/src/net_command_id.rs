/// 1:1 translation of `com.fumbbl.ffb.net.NetCommandId`.
/// The canonical enum lives in `ffb_model::enums::NetCommandId` where it is
/// shared with the engine. This module re-exports it for consumers that depend
/// only on `ffb-protocol`, preserving the original fully-qualified name.
pub use ffb_model::enums::NetCommandId;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_join_name_serializes() {
        let id = NetCommandId::ClientJoin;
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, r#""clientJoin""#);
    }

    #[test]
    fn server_game_state_round_trip() {
        let id = NetCommandId::ServerGameState;
        let json = serde_json::to_string(&id).unwrap();
        let back: NetCommandId = serde_json::from_str(&json).unwrap();
        assert_eq!(back, id);
    }
}
