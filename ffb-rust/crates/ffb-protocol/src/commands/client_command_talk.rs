use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of ClientCommandTalk (Java field: fTalk).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandTalk {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    pub talk: Option<String>,
}

impl ClientCommandTalk {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_talk(talk: impl Into<String>) -> Self {
        Self { entropy: None, talk: Some(talk.into()) }
    }

    pub fn get_talk(&self) -> Option<&str> {
        self.talk.as_deref()
    }

    /// Java: `ClientCommandTalk.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        if let Some(talk) = &self.talk {
            map.insert("talk".to_string(), serde_json::json!(talk));
        }
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandTalk.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            talk: json.get("talk").and_then(|v| v.as_str()).map(String::from),
        }
    }
}

impl NetCommand for ClientCommandTalk {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientTalk
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_no_talk() {
        let cmd = ClientCommandTalk::new();
        assert!(cmd.get_talk().is_none());
    }

    #[test]
    fn with_talk_stores_value() {
        let cmd = ClientCommandTalk::with_talk("hello");
        assert_eq!(cmd.get_talk(), Some("hello"));
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandTalk::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandTalk::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandTalk::default());
        assert!(s.contains("ClientCommandTalk"));
    }

    #[test]
    fn get_id_is_client_talk() {
        assert_eq!(ClientCommandTalk::new().get_id(), NetCommandId::ClientTalk);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_talk() {
        let cmd = ClientCommandTalk::with_talk("hi there");
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientTalk");
        assert_eq!(json["talk"], "hi there");
    }

    #[test]
    fn round_trip_with_talk_and_entropy() {
        let mut cmd = ClientCommandTalk::with_talk("gg");
        cmd.entropy = Some(1);
        let json = cmd.to_json_value();
        let restored = ClientCommandTalk::from_json(&json);
        assert_eq!(restored.entropy, Some(1));
        assert_eq!(restored.talk.as_deref(), Some("gg"));
    }

    #[test]
    fn round_trip_with_no_talk() {
        let cmd = ClientCommandTalk::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandTalk::from_json(&json);
        assert!(restored.talk.is_none());
    }
}
