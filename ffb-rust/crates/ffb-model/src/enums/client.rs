use serde::{Deserialize, Serialize};

/// Identifies which client-side state machine state is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ClientStateId {
    Login,
    StartGame,
    SelectPlayer,
    Move,
    Block,
    Blitz,
    HandOver,
    Pass,
    Spectate,
    Setup,
    Kickoff,
    Pushback,
    Interception,
    Foul,
    HighKick,
    QuickSnap,
    Touchback,
    WaitForOpponent,
    Replay,
    ThrowTeamMate,
    KickTeamMate,
    Swoop,
    DumpOff,
    WaitForSetup,
    Gaze,
    KickoffReturn,
    Swarming,
    Wizard,
    PassBlock,
    Bomb,
    IllegalSubstitution,
    GazeMove,
    SelectBlitzTarget,
    SelectGazeTarget,
    SynchronousMultiBlock,
    PlaceBall,
    SolidDefence,
    KickTeamMateThrow,
    ThrowKeg,
    RaidingParty,
    SelectBlockKind,
    MaximumCarnage,
    HitAndRun,
    PutridRegurgitationBlitz,
    Trickster,
    PutridRegurgitationBlock,
    KickEmBlock,
    KickEmBlitz,
    ThenIStartedBlastin,
    Stab,
    FuriousOutburst,
    Punt,
}

