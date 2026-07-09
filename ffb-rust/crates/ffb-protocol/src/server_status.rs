use serde::{Deserialize, Serialize};

/// 1:1 translation of `com.fumbbl.ffb.net.ServerStatus`.
/// Enumeration of well-known server-side error codes sent in status messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ServerStatus {
    /// Unknown coach login attempt.
    ErrorUnknownCoach,
    /// Wrong password supplied.
    ErrorWrongPassword,
    /// Game name is already in use.
    ErrorGameInUse,
    /// The joining team does not belong to this coach.
    ErrorNotYourTeam,
    /// No game exists with the requested id.
    ErrorUnknownGameId,
    /// A coach tried to play a team against itself.
    ErrorSameTeam,
    /// A FUMBBL API error occurred.
    FumbblError,
    /// The requested replay is not yet available.
    ReplayUnavailable,
}

impl ServerStatus {
    /// Human-readable short name (matches Java `getName()`).
    pub fn name(self) -> &'static str {
        match self {
            Self::ErrorUnknownCoach => "Unknown Coach",
            Self::ErrorWrongPassword => "Wrong Password",
            Self::ErrorGameInUse => "Game In Use",
            Self::ErrorNotYourTeam => "Not Your Team",
            Self::ErrorUnknownGameId => "Unknown Game Id",
            Self::ErrorSameTeam => "Same Team",
            Self::FumbblError => "Fumbbl Error",
            Self::ReplayUnavailable => "Replay Unavailable",
        }
    }

    /// Full user-facing error message (matches Java `getMessage()`).
    pub fn message(self) -> &'static str {
        match self {
            Self::ErrorUnknownCoach => "Unknown Coach!",
            Self::ErrorWrongPassword => "Wrong Password!",
            Self::ErrorGameInUse => "A Game with this name is already in use!",
            Self::ErrorNotYourTeam => "The team you wanted to join with is not yours!",
            Self::ErrorUnknownGameId => "There is no game with the given id!",
            Self::ErrorSameTeam => "You cannot play a team against itself!",
            Self::FumbblError => "Fumbbl Error",
            Self::ReplayUnavailable => "The replay for this game is currently unavailable.",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_matches_java() {
        assert_eq!(ServerStatus::ErrorUnknownCoach.name(), "Unknown Coach");
        assert_eq!(ServerStatus::ReplayUnavailable.name(), "Replay Unavailable");
    }

    #[test]
    fn message_non_empty_for_all_variants() {
        let variants = [
            ServerStatus::ErrorUnknownCoach,
            ServerStatus::ErrorWrongPassword,
            ServerStatus::ErrorGameInUse,
            ServerStatus::ErrorNotYourTeam,
            ServerStatus::ErrorUnknownGameId,
            ServerStatus::ErrorSameTeam,
            ServerStatus::FumbblError,
            ServerStatus::ReplayUnavailable,
        ];
        for v in variants {
            assert!(!v.message().is_empty(), "{v:?} message must not be empty");
        }
    }

    #[test]
    fn serde_round_trip() {
        let status = ServerStatus::ErrorGameInUse;
        let json = serde_json::to_string(&status).unwrap();
        let back: ServerStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(back, status);
    }
}
