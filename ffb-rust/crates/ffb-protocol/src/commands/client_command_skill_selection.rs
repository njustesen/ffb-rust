use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of com.fumbbl.ffb.net.commands.ClientCommandSkillSelection.
///
/// Java: `skill` is a complex Skill object. We use the skill's string identifier.

#[derive(Debug, Clone, Default)]
pub struct ClientCommandSkillSelection {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `playerId`
    pub player_id: Option<String>,
    /// Java: `skill` — stored as the skill's string identifier (DEFERRED: full Skill object).
    pub skill_id: Option<String>,
}

impl ClientCommandSkillSelection {
    pub fn new() -> Self {
        Self::default()
    }

    /// Java: `getPlayerId()`
    pub fn get_player_id(&self) -> Option<&str> {
        self.player_id.as_deref()
    }

    /// Java: `getSkill()` — returns the skill identifier string.
    pub fn get_skill_id(&self) -> Option<&str> {
        self.skill_id.as_deref()
    }

    /// Java: `ClientCommandSkillSelection.toJsonValue()`. `IJsonOption.SKILL` is
    /// a `JsonEnumWithNameOption` over a full `Skill` object in Java; since this
    /// Rust struct already simplifies `skill` down to a plain identifier string,
    /// it is serialized as a plain string under the `"skill"` wire key.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("playerId".to_string(), serde_json::json!(self.player_id));
        map.insert("skill".to_string(), serde_json::json!(self.skill_id));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandSkillSelection.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            player_id: json.get("playerId").and_then(|v| v.as_str()).map(|s| s.to_string()),
            skill_id: json.get("skill").and_then(|v| v.as_str()).map(|s| s.to_string()),
        }
    }
}

impl NetCommand for ClientCommandSkillSelection {
    /// Java: `getId()` deliberately returns `NetCommandId.CLIENT_PRAYER_SELECTION`,
    /// not a "skill selection" id — the class was renamed from a prayer-selection
    /// command but the wire id is kept unchanged so old replays still parse.
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientPrayerSelection
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_no_player_or_skill() {
        let cmd = ClientCommandSkillSelection::new();
        assert!(cmd.get_player_id().is_none());
        assert!(cmd.get_skill_id().is_none());
    }

    #[test]
    fn stores_player_id_and_skill_id() {
        let cmd = ClientCommandSkillSelection {
            entropy: None,
            player_id: Some("player_2".to_string()),
            skill_id: Some("BLOCK".to_string()),
        };
        assert_eq!(cmd.get_player_id(), Some("player_2"));
        assert_eq!(cmd.get_skill_id(), Some("BLOCK"));
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandSkillSelection::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandSkillSelection::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandSkillSelection::default());
        assert!(s.contains("ClientCommandSkillSelection"));
    }

    #[test]
    fn get_id_is_client_prayer_selection() {
        // Intentional: Java's getId() kept the old prayer-selection wire id.
        assert_eq!(ClientCommandSkillSelection::new().get_id(), NetCommandId::ClientPrayerSelection);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_skill_key() {
        let cmd = ClientCommandSkillSelection {
            entropy: None,
            player_id: Some("player_2".to_string()),
            skill_id: Some("BLOCK".to_string()),
        };
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientPrayerSelection");
        assert_eq!(json["skill"], "BLOCK");
    }

    #[test]
    fn round_trip_with_data() {
        let cmd = ClientCommandSkillSelection {
            entropy: Some(4),
            player_id: Some("player_3".to_string()),
            skill_id: Some("DODGE".to_string()),
        };
        let json = cmd.to_json_value();
        let restored = ClientCommandSkillSelection::from_json(&json);
        assert_eq!(restored.player_id, cmd.player_id);
        assert_eq!(restored.skill_id, cmd.skill_id);
        assert_eq!(restored.entropy, cmd.entropy);
    }

    #[test]
    fn round_trip_default() {
        let cmd = ClientCommandSkillSelection::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandSkillSelection::from_json(&json);
        assert!(restored.player_id.is_none());
        assert!(restored.skill_id.is_none());
        assert!(restored.entropy.is_none());
    }
}