impl ClientStateId {
    pub fn name(self) -> &'static str {
        match self {
            ClientStateId::Login => "login",
            ClientStateId::StartGame => "startGame",
            ClientStateId::SelectPlayer => "selectPlayer",
            ClientStateId::Move => "move",
            ClientStateId::Block => "block",
            ClientStateId::Blitz => "blitz",
            ClientStateId::HandOver => "handOver",
            ClientStateId::Pass => "pass",
            ClientStateId::Spectate => "spectate",
            ClientStateId::Setup => "setup",
            ClientStateId::Kickoff => "kickoff",
            ClientStateId::Pushback => "pushback",
            ClientStateId::Interception => "interception",
            ClientStateId::Foul => "foul",
            ClientStateId::HighKick => "highKick",
            ClientStateId::QuickSnap => "quickSnap",
            ClientStateId::Touchback => "touchback",
            ClientStateId::WaitForOpponent => "waitForOpponent",
            ClientStateId::Replay => "replay",
            ClientStateId::ThrowTeamMate => "throwTeamMate",
            ClientStateId::KickTeamMate => "kickTeamMate",
            ClientStateId::Swoop => "swoop",
            ClientStateId::DumpOff => "dumpOff",
            ClientStateId::WaitForSetup => "waitForSetup",
            ClientStateId::Gaze => "gaze",
            ClientStateId::KickoffReturn => "kickoffReturn",
            ClientStateId::Swarming => "swarming",
            ClientStateId::Wizard => "wizard",
            ClientStateId::PassBlock => "passBlock",
            ClientStateId::Bomb => "bomb",
            ClientStateId::IllegalSubstitution => "illegalSubstitution",
            ClientStateId::GazeMove => "gazeMove",
            ClientStateId::SelectBlitzTarget => "selectBlitzTarget",
            ClientStateId::SelectGazeTarget => "selectGazeTarget",
            ClientStateId::SynchronousMultiBlock => "synchronousMultiBlock",
            ClientStateId::PlaceBall => "safePairOfHands",
            ClientStateId::SolidDefence => "solidDefence",
            ClientStateId::KickTeamMateThrow => "kickTeamMateThrow",
            ClientStateId::ThrowKeg => "throwKeg",
            ClientStateId::RaidingParty => "raidingParty",
            ClientStateId::SelectBlockKind => "selectBlockKind",
            ClientStateId::MaximumCarnage => "maximumCarnage",
            ClientStateId::HitAndRun => "hitAndRun",
            ClientStateId::PutridRegurgitationBlitz => "putridRegurgitationBlitz",
            ClientStateId::Trickster => "trickster",
            ClientStateId::PutridRegurgitationBlock => "putridRegurgitationBlock",
            ClientStateId::KickEmBlock => "kickEmBlock",
            ClientStateId::KickEmBlitz => "kickEmBlitz",
            ClientStateId::ThenIStartedBlastin => "thenIStartedBlastin",
            ClientStateId::Stab => "stab",
            ClientStateId::FuriousOutburst => "furiousOutburst",
            ClientStateId::Punt => "punt",
        }
    }

    pub fn from_name(name: &str) -> Option<ClientStateId> {
        match name {
            "login" => Some(ClientStateId::Login),
            "startGame" => Some(ClientStateId::StartGame),
            "selectPlayer" => Some(ClientStateId::SelectPlayer),
            "move" => Some(ClientStateId::Move),
            "block" => Some(ClientStateId::Block),
            "blitz" => Some(ClientStateId::Blitz),
            "handOver" => Some(ClientStateId::HandOver),
            "pass" => Some(ClientStateId::Pass),
            "spectate" => Some(ClientStateId::Spectate),
            "setup" => Some(ClientStateId::Setup),
            "kickoff" => Some(ClientStateId::Kickoff),
            "pushback" => Some(ClientStateId::Pushback),
            "interception" => Some(ClientStateId::Interception),
            "foul" => Some(ClientStateId::Foul),
            "highKick" => Some(ClientStateId::HighKick),
            "quickSnap" => Some(ClientStateId::QuickSnap),
            "touchback" => Some(ClientStateId::Touchback),
            "waitForOpponent" => Some(ClientStateId::WaitForOpponent),
            "replay" => Some(ClientStateId::Replay),
            "throwTeamMate" => Some(ClientStateId::ThrowTeamMate),
            "kickTeamMate" => Some(ClientStateId::KickTeamMate),
            "swoop" => Some(ClientStateId::Swoop),
            "dumpOff" => Some(ClientStateId::DumpOff),
            "waitForSetup" => Some(ClientStateId::WaitForSetup),
            "gaze" => Some(ClientStateId::Gaze),
            "kickoffReturn" => Some(ClientStateId::KickoffReturn),
            "swarming" => Some(ClientStateId::Swarming),
            "wizard" => Some(ClientStateId::Wizard),
            "passBlock" => Some(ClientStateId::PassBlock),
            "bomb" => Some(ClientStateId::Bomb),
            "illegalSubstitution" => Some(ClientStateId::IllegalSubstitution),
            "gazeMove" => Some(ClientStateId::GazeMove),
            "selectBlitzTarget" => Some(ClientStateId::SelectBlitzTarget),
            "selectGazeTarget" => Some(ClientStateId::SelectGazeTarget),
            "synchronousMultiBlock" => Some(ClientStateId::SynchronousMultiBlock),
            "safePairOfHands" => Some(ClientStateId::PlaceBall),
            "solidDefence" => Some(ClientStateId::SolidDefence),
            "kickTeamMateThrow" => Some(ClientStateId::KickTeamMateThrow),
            "throwKeg" => Some(ClientStateId::ThrowKeg),
            "raidingParty" => Some(ClientStateId::RaidingParty),
            "selectBlockKind" => Some(ClientStateId::SelectBlockKind),
            "maximumCarnage" => Some(ClientStateId::MaximumCarnage),
            "hitAndRun" => Some(ClientStateId::HitAndRun),
            "putridRegurgitationBlitz" => Some(ClientStateId::PutridRegurgitationBlitz),
            "trickster" => Some(ClientStateId::Trickster),
            "putridRegurgitationBlock" => Some(ClientStateId::PutridRegurgitationBlock),
            "kickEmBlock" => Some(ClientStateId::KickEmBlock),
            "kickEmBlitz" => Some(ClientStateId::KickEmBlitz),
            "thenIStartedBlastin" => Some(ClientStateId::ThenIStartedBlastin),
            "stab" => Some(ClientStateId::Stab),
            "furiousOutburst" => Some(ClientStateId::FuriousOutburst),
            "punt" => Some(ClientStateId::Punt),
            _ => None,
        }
    }
}

