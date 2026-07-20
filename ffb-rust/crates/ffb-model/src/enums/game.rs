use serde::{Deserialize, Serialize};

/// Life-cycle status of a game on the server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GameStatus {
    Scheduled,
    Starting,
    Active,
    Paused,
    Finished,
    Uploaded,
    Backuped,
    /// Transient — not written to DB.
    Loading,
    /// Transient — not written to DB.
    Replaying,
}

impl GameStatus {
    pub fn name(self) -> &'static str {
        match self {
            GameStatus::Scheduled => "scheduled",
            GameStatus::Starting => "starting",
            GameStatus::Active => "active",
            GameStatus::Paused => "paused",
            GameStatus::Finished => "finished",
            GameStatus::Uploaded => "uploaded",
            GameStatus::Backuped => "backuped",
            GameStatus::Loading => "loading",
            GameStatus::Replaying => "replaying",
        }
    }

    pub fn type_string(self) -> &'static str {
        match self {
            GameStatus::Scheduled => "O",
            GameStatus::Starting => "S",
            GameStatus::Active => "A",
            GameStatus::Paused => "P",
            GameStatus::Finished => "F",
            GameStatus::Uploaded => "U",
            GameStatus::Backuped => "B",
            GameStatus::Loading => "L",
            GameStatus::Replaying => "R",
        }
    }

    pub fn from_name(name: &str) -> Option<GameStatus> {
        match name {
            "scheduled" => Some(GameStatus::Scheduled),
            "starting" => Some(GameStatus::Starting),
            "active" => Some(GameStatus::Active),
            "paused" => Some(GameStatus::Paused),
            "finished" => Some(GameStatus::Finished),
            "uploaded" => Some(GameStatus::Uploaded),
            "backuped" => Some(GameStatus::Backuped),
            "loading" => Some(GameStatus::Loading),
            "replaying" => Some(GameStatus::Replaying),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [GameStatus; 9] = [
        GameStatus::Scheduled, GameStatus::Starting, GameStatus::Active,
        GameStatus::Paused, GameStatus::Finished, GameStatus::Uploaded,
        GameStatus::Backuped, GameStatus::Loading, GameStatus::Replaying,
    ];

    #[test]
    fn game_status_all_values_found_by_name() {
        for s in ALL {
            assert_eq!(GameStatus::from_name(s.name()), Some(s));
        }
    }

    #[test]
    fn serde_round_trip() {
        let s = GameStatus::Active;
        let json = serde_json::to_string(&s).unwrap();
        let back: GameStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    #[test]
    fn game_status_count_is_nine() {
        assert_eq!(ALL.len(), 9);
    }

    #[test]
    fn game_status_all_have_non_null_name() {
        for s in ALL {
            assert!(!s.name().is_empty());
        }
    }

    #[test]
    fn game_status_all_have_non_null_type_string() {
        for s in ALL {
            assert!(!s.type_string().is_empty());
        }
    }

    #[test]
    fn game_status_active_name() {
        assert_eq!(GameStatus::Active.name(), "active");
    }

    #[test]
    fn game_status_active_type_string() {
        assert_eq!(GameStatus::Active.type_string(), "A");
    }

    #[test]
    fn game_status_finished_name() {
        assert_eq!(GameStatus::Finished.name(), "finished");
    }

    #[test]
    fn game_status_finished_type_string() {
        assert_eq!(GameStatus::Finished.type_string(), "F");
    }

    #[test]
    fn game_status_scheduled_type_string() {
        assert_eq!(GameStatus::Scheduled.type_string(), "O");
    }

    #[test]
    fn game_status_loading_name() {
        assert_eq!(GameStatus::Loading.name(), "loading");
    }

    #[test]
    fn game_status_type_strings_are_unique() {
        let unique: std::collections::HashSet<_> = ALL.iter().map(|s| s.type_string()).collect();
        assert_eq!(unique.len(), ALL.len());
    }
}
