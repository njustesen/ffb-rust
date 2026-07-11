use ffb_model::model::team_list::TeamList;
use ffb_model::enums::NetCommandId;
use crate::commands::server_command::ServerCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandTeamList`.
/// Sends the lobby team list to the client.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandTeamList {
    /// Java: base-class `ServerCommand.fCommandNr`.
    pub command_nr: i32,
    /// Java: `fTeamList` — the list of available teams. `None` corresponds
    /// to Java's `fTeamList == null`.
    pub team_list: Option<TeamList>,
}

impl ServerCommandTeamList {
    pub fn new(team_list: TeamList) -> Self { Self { command_nr: 0, team_list: Some(team_list) } }
    pub fn get_team_list(&self) -> Option<&TeamList> { self.team_list.as_ref() }

    /// Java: `ServerCommandTeamList.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ServerCommand { command_nr: self.command_nr };
        let mut map = base.base_json_fields(self.get_id());
        if let Some(team_list) = &self.team_list {
            map.insert("teamList".to_string(), team_list.to_json_value());
        }
        serde_json::Value::Object(map)
    }

    /// Java: `ServerCommandTeamList.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ServerCommand::base_from_json(json);
        let team_list = json.get("teamList").map(TeamList::from_json);
        Self { command_nr: base.command_nr, team_list }
    }
}

impl NetCommand for ServerCommandTeamList {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerTeamList
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ServerCommandTeamList::new(TeamList::default());
        let _ = cmd.get_team_list();
    }

    #[test]
    fn default_works() {
        let _ = ServerCommandTeamList::default();
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandTeamList::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ServerCommandTeamList::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ServerCommandTeamList::default());
        assert!(s.contains("ServerCommandTeamList"));
    }

    #[test]
    fn get_id_is_server_team_list() {
        assert_eq!(ServerCommandTeamList::default().get_id(), NetCommandId::ServerTeamList);
    }

    #[test]
    fn to_json_value_has_net_command_id() {
        let cmd = ServerCommandTeamList::default();
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "serverTeamList");
        assert!(json.get("teamList").is_none());
    }

    #[test]
    fn round_trip_with_team_list() {
        use ffb_model::model::team_list_entry::TeamListEntry;
        let mut tl = TeamList::default();
        tl.coach = Some("Alice".to_string());
        tl.add(TeamListEntry { team_id: "t1".into(), team_name: "Orcs".into(), coach: String::new(), race: "Orc".into() });
        let mut cmd = ServerCommandTeamList::new(tl);
        cmd.command_nr = 2;
        let json = cmd.to_json_value();
        let restored = ServerCommandTeamList::from_json(&json);
        assert_eq!(restored.command_nr, 2);
        let restored_list = restored.get_team_list().unwrap();
        assert_eq!(restored_list.coach, Some("Alice".to_string()));
        assert_eq!(restored_list.entries.len(), 1);
        assert_eq!(restored_list.entries[0].team_name, "Orcs");
    }

    #[test]
    fn round_trip_with_no_team_list() {
        let cmd = ServerCommandTeamList::default();
        let json = cmd.to_json_value();
        let restored = ServerCommandTeamList::from_json(&json);
        assert!(restored.get_team_list().is_none());
    }
}
