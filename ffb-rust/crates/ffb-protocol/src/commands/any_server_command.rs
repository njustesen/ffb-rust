//! Real dispatch layer over the 32 genuine 1:1-translated `ServerCommand*` structs.
//! 1:1 translation of the dispatch role played by `NetCommandId.createNetCommand()`
//! (the server-command half of the switch) plus `NetCommand`/`NetCommandFactory`.
//!
//! This is additive: the pre-existing `server_commands::ServerCommand` (a hand-rolled,
//! not-1:1 simplification used by the live WebSocket layer today) is untouched.

use ffb_model::enums::NetCommandId;
use crate::net_command::NetCommand;
use crate::commands::server_command_add_player::ServerCommandAddPlayer;
use crate::commands::server_command_add_sketches::ServerCommandAddSketches;
use crate::commands::server_command_admin_message::ServerCommandAdminMessage;
use crate::commands::server_command_automatic_player_markings::ServerCommandAutomaticPlayerMarkings;
use crate::commands::server_command_clear_sketches::ServerCommandClearSketches;
use crate::commands::server_command_game_list::ServerCommandGameList;
use crate::commands::server_command_game_state::ServerCommandGameState;
use crate::commands::server_command_game_time::ServerCommandGameTime;
use crate::commands::server_command_join::ServerCommandJoin;
use crate::commands::server_command_leave::ServerCommandLeave;
use crate::commands::server_command_model_sync::ServerCommandModelSync;
use crate::commands::server_command_password_challenge::ServerCommandPasswordChallenge;
use crate::commands::server_command_pong::ServerCommandPong;
use crate::commands::server_command_remove_player::ServerCommandRemovePlayer;
use crate::commands::server_command_remove_sketches::ServerCommandRemoveSketches;
use crate::commands::server_command_replay::ServerCommandReplay;
use crate::commands::server_command_replay_control::ServerCommandReplayControl;
use crate::commands::server_command_replay_status::ServerCommandReplayStatus;
use crate::commands::server_command_set_prevent_sketching::ServerCommandSetPreventSketching;
use crate::commands::server_command_sketch_add_coordinate::ServerCommandSketchAddCoordinate;
use crate::commands::server_command_sketch_set_color::ServerCommandSketchSetColor;
use crate::commands::server_command_sketch_set_label::ServerCommandSketchSetLabel;
use crate::commands::server_command_sound::ServerCommandSound;
use crate::commands::server_command_status::ServerCommandStatus;
use crate::commands::server_command_talk::ServerCommandTalk;
use crate::commands::server_command_team_list::ServerCommandTeamList;
use crate::commands::server_command_team_setup_list::ServerCommandTeamSetupList;
use crate::commands::server_command_unzap_player::ServerCommandUnzapPlayer;
use crate::commands::server_command_update_local_player_markers::ServerCommandUpdateLocalPlayerMarkers;
use crate::commands::server_command_user_settings::ServerCommandUserSettings;
use crate::commands::server_command_version::ServerCommandVersion;
use crate::commands::server_command_zap_player::ServerCommandZapPlayer;

/// Sum type over every genuine `ServerCommand*` struct, keyed the same way Java's
/// `NetCommandId.createNetCommand()` switch keys its instantiation.
///
/// Not `Clone`: `ServerCommandModelSync` holds a `ReportList` of `Box<dyn IReport>`
/// trait objects with no `Clone` impl (same pre-existing limitation noted in
/// `server_command_model_sync.rs`).
#[derive(Debug)]
pub enum AnyServerCommand {
    ServerAddPlayer(ServerCommandAddPlayer),
    ServerAddSketches(ServerCommandAddSketches),
    ServerAdminMessage(ServerCommandAdminMessage),
    ServerAutomaticPlayerMarkings(ServerCommandAutomaticPlayerMarkings),
    ServerClearSketches(ServerCommandClearSketches),
    ServerGameList(ServerCommandGameList),
    ServerGameState(ServerCommandGameState),
    ServerGameTime(ServerCommandGameTime),
    ServerJoin(ServerCommandJoin),
    ServerLeave(ServerCommandLeave),
    ServerModelSync(ServerCommandModelSync),
    ServerPasswordChallenge(ServerCommandPasswordChallenge),
    ServerPong(ServerCommandPong),
    ServerRemovePlayer(ServerCommandRemovePlayer),
    ServerRemoveSketches(ServerCommandRemoveSketches),
    ServerReplay(ServerCommandReplay),
    ServerReplayControl(ServerCommandReplayControl),
    ServerReplayStatus(ServerCommandReplayStatus),
    ServerSetPreventSketching(ServerCommandSetPreventSketching),
    ServerSketchAddCoordinate(ServerCommandSketchAddCoordinate),
    ServerSketchSetColor(ServerCommandSketchSetColor),
    ServerSketchSetLabel(ServerCommandSketchSetLabel),
    ServerSound(ServerCommandSound),
    ServerStatus(ServerCommandStatus),
    ServerTalk(ServerCommandTalk),
    ServerTeamList(ServerCommandTeamList),
    ServerTeamSetupList(ServerCommandTeamSetupList),
    ServerUnzapPlayer(ServerCommandUnzapPlayer),
    ServerUpdateLocalPlayerMarkers(ServerCommandUpdateLocalPlayerMarkers),
    ServerUserSettings(ServerCommandUserSettings),
    ServerVersion(ServerCommandVersion),
    ServerZapPlayer(ServerCommandZapPlayer),
}

