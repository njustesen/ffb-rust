use ffb_model::model::game_list::GameList;
use ffb_model::enums::NetCommandId;
use crate::commands::server_command::ServerCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandGameList`.
/// Sends the lobby game list to the client.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandGameList {
    /// Java: base-class `ServerCommand.fCommandNr`.
    pub command_nr: i32,
    /// Java: `fGameList` — the list of available games.
    pub game_list: GameList,
}

impl ServerCommandGameList {
    pub fn new(game_list: GameList) -> Self { Self { command_nr: 0, game_list } }
    pub fn get_game_list(&self) -> &GameList { &self.game_list }

    /// Java: `isReplayable()`.
    pub fn is_replayable(&self) -> bool {
        false
    }

    /// Java: `ServerCommandGameList.toJsonValue()`. `GameList` has no
    /// Java-matching `to_json_value()` of its own yet, so its serde derive
    /// is used for the nested `gameList` object — this keeps every field
    /// but with Rust field names rather than Java's `gameListEntries` shape.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ServerCommand { command_nr: self.command_nr };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("gameList".to_string(), serde_json::to_value(&self.game_list).unwrap_or(serde_json::Value::Null));
        serde_json::Value::Object(map)
    }

    /// Java: `ServerCommandGameList.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ServerCommand::base_from_json(json);
        let game_list = json
            .get("gameList")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();
        Self { command_nr: base.command_nr, game_list }
    }
}

impl NetCommand for ServerCommandGameList {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerGameList
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ServerCommandGameList::new(GameList::default());
        let _ = cmd.get_game_list();
    }

    #[test]
    fn default_works() {
        let _ = ServerCommandGameList::default();
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandGameList::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ServerCommandGameList::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ServerCommandGameList::default());
        assert!(s.contains("ServerCommandGameList"));
    }

    #[test]
    fn get_id_is_server_game_list() {
        assert_eq!(ServerCommandGameList::default().get_id(), NetCommandId::ServerGameList);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_game_list() {
        let mut cmd = ServerCommandGameList::new(GameList::default());
        cmd.command_nr = 2;
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "serverGameList");
        assert_eq!(json["commandNr"], 2);
        assert!(json.get("gameList").is_some());
    }

    #[test]
    fn round_trip_with_entries() {
        use ffb_model::model::game_list_entry::GameListEntry;
        let mut list = GameList::default();
        list.add(GameListEntry::default());
        let mut cmd = ServerCommandGameList::new(list);
        cmd.command_nr = 9;
        let json = cmd.to_json_value();
        let restored = ServerCommandGameList::from_json(&json);
        assert_eq!(restored.command_nr, 9);
        assert_eq!(restored.game_list.len(), 1);
    }

    #[test]
    fn round_trip_with_empty_game_list() {
        let cmd = ServerCommandGameList::default();
        let json = cmd.to_json_value();
        let restored = ServerCommandGameList::from_json(&json);
        assert!(restored.game_list.is_empty());
    }
}
