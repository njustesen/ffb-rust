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

/// Compare the Java and Rust parity JSONL logs for `seed` and a specific matchup.
///
/// Reads both log files from disk. Returns a `CompareResult` describing the
/// outcome. If either file is missing, the result is a failure.
pub fn compare_logs(seed: u64, home: &str, away: &str) -> CompareResult {
    let java_path = java_log_path_for(seed, home, away);
    let rust_path = rust_log_path_for(seed, home, away);

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

    let java_hash = GameLog::final_hash(&java_lines).unwrap_or("").to_string();
    let rust_hash = GameLog::final_hash(&rust_lines).unwrap_or("").to_string();

    // Fast path: if final hashes match, logs agree.
    if !java_hash.is_empty() && java_hash == rust_hash {
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

    // Find first divergent step line for diagnostics.
    // Only compare Step lines (skip game_start / game_end).
    let java_steps: Vec<(usize, &LogLine)> = java_lines.iter().enumerate()
        .filter(|(_, l)| matches!(l, LogLine::Step { .. }))
        .collect();
    let rust_steps: Vec<(usize, &LogLine)> = rust_lines.iter().enumerate()
        .filter(|(_, l)| matches!(l, LogLine::Step { .. }))
        .collect();

    let min_len = java_steps.len().min(rust_steps.len());
    let divergence_index = (0..min_len)
        .find(|&i| steps_differ(java_steps[i].1, rust_steps[i].1))
        .unwrap_or(min_len);

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
        (LogLine::Step { state_hash: ha, turn: ta, half: ha2, active: aa, .. },
         LogLine::Step { state_hash: hb, turn: tb, half: hb2, active: ab, .. }) => {
            // Compare game state hash and turn metadata only.
            // `chosen` is a log annotation (player IDs differ between Java/Rust naming
            // conventions) and does not affect game correctness — state hashes verify that.
            ha != hb || ta != tb || ha2 != hb2 || aa != ab
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
