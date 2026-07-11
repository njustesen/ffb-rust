use ffb_model::enums::NetCommandId;
use ffb_model::model::ReRolledAction;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandUseSkill`.
/// Sent when a player uses or declines to use a skill.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandUseSkill {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `fSkill` — stored as skill name string.
    pub skill_name: Option<String>,
    /// Java: `fSkillUsed`
    pub skill_used: bool,
    /// Java: `neverUse`
    pub never_use: bool,
    /// Java: `playerId`
    pub player_id: Option<String>,
    /// Java: `reRolledAction`
    pub re_rolled_action: Option<ReRolledAction>,
}

impl ClientCommandUseSkill {
    pub fn new() -> Self { Self::default() }
    pub fn get_skill_name(&self) -> Option<&str> { self.skill_name.as_deref() }
    pub fn is_skill_used(&self) -> bool { self.skill_used }
    pub fn is_never_use(&self) -> bool { self.never_use }
    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }

    /// Java: `ClientCommandUseSkill.toJsonValue()` (calls `super.toJsonValue()` first).
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        if let Some(skill) = &self.skill_name {
            map.insert("skill".to_string(), serde_json::json!(skill));
        }
        map.insert("skillUsed".to_string(), serde_json::json!(self.skill_used));
        if let Some(player_id) = &self.player_id {
            map.insert("playerId".to_string(), serde_json::json!(player_id));
        }
        if let Some(re_rolled_action) = &self.re_rolled_action {
            map.insert("reRolledAction".to_string(), serde_json::json!(re_rolled_action.get_name()));
        }
        map.insert("skillNeverUse".to_string(), serde_json::json!(self.never_use));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandUseSkill.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            skill_name: json.get("skill").and_then(|v| v.as_str()).map(|s| s.to_string()),
            skill_used: json.get("skillUsed").and_then(|v| v.as_bool()).unwrap_or(false),
            player_id: json.get("playerId").and_then(|v| v.as_str()).map(|s| s.to_string()),
            re_rolled_action: json.get("reRolledAction").and_then(|v| v.as_str()).map(ReRolledAction::new),
            never_use: json.get("skillNeverUse").and_then(|v| v.as_bool()).unwrap_or(false),
        }
    }
}

impl NetCommand for ClientCommandUseSkill {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientUseSkill
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn skill_used_flag() {
        let mut cmd = ClientCommandUseSkill::new();
        cmd.skill_used = true;
        assert!(cmd.is_skill_used());
    }
    #[test]
    fn default_all_false() {
        let cmd = ClientCommandUseSkill::new();
        assert!(!cmd.skill_used);
        assert!(!cmd.never_use);
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandUseSkill::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandUseSkill::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandUseSkill::default());
        assert!(s.contains("ClientCommandUseSkill"));
    }

    #[test]
    fn get_id_is_client_use_skill() {
        assert_eq!(ClientCommandUseSkill::new().get_id(), NetCommandId::ClientUseSkill);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_skill_used() {
        let mut cmd = ClientCommandUseSkill::new();
        cmd.skill_used = true;
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientUseSkill");
        assert_eq!(json["skillUsed"], true);
    }

    #[test]
    fn round_trip_with_all_fields_and_entropy() {
        let mut cmd = ClientCommandUseSkill::new();
        cmd.entropy = Some(5);
        cmd.skill_name = Some("Dodge".into());
        cmd.skill_used = true;
        cmd.never_use = true;
        cmd.player_id = Some("p1".into());
        cmd.re_rolled_action = Some(ReRolledAction::new("Dodge"));
        let json = cmd.to_json_value();
        let restored = ClientCommandUseSkill::from_json(&json);
        assert_eq!(restored.entropy, Some(5));
        assert_eq!(restored.skill_name.as_deref(), Some("Dodge"));
        assert!(restored.skill_used);
        assert!(restored.never_use);
        assert_eq!(restored.player_id.as_deref(), Some("p1"));
        assert_eq!(restored.re_rolled_action, Some(ReRolledAction::new("Dodge")));
    }

    #[test]
    fn round_trip_with_no_optional_fields() {
        let cmd = ClientCommandUseSkill::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandUseSkill::from_json(&json);
        assert!(restored.skill_name.is_none());
        assert!(!restored.skill_used);
        assert!(!restored.never_use);
        assert!(restored.player_id.is_none());
        assert!(restored.re_rolled_action.is_none());
    }
}
