/// 1:1 translation of com.fumbbl.ffb.net.commands.ClientCommandUseApothecary.
use ffb_model::enums::{ApothecaryType, NetCommandId, PlayerState, SeriousInjuryKind};
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

#[derive(Debug, Clone, Default)]
pub struct ClientCommandUseApothecary {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `fPlayerId`
    pub player_id: Option<String>,
    /// Java: `fApothecaryUsed`
    pub apothecary_used: bool,
    /// Java: `apothecaryType`
    pub apothecary_type: Option<ApothecaryType>,
    /// Java: `seriousInjury` — simplified to SeriousInjuryKind (the injury kind field).
    pub serious_injury: Option<SeriousInjuryKind>,
    /// Java: `playerState`
    pub player_state: Option<PlayerState>,
}

impl ClientCommandUseApothecary {
    pub fn new() -> Self {
        Self::default()
    }

    /// Java: `getPlayerId()`
    pub fn get_player_id(&self) -> Option<&str> {
        self.player_id.as_deref()
    }

    /// Java: `isApothecaryUsed()`
    pub fn is_apothecary_used(&self) -> bool {
        self.apothecary_used
    }

    /// Java: `getApothecaryType()`
    pub fn get_apothecary_type(&self) -> Option<ApothecaryType> {
        self.apothecary_type
    }

    /// Java: `getSeriousInjury()`
    pub fn get_serious_injury(&self) -> Option<SeriousInjuryKind> {
        self.serious_injury
    }

    /// Java: `getPlayerState()`
    pub fn get_player_state(&self) -> Option<PlayerState> {
        self.player_state
    }

    /// Java: `ClientCommandUseApothecary.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("playerId".to_string(), serde_json::json!(self.player_id));
        map.insert("apothecaryUsed".to_string(), serde_json::json!(self.apothecary_used));
        if let Some(apothecary_type) = self.apothecary_type {
            map.insert("apothecaryType".to_string(), serde_json::json!(apothecary_type.name()));
        }
        if let Some(serious_injury) = self.serious_injury {
            map.insert("seriousInjury".to_string(), serde_json::json!(serious_injury.name()));
        }
        if let Some(player_state) = self.player_state {
            map.insert("playerState".to_string(), serde_json::json!(player_state.id()));
        }
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandUseApothecary.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            player_id: json.get("playerId").and_then(|v| v.as_str()).map(|s| s.to_string()),
            apothecary_used: json.get("apothecaryUsed").and_then(|v| v.as_bool()).unwrap_or(false),
            apothecary_type: json.get("apothecaryType").and_then(|v| v.as_str()).and_then(ApothecaryType::from_name),
            serious_injury: json.get("seriousInjury").and_then(|v| v.as_str()).and_then(SeriousInjuryKind::from_name),
            player_state: json.get("playerState").and_then(|v| v.as_u64()).map(|id| PlayerState::new(id as u32)),
        }
    }
}

impl NetCommand for ClientCommandUseApothecary {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientUseApothecary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_apothecary_not_used() {
        let cmd = ClientCommandUseApothecary::new();
        assert!(!cmd.is_apothecary_used());
        assert!(cmd.get_player_id().is_none());
    }

    #[test]
    fn stores_apothecary_fields() {
        let cmd = ClientCommandUseApothecary {
            entropy: None,
            player_id: Some("player_3".to_string()),
            apothecary_used: true,
            apothecary_type: Some(ApothecaryType::Team),
            serious_injury: Some(SeriousInjuryKind::SeriouslyHurt),
            player_state: Some(PlayerState::new(1)),
        };
        assert_eq!(cmd.get_player_id(), Some("player_3"));
        assert!(cmd.is_apothecary_used());
        assert_eq!(cmd.get_apothecary_type(), Some(ApothecaryType::Team));
        assert_eq!(cmd.get_serious_injury(), Some(SeriousInjuryKind::SeriouslyHurt));
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandUseApothecary::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandUseApothecary::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandUseApothecary::default());
        assert!(s.contains("ClientCommandUseApothecary"));
    }

    #[test]
    fn get_id_is_client_use_apothecary() {
        assert_eq!(ClientCommandUseApothecary::new().get_id(), NetCommandId::ClientUseApothecary);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_apothecary_used() {
        let mut cmd = ClientCommandUseApothecary::new();
        cmd.player_id = Some("p1".to_string());
        cmd.apothecary_used = true;
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientUseApothecary");
        assert_eq!(json["apothecaryUsed"], true);
        assert_eq!(json["playerId"], "p1");
    }

    #[test]
    fn round_trip_with_all_fields_and_entropy() {
        let mut cmd = ClientCommandUseApothecary {
            entropy: None,
            player_id: Some("player_4".to_string()),
            apothecary_used: true,
            apothecary_type: Some(ApothecaryType::Wandering),
            serious_injury: Some(SeriousInjuryKind::Dead),
            player_state: Some(PlayerState::new(7)),
        };
        cmd.entropy = Some(17);
        let json = cmd.to_json_value();
        let restored = ClientCommandUseApothecary::from_json(&json);
        assert_eq!(restored.entropy, Some(17));
        assert_eq!(restored.get_player_id(), Some("player_4"));
        assert!(restored.is_apothecary_used());
        assert_eq!(restored.get_apothecary_type(), Some(ApothecaryType::Wandering));
        assert_eq!(restored.get_serious_injury(), Some(SeriousInjuryKind::Dead));
        assert_eq!(restored.get_player_state(), Some(PlayerState::new(7)));
    }

    #[test]
    fn round_trip_with_no_optional_fields() {
        let cmd = ClientCommandUseApothecary::new();
        let json = cmd.to_json_value();
        assert!(json.get("apothecaryType").is_none());
        assert!(json.get("seriousInjury").is_none());
        assert!(json.get("playerState").is_none());
        let restored = ClientCommandUseApothecary::from_json(&json);
        assert!(restored.get_apothecary_type().is_none());
        assert!(restored.get_serious_injury().is_none());
        assert!(restored.get_player_state().is_none());
    }
}
