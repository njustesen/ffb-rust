use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandBlock`.
/// Sent when a player initiates a block action.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandBlock {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `fActingPlayerId`
    pub acting_player_id: Option<String>,
    /// Java: `fDefenderId`
    pub defender_id: Option<String>,
    /// Java: `fUsingStab`
    pub using_stab: bool,
    /// Java: `usingChainsaw`
    pub using_chainsaw: bool,
    /// Java: `usingVomit`
    pub using_vomit: bool,
    /// Java: `usingBreatheFire`
    pub using_breathe_fire: bool,
    /// Java: `usingChomp`
    pub using_chomp: bool,
}

impl ClientCommandBlock {
    pub fn new(
        acting_player_id: impl Into<String>,
        defender_id: impl Into<String>,
        using_stab: bool,
        using_chainsaw: bool,
        using_vomit: bool,
        using_breathe_fire: bool,
        using_chomp: bool,
    ) -> Self {
        Self {
            entropy: None,
            acting_player_id: Some(acting_player_id.into()),
            defender_id: Some(defender_id.into()),
            using_stab,
            using_chainsaw,
            using_vomit,
            using_breathe_fire,
            using_chomp,
        }
    }

    pub fn get_acting_player_id(&self) -> Option<&str> { self.acting_player_id.as_deref() }
    pub fn get_defender_id(&self) -> Option<&str> { self.defender_id.as_deref() }
    pub fn is_using_stab(&self) -> bool { self.using_stab }
    pub fn is_using_chainsaw(&self) -> bool { self.using_chainsaw }
    pub fn is_using_vomit(&self) -> bool { self.using_vomit }
    pub fn is_using_breathe_fire(&self) -> bool { self.using_breathe_fire }
    pub fn is_using_chomp(&self) -> bool { self.using_chomp }

    /// Java: `ClientCommandBlock.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        if let Some(acting_player_id) = &self.acting_player_id {
            map.insert("actingPlayerId".to_string(), serde_json::json!(acting_player_id));
        }
        if let Some(defender_id) = &self.defender_id {
            map.insert("defenderId".to_string(), serde_json::json!(defender_id));
        }
        map.insert("usingStab".to_string(), serde_json::json!(self.using_stab));
        map.insert("usingChainsaw".to_string(), serde_json::json!(self.using_chainsaw));
        map.insert("usingVomit".to_string(), serde_json::json!(self.using_vomit));
        map.insert("usingBreatheFire".to_string(), serde_json::json!(self.using_breathe_fire));
        map.insert("usingChomp".to_string(), serde_json::json!(self.using_chomp));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandBlock.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            acting_player_id: json.get("actingPlayerId").and_then(|v| v.as_str()).map(String::from),
            defender_id: json.get("defenderId").and_then(|v| v.as_str()).map(String::from),
            using_stab: json.get("usingStab").and_then(|v| v.as_bool()).unwrap_or(false),
            using_chainsaw: json.get("usingChainsaw").and_then(|v| v.as_bool()).unwrap_or(false),
            using_vomit: json.get("usingVomit").and_then(|v| v.as_bool()).unwrap_or(false),
            using_breathe_fire: json.get("usingBreatheFire").and_then(|v| v.as_bool()).unwrap_or(false),
            using_chomp: json.get("usingChomp").and_then(|v| v.as_bool()).unwrap_or(false),
        }
    }
}

impl NetCommand for ClientCommandBlock {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientBlock
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored_correctly() {
        let cmd = ClientCommandBlock::new("atk1", "def1", true, false, false, false, false);
        assert_eq!(cmd.get_acting_player_id(), Some("atk1"));
        assert_eq!(cmd.get_defender_id(), Some("def1"));
        assert!(cmd.is_using_stab());
        assert!(!cmd.is_using_chainsaw());
    }

    #[test]
    fn chainsaw_flag() {
        let cmd = ClientCommandBlock::new("a", "b", false, true, false, false, false);
        assert!(cmd.is_using_chainsaw());
        assert!(!cmd.is_using_stab());
    }

    #[test]
    fn default_all_false() {
        let cmd = ClientCommandBlock::default();
        assert!(!cmd.using_stab);
        assert!(!cmd.using_chainsaw);
        assert!(!cmd.using_vomit);
        assert!(!cmd.using_breathe_fire);
        assert!(!cmd.using_chomp);
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandBlock::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandBlock::default().clone();
    }

    #[test]
    fn get_id_is_client_block() {
        assert_eq!(ClientCommandBlock::default().get_id(), NetCommandId::ClientBlock);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_using_stab() {
        let cmd = ClientCommandBlock::new("atk1", "def1", true, false, false, false, false);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientBlock");
        assert_eq!(json["usingStab"], true);
    }

    #[test]
    fn round_trip_with_populated_data() {
        let mut cmd = ClientCommandBlock::new("atk1", "def1", true, true, true, true, true);
        cmd.entropy = Some(5);
        let json = cmd.to_json_value();
        let restored = ClientCommandBlock::from_json(&json);
        assert_eq!(restored.entropy, Some(5));
        assert_eq!(restored.get_acting_player_id(), Some("atk1"));
        assert_eq!(restored.get_defender_id(), Some("def1"));
        assert!(restored.is_using_stab());
        assert!(restored.is_using_chainsaw());
        assert!(restored.is_using_vomit());
        assert!(restored.is_using_breathe_fire());
        assert!(restored.is_using_chomp());
    }

    #[test]
    fn round_trip_with_default_data() {
        let cmd = ClientCommandBlock::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandBlock::from_json(&json);
        assert!(restored.entropy.is_none());
        assert!(restored.acting_player_id.is_none());
        assert!(restored.defender_id.is_none());
        assert!(!restored.is_using_stab());
        assert!(!restored.is_using_chainsaw());
    }
}
