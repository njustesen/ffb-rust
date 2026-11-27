use crate::log_format::{GameLog, LogEntry, LogLine, java_log_path_for, rust_log_path_for};

/// Result of comparing a Java log against a Rust log for one seed.
#[derive(Debug)]
pub struct CompareResult {
    pub matches: bool,
    /// Index of the first divergent line (0-based), or total line count if all match.
    pub divergence_index: usize,
    pub java_event: Option<LogEntry>,
    pub rust_event: Option<LogEntry>,
    pub java_event_count: usize,
    pub rust_event_count: usize,
    pub java_hash: String,
    pub rust_hash: String,
}

impl CompareResult {
    /// A Rust panic is a parity divergence: Java played the game, Rust could not. `main` records the
    /// seed with this so the sweep keeps running and the summary counts it, instead of the process
    /// aborting and leaving a "PARITY FAIL"-counting caller to read zero failures as success.
    pub fn rust_panic(msg: String) -> Self {
        Self {
            matches: false,
            divergence_index: 0,
            java_event: None,
            rust_event: None,
            java_event_count: 0,
            rust_event_count: 0,
            java_hash: String::new(),
            rust_hash: format!("<rust panicked: {msg}>"),
        }
    }
}

/// Compare the Java and Rust parity JSONL logs for `seed` and a specific matchup.
///
/// Reads both log files from disk. Returns a `CompareResult` describing the
/// outcome. If either file is missing, the result is a failure.
pub fn compare_logs(seed: u64, edition: &str, home: &str, away: &str) -> CompareResult {
    let java_path = java_log_path_for(seed, edition, home, away);
    let rust_path = rust_log_path_for(seed, edition, home, away);

    let java_lines = match GameLog::read_from_file(&java_path) {
        Ok(l) => l,
        Err(e) => {
            log::warn!("Could not read Java log for seed {seed} at {java_path}: {e}");
            return failure(0, 0, String::new(), String::new());
        }
    };

    let rust_lines = match GameLog::read_from_file(&rust_path) {
        Ok(l) => l,
        Err(e) => {
            log::warn!("Could not read Rust log for seed {seed} at {rust_path}: {e}");
            let java_hash = GameLog::final_hash(&java_lines).unwrap_or("").to_string();
            return failure(java_lines.len(), 0, java_hash, String::new());
        }
    };

    compare_lines(&java_lines, &rust_lines)
}

/// The verdict for one game from its two parsed logs.
///
/// A game matches only when EVERY step agrees (state hash entering the step, post hash, turn, half,
/// active team), both logs hold the same number of steps, the start states agree, and the final
/// hashes agree. Before §Q1.1 the final hash alone decided: a mid-game difference that converged
/// again, or an extra or missing activation, could never fail a game.
pub fn compare_lines(java_lines: &[LogLine], rust_lines: &[LogLine]) -> CompareResult {
    let java_hash = GameLog::final_hash(java_lines).unwrap_or("").to_string();
    let rust_hash = GameLog::final_hash(rust_lines).unwrap_or("").to_string();

    // Only compare Step lines here (game_start / game_end are checked below).
    let java_steps: Vec<(usize, &LogLine)> = java_lines.iter().enumerate()
        .filter(|(_, l)| matches!(l, LogLine::Step { .. }))
        .collect();
    let rust_steps: Vec<(usize, &LogLine)> = rust_lines.iter().enumerate()
        .filter(|(_, l)| matches!(l, LogLine::Step { .. }))
        .collect();

    let min_len = java_steps.len().min(rust_steps.len());
    let first_step_diff = (0..min_len).find(|&i| steps_differ(java_steps[i].1, rust_steps[i].1));
    let start_hash = |lines: &[LogLine]| lines.iter().find_map(|l| match l {
        LogLine::GameStart { state_hash, .. } => Some(state_hash.clone()),
        _ => None,
    });
    let starts_agree = start_hash(java_lines) == start_hash(rust_lines);

    if first_step_diff.is_none()
        && java_steps.len() == rust_steps.len()
        && starts_agree
        && !java_hash.is_empty()
        && java_hash == rust_hash
    {
        let n = java_lines.len();
        return CompareResult {
            matches: true,
            divergence_index: n,
            java_event: None,
            rust_event: None,
            java_event_count: n,
            rust_event_count: rust_lines.len(),
            java_hash,
            rust_hash,
        };
    }

    // The first differing step; with every shared step equal, the first step one side lacks
    // (min_len), which points at the extra activation.
    let divergence_index = first_step_diff.unwrap_or(min_len);

    let java_event = java_steps.get(divergence_index).map(|(idx, line)| LogEntry {
        index: *idx as u64,
        line: (*line).clone(),
    });
    let rust_event = rust_steps.get(divergence_index).map(|(idx, line)| LogEntry {
        index: *idx as u64,
        line: (*line).clone(),
    });

    CompareResult {
        matches: false,
        divergence_index,
        java_event,
        rust_event,
        java_event_count: java_lines.len(),
        rust_event_count: rust_lines.len(),
        java_hash,
        rust_hash,
    }
}

