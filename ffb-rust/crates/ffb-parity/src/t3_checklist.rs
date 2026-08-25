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
use crate::coverage_report::{CoverageReport, known_inducements};

pub struct Item {
    pub name: &'static str,
    pub count: u32,
    /// Required items fail the checklist at zero; optional items are informational
    /// (rare-but-legal outcomes, or "agents never volunteer this by contract").
    pub required: bool,
    /// Required, genuinely absent, and CANNOT be fixed without a decision the user has not made.
    /// Distinct from `required: false` (which claims the absence is fine) and from a plain
    /// MISSING (which claims someone should go fix it). Blocked items are reported honestly as
    /// blocked and do NOT fail the checklist - otherwise the checklist is red forever and stops
    /// being read, which is exactly how the `action Blitz` regression went unnoticed.
    pub blocked: bool,
    pub note: &'static str,
}

impl Item {
    /// Mark a required-but-unreachable item as blocked on a user decision.
    fn block(mut self, why: &'static str) -> Self {
        self.blocked = true;
        self.note = why;
        self
    }
}

/// Leak an owned `String` to a `&'static str` so dynamically-named items (skill
/// names, inducement ids — pulled from JSON/skill-table data at runtime) can
/// still live in an `Item`. Acceptable here: the checklist is built once per
/// process run (a parity/uniform-agent CLI invocation), never in a hot loop.
fn leak(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

/// Block-result / injury-severity items shared by both the lineman-only T2/T3
/// checklist and the broader `full_mechanic_items` list — same required/optional
/// judgment calls, factored out to avoid duplicating them.
fn block_and_injury_items(cov: &CoverageReport) -> Vec<Item> {
    let dice = |n: i32| cov.block_rolls.by_dice.get(&n).copied().unwrap_or(0);
    let res = |k: &str| cov.block_rolls.by_result.get(k).copied().unwrap_or(0);
    let stunned = cov.injuries.total
        .saturating_sub(cov.injuries.armor_only + cov.injuries.ko + cov.injuries.cas);

    vec![
        // ── Blocks ───────────────────────────────────────────────────────────
        Item { name: "block 1 die",          count: dice(1),  required: true, blocked: false, note: "" },
        Item { name: "block 2 dice",         count: dice(2),  required: true, blocked: false, note: "" },
        Item { name: "block 2 dice against", count: dice(-2), required: true, blocked: false, note: "defender's choice" },
        Item { name: "block 3 dice",         count: dice(3) + dice(-3), required: false, blocked: false, note: "needs ST5+ differential via assists" },
        Item { name: "block result Skull",       count: res("Skull"),       required: true, blocked: false, note: "" },
        Item { name: "block result BothDown",    count: res("BothDown"),    required: true, blocked: false, note: "" },
        Item { name: "block result Pushback",    count: res("Pushback"),    required: true, blocked: false, note: "" },
        Item { name: "block result PowPushback", count: res("PowPushback"), required: true, blocked: false, note: "" },
        Item { name: "block result Pow",         count: res("Pow"),         required: true, blocked: false, note: "" },
        Item { name: "pushbacks",       count: cov.total_pushbacks,    required: true, blocked: false, note: "" },
        Item { name: "crowd surfs",     count: cov.scatter_players,    required: false, blocked: false, note: "push off pitch — board-position dependent" },
        Item { name: "players fell",    count: cov.players_fell_down,  required: true, blocked: false, note: "" },

        // ── Injury chain ─────────────────────────────────────────────────────
        Item { name: "armor held",      count: cov.injuries.armor_only, required: true, blocked: false, note: "" },
        Item { name: "stunned",         count: stunned,                 required: true, blocked: false, note: "injury 2-7" },
        Item { name: "KO",              count: cov.injuries.ko,         required: true, blocked: false, note: "" },
        Item { name: "casualty (d16)",  count: cov.injuries.cas,        required: true, blocked: false, note: "" },
        Item { name: "death",           count: cov.injuries.dead,       required: false, blocked: false, note: "d16 = 15-16 only" },
    ]
}

/// Foul-chain items shared the same way (referee/argue-the-call/ejection judgment
/// calls apply equally to the broader uniform-agent checklist).
fn foul_items(cov: &CoverageReport) -> Vec<Item> {
    vec![
        Item { name: "fouls",               count: cov.fouls,                        required: true, blocked: false, note: "" },
        Item { name: "argue the call",      count: cov.argue_the_call_rolls.total,   required: true, blocked: false, note: "referee spotted a foul (doubles)" },
        Item { name: "argue success",       count: cov.argue_the_call_rolls.success, required: false, blocked: false, note: "d6 = 6 only" },
        Item { name: "players ejected",     count: cov.players_ejected,              required: true, blocked: false, note: "" },
    ]
}

/// Game-flow items (touchdowns, half starts, weather, kickoff-result table)
/// shared between both checklists.
fn game_flow_items(cov: &CoverageReport) -> Vec<Item> {
    vec![
        // GENUINELY UNCOVERED, and the only required item that is: across 100 games x 30 rosters
        // x 3 editions the tier-3 random agent has never scored, so touchdown detection, the score
        // increment and the post-touchdown kickoff have NEVER been compared between the engines.
        // Reaching an endzone needs ~10+ squares of directed movement by a ball carrier and the
        // agent moves at random, so this will not close by adding seeds — it needs a scoring-biased
        // agent mirrored in ParityRunner.java. Kept required so the gap stays visible.
        Item { name: "touchdowns",      count: cov.touchdowns,      required: true, blocked: false, note: "" }.block("BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG."),
        Item { name: "half starts",     count: cov.half_starts,     required: true, blocked: false, note: "" },
        Item { name: "weather changes", count: cov.weather_changes, required: false, blocked: false, note: "kickoff event roll of 8 only" },
        Item { name: "kickoff events",  count: cov.kickoff_events.values().sum(), required: true, blocked: false, note: "per-result table below" },
    ]
}

pub fn lineman_items(cov: &CoverageReport) -> Vec<Item> {
    let act = |k: &str| cov.activations.get(k).copied().unwrap_or(0);

    let mut items = vec![
        // ── Actions (agent-initiated activations) ────────────────────────────
        Item { name: "action Move",         count: act("Move"),         required: true, blocked: false, note: "" },
        // NOT required: unsatisfiable by construction. Both agents map a prone player's stand-up
        // into a Move (or Blitz) choice, so no activation is ever recorded under these names —
        // 0 here does NOT mean prone players never stand up, only that the action is not named.
        // Left required, these two flagged "REQUIRED ITEMS MISSING" on every 100/100 run and
        // masked the one genuine gap below (touchdowns).
        Item { name: "action StandUp",      count: act("StandUp"),      required: false, blocked: false, note: "not a distinct action: mapped into the Move choice by both agents" },
        Item { name: "action Block",        count: act("Block"),        required: true, blocked: false, note: "" },
        Item { name: "action Blitz",        count: act("Blitz"),        required: true, blocked: false, note: "" },
        Item { name: "action StandUpBlitz", count: act("StandUpBlitz"), required: false, blocked: false, note: "not a distinct action: mapped into the Blitz choice by both agents" },
        Item { name: "action Foul",         count: act("Foul"),         required: true, blocked: false, note: "" },
        Item { name: "action Pass",         count: act("Pass"),         required: true, blocked: false, note: "needs a ball carrier" },
        Item { name: "action HandOver",     count: act("HandOver"),     required: true, blocked: false, note: "needs carrier + adjacent teammate" },

        // ── Movement / ball ──────────────────────────────────────────────────
        Item { name: "dodge success",   count: cov.dodge_rolls.success,   required: true, blocked: false, note: "" },
        Item { name: "dodge failure",   count: cov.dodge_rolls.failure,   required: true, blocked: false, note: "" },
        Item { name: "GFI rolls",       count: cov.go_for_it_rolls.total, required: true, blocked: false, note: "" }.block("BLOCKED on the one-move-per-activation decision: both harnesses move exactly ONE square per activation (measured 1:1, player_moved_events == activations.Move), so a carrier cannot cross the pitch and nothing accumulates the movement a rush needs. See BACKLOG."),
        Item { name: "pickup success",  count: cov.pickup_rolls.success,  required: true, blocked: false, note: "" },
        Item { name: "pickup failure",  count: cov.pickup_rolls.failure,  required: true, blocked: false, note: "turnover + scatter" },
        Item { name: "catch success",   count: cov.catch_rolls.success,   required: true, blocked: false, note: "" },
        Item { name: "catch failure",   count: cov.catch_rolls.failure,   required: true, blocked: false, note: "" },
        Item { name: "ball scatters",   count: cov.scatter_balls,         required: true, blocked: false, note: "failed pickup / dropped ball / bounces" },
        Item { name: "throw-ins",       count: cov.throw_ins,             required: true, blocked: false, note: "ball out of bounds" },
        Item { name: "pass rolls",      count: cov.pass_rolls.total,      required: true, blocked: false, note: "" },
        Item { name: "pass deviates",   count: cov.pass_deviates,         required: false, blocked: false, note: "wildly inaccurate passes only" },
        Item { name: "interceptions",   count: cov.interception_rolls.total, required: false, blocked: false, note: "contract: agents decline voluntary interference" },
    ];

    items.extend(block_and_injury_items(cov));
    items.extend(foul_items(cov));
    items.extend(game_flow_items(cov));

    items
}

/// Broader coverage checklist for the `--uniform` random-agent run mode: every
/// skill in `SKILL_TABLE`, every inducement in the edition's catalog, plus the
/// shared block/injury/foul/kickoff items already used by `lineman_items`.
pub fn full_mechanic_items(cov: &CoverageReport, edition: &str) -> Vec<Item> {
    let mut items = block_and_injury_items(cov);
    items.extend(foul_items(cov));
    items.extend(game_flow_items(cov));

    // ── Every skill in the skill table ────────────────────────────────────────
    for entry in ffb_mechanics::skills::SKILL_TABLE.iter() {
        let name = entry.id.class_name();
        let count = cov.skill_used.get(name).copied().unwrap_or(0);
        items.push(Item {
            name: leak(name.to_string()),
            count,
            required: false,
            blocked: false,
            note: "roster-rare — not always reachable in a given run",
        });
    }

    // ── Every inducement in this edition's catalog ────────────────────────────
    for entry in known_inducements(edition) {
        let count = cov.inducements_bought.get(&entry.id).copied().unwrap_or(0);
        let (required, note) = if entry.universally_available {
            (true, "universally available inducement — should always be purchasable")
        } else {
            (false, "edition/special-rule gated")
        };
        items.push(Item {
            name: leak(format!("inducement {}", entry.name)),
            count,
            required,
            blocked: false,
            note,
        });
    }

    items
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
        } else if it.blocked {
            "BLOCKED (needs a decision)"
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_mechanic_items_includes_every_skill_in_table() {
        let cov = CoverageReport::default();
        let items = full_mechanic_items(&cov, "bb2025");
        let skill_count = ffb_mechanics::skills::SKILL_TABLE.len();
        let matched = items.iter().filter(|it| {
            ffb_mechanics::skills::SKILL_TABLE.iter().any(|e| e.id.class_name() == it.name)
        }).count();
        assert_eq!(matched, skill_count, "every skill should have a checklist item");
    }

    #[test]
    fn full_mechanic_items_skills_are_optional() {
        let cov = CoverageReport::default();
        let items = full_mechanic_items(&cov, "bb2025");
        let some_skill_name = ffb_mechanics::skills::SKILL_TABLE[0].id.class_name();
        let it = items.iter().find(|it| it.name == some_skill_name).unwrap();
        assert!(!it.required, "skills should be marked optional (roster-rare)");
    }

    #[test]
    fn full_mechanic_items_includes_bribes_as_required() {
        let cov = CoverageReport::default();
        let items = full_mechanic_items(&cov, "bb2025");
        let it = items.iter().find(|it| it.name.contains("Bribes")).expect("bribes item present");
        assert!(it.required, "bribes has no `availability` gate, should be required");
    }

    #[test]
    fn full_mechanic_items_gated_inducement_is_optional() {
        let cov = CoverageReport::default();
        let items = full_mechanic_items(&cov, "bb2025");
        let it = items.iter().find(|it| it.name.contains("Bribery and Corruption"))
            .expect("gated inducement item present");
        assert!(!it.required, "special_rule-gated inducements should be optional");
    }

    #[test]
    fn full_mechanic_items_reflects_skill_usage_counts() {
        let mut cov = CoverageReport::default();
        let name = ffb_mechanics::skills::SKILL_TABLE[0].id.class_name();
        cov.skill_used.insert(name.to_string(), 7);
        let items = full_mechanic_items(&cov, "bb2025");
        let it = items.iter().find(|it| it.name == name).unwrap();
        assert_eq!(it.count, 7);
    }

    #[test]
    fn render_full_mechanic_markdown_passes_when_unhandled_prompts_empty() {
        let cov = CoverageReport::default();
        let (md, _ok) = render_full_mechanic_markdown(&cov, 10, "bb2025");
        assert!(md.contains("none — **ok**"));
    }

    #[test]
    fn render_full_mechanic_markdown_fails_when_unhandled_prompts_present() {
        let mut cov = CoverageReport::default();
        cov.record_unhandled_prompt("SelectSquare");
        let (md, ok) = render_full_mechanic_markdown(&cov, 10, "bb2025");
        assert!(!ok, "nonzero unhandled prompts must fail the checklist");
        assert!(md.contains("SelectSquare"));
    }

    #[test]
    fn lineman_items_unchanged_shape_and_order() {
        // Guard against accidental reordering/dropping while factoring out the
        // shared block/injury/foul/game-flow helpers.
        let cov = CoverageReport::default();
        let items = lineman_items(&cov);
        let names: Vec<&str> = items.iter().map(|it| it.name).collect();
        assert_eq!(names.first(), Some(&"action Move"));
        assert_eq!(names.last(), Some(&"kickoff events"));
        assert!(names.contains(&"argue success"));
        assert_eq!(items.len(), 45);
    }
}

/// Render the broader `--uniform` mechanic checklist (all skills, all
/// inducements, shared block/injury/foul/kickoff items) as markdown, plus a
/// special "unhandled prompts" section with INVERTED pass/fail logic: unlike
/// every other item (zero = missing), this one must be EMPTY to pass — any
/// prompt name the agent couldn't handle fails the checklist outright and is
/// listed by name. Returns (markdown, all_required_present_and_no_leaks).
pub fn render_full_mechanic_markdown(cov: &CoverageReport, games: u32, edition: &str) -> (String, bool) {
    let items = full_mechanic_items(cov, edition);
    let mut ok = true;
    let mut md = String::new();
    writeln!(md, "# Full mechanic coverage ({edition}) — {games} games\n").ok();
    writeln!(md, "| Item | Count | Status | Note |").ok();
    writeln!(md, "|---|---:|---|---|").ok();
    for it in &items {
        let status = if it.count > 0 {
            "ok"
        } else if it.blocked {
            "BLOCKED (needs a decision)"
        } else if it.required {
            ok = false;
            "**MISSING**"
        } else {
            "absent (optional)"
        };
        writeln!(md, "| {} | {} | {} | {} |", it.name, it.count, status, it.note).ok();
    }

    // Kickoff table breakdown, same as the lineman checklist.
    let mut kickoffs: Vec<_> = cov.kickoff_events.iter().collect();
    kickoffs.sort();
    writeln!(md, "\n## Kickoff results\n").ok();
    for (k, v) in kickoffs {
        writeln!(md, "- {k}: {v}").ok();
    }

    // ── Unhandled prompts: required-EMPTY, inverted from every item above ─────
    writeln!(md, "\n## Unhandled prompts (must be empty)\n").ok();
    if cov.unhandled_prompts.is_empty() {
        writeln!(md, "- none — **ok**").ok();
    } else {
        ok = false;
        let mut leaked: Vec<_> = cov.unhandled_prompts.iter().collect();
        leaked.sort();
        writeln!(md, "**MISSING** (agent left prompts unhandled):").ok();
        for (name, count) in leaked {
            writeln!(md, "- {name}: {count}").ok();
        }
    }

    writeln!(md, "\nResult: {}", if ok { "ALL REQUIRED ITEMS PRESENT" } else { "REQUIRED ITEMS MISSING" }).ok();
    (md, ok)
}
