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
        /// Full state string for human-readable diagnosis (Rust --verbose only; absent in Java logs).
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        state: Option<String>,
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

/// Path of the Java-generated JSONL log for a given seed and matchup.
/// Uses home+away roster names to avoid cross-race contamination when testing multiple races.
pub fn java_log_path(seed: u64) -> String {
    format!("parity/seed_{seed}_java.jsonl")
}

/// Path of the Rust-generated JSONL log for a given seed and matchup.
pub fn rust_log_path(seed: u64) -> String {
    format!("parity/seed_{seed}_rust.jsonl")
}

/// Directory holding one matchup's logs, scoped by edition AND by the two rosters.
///
/// The edition segment is not cosmetic. Without it the three edition matrices share
/// `parity/<matchup>/`, so gating bb2016, bb2020 and bb2025 concurrently has each run
/// overwriting the other's `seed_N_java.jsonl` mid-comparison — the same class of
/// phantom red that a `cargo build` rewriting `ffb-parity.exe` mid-gate produces.
/// Scoped this way the three matrices are independent and can gate in parallel.
pub fn matchup_dir(edition: &str, home: &str, away: &str) -> String {
    matchup_dir_in(&parity_root(), edition, home, away)
}

/// The root every log path hangs off. `FFB_PARITY_ROOT` overrides the default `parity`.
///
/// The edition/matchup scoping above keeps CONCURRENT runs apart, but it cannot keep two
/// runs of the same matchup with different AGENTS apart: the `--agent random` control that
/// follows every heuristic gate wrote into the same directory and destroyed the heuristic's
/// evidence — 14 of 19 bb2020 reds had no analysable log by the time anyone looked. The control
/// runs with `FFB_PARITY_ROOT=parity_random` so the gate's files survive it.
pub fn parity_root() -> String {
    std::env::var("FFB_PARITY_ROOT").unwrap_or_else(|_| "parity".to_string())
}

pub fn matchup_dir_in(root: &str, edition: &str, home: &str, away: &str) -> String {
    format!("{root}/{edition}/{home}_vs_{away}")
}

/// Race-specific log paths — include edition + home + away so no two runs collide.
pub fn java_log_path_for(seed: u64, edition: &str, home: &str, away: &str) -> String {
    format!("{}/seed_{seed}_java.jsonl", matchup_dir(edition, home, away))
}

pub fn rust_log_path_for(seed: u64, edition: &str, home: &str, away: &str) -> String {
    format!("{}/seed_{seed}_rust.jsonl", matchup_dir(edition, home, away))
}

pub fn rust_events_path_for(seed: u64, edition: &str, home: &str, away: &str) -> String {
    format!("{}/seed_{seed}_rust_events.jsonl", matchup_dir(edition, home, away))
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

    /// The three edition matrices must never share a log directory: before the edition
    /// segment existed, gating bb2016 and bb2025 concurrently had each run overwriting the
    /// other's `seed_N_java.jsonl` and reporting reds that did not reproduce in isolation.
    #[test]
    fn parity_root_override_moves_every_path() {
        // Env-free: the override is exercised through the root-taking form so this test cannot
        // race another test that reads the process environment.
        assert_eq!(matchup_dir_in("parity_random", "bb2025", "amazon", "amazon"),
                   "parity_random/bb2025/amazon_vs_amazon");
        assert_eq!(matchup_dir_in("parity", "bb2025", "amazon", "amazon"),
                   matchup_dir("bb2025", "amazon", "amazon"),
                   "default root must be `parity` (no FFB_PARITY_ROOT in the test env)");
    }

    #[test]
    fn log_paths_are_scoped_by_edition_and_matchup() {
        let a = java_log_path_for(7, "bb2016", "ogre", "ogre");
        let b = java_log_path_for(7, "bb2025", "ogre", "ogre");
        assert_ne!(a, b, "two editions must not write the same Java log path");
        assert_eq!(a, "parity/bb2016/ogre_vs_ogre/seed_7_java.jsonl");
        assert_eq!(rust_log_path_for(7, "bb2025", "ogre", "orc"),
                   "parity/bb2025/ogre_vs_orc/seed_7_rust.jsonl");
        assert_eq!(rust_events_path_for(7, "bb2020", "elf", "elf"),
                   "parity/bb2020/elf_vs_elf/seed_7_rust_events.jsonl");
        // Rust and Java logs for one seed live side by side but never collide.
        assert_ne!(java_log_path_for(7, "bb2020", "elf", "elf"),
                   rust_log_path_for(7, "bb2020", "elf", "elf"));
    }

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
            state: None,
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

    #[test]
    fn final_hash_returns_last_game_end_hash() {
        let lines = vec![
            LogLine::GameEnd { i: 0, home_score: 1, away_score: 0, state_hash: "aabb".into() },
        ];
        assert_eq!(GameLog::final_hash(&lines), Some("aabb"));
    }

    #[test]
    fn final_hash_returns_none_for_empty_lines() {
        assert_eq!(GameLog::final_hash(&[]), None);
    }
}
