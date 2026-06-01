use serde::{Deserialize, Serialize};

/// Identifies every command type that can flow over the WebSocket.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NetCommandId {
    InternalServerSocketClosed,
    ClientJoin,
    ClientTalk,
    ServerGameState,
    ServerTeamList,
    ServerStatus,
    ServerJoin,
    ServerLeave,
    ServerTalk,
    ClientSetupPlayer,
    ClientStartGame,
    ClientActingPlayer,
    ClientMove,
    ClientBlitzMove,
    ClientBlitzTargetSelected,
    ClientTargetSelected,
    ClientUseReRoll,
    ClientUseReRollForTarget,
    ServerSound,
    ClientCoinChoice,
    ClientReceiveChoice,
    ClientEndTurn,
    ClientKickoff,
    ClientTouchback,
    ClientHandOver,
    ClientPass,
    ClientBlock,
    ClientBlockChoice,
    ClientPushback,
    ClientUseConsummateReRollForBlock,
    ClientUseProReRollForBlock,
    ClientFollowupChoice,
    ClientInterceptorChoice,
    ClientUseSkill,
    ServerTeamSetupList,
    ClientTeamSetupLoad,
    ClientTeamSetupSave,
    ClientTeamSetupDelete,
    ClientFoul,
    ClientUseApothecary,
    ClientApothecaryChoice,
    ClientPasswordChallenge,
    ServerPasswordChallenge,
    ServerModelSync,
    ServerVersion,
    ClientRequestVersion,
    ClientDebugClientState,
    ServerGameList,
    ClientUserSettings,
    ServerUserSettings,
    ClientReplay,
    ServerReplay,
    ClientThrowTeamMate,
    ClientKickTeamMate,
    ClientSwoop,
    ClientPlayerChoice,
    ClientIllegalProcedure,
    ClientConcedeGame,
    ServerAdminMessage,
    ClientUseInducement,
    ClientBuyInducements,
    ServerAddPlayer,
    ServerZapPlayer,
    ServerUnzapPlayer,
    ClientJourneymen,
    ClientGaze,
    ClientConfirm,
    ClientSetMarker,
    InternalServerFumbblGameCreated,
    InternalServerFumbblTeamLoaded,
    InternalServerFumbblGameChecked,
    InternalServerJoinApproved,
    InternalServerReplayLoaded,
    ClientPettyCash,
    ServerRemovePlayer,
    ClientWizardSpell,
    ClientBuyCard,
    ClientSelectCardToBuy,
    InternalServerCloseGame,
    InternalServerDeleteGame,
    InternalServerUploadGame,
    InternalServerScheduleGame,
    InternalServerClearCache,
    ClientCloseSession,
    ClientArgueTheCall,
    ClientUseApothecaries,
    ClientUseIgors,
    ServerGameTime,
    ClientPing,
    ServerPong,
    ClientSetBlockTargetSelection,
    ClientUnsetBlockTargetSelection,
    ClientSynchronousMultiBlock,
    ClientBlockOrReRollChoiceForTarget,
    ClientPileDriver,
    ClientUseChainsaw,
    ClientUseBrawler,
    ClientFieldCoordinate,
    ClientUseFumblerooskie,
    ClientPrayerSelection,
    ClientUseTeamMatesWisdom,
    ClientThrowKeg,
    ClientSelectWeather,
    ClientUpdatePlayerMarkings,
    ClientKickOffResultChoice,
    ClientBloodlustAction,
    ServerUpdateLocalPlayerMarkers,
    InternalServerAddLoadedTeam,
    InternalApplyAutomaticPlayerMarkings,
    ClientUseSingleBlockDieReRoll,
    ClientUseMultiBlockDiceReRoll,
    InternalCalculateAutomaticPlayerMarkings,
    ClientLoadAutomaticPlayerMarkings,
    ServerAutomaticPlayerMarkings,
    ClientReplayStatus,
    ServerReplayStatus,
    ClientJoinReplay,
    ServerReplayControl,
    ClientTransferReplayControl,
    ClientAddSketch,
    ClientRemoveSketches,
    ClientSketchAddCoordinate,
    ClientSketchSetColor,
    ClientSketchSetLabel,
    ClientClearSketches,
    ServerAddSketches,
    ServerRemoveSketches,
    ServerSketchAddCoordinate,
    ServerSketchSetColor,
    ServerSketchSetLabel,
    ServerClearSketches,
    ClientSetPreventSketching,
    ServerSetPreventSketching,
    ClientPickUpChoice,
    ClientKeywordSelection,
    ClientUseHatred,
    ClientPositionSelection,
    ClientPuntToCrowd,
}

