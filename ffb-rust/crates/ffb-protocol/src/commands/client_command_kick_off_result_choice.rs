/// 1:1 translation of com.fumbbl.ffb.net.commands.ClientCommandKickOffResultChoice.
use ffb_model::enums::{KickoffResult, NetCommandId};
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

#[derive(Debug, Clone, Default)]
pub struct ClientCommandKickOffResultChoice {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `kickoffResult`
    pub kickoff_result: Option<KickoffResult>,
}

impl ClientCommandKickOffResultChoice {
    pub fn new() -> Self {
        Self::default()
    }

    /// Java: `getKickoffResult()`
    pub fn get_kickoff_result(&self) -> Option<KickoffResult> {
        self.kickoff_result
    }

    /// Java: `ClientCommandKickOffResultChoice.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        if let Some(kickoff_result) = self.kickoff_result {
            map.insert("kickoffResult".to_string(), serde_json::json!(kickoff_result.name()));
        }
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandKickOffResultChoice.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            kickoff_result: json.get("kickoffResult").and_then(|v| v.as_str()).and_then(KickoffResult::from_name),
        }
    }
}

impl NetCommand for ClientCommandKickOffResultChoice {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientKickOffResultChoice
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_no_kickoff_result() {
        let cmd = ClientCommandKickOffResultChoice::new();
        assert!(cmd.get_kickoff_result().is_none());
    }

    #[test]
    fn stores_kickoff_result() {
        let cmd = ClientCommandKickOffResultChoice {
            entropy: None,
            kickoff_result: Some(KickoffResult::QuickSnap),
        };
        assert_eq!(cmd.get_kickoff_result(), Some(KickoffResult::QuickSnap));
    }

    #[test]
    fn blitz_variant_stored() {
        let cmd = ClientCommandKickOffResultChoice {
            entropy: None,
            kickoff_result: Some(KickoffResult::Blitz),
        };
        assert_eq!(cmd.get_kickoff_result(), Some(KickoffResult::Blitz));
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandKickOffResultChoice::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandKickOffResultChoice::default().clone();
    }

    #[test]
    fn get_id_is_client_kick_off_result_choice() {
        assert_eq!(ClientCommandKickOffResultChoice::new().get_id(), NetCommandId::ClientKickOffResultChoice);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_kickoff_result() {
        let cmd = ClientCommandKickOffResultChoice {
            entropy: None,
            kickoff_result: Some(KickoffResult::Blitz),
        };
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientKickOffResultChoice");
        assert_eq!(json["kickoffResult"], "Blitz");
    }

    #[test]
    fn round_trip_with_result_and_entropy() {
        let cmd = ClientCommandKickOffResultChoice {
            entropy: Some(3),
            kickoff_result: Some(KickoffResult::QuickSnap),
        };
        let json = cmd.to_json_value();
        let restored = ClientCommandKickOffResultChoice::from_json(&json);
        assert_eq!(restored.entropy, Some(3));
        assert_eq!(restored.get_kickoff_result(), Some(KickoffResult::QuickSnap));
    }

    #[test]
    fn round_trip_with_no_result() {
        let cmd = ClientCommandKickOffResultChoice::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandKickOffResultChoice::from_json(&json);
        assert!(restored.get_kickoff_result().is_none());
    }
}