fn steps_differ(a: &LogLine, b: &LogLine) -> bool {
    match (a, b) {
        (LogLine::Step { state_hash: ha, turn: ta, half: ha2, active: aa, post_hash: pa, .. },
         LogLine::Step { state_hash: hb, turn: tb, half: hb2, active: ab, post_hash: pb, .. }) => {
            // Compare the state hashes (entering and after the step) and turn metadata.
            // `chosen` is a log annotation (player IDs differ between Java/Rust naming
            // conventions) and does not affect game correctness — state hashes verify that.
            ha != hb || pa != pb || ta != tb || ha2 != hb2 || aa != ab
        }
        _ => true,
    }
}

fn failure(java_count: usize, rust_count: usize, java_hash: String, rust_hash: String) -> CompareResult {
    CompareResult {
        matches: false,
        divergence_index: 0,
        java_event: None,
        rust_event: None,
        java_event_count: java_count,
        rust_event_count: rust_count,
        java_hash,
        rust_hash,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn start(h: &str) -> LogLine {
        LogLine::GameStart { i: 0, home: "h".into(), away: "a".into(), seed: 1, state_hash: h.into() }
    }
    fn step(i: u64, h: &str, post: &str) -> LogLine {
        LogLine::Step {
            i, turn: 1, half: 1, active: "home".into(), dialog: String::new(), state_hash: h.into(),
            actions: vec![], chosen: String::new(), dice: vec![], post_hash: post.into(), state: None,
        }
    }
    fn end(h: &str) -> LogLine {
        LogLine::GameEnd { i: 9, home_score: 0, away_score: 0, state_hash: h.into() }
    }

    #[test]
    fn identical_logs_match() {
        let log = vec![start("s"), step(1, "a", "b"), step(2, "b", "c"), end("c")];
        let r = compare_lines(&log, &log.clone());
        assert!(r.matches);
    }

    #[test]
    fn a_mid_game_difference_that_converges_fails() {
        let java = vec![start("s"), step(1, "a", "b"), step(2, "b", "c"), end("c")];
        let rust = vec![start("s"), step(1, "a", "X"), step(2, "X", "c"), end("c")];
        let r = compare_lines(&java, &rust);
        assert!(!r.matches, "same final hash, different middle must fail");
        assert_eq!(r.divergence_index, 0);
    }

    #[test]
    fn an_extra_activation_fails() {
        let java = vec![start("s"), step(1, "a", "b"), end("b")];
        let rust = vec![start("s"), step(1, "a", "b"), step(2, "b", "b"), end("b")];
        let r = compare_lines(&java, &rust);
        assert!(!r.matches, "one more step on one side must fail");
        assert_eq!(r.divergence_index, 1);
    }

    #[test]
    fn a_different_start_state_fails() {
        let java = vec![start("s"), step(1, "a", "b"), end("b")];
        let rust = vec![start("t"), step(1, "a", "b"), end("b")];
        assert!(!compare_lines(&java, &rust).matches);
    }

    #[test]
    fn a_different_final_hash_fails() {
        let java = vec![start("s"), step(1, "a", "b"), end("b")];
        let rust = vec![start("s"), step(1, "a", "b"), end("z")];
        assert!(!compare_lines(&java, &rust).matches);
    }
}