impl NetCommandId {
    pub fn name(self) -> &'static str {
        match self {
            NetCommandId::InternalServerSocketClosed => "internalServerSocketClosed",
            NetCommandId::ClientJoin => "clientJoin",
            NetCommandId::ClientTalk => "clientTalk",
            NetCommandId::ServerGameState => "serverGameState",
            NetCommandId::ServerTeamList => "serverTeamList",
            NetCommandId::ServerStatus => "serverStatus",
            NetCommandId::ServerJoin => "serverJoin",
            NetCommandId::ServerLeave => "serverLeave",
            NetCommandId::ServerTalk => "serverTalk",
            NetCommandId::ClientSetupPlayer => "clientSetupPlayer",
            NetCommandId::ClientStartGame => "clientStartGame",
            NetCommandId::ClientActingPlayer => "clientActingPlayer",
            NetCommandId::ClientMove => "clientMove",
            NetCommandId::ClientBlitzMove => "clientBlitzMove",
            NetCommandId::ClientBlitzTargetSelected => "blitzTargetSelected",
            NetCommandId::ClientTargetSelected => "targetSelected",
            NetCommandId::ClientUseReRoll => "clientUseReRoll",
            NetCommandId::ClientUseReRollForTarget => "clientUseReRollForTarget",
            NetCommandId::ServerSound => "serverSound",
            NetCommandId::ClientCoinChoice => "clientCoinChoice",
            NetCommandId::ClientReceiveChoice => "clientReceiveChoice",
            NetCommandId::ClientEndTurn => "clientEndTurn",
            NetCommandId::ClientKickoff => "clientKickoff",
            NetCommandId::ClientTouchback => "clientTouchback",
            NetCommandId::ClientHandOver => "clientHandOver",
            NetCommandId::ClientPass => "clientPass",
            NetCommandId::ClientBlock => "clientBlock",
            NetCommandId::ClientBlockChoice => "clientBlockChoice",
            NetCommandId::ClientPushback => "clientPushback",
            NetCommandId::ClientUseConsummateReRollForBlock => "clientUseConsummateReRollForBlock",
            NetCommandId::ClientUseProReRollForBlock => "clientUseProReRollForBlock",
            NetCommandId::ClientFollowupChoice => "clientFollowupChoice",
            NetCommandId::ClientInterceptorChoice => "clientInterceptorChoice",
            NetCommandId::ClientUseSkill => "clientUseSkill",
            NetCommandId::ServerTeamSetupList => "serverTeamSetupList",
            NetCommandId::ClientTeamSetupLoad => "clientTeamSetupLoad",
            NetCommandId::ClientTeamSetupSave => "clientTeamSetupSave",
            NetCommandId::ClientTeamSetupDelete => "clientTeamSetupDelete",
            NetCommandId::ClientFoul => "clientFoul",
            NetCommandId::ClientUseApothecary => "clientUseApothecary",
            NetCommandId::ClientApothecaryChoice => "clientApothecaryChoice",
            NetCommandId::ClientPasswordChallenge => "clientPasswordChallenge",
            NetCommandId::ServerPasswordChallenge => "serverPasswordChallenge",
            NetCommandId::ServerModelSync => "serverModelSync",
            NetCommandId::ServerVersion => "serverVersion",
            NetCommandId::ClientRequestVersion => "clientRequestVersion",
            NetCommandId::ClientDebugClientState => "clientDebugClientState",
            NetCommandId::ServerGameList => "serverGameList",
            NetCommandId::ClientUserSettings => "clientUserSettings",
            NetCommandId::ServerUserSettings => "serverUserSettings",
            NetCommandId::ClientReplay => "clientReplay",
            NetCommandId::ServerReplay => "serverReplay",
            NetCommandId::ClientThrowTeamMate => "clientThrowTeamMate",
            NetCommandId::ClientKickTeamMate => "clientKickTeamMate",
            NetCommandId::ClientSwoop => "clientSwoop",
            NetCommandId::ClientPlayerChoice => "clientPlayerChoice",
            NetCommandId::ClientIllegalProcedure => "clientIllegalProcedure",
            NetCommandId::ClientConcedeGame => "clientConcedeGame",
            NetCommandId::ServerAdminMessage => "serverAdminMessage",
            NetCommandId::ClientUseInducement => "clientUseInducement",
            NetCommandId::ClientBuyInducements => "clientBuyInducements",
            NetCommandId::ServerAddPlayer => "serverAddPlayer",
            NetCommandId::ServerZapPlayer => "serverZapPlayer",
            NetCommandId::ServerUnzapPlayer => "serverUnzapPlayer",
            NetCommandId::ClientJourneymen => "clientJourneymen",
            NetCommandId::ClientGaze => "clientGaze",
            NetCommandId::ClientConfirm => "clientConfirm",
            NetCommandId::ClientSetMarker => "clientSetMarker",
            NetCommandId::InternalServerFumbblGameCreated => "internalServerFumbblGameCreated",
            NetCommandId::InternalServerFumbblTeamLoaded => "internalServerFumbblTeamLoaded",
            NetCommandId::InternalServerFumbblGameChecked => "internalServerFumbblGameChecked",
            NetCommandId::InternalServerJoinApproved => "internalServerJoinApproved",
            NetCommandId::InternalServerReplayLoaded => "internalServerReplayGameLoaded",
            NetCommandId::ClientPettyCash => "clientPettyCash",
            NetCommandId::ServerRemovePlayer => "serverRemovePlayer",
            NetCommandId::ClientWizardSpell => "clientWizardSpell",
            NetCommandId::ClientBuyCard => "clientBuyCard",
            NetCommandId::ClientSelectCardToBuy => "clientSelectCardToBuy",
            NetCommandId::InternalServerCloseGame => "internalServerCloseGame",
            NetCommandId::InternalServerDeleteGame => "internalServerDeleteGame",
            NetCommandId::InternalServerUploadGame => "internalServerUploadGame",
            NetCommandId::InternalServerScheduleGame => "internalServerScheduleGame",
            NetCommandId::InternalServerClearCache => "internalServerClearCache",
            NetCommandId::ClientCloseSession => "clientCloseSession",
            NetCommandId::ClientArgueTheCall => "clientArgueTheCall",
            NetCommandId::ClientUseApothecaries => "clientUseApothecaries",
            NetCommandId::ClientUseIgors => "clientUseIgors",
            NetCommandId::ServerGameTime => "serverGameTime",
            NetCommandId::ClientPing => "clientPing",
            NetCommandId::ServerPong => "serverPong",
            NetCommandId::ClientSetBlockTargetSelection => "clientSetBlockTargetSelection",
            NetCommandId::ClientUnsetBlockTargetSelection => "clientUnsetBlockTargetSelection",
            NetCommandId::ClientSynchronousMultiBlock => "clientSynchronousMultiBlock",
            NetCommandId::ClientBlockOrReRollChoiceForTarget => "clientBlockOrReRollChoiceForTarget",
            NetCommandId::ClientPileDriver => "clientPileDriver",
            NetCommandId::ClientUseChainsaw => "clientUseChainsaw",
            NetCommandId::ClientUseBrawler => "clientUseBrawler",
            NetCommandId::ClientFieldCoordinate => "clientFieldCoordinate",
            NetCommandId::ClientUseFumblerooskie => "clientUseFumblerooskie",
            NetCommandId::ClientPrayerSelection => "clientPrayerSelection",
            NetCommandId::ClientUseTeamMatesWisdom => "clientUseTeamMatesWisdom",
            NetCommandId::ClientThrowKeg => "clientThrowKeg",
            NetCommandId::ClientSelectWeather => "clientSelectWeather",
            NetCommandId::ClientUpdatePlayerMarkings => "clientUpdatePlayerMarkings",
            NetCommandId::ClientKickOffResultChoice => "clientKickOffResultChoice",
            NetCommandId::ClientBloodlustAction => "clientBloodlustAction",
            NetCommandId::ServerUpdateLocalPlayerMarkers => "serverUpdateLocalPlayerMarkers",
            NetCommandId::InternalServerAddLoadedTeam => "internalServerAddLoadedTeam",
            NetCommandId::InternalApplyAutomaticPlayerMarkings => {
                "internalApplyAutomaticPlayerMarkings"
            }
            NetCommandId::ClientUseSingleBlockDieReRoll => "clientUseSingleBlockDieReRoll",
            NetCommandId::ClientUseMultiBlockDiceReRoll => "clientUseMultiBlockDiceReRoll",
            NetCommandId::InternalCalculateAutomaticPlayerMarkings => {
                "internalCalculateAutomaticPlayerMarkings"
            }
            NetCommandId::ClientLoadAutomaticPlayerMarkings => "clientLoadPlayerMarkings",
            NetCommandId::ServerAutomaticPlayerMarkings => "serverAutomaticPlayerMarkings",
            NetCommandId::ClientReplayStatus => "clientReplayStatus",
            NetCommandId::ServerReplayStatus => "serverReplayStatus",
            NetCommandId::ClientJoinReplay => "clientJoinReplay",
            NetCommandId::ServerReplayControl => "serverReplayControl",
            NetCommandId::ClientTransferReplayControl => "clientTransferReplayControl",
            NetCommandId::ClientAddSketch => "clientAddSketch",
            NetCommandId::ClientRemoveSketches => "clientRemoveSketches",
            NetCommandId::ClientSketchAddCoordinate => "clientSketchAddCoordinate",
            NetCommandId::ClientSketchSetColor => "clientSketchSetColor",
            NetCommandId::ClientSketchSetLabel => "clientSketchSetLabel",
            NetCommandId::ClientClearSketches => "clientClearSketches",
            NetCommandId::ServerAddSketches => "serverAddSketches",
            NetCommandId::ServerRemoveSketches => "serverRemoveSketches",
            NetCommandId::ServerSketchAddCoordinate => "serverSketchAddCoordinate",
            NetCommandId::ServerSketchSetColor => "serverSketchSetColor",
            NetCommandId::ServerSketchSetLabel => "serverSketchSetLabel",
            NetCommandId::ServerClearSketches => "serverClearSketches",
            NetCommandId::ClientSetPreventSketching => "clientSetPreventSketching",
            NetCommandId::ServerSetPreventSketching => "serverSetPreventSketching",
            NetCommandId::ClientPickUpChoice => "clientPickUpChoice",
            NetCommandId::ClientKeywordSelection => "clientKeywordSelection",
            NetCommandId::ClientUseHatred => "clientUseHatred",
            NetCommandId::ClientPositionSelection => "clientPositionSelection",
            NetCommandId::ClientPuntToCrowd => "clientPuntToCrowd",
        }
    }

    pub fn is_client_command(self) -> bool {
        self.name().starts_with("client")
    }

    pub fn is_server_command(self) -> bool {
        self.name().starts_with("server")
    }
}

