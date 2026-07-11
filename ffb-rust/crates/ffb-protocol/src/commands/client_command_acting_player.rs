use ffb_model::enums::{NetCommandId, PlayerAction};
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandActingPlayer`.
/// Sent to declare which player will act next and what action they perform.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandActingPlayer {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `fPlayerId`
    pub player_id: Option<String>,
    /// Java: `fPlayerAction`
    pub player_action: Option<PlayerAction>,
    /// Java: `jumping`
    pub jumping: bool,
}

impl ClientCommandActingPlayer {
    pub fn new(player_id: impl Into<String>, player_action: PlayerAction, jumping: bool) -> Self {
        Self {
            entropy: None,
            player_id: Some(player_id.into()),
            player_action: Some(player_action),
            jumping,
        }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_player_action(&self) -> Option<PlayerAction> { self.player_action }
    pub fn is_jumping(&self) -> bool { self.jumping }

    /// Java: `ClientCommandActingPlayer.toJsonValue()` (calls `super.toJsonValue()` first).
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        if let Some(player_id) = &self.player_id {
            map.insert("playerId".to_string(), serde_json::json!(player_id));
        }
        if let Some(player_action) = self.player_action {
            map.insert("playerAction".to_string(), serde_json::json!(player_action.name()));
        }
        // IJsonOption.JUMPING wire key is "leaping" (Java naming mismatch, verified in IJsonOption.java).
        map.insert("leaping".to_string(), serde_json::json!(self.jumping));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandActingPlayer.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            player_id: json.get("playerId").and_then(|v| v.as_str()).map(|s| s.to_string()),
            player_action: json.get("playerAction").and_then(|v| v.as_str()).and_then(PlayerAction::from_name),
            jumping: json.get("leaping").and_then(|v| v.as_bool()).unwrap_or(false),
        }
    }
}

impl NetCommand for ClientCommandActingPlayer {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientActingPlayer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ClientCommandActingPlayer::new("p1", PlayerAction::Move, false);
        assert_eq!(cmd.get_player_id(), Some("p1"));
        assert_eq!(cmd.get_player_action(), Some(PlayerAction::Move));
        assert!(!cmd.is_jumping());
    }

    #[test]
    fn jumping_flag() {
        let cmd = ClientCommandActingPlayer::new("p2", PlayerAction::Block, true);
        assert!(cmd.is_jumping());
    }

    #[test]
    fn default_is_empty() {
        let cmd = ClientCommandActingPlayer::default();
        assert!(cmd.player_id.is_none());
        assert!(cmd.player_action.is_none());
        assert!(!cmd.jumping);
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandActingPlayer::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandActingPlayer::default().clone();
    }

    #[test]
    fn get_id_is_client_acting_player() {
        assert_eq!(ClientCommandActingPlayer::default().get_id(), NetCommandId::ClientActingPlayer);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_player_action() {
        let cmd = ClientCommandActingPlayer::new("p1", PlayerAction::Block, true);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientActingPlayer");
        assert_eq!(json["playerAction"], "block");
        assert_eq!(json["leaping"], true);
    }

    #[test]
    fn round_trip_with_populated_fields() {
        let mut cmd = ClientCommandActingPlayer::new("p1", PlayerAction::Blitz, true);
        cmd.entropy = Some(5);
        let json = cmd.to_json_value();
        let restored = ClientCommandActingPlayer::from_json(&json);
        assert_eq!(restored.entropy, Some(5));
        assert_eq!(restored.player_id.as_deref(), Some("p1"));
        assert_eq!(restored.player_action, Some(PlayerAction::Blitz));
        assert!(restored.jumping);
    }

    #[test]
    fn round_trip_with_default_data() {
        let cmd = ClientCommandActingPlayer::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandActingPlayer::from_json(&json);
        assert!(restored.player_id.is_none());
        assert!(restored.player_action.is_none());
        assert!(!restored.jumping);
    }
}