impl NetCommand for AnyServerCommand {
    fn get_id(&self) -> NetCommandId {
        match self {
            AnyServerCommand::ServerAddPlayer(_) => NetCommandId::ServerAddPlayer,
            AnyServerCommand::ServerAddSketches(_) => NetCommandId::ServerAddSketches,
            AnyServerCommand::ServerAdminMessage(_) => NetCommandId::ServerAdminMessage,
            AnyServerCommand::ServerAutomaticPlayerMarkings(_) => NetCommandId::ServerAutomaticPlayerMarkings,
            AnyServerCommand::ServerClearSketches(_) => NetCommandId::ServerClearSketches,
            AnyServerCommand::ServerGameList(_) => NetCommandId::ServerGameList,
            AnyServerCommand::ServerGameState(_) => NetCommandId::ServerGameState,
            AnyServerCommand::ServerGameTime(_) => NetCommandId::ServerGameTime,
            AnyServerCommand::ServerJoin(_) => NetCommandId::ServerJoin,
            AnyServerCommand::ServerLeave(_) => NetCommandId::ServerLeave,
            AnyServerCommand::ServerModelSync(_) => NetCommandId::ServerModelSync,
            AnyServerCommand::ServerPasswordChallenge(_) => NetCommandId::ServerPasswordChallenge,
            AnyServerCommand::ServerPong(_) => NetCommandId::ServerPong,
            AnyServerCommand::ServerRemovePlayer(_) => NetCommandId::ServerRemovePlayer,
            AnyServerCommand::ServerRemoveSketches(_) => NetCommandId::ServerRemoveSketches,
            AnyServerCommand::ServerReplay(_) => NetCommandId::ServerReplay,
            AnyServerCommand::ServerReplayControl(_) => NetCommandId::ServerReplayControl,
            AnyServerCommand::ServerReplayStatus(_) => NetCommandId::ServerReplayStatus,
            AnyServerCommand::ServerSetPreventSketching(_) => NetCommandId::ServerSetPreventSketching,
            AnyServerCommand::ServerSketchAddCoordinate(_) => NetCommandId::ServerSketchAddCoordinate,
            AnyServerCommand::ServerSketchSetColor(_) => NetCommandId::ServerSketchSetColor,
            AnyServerCommand::ServerSketchSetLabel(_) => NetCommandId::ServerSketchSetLabel,
            AnyServerCommand::ServerSound(_) => NetCommandId::ServerSound,
            AnyServerCommand::ServerStatus(_) => NetCommandId::ServerStatus,
            AnyServerCommand::ServerTalk(_) => NetCommandId::ServerTalk,
            AnyServerCommand::ServerTeamList(_) => NetCommandId::ServerTeamList,
            AnyServerCommand::ServerTeamSetupList(_) => NetCommandId::ServerTeamSetupList,
            AnyServerCommand::ServerUnzapPlayer(_) => NetCommandId::ServerUnzapPlayer,
            AnyServerCommand::ServerUpdateLocalPlayerMarkers(_) => NetCommandId::ServerUpdateLocalPlayerMarkers,
            AnyServerCommand::ServerUserSettings(_) => NetCommandId::ServerUserSettings,
            AnyServerCommand::ServerVersion(_) => NetCommandId::ServerVersion,
            AnyServerCommand::ServerZapPlayer(_) => NetCommandId::ServerZapPlayer,
        }
    }
}

