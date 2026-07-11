/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandInterceptorChoice`.
/// Sent when a coach selects an interceptor for a pass.
///
/// Note: `interceptionSkill` (a Skill object, `IJsonOption.SKILL`, wire key `"skill"`) is
/// represented here as the skill's name string rather than the full `Skill` model — `Skill`
/// carries trait-object modifier lists (`Box<dyn ISkillProperty>`) with no `Serialize`/`Clone`
/// impl, so round-tripping the whole object isn't practical yet. The interceptor player ID and
/// the skill name are the fields that matter on the wire.
use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

#[derive(Debug, Clone, Default)]
pub struct ClientCommandInterceptorChoice {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `fInterceptorId`
    pub interceptor_id: Option<String>,
    /// Java: `interceptionSkill` — skill name string (simplified from full Skill object).
    pub interception_skill_id: Option<String>,
}

impl ClientCommandInterceptorChoice {
    pub fn new() -> Self { Self::default() }

    pub fn with_interceptor(interceptor_id: impl Into<String>) -> Self {
        Self {
            entropy: None,
            interceptor_id: Some(interceptor_id.into()),
            interception_skill_id: None,
        }
    }

    pub fn get_interceptor_id(&self) -> Option<&str> { self.interceptor_id.as_deref() }
    pub fn get_interception_skill_id(&self) -> Option<&str> { self.interception_skill_id.as_deref() }

    /// Java: `ClientCommandInterceptorChoice.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("interceptorId".to_string(), serde_json::json!(self.interceptor_id));
        map.insert("skill".to_string(), serde_json::json!(self.interception_skill_id));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandInterceptorChoice.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            interceptor_id: json.get("interceptorId").and_then(|v| v.as_str()).map(String::from),
            interception_skill_id: json.get("skill").and_then(|v| v.as_str()).map(String::from),
        }
    }
}

impl NetCommand for ClientCommandInterceptorChoice {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientInterceptorChoice
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interceptor_id_stored() {
        let cmd = ClientCommandInterceptorChoice::with_interceptor("p5");
        assert_eq!(cmd.get_interceptor_id(), Some("p5"));
        assert!(cmd.get_interception_skill_id().is_none());
    }

    #[test]
    fn default_both_none() {
        let cmd = ClientCommandInterceptorChoice::new();
        assert!(cmd.interceptor_id.is_none());
    }
#[test]    fn debug_format_nonempty() {        let v = ClientCommandInterceptorChoice::default();        assert!(!format!("{:?}", v).is_empty());    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandInterceptorChoice::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandInterceptorChoice::default());
        assert!(s.contains("ClientCommandInterceptorChoice"));
    }

    #[test]
    fn get_id_is_client_interceptor_choice() {
        assert_eq!(ClientCommandInterceptorChoice::new().get_id(), NetCommandId::ClientInterceptorChoice);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_interceptor_id() {
        let cmd = ClientCommandInterceptorChoice::with_interceptor("p5");
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientInterceptorChoice");
        assert_eq!(json["interceptorId"], "p5");
    }

    #[test]
    fn round_trip_with_skill_and_entropy() {
        let mut cmd = ClientCommandInterceptorChoice::with_interceptor("p5");
        cmd.interception_skill_id = Some("Dodge".to_string());
        cmd.entropy = Some(8);
        let json = cmd.to_json_value();
        let restored = ClientCommandInterceptorChoice::from_json(&json);
        assert_eq!(restored.entropy, Some(8));
        assert_eq!(restored.get_interceptor_id(), Some("p5"));
        assert_eq!(restored.get_interception_skill_id(), Some("Dodge"));
    }

    #[test]
    fn round_trip_default() {
        let cmd = ClientCommandInterceptorChoice::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandInterceptorChoice::from_json(&json);
        assert!(restored.interceptor_id.is_none());
        assert!(restored.interception_skill_id.is_none());
    }
}
