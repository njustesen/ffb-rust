use ffb_model::enums::{NetCommandId, PlayerState, SeriousInjuryKind};
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandApothecaryChoice`.
/// Sent when a coach makes an apothecary decision for an injured player.
#[derive(Debug, Clone)]
pub struct ClientCommandApothecaryChoice {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `fPlayerId`
    pub player_id: Option<String>,
    /// Java: `fPlayerState`
    pub player_state: Option<PlayerState>,
    /// Java: `oldPlayerState`
    pub old_player_state: Option<PlayerState>,
    /// Java: `fSeriousInjury`
    pub serious_injury: Option<SeriousInjuryKind>,
}

impl Default for ClientCommandApothecaryChoice {
    fn default() -> Self {
        Self {
            entropy: None,
            player_id: None,
            player_state: None,
            old_player_state: None,
            serious_injury: None,
        }
    }
}

impl ClientCommandApothecaryChoice {
    pub fn new() -> Self { Self::default() }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_player_state(&self) -> Option<PlayerState> { self.player_state }
    pub fn get_old_player_state(&self) -> Option<PlayerState> { self.old_player_state }
    pub fn get_serious_injury(&self) -> Option<SeriousInjuryKind> { self.serious_injury }

    /// Java: `ClientCommandApothecaryChoice.toJsonValue()` (calls `super.toJsonValue()` first).
    /// NOTE: Java's `PLAYER_STATE`/`PLAYER_STATE_OLD` options (`JsonPlayerStateOption`) serialize
    /// `PlayerState` as its raw `int` id (`UtilJson.toPlayerState`/`toJsonValue`), matching the
    /// Rust `PlayerState(u32)` newtype's `.id()`. `SERIOUS_INJURY` is a `JsonEnumWithNameOption`
    /// in Java (serializes via the edition-specific `SeriousInjury.getName()`, e.g. "Head Injury
    /// (-AV)"), but the Rust `SeriousInjuryKind` is a flattened, edition-agnostic engine enum with
    /// no such display-name mapping and no `.name()/.from_name()` pair — it does derive
    /// Serialize/Deserialize, so `serde_json::to_value`/`from_value` is used instead (round-trips
    /// self-consistently but does not match Java's wire string).
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        if let Some(player_id) = &self.player_id {
            map.insert("playerId".to_string(), serde_json::json!(player_id));
        }
        if let Some(player_state) = self.player_state {
            map.insert("playerState".to_string(), serde_json::json!(player_state.id()));
        }
        if let Some(serious_injury) = self.serious_injury {
            if let Ok(value) = serde_json::to_value(serious_injury) {
                map.insert("seriousInjury".to_string(), value);
            }
        }
        if let Some(old_player_state) = self.old_player_state {
            map.insert("playerStateOld".to_string(), serde_json::json!(old_player_state.id()));
        }
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandApothecaryChoice.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            player_id: json.get("playerId").and_then(|v| v.as_str()).map(|s| s.to_string()),
            player_state: json.get("playerState").and_then(|v| v.as_u64()).map(|v| PlayerState::new(v as u32)),
            old_player_state: json.get("playerStateOld").and_then(|v| v.as_u64()).map(|v| PlayerState::new(v as u32)),
            serious_injury: json.get("seriousInjury").and_then(|v| serde_json::from_value(v.clone()).ok()),
        }
    }
}

impl NetCommand for ClientCommandApothecaryChoice {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientApothecaryChoice
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_all_none() {
        let cmd = ClientCommandApothecaryChoice::new();
        assert!(cmd.player_id.is_none());
        assert!(cmd.player_state.is_none());
        assert!(cmd.old_player_state.is_none());
        assert!(cmd.serious_injury.is_none());
    }

    #[test]
    fn fields_accessible() {
        let mut cmd = ClientCommandApothecaryChoice::new();
        cmd.player_id = Some("p1".into());
        cmd.serious_injury = Some(SeriousInjuryKind::Dead);
        assert_eq!(cmd.get_player_id(), Some("p1"));
        assert_eq!(cmd.get_serious_injury(), Some(SeriousInjuryKind::Dead));
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandApothecaryChoice::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_roundtrip() {
        let cmd = ClientCommandApothecaryChoice::default();
        let _ = cmd.clone();
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandApothecaryChoice::default().clone();
    }

    #[test]
    fn get_id_is_client_apothecary_choice() {
        assert_eq!(ClientCommandApothecaryChoice::new().get_id(), NetCommandId::ClientApothecaryChoice);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_player_state() {
        let mut cmd = ClientCommandApothecaryChoice::new();
        cmd.player_state = Some(PlayerState::new(3));
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientApothecaryChoice");
        assert_eq!(json["playerState"], 3);
    }

    #[test]
    fn round_trip_with_populated_fields() {
        let mut cmd = ClientCommandApothecaryChoice::new();
        cmd.entropy = Some(6);
        cmd.player_id = Some("p1".into());
        cmd.player_state = Some(PlayerState::new(7));
        cmd.old_player_state = Some(PlayerState::new(2));
        cmd.serious_injury = Some(SeriousInjuryKind::Dead);
        let json = cmd.to_json_value();
        let restored = ClientCommandApothecaryChoice::from_json(&json);
        assert_eq!(restored.entropy, Some(6));
        assert_eq!(restored.player_id.as_deref(), Some("p1"));
        assert_eq!(restored.player_state, Some(PlayerState::new(7)));
        assert_eq!(restored.old_player_state, Some(PlayerState::new(2)));
        assert_eq!(restored.serious_injury, Some(SeriousInjuryKind::Dead));
    }

    #[test]
    fn round_trip_with_default_data() {
        let cmd = ClientCommandApothecaryChoice::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandApothecaryChoice::from_json(&json);
        assert!(restored.player_id.is_none());
        assert!(restored.player_state.is_none());
        assert!(restored.old_player_state.is_none());
        assert!(restored.serious_injury.is_none());
    }
}