impl AnyServerCommand {
    /// Java: `NetCommand.toJsonValue()` dispatched to the concrete subclass.
    pub fn to_json_value(&self) -> serde_json::Value {
        match self {
            AnyServerCommand::ServerAddPlayer(c) => c.to_json_value(),
            AnyServerCommand::ServerAddSketches(c) => c.to_json_value(),
            AnyServerCommand::ServerAdminMessage(c) => c.to_json_value(),
            AnyServerCommand::ServerAutomaticPlayerMarkings(c) => c.to_json_value(),
            AnyServerCommand::ServerClearSketches(c) => c.to_json_value(),
            AnyServerCommand::ServerGameList(c) => c.to_json_value(),
            AnyServerCommand::ServerGameState(c) => c.to_json_value(),
            AnyServerCommand::ServerGameTime(c) => c.to_json_value(),
            AnyServerCommand::ServerJoin(c) => c.to_json_value(),
            AnyServerCommand::ServerLeave(c) => c.to_json_value(),
            AnyServerCommand::ServerModelSync(c) => c.to_json_value(),
            AnyServerCommand::ServerPasswordChallenge(c) => c.to_json_value(),
            AnyServerCommand::ServerPong(c) => c.to_json_value(),
            AnyServerCommand::ServerRemovePlayer(c) => c.to_json_value(),
            AnyServerCommand::ServerRemoveSketches(c) => c.to_json_value(),
            AnyServerCommand::ServerReplay(c) => c.to_json_value(),
            AnyServerCommand::ServerReplayControl(c) => c.to_json_value(),
            AnyServerCommand::ServerReplayStatus(c) => c.to_json_value(),
            AnyServerCommand::ServerSetPreventSketching(c) => c.to_json_value(),
            AnyServerCommand::ServerSketchAddCoordinate(c) => c.to_json_value(),
            AnyServerCommand::ServerSketchSetColor(c) => c.to_json_value(),
            AnyServerCommand::ServerSketchSetLabel(c) => c.to_json_value(),
            AnyServerCommand::ServerSound(c) => c.to_json_value(),
            AnyServerCommand::ServerStatus(c) => c.to_json_value(),
            AnyServerCommand::ServerTalk(c) => c.to_json_value(),
            AnyServerCommand::ServerTeamList(c) => c.to_json_value(),
            AnyServerCommand::ServerTeamSetupList(c) => c.to_json_value(),
            AnyServerCommand::ServerUnzapPlayer(c) => c.to_json_value(),
            AnyServerCommand::ServerUpdateLocalPlayerMarkers(c) => c.to_json_value(),
            AnyServerCommand::ServerUserSettings(c) => c.to_json_value(),
            AnyServerCommand::ServerVersion(c) => c.to_json_value(),
            AnyServerCommand::ServerZapPlayer(c) => c.to_json_value(),
        }
    }

