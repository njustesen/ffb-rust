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

    /// Reverse of `name()` -- the Java-side lookup performed by
    /// `NetCommandIdFactory.forName(String)`.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "internalServerSocketClosed" => Some(NetCommandId::InternalServerSocketClosed),
            "clientJoin" => Some(NetCommandId::ClientJoin),
            "clientTalk" => Some(NetCommandId::ClientTalk),
            "serverGameState" => Some(NetCommandId::ServerGameState),
            "serverTeamList" => Some(NetCommandId::ServerTeamList),
            "serverStatus" => Some(NetCommandId::ServerStatus),
            "serverJoin" => Some(NetCommandId::ServerJoin),
            "serverLeave" => Some(NetCommandId::ServerLeave),
            "serverTalk" => Some(NetCommandId::ServerTalk),
            "clientSetupPlayer" => Some(NetCommandId::ClientSetupPlayer),
            "clientStartGame" => Some(NetCommandId::ClientStartGame),
            "clientActingPlayer" => Some(NetCommandId::ClientActingPlayer),
            "clientMove" => Some(NetCommandId::ClientMove),
            "clientBlitzMove" => Some(NetCommandId::ClientBlitzMove),
            "blitzTargetSelected" => Some(NetCommandId::ClientBlitzTargetSelected),
            "targetSelected" => Some(NetCommandId::ClientTargetSelected),
            "clientUseReRoll" => Some(NetCommandId::ClientUseReRoll),
            "clientUseReRollForTarget" => Some(NetCommandId::ClientUseReRollForTarget),
            "serverSound" => Some(NetCommandId::ServerSound),
            "clientCoinChoice" => Some(NetCommandId::ClientCoinChoice),
            "clientReceiveChoice" => Some(NetCommandId::ClientReceiveChoice),
            "clientEndTurn" => Some(NetCommandId::ClientEndTurn),
            "clientKickoff" => Some(NetCommandId::ClientKickoff),
            "clientTouchback" => Some(NetCommandId::ClientTouchback),
            "clientHandOver" => Some(NetCommandId::ClientHandOver),
            "clientPass" => Some(NetCommandId::ClientPass),
            "clientBlock" => Some(NetCommandId::ClientBlock),
            "clientBlockChoice" => Some(NetCommandId::ClientBlockChoice),
            "clientPushback" => Some(NetCommandId::ClientPushback),
            "clientUseConsummateReRollForBlock" => Some(NetCommandId::ClientUseConsummateReRollForBlock),
            "clientUseProReRollForBlock" => Some(NetCommandId::ClientUseProReRollForBlock),
            "clientFollowupChoice" => Some(NetCommandId::ClientFollowupChoice),
            "clientInterceptorChoice" => Some(NetCommandId::ClientInterceptorChoice),
            "clientUseSkill" => Some(NetCommandId::ClientUseSkill),
            "serverTeamSetupList" => Some(NetCommandId::ServerTeamSetupList),
            "clientTeamSetupLoad" => Some(NetCommandId::ClientTeamSetupLoad),
            "clientTeamSetupSave" => Some(NetCommandId::ClientTeamSetupSave),
            "clientTeamSetupDelete" => Some(NetCommandId::ClientTeamSetupDelete),
            "clientFoul" => Some(NetCommandId::ClientFoul),
            "clientUseApothecary" => Some(NetCommandId::ClientUseApothecary),
            "clientApothecaryChoice" => Some(NetCommandId::ClientApothecaryChoice),
            "clientPasswordChallenge" => Some(NetCommandId::ClientPasswordChallenge),
            "serverPasswordChallenge" => Some(NetCommandId::ServerPasswordChallenge),
            "serverModelSync" => Some(NetCommandId::ServerModelSync),
            "serverVersion" => Some(NetCommandId::ServerVersion),
            "clientRequestVersion" => Some(NetCommandId::ClientRequestVersion),
            "clientDebugClientState" => Some(NetCommandId::ClientDebugClientState),
            "serverGameList" => Some(NetCommandId::ServerGameList),
            "clientUserSettings" => Some(NetCommandId::ClientUserSettings),
            "serverUserSettings" => Some(NetCommandId::ServerUserSettings),
            "clientReplay" => Some(NetCommandId::ClientReplay),
            "serverReplay" => Some(NetCommandId::ServerReplay),
            "clientThrowTeamMate" => Some(NetCommandId::ClientThrowTeamMate),
            "clientKickTeamMate" => Some(NetCommandId::ClientKickTeamMate),
            "clientSwoop" => Some(NetCommandId::ClientSwoop),
            "clientPlayerChoice" => Some(NetCommandId::ClientPlayerChoice),
            "clientIllegalProcedure" => Some(NetCommandId::ClientIllegalProcedure),
            "clientConcedeGame" => Some(NetCommandId::ClientConcedeGame),
            "serverAdminMessage" => Some(NetCommandId::ServerAdminMessage),
            "clientUseInducement" => Some(NetCommandId::ClientUseInducement),
            "clientBuyInducements" => Some(NetCommandId::ClientBuyInducements),
            "serverAddPlayer" => Some(NetCommandId::ServerAddPlayer),
            "serverZapPlayer" => Some(NetCommandId::ServerZapPlayer),
            "serverUnzapPlayer" => Some(NetCommandId::ServerUnzapPlayer),
            "clientJourneymen" => Some(NetCommandId::ClientJourneymen),
            "clientGaze" => Some(NetCommandId::ClientGaze),
            "clientConfirm" => Some(NetCommandId::ClientConfirm),
            "clientSetMarker" => Some(NetCommandId::ClientSetMarker),
            "internalServerFumbblGameCreated" => Some(NetCommandId::InternalServerFumbblGameCreated),
            "internalServerFumbblTeamLoaded" => Some(NetCommandId::InternalServerFumbblTeamLoaded),
            "internalServerFumbblGameChecked" => Some(NetCommandId::InternalServerFumbblGameChecked),
            "internalServerJoinApproved" => Some(NetCommandId::InternalServerJoinApproved),
            "internalServerReplayGameLoaded" => Some(NetCommandId::InternalServerReplayLoaded),
            "clientPettyCash" => Some(NetCommandId::ClientPettyCash),
            "serverRemovePlayer" => Some(NetCommandId::ServerRemovePlayer),
            "clientWizardSpell" => Some(NetCommandId::ClientWizardSpell),
            "clientBuyCard" => Some(NetCommandId::ClientBuyCard),
            "clientSelectCardToBuy" => Some(NetCommandId::ClientSelectCardToBuy),
            "internalServerCloseGame" => Some(NetCommandId::InternalServerCloseGame),
            "internalServerDeleteGame" => Some(NetCommandId::InternalServerDeleteGame),
            "internalServerUploadGame" => Some(NetCommandId::InternalServerUploadGame),
            "internalServerScheduleGame" => Some(NetCommandId::InternalServerScheduleGame),
            "internalServerClearCache" => Some(NetCommandId::InternalServerClearCache),
            "clientCloseSession" => Some(NetCommandId::ClientCloseSession),
            "clientArgueTheCall" => Some(NetCommandId::ClientArgueTheCall),
            "clientUseApothecaries" => Some(NetCommandId::ClientUseApothecaries),
            "clientUseIgors" => Some(NetCommandId::ClientUseIgors),
            "serverGameTime" => Some(NetCommandId::ServerGameTime),
            "clientPing" => Some(NetCommandId::ClientPing),
            "serverPong" => Some(NetCommandId::ServerPong),
            "clientSetBlockTargetSelection" => Some(NetCommandId::ClientSetBlockTargetSelection),
            "clientUnsetBlockTargetSelection" => Some(NetCommandId::ClientUnsetBlockTargetSelection),
            "clientSynchronousMultiBlock" => Some(NetCommandId::ClientSynchronousMultiBlock),
            "clientBlockOrReRollChoiceForTarget" => Some(NetCommandId::ClientBlockOrReRollChoiceForTarget),
            "clientPileDriver" => Some(NetCommandId::ClientPileDriver),
            "clientUseChainsaw" => Some(NetCommandId::ClientUseChainsaw),
            "clientUseBrawler" => Some(NetCommandId::ClientUseBrawler),
            "clientFieldCoordinate" => Some(NetCommandId::ClientFieldCoordinate),
            "clientUseFumblerooskie" => Some(NetCommandId::ClientUseFumblerooskie),
            "clientPrayerSelection" => Some(NetCommandId::ClientPrayerSelection),
            "clientUseTeamMatesWisdom" => Some(NetCommandId::ClientUseTeamMatesWisdom),
            "clientThrowKeg" => Some(NetCommandId::ClientThrowKeg),
            "clientSelectWeather" => Some(NetCommandId::ClientSelectWeather),
            "clientUpdatePlayerMarkings" => Some(NetCommandId::ClientUpdatePlayerMarkings),
            "clientKickOffResultChoice" => Some(NetCommandId::ClientKickOffResultChoice),
            "clientBloodlustAction" => Some(NetCommandId::ClientBloodlustAction),
            "serverUpdateLocalPlayerMarkers" => Some(NetCommandId::ServerUpdateLocalPlayerMarkers),
            "internalServerAddLoadedTeam" => Some(NetCommandId::InternalServerAddLoadedTeam),
            "internalApplyAutomaticPlayerMarkings" => Some(NetCommandId::InternalApplyAutomaticPlayerMarkings),
            "clientUseSingleBlockDieReRoll" => Some(NetCommandId::ClientUseSingleBlockDieReRoll),
            "clientUseMultiBlockDiceReRoll" => Some(NetCommandId::ClientUseMultiBlockDiceReRoll),
            "internalCalculateAutomaticPlayerMarkings" => Some(NetCommandId::InternalCalculateAutomaticPlayerMarkings),
            "clientLoadPlayerMarkings" => Some(NetCommandId::ClientLoadAutomaticPlayerMarkings),
            "serverAutomaticPlayerMarkings" => Some(NetCommandId::ServerAutomaticPlayerMarkings),
            "clientReplayStatus" => Some(NetCommandId::ClientReplayStatus),
            "serverReplayStatus" => Some(NetCommandId::ServerReplayStatus),
            "clientJoinReplay" => Some(NetCommandId::ClientJoinReplay),
            "serverReplayControl" => Some(NetCommandId::ServerReplayControl),
            "clientTransferReplayControl" => Some(NetCommandId::ClientTransferReplayControl),
            "clientAddSketch" => Some(NetCommandId::ClientAddSketch),
            "clientRemoveSketches" => Some(NetCommandId::ClientRemoveSketches),
            "clientSketchAddCoordinate" => Some(NetCommandId::ClientSketchAddCoordinate),
            "clientSketchSetColor" => Some(NetCommandId::ClientSketchSetColor),
            "clientSketchSetLabel" => Some(NetCommandId::ClientSketchSetLabel),
            "clientClearSketches" => Some(NetCommandId::ClientClearSketches),
            "serverAddSketches" => Some(NetCommandId::ServerAddSketches),
            "serverRemoveSketches" => Some(NetCommandId::ServerRemoveSketches),
            "serverSketchAddCoordinate" => Some(NetCommandId::ServerSketchAddCoordinate),
            "serverSketchSetColor" => Some(NetCommandId::ServerSketchSetColor),
            "serverSketchSetLabel" => Some(NetCommandId::ServerSketchSetLabel),
            "serverClearSketches" => Some(NetCommandId::ServerClearSketches),
            "clientSetPreventSketching" => Some(NetCommandId::ClientSetPreventSketching),
            "serverSetPreventSketching" => Some(NetCommandId::ServerSetPreventSketching),
            "clientPickUpChoice" => Some(NetCommandId::ClientPickUpChoice),
            "clientKeywordSelection" => Some(NetCommandId::ClientKeywordSelection),
            "clientUseHatred" => Some(NetCommandId::ClientUseHatred),
            "clientPositionSelection" => Some(NetCommandId::ClientPositionSelection),
            "clientPuntToCrowd" => Some(NetCommandId::ClientPuntToCrowd),
            _ => None,
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

    pub fn from_name(name: &str) -> Option<Self> {
        [
            ServerStatus::ErrorUnknownCoach,
            ServerStatus::ErrorWrongPassword,
            ServerStatus::ErrorGameInUse,
            ServerStatus::ErrorNotYourTeam,
            ServerStatus::ErrorUnknownGameId,
            ServerStatus::ErrorSameTeam,
            ServerStatus::FumbblError,
            ServerStatus::ReplayUnavailable,
        ]
        .iter()
        .copied()
        .find(|v| v.name().eq_ignore_ascii_case(name))
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
    fn from_name_is_the_exact_inverse_of_name() {
        for id in [
            NetCommandId::ClientJoin,
            NetCommandId::ServerGameTime,
            NetCommandId::ClientBlitzTargetSelected,
            NetCommandId::ClientTargetSelected,
            NetCommandId::InternalApplyAutomaticPlayerMarkings,
            NetCommandId::InternalCalculateAutomaticPlayerMarkings,
            NetCommandId::ClientLoadAutomaticPlayerMarkings,
        ] {
            assert_eq!(NetCommandId::from_name(id.name()), Some(id));
        }
    }

    #[test]
    fn from_name_unknown_string_returns_none() {
        assert_eq!(NetCommandId::from_name("notARealCommand"), None);
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