/// Java: `ClientStateId.toString()` returns `getName()`.
impl std::fmt::Display for ClientStateId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [ClientStateId; 52] = [
        ClientStateId::Login, ClientStateId::StartGame, ClientStateId::SelectPlayer,
        ClientStateId::Move, ClientStateId::Block, ClientStateId::Blitz,
        ClientStateId::HandOver, ClientStateId::Pass, ClientStateId::Spectate,
        ClientStateId::Setup, ClientStateId::Kickoff, ClientStateId::Pushback,
        ClientStateId::Interception, ClientStateId::Foul, ClientStateId::HighKick,
        ClientStateId::QuickSnap, ClientStateId::Touchback, ClientStateId::WaitForOpponent,
        ClientStateId::Replay, ClientStateId::ThrowTeamMate, ClientStateId::KickTeamMate,
        ClientStateId::Swoop, ClientStateId::DumpOff, ClientStateId::WaitForSetup,
        ClientStateId::Gaze, ClientStateId::KickoffReturn, ClientStateId::Swarming,
        ClientStateId::Wizard, ClientStateId::PassBlock, ClientStateId::Bomb,
        ClientStateId::IllegalSubstitution, ClientStateId::GazeMove,
        ClientStateId::SelectBlitzTarget, ClientStateId::SelectGazeTarget,
        ClientStateId::SynchronousMultiBlock, ClientStateId::PlaceBall,
        ClientStateId::SolidDefence, ClientStateId::KickTeamMateThrow,
        ClientStateId::ThrowKeg, ClientStateId::RaidingParty,
        ClientStateId::SelectBlockKind, ClientStateId::MaximumCarnage,
        ClientStateId::HitAndRun, ClientStateId::PutridRegurgitationBlitz,
        ClientStateId::Trickster, ClientStateId::PutridRegurgitationBlock,
        ClientStateId::KickEmBlock, ClientStateId::KickEmBlitz,
        ClientStateId::ThenIStartedBlastin, ClientStateId::Stab,
        ClientStateId::FuriousOutburst, ClientStateId::Punt,
    ];

    #[test]
    fn serde_round_trip() {
        let id = ClientStateId::Swarming;
        let json = serde_json::to_string(&id).unwrap();
        let back: ClientStateId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, back);
    }

    #[test]
    fn client_state_id_all_have_non_null_name() {
        for id in ALL {
            assert!(!id.name().is_empty());
        }
    }

    #[test]
    fn client_state_id_login_name() {
        assert_eq!(ClientStateId::Login.name(), "login");
    }

    #[test]
    fn client_state_id_select_player_name() {
        assert_eq!(ClientStateId::SelectPlayer.name(), "selectPlayer");
    }

    #[test]
    fn client_state_id_block_name() {
        assert_eq!(ClientStateId::Block.name(), "block");
    }

    #[test]
    fn client_state_id_setup_name() {
        assert_eq!(ClientStateId::Setup.name(), "setup");
    }

    #[test]
    fn client_state_id_move_name() {
        assert_eq!(ClientStateId::Move.name(), "move");
    }

    #[test]
    fn client_state_id_all_from_name_round_trip() {
        for id in ALL {
            assert_eq!(ClientStateId::from_name(id.name()), Some(id));
        }
    }

    #[test]
    fn client_state_id_names_are_unique() {
        let unique: std::collections::HashSet<_> = ALL.iter().map(|id| id.name()).collect();
        assert_eq!(unique.len(), ALL.len());
    }

    #[test]
    fn client_state_id_to_string_equals_name() {
        // Java: for each id, assertEquals(id.getName(), id.toString());
        for id in ALL {
            assert_eq!(id.to_string(), id.name());
        }
    }

    #[test]
    fn client_state_id_at_least_forty_variants() {
        assert!(ALL.len() >= 40);
    }
}