    /// Java: `NetCommandId.createNetCommand()` + `NetCommand.initFrom(...)`.
    pub fn from_json(id: NetCommandId, json: &serde_json::Value) -> Option<Self> {
        Some(match id {
            NetCommandId::ServerAddPlayer => AnyServerCommand::ServerAddPlayer(ServerCommandAddPlayer::from_json(json)),
            NetCommandId::ServerAddSketches => AnyServerCommand::ServerAddSketches(ServerCommandAddSketches::from_json(json)),
            NetCommandId::ServerAdminMessage => AnyServerCommand::ServerAdminMessage(ServerCommandAdminMessage::from_json(json)),
            NetCommandId::ServerAutomaticPlayerMarkings => AnyServerCommand::ServerAutomaticPlayerMarkings(ServerCommandAutomaticPlayerMarkings::from_json(json)),
            NetCommandId::ServerClearSketches => AnyServerCommand::ServerClearSketches(ServerCommandClearSketches::from_json(json)),
            NetCommandId::ServerGameList => AnyServerCommand::ServerGameList(ServerCommandGameList::from_json(json)),
            NetCommandId::ServerGameState => AnyServerCommand::ServerGameState(ServerCommandGameState::from_json(json)),
            NetCommandId::ServerGameTime => AnyServerCommand::ServerGameTime(ServerCommandGameTime::from_json(json)),
            NetCommandId::ServerJoin => AnyServerCommand::ServerJoin(ServerCommandJoin::from_json(json)),
            NetCommandId::ServerLeave => AnyServerCommand::ServerLeave(ServerCommandLeave::from_json(json)),
            NetCommandId::ServerModelSync => AnyServerCommand::ServerModelSync(ServerCommandModelSync::from_json(json)),
            NetCommandId::ServerPasswordChallenge => AnyServerCommand::ServerPasswordChallenge(ServerCommandPasswordChallenge::from_json(json)),
            NetCommandId::ServerPong => AnyServerCommand::ServerPong(ServerCommandPong::from_json(json)),
            NetCommandId::ServerRemovePlayer => AnyServerCommand::ServerRemovePlayer(ServerCommandRemovePlayer::from_json(json)),
            NetCommandId::ServerRemoveSketches => AnyServerCommand::ServerRemoveSketches(ServerCommandRemoveSketches::from_json(json)),
            NetCommandId::ServerReplay => AnyServerCommand::ServerReplay(ServerCommandReplay::from_json(json)),
            NetCommandId::ServerReplayControl => AnyServerCommand::ServerReplayControl(ServerCommandReplayControl::from_json(json)),
            NetCommandId::ServerReplayStatus => AnyServerCommand::ServerReplayStatus(ServerCommandReplayStatus::from_json(json)),
            NetCommandId::ServerSetPreventSketching => AnyServerCommand::ServerSetPreventSketching(ServerCommandSetPreventSketching::from_json(json)),
            NetCommandId::ServerSketchAddCoordinate => AnyServerCommand::ServerSketchAddCoordinate(ServerCommandSketchAddCoordinate::from_json(json)),
            NetCommandId::ServerSketchSetColor => AnyServerCommand::ServerSketchSetColor(ServerCommandSketchSetColor::from_json(json)),
            NetCommandId::ServerSketchSetLabel => AnyServerCommand::ServerSketchSetLabel(ServerCommandSketchSetLabel::from_json(json)),
            NetCommandId::ServerSound => AnyServerCommand::ServerSound(ServerCommandSound::from_json(json)),
            NetCommandId::ServerStatus => AnyServerCommand::ServerStatus(ServerCommandStatus::from_json(json)),
            NetCommandId::ServerTalk => AnyServerCommand::ServerTalk(ServerCommandTalk::from_json(json)),
            NetCommandId::ServerTeamList => AnyServerCommand::ServerTeamList(ServerCommandTeamList::from_json(json)),
            NetCommandId::ServerTeamSetupList => AnyServerCommand::ServerTeamSetupList(ServerCommandTeamSetupList::from_json(json)),
            NetCommandId::ServerUnzapPlayer => AnyServerCommand::ServerUnzapPlayer(ServerCommandUnzapPlayer::from_json(json)),
            NetCommandId::ServerUpdateLocalPlayerMarkers => AnyServerCommand::ServerUpdateLocalPlayerMarkers(ServerCommandUpdateLocalPlayerMarkers::from_json(json)),
            NetCommandId::ServerUserSettings => AnyServerCommand::ServerUserSettings(ServerCommandUserSettings::from_json(json)),
            NetCommandId::ServerVersion => AnyServerCommand::ServerVersion(ServerCommandVersion::from_json(json)),
            NetCommandId::ServerZapPlayer => AnyServerCommand::ServerZapPlayer(ServerCommandZapPlayer::from_json(json)),
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_id_matches_wrapped_variant() {
        let cmd = AnyServerCommand::ServerGameTime(ServerCommandGameTime::new(1, 2));
        assert_eq!(cmd.get_id(), NetCommandId::ServerGameTime);
    }

    #[test]
    fn to_json_value_delegates_to_wrapped_command() {
        let cmd = AnyServerCommand::ServerGameTime(ServerCommandGameTime::new(1, 2));
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "serverGameTime");
    }

    #[test]
    fn from_json_round_trip_for_known_id() {
        let original = AnyServerCommand::ServerGameTime(ServerCommandGameTime::new(60_000, 30_000));
        let json = original.to_json_value();
        let restored = AnyServerCommand::from_json(NetCommandId::ServerGameTime, &json).unwrap();
        assert_eq!(restored.get_id(), NetCommandId::ServerGameTime);
    }

    #[test]
    fn from_json_returns_none_for_a_client_only_id() {
        let json = serde_json::json!({});
        assert!(AnyServerCommand::from_json(NetCommandId::ClientEndTurn, &json).is_none());
    }

    #[test]
    fn from_json_dispatches_to_the_matching_struct_type() {
        let json = serde_json::json!({"netCommandId": "serverPong"});
        let restored = AnyServerCommand::from_json(NetCommandId::ServerPong, &json).unwrap();
        assert!(matches!(restored, AnyServerCommand::ServerPong(_)));
    }
}
