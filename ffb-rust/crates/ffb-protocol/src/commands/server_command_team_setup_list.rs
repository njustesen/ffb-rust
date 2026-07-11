use ffb_model::enums::NetCommandId;
use crate::commands::server_command::ServerCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandTeamSetupList`.
/// Sends the list of saved team setup names to the client.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandTeamSetupList {
    /// Java: base-class `ServerCommand.fCommandNr`.
    pub command_nr: i32,
    /// Java: `fSetupNames` — ordered list of saved setup names.
    pub setup_names: Vec<String>,
}

impl ServerCommandTeamSetupList {
    pub fn new(setup_names: Vec<String>) -> Self { Self { command_nr: 0, setup_names } }
    pub fn get_setup_names(&self) -> &[String] { &self.setup_names }
    pub fn add_setup_name(&mut self, name: impl Into<String>) { self.setup_names.push(name.into()); }

    /// Java: `ServerCommandTeamSetupList.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ServerCommand { command_nr: self.command_nr };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("setupNames".to_string(), serde_json::json!(self.setup_names));
        serde_json::Value::Object(map)
    }

    /// Java: `ServerCommandTeamSetupList.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ServerCommand::base_from_json(json);
        let setup_names: Vec<String> = json
            .get("setupNames")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
            .unwrap_or_default();
        Self { command_nr: base.command_nr, setup_names }
    }
}

impl NetCommand for ServerCommandTeamSetupList {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerTeamSetupList
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ServerCommandTeamSetupList::new(vec!["Wide".to_string(), "Cage".to_string()]);
        assert_eq!(cmd.get_setup_names(), &["Wide", "Cage"]);
    }

    #[test]
    fn add_name() {
        let mut cmd = ServerCommandTeamSetupList::default();
        cmd.add_setup_name("Press");
        assert_eq!(cmd.setup_names.len(), 1);
        assert_eq!(cmd.setup_names[0], "Press");
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandTeamSetupList::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ServerCommandTeamSetupList::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ServerCommandTeamSetupList::default());
        assert!(s.contains("ServerCommandTeamSetupList"));
    }

    #[test]
    fn get_id_is_server_team_setup_list() {
        assert_eq!(ServerCommandTeamSetupList::default().get_id(), NetCommandId::ServerTeamSetupList);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_setup_names() {
        let cmd = ServerCommandTeamSetupList::new(vec!["Wide".into()]);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "serverTeamSetupList");
        assert_eq!(json["setupNames"][0], "Wide");
    }

    #[test]
    fn round_trip_with_names() {
        let mut cmd = ServerCommandTeamSetupList::new(vec!["Wide".into(), "Cage".into()]);
        cmd.command_nr = 6;
        let json = cmd.to_json_value();
        let restored = ServerCommandTeamSetupList::from_json(&json);
        assert_eq!(restored.command_nr, 6);
        assert_eq!(restored.setup_names, vec!["Wide".to_string(), "Cage".to_string()]);
    }

    #[test]
    fn round_trip_with_no_names() {
        let cmd = ServerCommandTeamSetupList::default();
        let json = cmd.to_json_value();
        let restored = ServerCommandTeamSetupList::from_json(&json);
        assert!(restored.setup_names.is_empty());
    }
}
