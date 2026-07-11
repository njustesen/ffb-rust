/// Rust-only sum type over the 14 concrete `internal_server_command_*` structs, mirroring
/// Java's `InternalServerCommand extends NetCommand` — the queue carries a `NetCommand` that
/// is either a `ClientCommand` or an `InternalServerCommand`; this enum is the "internal" half
/// of that union (see `ffb_protocol::client_commands::ClientCommand` for the client half, and
/// `crate::model::received_command::ReceivedNetCommand` for where the two are joined).
///
/// Precedent: `ffb_protocol::commands::any_server_command::AnyServerCommand` /
/// `any_client_command::AnyClientCommand` (Phase ZW.2c) use the same pattern for the
/// server↔client wire command hierarchy.
use super::internal_server_command::InternalServerCommand;
use super::internal_server_command_add_loaded_team::InternalServerCommandAddLoadedTeam;
use super::internal_server_command_apply_automated_player_markings::InternalServerCommandApplyAutomatedPlayerMarkings;
use super::internal_server_command_calculate_automatic_player_markings::InternalServerCommandCalculateAutomaticPlayerMarkings;
use super::internal_server_command_clear_cache::InternalServerCommandClearCache;
use super::internal_server_command_close_game::InternalServerCommandCloseGame;
use super::internal_server_command_delete_game::InternalServerCommandDeleteGame;
use super::internal_server_command_fumbbl_game_checked::InternalServerCommandFumbblGameChecked;
use super::internal_server_command_fumbbl_game_created::InternalServerCommandFumbblGameCreated;
use super::internal_server_command_fumbbl_team_loaded::InternalServerCommandFumbblTeamLoaded;
use super::internal_server_command_join_approved::InternalServerCommandJoinApproved;
use super::internal_server_command_replay_loaded::InternalServerCommandReplayLoaded;
use super::internal_server_command_schedule_game::InternalServerCommandScheduleGame;
use super::internal_server_command_socket_closed::InternalServerCommandSocketClosed;
use super::internal_server_command_upload_game::InternalServerCommandUploadGame;

pub enum AnyInternalServerCommand {
    AddLoadedTeam(InternalServerCommandAddLoadedTeam),
    ApplyAutomatedPlayerMarkings(InternalServerCommandApplyAutomatedPlayerMarkings),
    CalculateAutomaticPlayerMarkings(InternalServerCommandCalculateAutomaticPlayerMarkings),
    ClearCache(InternalServerCommandClearCache),
    CloseGame(InternalServerCommandCloseGame),
    DeleteGame(InternalServerCommandDeleteGame),
    FumbblGameChecked(InternalServerCommandFumbblGameChecked),
    FumbblGameCreated(InternalServerCommandFumbblGameCreated),
    FumbblTeamLoaded(InternalServerCommandFumbblTeamLoaded),
    JoinApproved(InternalServerCommandJoinApproved),
    ReplayLoaded(InternalServerCommandReplayLoaded),
    ScheduleGame(InternalServerCommandScheduleGame),
    SocketClosed(InternalServerCommandSocketClosed),
    UploadGame(InternalServerCommandUploadGame),
}

impl AnyInternalServerCommand {
    /// Java: `NetCommand.getId()` — dispatched to whichever concrete command is wrapped.
    pub fn get_id(&self) -> &'static str {
        match self {
            AnyInternalServerCommand::AddLoadedTeam(c) => c.get_id(),
            AnyInternalServerCommand::ApplyAutomatedPlayerMarkings(c) => c.get_id(),
            AnyInternalServerCommand::CalculateAutomaticPlayerMarkings(c) => c.get_id(),
            AnyInternalServerCommand::ClearCache(c) => c.get_id(),
            AnyInternalServerCommand::CloseGame(c) => c.get_id(),
            AnyInternalServerCommand::DeleteGame(c) => c.get_id(),
            AnyInternalServerCommand::FumbblGameChecked(c) => c.get_id(),
            AnyInternalServerCommand::FumbblGameCreated(c) => c.get_id(),
            AnyInternalServerCommand::FumbblTeamLoaded(c) => c.get_id(),
            AnyInternalServerCommand::JoinApproved(c) => c.get_id(),
            AnyInternalServerCommand::ReplayLoaded(c) => c.get_id(),
            AnyInternalServerCommand::ScheduleGame(c) => c.get_id(),
            AnyInternalServerCommand::SocketClosed(c) => c.get_id(),
            AnyInternalServerCommand::UploadGame(c) => c.get_id(),
        }
    }

    /// Java: `NetCommand.getGameId()`.
    pub fn get_game_id(&self) -> i64 {
        match self {
            AnyInternalServerCommand::AddLoadedTeam(c) => c.get_game_id(),
            AnyInternalServerCommand::ApplyAutomatedPlayerMarkings(c) => c.get_game_id(),
            AnyInternalServerCommand::CalculateAutomaticPlayerMarkings(c) => c.get_game_id(),
            AnyInternalServerCommand::ClearCache(c) => c.get_game_id(),
            AnyInternalServerCommand::CloseGame(c) => c.get_game_id(),
            AnyInternalServerCommand::DeleteGame(c) => c.get_game_id(),
            AnyInternalServerCommand::FumbblGameChecked(c) => c.get_game_id(),
            AnyInternalServerCommand::FumbblGameCreated(c) => c.get_game_id(),
            AnyInternalServerCommand::FumbblTeamLoaded(c) => c.get_game_id(),
            AnyInternalServerCommand::JoinApproved(c) => c.get_game_id(),
            AnyInternalServerCommand::ReplayLoaded(c) => c.get_game_id(),
            AnyInternalServerCommand::ScheduleGame(c) => c.get_game_id(),
            AnyInternalServerCommand::SocketClosed(c) => c.get_game_id(),
            AnyInternalServerCommand::UploadGame(c) => c.get_game_id(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_id_dispatches_to_upload_game() {
        let cmd = AnyInternalServerCommand::UploadGame(InternalServerCommandUploadGame::new(1));
        assert_eq!(cmd.get_id(), "internalServerUploadGame");
    }

    #[test]
    fn get_id_dispatches_to_join_approved() {
        let cmd = AnyInternalServerCommand::JoinApproved(InternalServerCommandJoinApproved::new(
            1, String::new(), "coach".into(), String::new(), "PLAYER".into(), vec![],
        ));
        assert_eq!(cmd.get_id(), InternalServerCommandJoinApproved::new(
            1, String::new(), "coach".into(), String::new(), "PLAYER".into(), vec![],
        ).get_id());
    }

    #[test]
    fn get_id_dispatches_to_schedule_game() {
        let cmd = AnyInternalServerCommand::ScheduleGame(
            InternalServerCommandScheduleGame::new("home".into(), "away".into()),
        );
        assert_eq!(cmd.get_id(), "internalServerScheduleGame");
    }

    #[test]
    fn get_game_id_dispatches_to_upload_game() {
        let cmd = AnyInternalServerCommand::UploadGame(InternalServerCommandUploadGame::new(42));
        assert_eq!(cmd.get_game_id(), 42);
    }

    #[test]
    fn get_id_dispatches_to_socket_closed() {
        let cmd = AnyInternalServerCommand::SocketClosed(InternalServerCommandSocketClosed);
        assert!(!cmd.get_id().is_empty());
    }

    #[test]
    fn get_id_dispatches_to_clear_cache() {
        let cmd = AnyInternalServerCommand::ClearCache(InternalServerCommandClearCache);
        assert!(!cmd.get_id().is_empty());
    }
}
