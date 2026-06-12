//! Tier-3 lineman coverage checklist.
//!
//! Verifies that a tier-3 parity suite actually exercised every action and event
//! reachable by skill-less linemen — so "100/100 parity" provably covers the full
//! lineman mechanic surface, not just whatever the seeds happened to roll.
//!
//! Counts come from `CoverageReport` tallies of the Rust engine's `GameEvent`s.
//! Because every per-activation state hash matches Java, the Rust events are a
//! faithful proxy for what happened in both engines.

use std::fmt::Write as _;
use crate::coverage_report::CoverageReport;

pub struct Item {
    pub name: &'static str,
    pub count: u32,
    /// Required items fail the checklist at zero; optional items are informational
    /// (rare-but-legal outcomes, or "agents never volunteer this by contract").
    pub required: bool,
    pub note: &'static str,
}

pub fn lineman_items(cov: &CoverageReport) -> Vec<Item> {
    let act = |k: &str| cov.activations.get(k).copied().unwrap_or(0);
    let dice = |n: i32| cov.block_rolls.by_dice.get(&n).copied().unwrap_or(0);
    let res = |k: &str| cov.block_rolls.by_result.get(k).copied().unwrap_or(0);
    let stunned = cov.injuries.total
        .saturating_sub(cov.injuries.armor_only + cov.injuries.ko + cov.injuries.cas);

    vec![
        // ── Actions (agent-initiated activations) ────────────────────────────
        Item { name: "action Move",         count: act("Move"),         required: true,  note: "" },
        Item { name: "action StandUp",      count: act("StandUp"),      required: true,  note: "prone player stands (mapped from Move choice)" },
        Item { name: "action Block",        count: act("Block"),        required: true,  note: "" },
        Item { name: "action Blitz",        count: act("Blitz"),        required: true,  note: "" },
        Item { name: "action StandUpBlitz", count: act("StandUpBlitz"), required: true,  note: "prone + adjacent + blitz available" },
        Item { name: "action Foul",         count: act("Foul"),         required: true,  note: "" },
        Item { name: "action Pass",         count: act("Pass"),         required: true,  note: "needs a ball carrier" },
        Item { name: "action HandOver",     count: act("HandOver"),     required: true,  note: "needs carrier + adjacent teammate" },

        // ── Movement / ball ──────────────────────────────────────────────────
        Item { name: "dodge success",   count: cov.dodge_rolls.success,   required: true,  note: "" },
        Item { name: "dodge failure",   count: cov.dodge_rolls.failure,   required: true,  note: "" },
        Item { name: "GFI rolls",       count: cov.go_for_it_rolls.total, required: true,  note: "" },
        Item { name: "pickup success",  count: cov.pickup_rolls.success,  required: true,  note: "" },
        Item { name: "pickup failure",  count: cov.pickup_rolls.failure,  required: true,  note: "turnover + scatter" },
        Item { name: "catch success",   count: cov.catch_rolls.success,   required: true,  note: "" },
        Item { name: "catch failure",   count: cov.catch_rolls.failure,   required: true,  note: "" },
        Item { name: "ball scatters",   count: cov.scatter_balls,         required: true,  note: "failed pickup / dropped ball / bounces" },
        Item { name: "throw-ins",       count: cov.throw_ins,             required: true,  note: "ball out of bounds" },
        Item { name: "pass rolls",      count: cov.pass_rolls.total,      required: true,  note: "" },
        Item { name: "pass deviates",   count: cov.pass_deviates,         required: false, note: "wildly inaccurate passes only" },
        Item { name: "interceptions",   count: cov.interception_rolls.total, required: false, note: "contract: agents decline voluntary interference" },

        // ── Blocks ───────────────────────────────────────────────────────────
        Item { name: "block 1 die",          count: dice(1),  required: true,  note: "" },
        Item { name: "block 2 dice",         count: dice(2),  required: true,  note: "" },
        Item { name: "block 2 dice against", count: dice(-2), required: true,  note: "defender's choice" },
        Item { name: "block 3 dice",         count: dice(3) + dice(-3), required: false, note: "needs ST5+ differential via assists" },
        Item { name: "block result Skull",       count: res("Skull"),       required: true, note: "" },
        Item { name: "block result BothDown",    count: res("BothDown"),    required: true, note: "" },
        Item { name: "block result Pushback",    count: res("Pushback"),    required: true, note: "" },
        Item { name: "block result PowPushback", count: res("PowPushback"), required: true, note: "" },
        Item { name: "block result Pow",         count: res("Pow"),         required: true, note: "" },
        Item { name: "pushbacks",       count: cov.total_pushbacks,    required: true,  note: "" },
        Item { name: "crowd surfs",     count: cov.scatter_players,    required: false, note: "push off pitch — board-position dependent" },
        Item { name: "players fell",    count: cov.players_fell_down,  required: true,  note: "" },

        // ── Injury chain ─────────────────────────────────────────────────────
        Item { name: "armor held",      count: cov.injuries.armor_only, required: true,  note: "" },
        Item { name: "stunned",         count: stunned,                 required: true,  note: "injury 2-7" },
        Item { name: "KO",              count: cov.injuries.ko,         required: true,  note: "" },
        Item { name: "casualty (d16)",  count: cov.injuries.cas,        required: true,  note: "" },
        Item { name: "death",           count: cov.injuries.dead,       required: false, note: "d16 = 15-16 only" },

        // ── Foul chain ───────────────────────────────────────────────────────
        Item { name: "fouls",               count: cov.fouls,                        required: true,  note: "" },
        Item { name: "argue the call",      count: cov.argue_the_call_rolls.total,   required: true,  note: "referee spotted a foul (doubles)" },
        Item { name: "argue success",       count: cov.argue_the_call_rolls.success, required: false, note: "d6 = 6 only" },
        Item { name: "players ejected",     count: cov.players_ejected,              required: true,  note: "" },

        // ── Game flow ────────────────────────────────────────────────────────
        Item { name: "touchdowns",      count: cov.touchdowns,      required: true,  note: "" },
        Item { name: "half starts",     count: cov.half_starts,     required: true,  note: "" },
        Item { name: "weather changes", count: cov.weather_changes, required: false, note: "kickoff event roll of 8 only" },
        Item { name: "kickoff events",  count: cov.kickoff_events.values().sum(), required: true, note: "per-result table below" },
    ]
}

