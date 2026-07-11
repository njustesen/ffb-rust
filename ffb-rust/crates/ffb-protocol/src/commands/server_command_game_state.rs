use serde::{Deserialize, Serialize};
use ffb_model::model::game::Game;
use ffb_model::enums::NetCommandId;
use ffb_model::model::factory_type::FactoryContext;
use crate::commands::server_command::ServerCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandGameState`.
/// Carries a full `Game` snapshot; sent on join and after reconnects.
/// Java: extends `ServerCommand`; `isReplayable()` returns `false`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCommandGameState {
    /// Java: `fCommandNr` inherited from `ServerCommand`.
    pub command_nr: i32,
    /// Java: `fGame` — the complete game state.
    pub game: Option<Box<Game>>,
}

impl ServerCommandGameState {
    pub const ID: NetCommandId = NetCommandId::ServerGameState;

    pub fn new(game: Option<Game>) -> Self {
        Self {
            command_nr: 0,
            game: game.map(Box::new),
        }
    }

    /// Java: `isReplayable()` — game state snapshots are NOT stored in replays.
    pub fn is_replayable(&self) -> bool {
        false
    }

    pub fn id(&self) -> NetCommandId {
        Self::ID
    }

    /// Java: `ServerCommandGameState.toJsonValue()`. `Game` has no
    /// Java-matching `to_json_value()` of its own yet, so its serde derive
    /// is used for the nested `game` object — every field is preserved but
    /// under Rust field names rather than Java's camelCase keys.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ServerCommand { command_nr: self.command_nr };
        let mut map = base.base_json_fields(self.get_id());
        if let Some(game) = &self.game {
            map.insert("game".to_string(), serde_json::to_value(game.as_ref()).unwrap_or(serde_json::Value::Null));
        }
        serde_json::Value::Object(map)
    }

    /// Java: `ServerCommandGameState.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ServerCommand::base_from_json(json);
        let game = json
            .get("game")
            .and_then(|v| if v.is_null() { None } else { serde_json::from_value(v.clone()).ok() })
            .map(Box::new);
        Self { command_nr: base.command_nr, game }
    }
}

impl NetCommand for ServerCommandGameState {
    fn get_id(&self) -> NetCommandId {
        Self::ID
    }

    /// Java: `getContext()` override — returns `FactoryContext.APPLICATION`.
    fn get_context(&self) -> FactoryContext {
        FactoryContext::APPLICATION
    }
}

impl Default for ServerCommandGameState {
    fn default() -> Self {
        Self::new(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_replayable() {
        assert!(!ServerCommandGameState::default().is_replayable());
    }

    #[test]
    fn id_is_server_game_state() {
        assert_eq!(ServerCommandGameState::default().id(), NetCommandId::ServerGameState);
    }

    #[test]
    fn new_with_no_game() {
        let cmd = ServerCommandGameState::new(None);
        assert!(cmd.game.is_none());
    }

    #[test]
    fn serde_round_trip_no_game() {
        let cmd = ServerCommandGameState { command_nr: 5, game: None };
        let json = serde_json::to_string(&cmd).unwrap();
        let back: ServerCommandGameState = serde_json::from_str(&json).unwrap();
        assert_eq!(back.command_nr, 5);
        assert!(back.game.is_none());
    }

    #[test]
    fn get_id_via_net_command_trait() {
        assert_eq!(NetCommand::get_id(&ServerCommandGameState::default()), NetCommandId::ServerGameState);
    }

    #[test]
    fn get_context_is_application() {
        assert_eq!(NetCommand::get_context(&ServerCommandGameState::default()), FactoryContext::APPLICATION);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_no_game() {
        let mut cmd = ServerCommandGameState::default();
        cmd.command_nr = 3;
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "serverGameState");
        assert_eq!(json["commandNr"], 3);
        assert!(json.get("game").is_none());
    }

    fn empty_team(id: &str) -> ffb_model::model::team::Team {
        ffb_model::model::team::Team {
            id: id.into(),
            name: id.into(),
            race: "Human".into(),
            roster_id: "human".into(),
            coach: "Coach".into(),
            rerolls: 0,
            apothecaries: 0,
            bribes: 0,
            master_chefs: 0,
            prayers_to_nuffle: 0,
            bloodweiser_kegs: 0,
            riotous_rookies: 0,
            cheerleaders: 0,
            assistant_coaches: 0,
            fan_factor: 0,
            dedicated_fans: 0,
            team_value: 0,
            treasury: 0,
            special_rules: vec![],
            players: vec![],
            vampire_lord: false,
            necromancer: false,
        }
    }

    #[test]
    fn round_trip_with_game() {
        use ffb_model::enums::Rules;
        let mut cmd = ServerCommandGameState::new(Some(Game::new(empty_team("home"), empty_team("away"), Rules::Bb2020)));
        cmd.command_nr = 11;
        let json = cmd.to_json_value();
        let restored = ServerCommandGameState::from_json(&json);
        assert_eq!(restored.command_nr, 11);
        assert!(restored.game.is_some());
    }

    #[test]
    fn round_trip_with_no_game() {
        let cmd = ServerCommandGameState::new(None);
        let json = cmd.to_json_value();
        let restored = ServerCommandGameState::from_json(&json);
        assert!(restored.game.is_none());
    }
}
