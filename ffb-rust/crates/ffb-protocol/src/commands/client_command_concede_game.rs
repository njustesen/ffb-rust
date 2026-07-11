use ffb_model::model::ConcedeGameStatus;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;
use ffb_model::enums::NetCommandId;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandConcedeGame`.
/// Sent when a coach concedes the game.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandConcedeGame {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `fConcedeGameStatus`
    pub concede_game_status: Option<ConcedeGameStatus>,
}

impl ClientCommandConcedeGame {
    pub fn new() -> Self { Self::default() }
    pub fn get_concede_game_status(&self) -> Option<&ConcedeGameStatus> { self.concede_game_status.as_ref() }

    /// Java: `ClientCommandConcedeGame.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        if let Some(status) = self.concede_game_status {
            map.insert("concedeGameStatus".to_string(), serde_json::json!(status.get_name()));
        }
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandConcedeGame.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            concede_game_status: json.get("concedeGameStatus").and_then(|v| v.as_str()).and_then(ConcedeGameStatus::from_name),
        }
    }
}

impl NetCommand for ClientCommandConcedeGame {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientConcedeGame
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_status_none() {
        let cmd = ClientCommandConcedeGame::new();
        assert!(cmd.concede_game_status.is_none());
    }

    #[test]
    fn default_same_as_new() { let _ = ClientCommandConcedeGame::default(); }

    #[test]
    fn stores_concede_status() {
        let cmd = ClientCommandConcedeGame {
            entropy: None,
            concede_game_status: Some(ConcedeGameStatus::REQUESTED),
        };
        assert!(cmd.get_concede_game_status().is_some());
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandConcedeGame::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandConcedeGame::default().clone();
    }

    #[test]
    fn get_id_is_client_concede_game() {
        assert_eq!(ClientCommandConcedeGame::new().get_id(), NetCommandId::ClientConcedeGame);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_concede_game_status() {
        let cmd = ClientCommandConcedeGame {
            entropy: None,
            concede_game_status: Some(ConcedeGameStatus::CONFIRMED),
        };
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientConcedeGame");
        assert_eq!(json["concedeGameStatus"], "confirmed");
    }

    #[test]
    fn round_trip_with_status_and_entropy() {
        let cmd = ClientCommandConcedeGame {
            entropy: Some(2),
            concede_game_status: Some(ConcedeGameStatus::DENIED),
        };
        let json = cmd.to_json_value();
        let restored = ClientCommandConcedeGame::from_json(&json);
        assert_eq!(restored.entropy, Some(2));
        assert_eq!(restored.concede_game_status, Some(ConcedeGameStatus::DENIED));
    }

    #[test]
    fn round_trip_with_no_status() {
        let cmd = ClientCommandConcedeGame::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandConcedeGame::from_json(&json);
        assert!(restored.concede_game_status.is_none());
    }
}