/// Render the checklist (plus diagnostics) as markdown. Returns (markdown, all_required_present).
pub fn render_markdown(cov: &CoverageReport, games: u32) -> (String, bool) {
    let items = lineman_items(cov);
    let mut ok = true;
    let mut md = String::new();
    writeln!(md, "# T3 lineman coverage — {games} games\n").ok();
    writeln!(md, "| Item | Count | Status | Note |").ok();
    writeln!(md, "|---|---:|---|---|").ok();
    for it in &items {
        let status = if it.count > 0 {
            "ok"
        } else if it.required {
            ok = false;
            "**MISSING**"
        } else {
            "absent (optional)"
        };
        writeln!(md, "| {} | {} | {} | {} |", it.name, it.count, status, it.note).ok();
    }

    // Kickoff table breakdown (11 results; ~2-4 kickoffs per game makes full
    // coverage near-certain over 100 seeds — report whichever occurred).
    let mut kickoffs: Vec<_> = cov.kickoff_events.iter().collect();
    kickoffs.sort();
    writeln!(md, "\n## Kickoff results\n").ok();
    for (k, v) in kickoffs {
        writeln!(md, "- {k}: {v}").ok();
    }

    writeln!(md, "\n## Hash-verified (not evented)\n").ok();
    writeln!(md, "- KO recovery rolls, stunned→prone wake cycle, turnover sequencing and").ok();
    writeln!(md, "  banned-players-stay-off are not separate GameEvents; they are covered by").ok();
    writeln!(md, "  the per-activation state hashes that must match Java exactly.").ok();

    writeln!(md, "\nResult: {}", if ok { "ALL REQUIRED ITEMS PRESENT" } else { "REQUIRED ITEMS MISSING" }).ok();
    (md, ok)
}
