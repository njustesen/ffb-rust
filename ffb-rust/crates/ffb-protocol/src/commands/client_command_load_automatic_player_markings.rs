/// 1:1 translation of com.fumbbl.ffb.net.commands.ClientCommandLoadAutomaticPlayerMarkings.
///
/// Java: `game` field is the full `Game` object (`IJsonOption.GAME`, wire key `"game"`,
/// serialized via `game.toJsonValue()` / deserialized via `new Game(...).initFrom(...)`).
/// `ffb_model::model::Game` already derives `Serialize`/`Deserialize`, so it round-trips via
/// `serde_json::to_value`/`from_value` rather than a hand-rolled `to_json_value`/`from_json`
/// pair (its wire shape therefore differs from the Java minimal-json layout, but nothing is
/// silently dropped).
use ffb_model::enums::NetCommandId;
use ffb_model::model::Game;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

#[derive(Debug, Clone, Default)]
pub struct ClientCommandLoadAutomaticPlayerMarkings {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `index`
    pub index: i32,
    /// Java: `coach`
    pub coach: Option<String>,
    /// Java: `game`
    pub game: Option<Game>,
}

impl ClientCommandLoadAutomaticPlayerMarkings {
    pub fn new() -> Self {
        Self::default()
    }

    /// Java: `getIndex()`
    pub fn get_index(&self) -> i32 {
        self.index
    }

    /// Java: `getCoach()`
    pub fn get_coach(&self) -> Option<&str> {
        self.coach.as_deref()
    }

    /// Java: `getGame()`
    pub fn get_game(&self) -> Option<&Game> {
        self.game.as_ref()
    }

    /// Java: `ClientCommandLoadAutomaticPlayerMarkings.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("selectedIndex".to_string(), serde_json::json!(self.index));
        if let Some(game) = &self.game {
            map.insert("game".to_string(), serde_json::to_value(game).unwrap_or(serde_json::Value::Null));
        }
        map.insert("coach".to_string(), serde_json::json!(self.coach));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandLoadAutomaticPlayerMarkings.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            index: json.get("selectedIndex").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            coach: json.get("coach").and_then(|v| v.as_str()).map(String::from),
            game: json.get("game").and_then(|v| serde_json::from_value(v.clone()).ok()),
        }
    }
}

impl NetCommand for ClientCommandLoadAutomaticPlayerMarkings {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientLoadAutomaticPlayerMarkings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_index_is_zero() {
        let cmd = ClientCommandLoadAutomaticPlayerMarkings::new();
        assert_eq!(cmd.get_index(), 0);
    }

    #[test]
    fn stores_index_and_coach() {
        let cmd = ClientCommandLoadAutomaticPlayerMarkings {
            entropy: None,
            index: 3,
            coach: Some("CoachB".to_string()),
            game: None,
        };
        assert_eq!(cmd.get_index(), 3);
        assert_eq!(cmd.get_coach(), Some("CoachB"));
    }

    #[test]
    fn coach_none_by_default() {
        let cmd = ClientCommandLoadAutomaticPlayerMarkings::default();
        assert!(cmd.get_coach().is_none());
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandLoadAutomaticPlayerMarkings::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandLoadAutomaticPlayerMarkings::default().clone();
    }

    #[test]
    fn get_id_is_client_load_automatic_player_markings() {
        assert_eq!(
            ClientCommandLoadAutomaticPlayerMarkings::new().get_id(),
            NetCommandId::ClientLoadAutomaticPlayerMarkings
        );
    }

    #[test]
    fn to_json_value_has_net_command_id_and_selected_index() {
        let cmd = ClientCommandLoadAutomaticPlayerMarkings {
            entropy: None,
            index: 5,
            coach: Some("CoachB".to_string()),
            game: None,
        };
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientLoadPlayerMarkings");
        assert_eq!(json["selectedIndex"], 5);
        assert_eq!(json["coach"], "CoachB");
    }

    #[test]
    fn round_trip_with_index_coach_and_entropy() {
        let cmd = ClientCommandLoadAutomaticPlayerMarkings {
            entropy: Some(6),
            index: 2,
            coach: Some("CoachC".to_string()),
            game: None,
        };
        let json = cmd.to_json_value();
        let restored = ClientCommandLoadAutomaticPlayerMarkings::from_json(&json);
        assert_eq!(restored.entropy, Some(6));
        assert_eq!(restored.get_index(), 2);
        assert_eq!(restored.get_coach(), Some("CoachC"));
        assert!(restored.get_game().is_none());
    }

    #[test]
    fn round_trip_default() {
        let cmd = ClientCommandLoadAutomaticPlayerMarkings::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandLoadAutomaticPlayerMarkings::from_json(&json);
        assert_eq!(restored.get_index(), 0);
        assert!(restored.get_coach().is_none());
        assert!(restored.get_game().is_none());
    }
}
