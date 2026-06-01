use serde::{Deserialize, Serialize};

/// One line in a parity JSONL log.
/// The Java format has three line types: game_start, step, game_end.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LogLine {
    GameStart {
        i: u64,
        home: String,
        away: String,
        seed: u64,
        state_hash: String,
    },
    Step {
        i: u64,
        turn: i32,
        half: i32,
        active: String,
        dialog: String,
        state_hash: String,
        actions: Vec<String>,
        chosen: String,
        dice: Vec<serde_json::Value>,
        post_hash: String,
    },
    GameEnd {
        i: u64,
        home_score: i32,
        away_score: i32,
        state_hash: String,
    },
}

/// A complete game log parsed from a JSONL file.
#[derive(Debug, Clone)]
pub struct GameLog {
    pub seed: u64,
    pub home_roster: String,
    pub away_roster: String,
    pub lines: Vec<LogLine>,
}

impl GameLog {
    /// Write as JSONL (one JSON object per line).
    pub fn write_to_file(&self, path: &str) -> std::io::Result<()> {
        use std::io::Write;
        let mut file = std::fs::File::create(path)?;
        for line in &self.lines {
            let json = serde_json::to_string(line)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            writeln!(file, "{json}")?;
        }
        Ok(())
    }

    /// Read a JSONL log file produced by Java's ParityRunner or by Rust.
    pub fn read_from_file(path: &str) -> std::io::Result<Vec<LogLine>> {
        let content = std::fs::read_to_string(path)?;
        let mut lines = Vec::new();
        for raw in content.lines() {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                continue;
            }
            let line: LogLine = serde_json::from_str(trimmed)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other,
                    format!("parse error in {path}: {e} — line: {trimmed}")))?;
            lines.push(line);
        }
        Ok(lines)
    }

    /// The final state_hash from the game_end line.
    pub fn final_hash(lines: &[LogLine]) -> Option<&str> {
        lines.iter().rev().find_map(|l| {
            if let LogLine::GameEnd { state_hash, .. } = l {
                Some(state_hash.as_str())
            } else {
                None
            }
        })
    }
}

/// Path of the Java-generated JSONL log for a given seed.
pub fn java_log_path(seed: u64) -> String {
    format!("parity/seed_{seed}_java.jsonl")
}

/// Path of the Rust-generated JSONL log for a given seed.
pub fn rust_log_path(seed: u64) -> String {
    format!("parity/seed_{seed}_rust.jsonl")
}

/// A minimal LogEntry type used by comparator for per-line diffs.
#[derive(Debug, Clone, PartialEq)]
pub struct LogEntry {
    pub index: u64,
    pub line: LogLine,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_start_round_trip() {
        let line = LogLine::GameStart {
            i: 0,
            home: "human".into(),
            away: "orc".into(),
            seed: 42,
            state_hash: "abcd1234abcd1234".into(),
        };
        let json = serde_json::to_string(&line).unwrap();
        assert!(json.contains("\"type\":\"game_start\""));
        let back: LogLine = serde_json::from_str(&json).unwrap();
        assert_eq!(line, back);
    }

    #[test]
    fn step_round_trip() {
        let line = LogLine::Step {
            i: 1,
            turn: 1,
            half: 1,
            active: "home".into(),
            dialog: "None".into(),
            state_hash: "1234abcd1234abcd".into(),
            actions: vec!["EndTurn".into()],
            chosen: "EndTurn".into(),
            dice: vec![],
            post_hash: "deadbeefdeadbeef".into(),
        };
        let json = serde_json::to_string(&line).unwrap();
        assert!(json.contains("\"type\":\"step\""));
        let back: LogLine = serde_json::from_str(&json).unwrap();
        assert_eq!(line, back);
    }

    #[test]
    fn game_end_round_trip() {
        let line = LogLine::GameEnd {
            i: 99,
            home_score: 2,
            away_score: 1,
            state_hash: "0000000000000000".into(),
        };
        let json = serde_json::to_string(&line).unwrap();
        assert!(json.contains("\"type\":\"game_end\""));
        let back: LogLine = serde_json::from_str(&json).unwrap();
        assert_eq!(line, back);
    }
}
