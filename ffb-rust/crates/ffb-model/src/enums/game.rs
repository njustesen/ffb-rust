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

    #[test]
    fn round_trip_name() {
        let statuses = [
            GameStatus::Scheduled,
            GameStatus::Active,
            GameStatus::Finished,
            GameStatus::Loading,
        ];
        for s in &statuses {
            assert_eq!(GameStatus::from_name(s.name()), Some(*s));
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
    fn count_is_nine() {
        let all = [
            GameStatus::Scheduled, GameStatus::Starting, GameStatus::Active,
            GameStatus::Paused, GameStatus::Finished, GameStatus::Uploaded,
            GameStatus::Backuped, GameStatus::Loading, GameStatus::Replaying,
        ];
        assert_eq!(all.len(), 9);
    }

    #[test]
    fn all_have_non_empty_names() {
        for s in [
            GameStatus::Scheduled, GameStatus::Starting, GameStatus::Active,
            GameStatus::Paused, GameStatus::Finished, GameStatus::Uploaded,
            GameStatus::Backuped, GameStatus::Loading, GameStatus::Replaying,
        ] {
            assert!(!s.name().is_empty());
        }
    }

    #[test]
    fn active_type_string_is_a() {
        assert_eq!(GameStatus::Active.type_string(), "A");
    }

    #[test]
    fn finished_type_string_is_f() {
        assert_eq!(GameStatus::Finished.type_string(), "F");
    }

    #[test]
    fn scheduled_type_string_is_o() {
        assert_eq!(GameStatus::Scheduled.type_string(), "O");
    }

    #[test]
    fn loading_name_is_loading() {
        assert_eq!(GameStatus::Loading.name(), "loading");
    }

    #[test]
    fn type_strings_are_unique() {
        let all = [
            GameStatus::Scheduled, GameStatus::Starting, GameStatus::Active,
            GameStatus::Paused, GameStatus::Finished, GameStatus::Uploaded,
            GameStatus::Backuped, GameStatus::Loading, GameStatus::Replaying,
        ];
        let unique: std::collections::HashSet<_> = all.iter().map(|s| s.type_string()).collect();
        assert_eq!(unique.len(), all.len());
    }
}