/// Error statuses the server can send back on join.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServerStatus {
    ErrorUnknownCoach,
    ErrorWrongPassword,
    ErrorGameInUse,
    ErrorNotYourTeam,
    ErrorUnknownGameId,
    ErrorSameTeam,
    FumbblError,
    ReplayUnavailable,
}

impl ServerStatus {
    pub fn name(self) -> &'static str {
        match self {
            ServerStatus::ErrorUnknownCoach => "Unknown Coach",
            ServerStatus::ErrorWrongPassword => "Wrong Password",
            ServerStatus::ErrorGameInUse => "Game In Use",
            ServerStatus::ErrorNotYourTeam => "Not Your Team",
            ServerStatus::ErrorUnknownGameId => "Unknown Game Id",
            ServerStatus::ErrorSameTeam => "Same Team",
            ServerStatus::FumbblError => "Fumbbl Error",
            ServerStatus::ReplayUnavailable => "Replay Unavailable",
        }
    }

    pub fn message(self) -> &'static str {
        match self {
            ServerStatus::ErrorUnknownCoach => "Unknown Coach!",
            ServerStatus::ErrorWrongPassword => "Wrong Password!",
            ServerStatus::ErrorGameInUse => "A Game with this name is already in use!",
            ServerStatus::ErrorNotYourTeam => "The team you wanted to join with is not yours!",
            ServerStatus::ErrorUnknownGameId => "There is no game with the given id!",
            ServerStatus::ErrorSameTeam => "You cannot play a team against itself!",
            ServerStatus::FumbblError => "Fumbbl Error",
            ServerStatus::ReplayUnavailable => "The replay for this game is currently unavailable.",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_commands_are_client() {
        assert!(NetCommandId::ClientJoin.is_client_command());
        assert!(!NetCommandId::ServerGameState.is_client_command());
    }

    #[test]
    fn serde_net_command_id() {
        let id = NetCommandId::ServerGameState;
        let json = serde_json::to_string(&id).unwrap();
        let back: NetCommandId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, back);
    }

    #[test]
    fn client_join_name_is_client_join() {
        assert_eq!(NetCommandId::ClientJoin.name(), "clientJoin");
    }

    #[test]
    fn server_game_state_name() {
        assert_eq!(NetCommandId::ServerGameState.name(), "serverGameState");
    }

    #[test]
    fn client_end_turn_name() {
        assert_eq!(NetCommandId::ClientEndTurn.name(), "clientEndTurn");
    }

    #[test]
    fn server_commands_are_server() {
        assert!(NetCommandId::ServerGameState.is_server_command());
        assert!(!NetCommandId::ClientJoin.is_server_command());
    }

    #[test]
    fn server_status_count_is_eight() {
        let all = [
            ServerStatus::ErrorUnknownCoach, ServerStatus::ErrorWrongPassword,
            ServerStatus::ErrorGameInUse, ServerStatus::ErrorNotYourTeam,
            ServerStatus::ErrorUnknownGameId, ServerStatus::ErrorSameTeam,
            ServerStatus::FumbblError, ServerStatus::ReplayUnavailable,
        ];
        assert_eq!(all.len(), 8);
    }

    #[test]
    fn server_status_all_have_non_empty_names() {
        for s in [
            ServerStatus::ErrorUnknownCoach, ServerStatus::ErrorWrongPassword,
            ServerStatus::ErrorGameInUse, ServerStatus::ErrorNotYourTeam,
            ServerStatus::ErrorUnknownGameId, ServerStatus::ErrorSameTeam,
            ServerStatus::FumbblError, ServerStatus::ReplayUnavailable,
        ] {
            assert!(!s.name().is_empty());
            assert!(!s.message().is_empty());
        }
    }

    #[test]
    fn server_status_wrong_password_name() {
        assert_eq!(ServerStatus::ErrorWrongPassword.name(), "Wrong Password");
    }

    #[test]
    fn server_status_fumbbl_error_name() {
        assert_eq!(ServerStatus::FumbblError.name(), "Fumbbl Error");
    }

    #[test]
    fn server_status_unknown_coach_message() {
        assert_eq!(ServerStatus::ErrorUnknownCoach.message(), "Unknown Coach!");
    }

    #[test]
    fn net_command_id_client_join_is_client_command() {
        assert!(NetCommandId::ClientJoin.is_client_command());
    }

    #[test]
    fn net_command_id_server_pong_is_server_command() {
        assert!(NetCommandId::ServerPong.is_server_command());
    }

    #[test]
    fn server_status_error_game_in_use_name() {
        assert_eq!(ServerStatus::ErrorGameInUse.name(), "Game In Use");
    }

    #[test]
    fn net_command_id_punt_to_crowd_name() {
        assert_eq!(NetCommandId::ClientPuntToCrowd.name(), "clientPuntToCrowd");
    }

    #[test]
    fn internal_command_is_not_client_or_server() {
        assert!(!NetCommandId::InternalServerSocketClosed.is_client_command());
        assert!(!NetCommandId::InternalServerSocketClosed.is_server_command());
    }
}
