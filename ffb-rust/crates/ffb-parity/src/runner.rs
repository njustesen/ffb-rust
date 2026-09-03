use std::collections::HashSet;
use std::process::Command;
use ffb_engine::agent::{RandomAgent, Agent};
use ffb_engine::step::GameState;
use ffb_engine::legal_actions::TeamSide;
use ffb_model::events::GameEvent;
use ffb_model::data::loader::position_json_to_roster_position;
use ffb_model::data::roster_json::PositionJson;
use ffb_model::data::{bb2016_rosters, bb2020_rosters, bb2025_rosters};
use ffb_model::enums::{PlayerGender, PlayerType, Rules};
use ffb_model::enums::SkillId;
use ffb_model::enums::SkillCategory;
use ffb_model::model::player::Player;
use ffb_model::model::roster_position::RosterPosition;
use ffb_model::model::team::Team;
use ffb_model::prompts::AgentPrompt;
use ffb_model::option::game_option_id::{INDUCEMENTS, MAX_PLAYERS_ON_FIELD, MIN_PLAYERS_ON_LOS, MAX_PLAYERS_IN_WIDE_ZONE, MB_STACKS_AGAINST_CHAINSAW, CLAW_DOES_NOT_STACK, ENABLE_STALLING_CHECK, ALLOW_BALL_AND_CHAIN_RE_ROLL};
use crate::log_format::{GameLog, LogLine, java_log_path_for, rust_log_path_for, rust_events_path_for};
use crate::state_hash::state_hash;
use ffb_model::util::state_hash::state_string;

/// Baseline setup-validity options every headless full-game run needs: `Game::new` leaves every
/// option unset (0/false), but `SetupMechanic::check_setup` treats `MAX_PLAYERS_ON_FIELD`/
/// `MIN_PLAYERS_ON_LOS`/`MAX_PLAYERS_IN_WIDE_ZONE` literally — left at their 0 default, it
/// rejects a legitimate 11-player formation as "too many players on the field" (`11 > 0`),
/// stalling the game at the very first kickoff. Values match Java's real
/// `GameOptionFactory` defaults (11 / 3 / 2) exactly.
pub const BASELINE_SETUP_OPTIONS: &[(&str, &str)] = &[
    (MAX_PLAYERS_ON_FIELD, "11"),
    (MIN_PLAYERS_ON_LOS, "3"),
    (MAX_PLAYERS_IN_WIDE_ZONE, "2"),
    // Java's parity game runs with mbStacksAgainstChainsaw enabled (confirmed via JAVA_AVBROKE:
    // a knocked-down Chainsaw wielder's own chainsaw AND the opponent's Mighty Blow both apply to
    // its fall armour). The factory default is false in both engines, so Rust must set it to match
    // Java — otherwise a fallen Looney's armour holds (chainsaw-only +3) where Java breaks it
    // (chainsaw +3 + Mighty Blow +2), diverging the dice (goblin seed 99).
    (MB_STACKS_AGAINST_CHAINSAW, "true"),
    // Java UtilServerStartGame.addDefaultGameOptions (STANDALONE = the parity harness):
    // `ballAndChainRr.setValue(true); game.getOptions().addOption(ballAndChainRr)` — the
    // mixed StepMoveBallAndChain offers a TEAM re-roll of the scatter direction only when
    // this is enabled; Java re-rolled a Fanatic's direction (JBCMOVE same from/orig, fresh
    // roll, r3→r2) while Rust never offered (goblin bb2020 seed 6 i=103).
    (ALLOW_BALL_AND_CHAIN_RE_ROLL, "true"),
    // Same shape, opposite direction. Java's `UtilServerStartGame:247-249` explicitly sets
    // clawDoesNotStack=false, but Rust never applies its equivalent defaults: the list in
    // `util_server_start_game.rs` exists and `add_default_game_options` is DEAD CODE, called from
    // nowhere in any crate. An unset option falls back to the FACTORY default, and this one's
    // factory default is TRUE, so Rust silently ran with Claws-does-not-stack while Java did not.
    //
    // Effect: with Claws AND Mighty Blow on the attacker, Java stacks them (JAVA_AVBROKE
    // `mods=Claws,Mighty Blow modTotal=1 reduced=8 broken=true` on a roll of 7), while Rust
    // discarded the stack and kept Claws alone (`mods=[Claws] modTotal=0 broken=false`), leaving
    // the defender Prone instead of KO'd — necromantic bb2020 seed 65, where a Knuckle Dusters
    // prayer grants the Claws Werewolf a temporary Mighty Blow (+1).
    (CLAW_DOES_NOT_STACK, "false"),
    // Third of the same shape. `UtilServerStartGame:301-303` builds an ENABLE_STALLING_CHECK option
    // set to FALSE and then does NOT add it -- the `addOption` line is commented out -- so Java
    // falls back to `GameOptionFactory`'s default, which is TRUE. Rust treats an unset option as
    // disabled, so its BB2025 stalling rule never ran at all.
    //
    // Effect: Java rolls a d6 in `StallingExtension.handleStaller` for a lone ball-carrier with an
    // open path to the endzone, and Rust rolls nothing (bb2025 seed 26 step 25, uniform sampling:
    // identical boards, rng_calls 46 against 45). Every die after it is then read one position
    // early. The whole rule -- `StepForgoneStalling`, `StallingExtension`, `ReportStallerDetected`
    // -- is implemented and faithful on the Rust side; it was simply switched off.
    (ENABLE_STALLING_CHECK, "true"),
];

/// Invoke the Java parity runner as a subprocess.
///
/// Uses `java -cp <classpath> com.fumbbl.ffb.ai.parity.ParityRunner`
/// Env vars:
///   PARITY_CP  — Java classpath (default: scans for ffb-ai fat jar)
///   FFB_SERVER_DIR — path to the ffb-server directory
/// Writes `parity/seed_{seed}_java.jsonl`.
/// `home_team_id` / `away_team_id` are Java server team IDs (e.g. "teamHumanKalimar").
/// `home_race` / `away_race` are the Rust race names used for the output directory.
pub fn run_java_headless(seed: u64, home_team_id: &str, away_team_id: &str, home_race: &str, away_race: &str, tier: u8, edition: &str) {
    let output_path = java_log_path_for(seed, edition, home_race, away_race);
    let dir = std::path::Path::new(&output_path).parent().unwrap_or(std::path::Path::new("parity"));
    std::fs::create_dir_all(dir).ok();

    let cp = std::env::var("PARITY_CP").unwrap_or_else(|_| {
        let candidates = [
            r"C:\Users\Admin\niels\ffb\ffb\ffb-ai\target\ffb-ai-jar-with-dependencies.jar",
            "../../ffb/ffb/ffb-ai/target/ffb-ai-jar-with-dependencies.jar",
            "../../ffb/ffb-ai/target/ffb-ai-jar-with-dependencies.jar",
            "../ffb/ffb-ai/target/ffb-ai-jar-with-dependencies.jar",
            "ffb-ai/target/ffb-ai-jar-with-dependencies.jar",
        ];
        for c in &candidates {
            if std::path::Path::new(c).exists() {
                return c.to_string();
            }
        }
        "ffb-ai-jar-with-dependencies.jar".to_string()
    });

    let server_dir = std::env::var("FFB_SERVER_DIR").unwrap_or_else(|_| {
        let candidates = [
            r"C:\Users\Admin\niels\ffb\ffb\ffb-server",
            "../../ffb/ffb/ffb-server",
            "../../ffb/ffb-server",
            "../ffb/ffb-server",
            "ffb-server",
        ];
        for c in &candidates {
            if std::path::Path::new(c).exists() {
                return c.to_string();
            }
        }
        "ffb-server".to_string()
    });

    let mut args: Vec<String> = vec!["-cp".into(), cp.clone()];
    if let Some(opt) = jvm_core_opt() { args.push(opt); }
    // `java.util.Collections`' one-arg `shuffle(List)` uses a private shared `Random` seeded from
    // system entropy — non-deterministic within Java itself, and reachable in bb2020 via
    // `StepApplyKickoffResult.handleCheeringFans` (which picks a Prayer to Nuffle). ParityRunner
    // seeds that field by reflection so the engine becomes reproducible; JDK 17 needs the module
    // opened for it. Rust reproduces the identical permutation via
    // `ffb_model::util::java_random::collections_shuffle`.
    args.push("--add-opens".into());
    args.push("java.base/java.util=ALL-UNNAMED".into());

    // Mirror the Rust-side trace env vars onto the Java process.
    if std::env::var_os("FFB_DICE_TRACE").is_some() {
        args.push("-Dffb.diceTrace=true".into());
    }
    if std::env::var_os("FFB_TRACE").is_some() {
        args.push("-Dffb.parityDebug=true".into());
        if std::env::var_os("FFB_DICE_DEEP").is_some() { args.push("-Dffb.parityDebugDeep=true".into()); }
    }
    args.extend([
        "com.fumbbl.ffb.ai.parity.ParityRunner".into(),
        server_dir.clone(),
        home_team_id.into(),
        away_team_id.into(),
        seed.to_string(),
        output_path.clone(),
    ]);
    // Tier 2 invocations stay byte-identical to the historical CLI so older jars work.
    if tier >= 3 {
        args.push("--tier".into());
        args.push(tier.to_string());
    }
    if let Some(rs) = java_ruleset_arg(edition) {
        args.push("--ruleset".into());
        args.push(rs.into());
    }
    let status = Command::new("java").args(&args).status();

    match status {
        Ok(s) if s.success() => {}
        Ok(s) => log::warn!("Java parity runner exited with status {s} for seed {seed}"),
        Err(e) => log::warn!("Could not launch Java parity runner for seed {seed}: {e}"),
    }
}

/// Java ruleset override for non-default editions. The Java side hardcodes RULESVERSION=BB2025
/// (UtilServerStartGame.addDefaultGameOptions), so bb2025 passes nothing (older jars keep
/// working) and other editions send `--ruleset <RULES>` for ParityRunner/HeadlessGameSetup.
fn java_ruleset_arg(edition: &str) -> Option<&'static str> {
    match edition {
        "bb2016" => Some("BB2016"),
        "bb2020" => Some("BB2020"),
        _ => None,
    }
}

/// When many JVMs run concurrently (parallel matrix), each one otherwise sizes its parallel-GC
/// and JIT-compiler thread pools to ALL visible cores, so N JVMs spawn ~N×cores threads and
/// thrash. `PARITY_JVM_CORES=k` passes `-XX:ActiveProcessorCount=k` so each JVM only sees k cores.
/// Unset (solo runs) → no flag, JVM uses all cores at full speed.
fn jvm_core_opt() -> Option<String> {
    let n = std::env::var("PARITY_JVM_CORES").ok()?;
    let n = n.trim();
    n.parse::<u32>().ok().filter(|&k| k >= 1).map(|k| format!("-XX:ActiveProcessorCount={k}"))
}

/// Resolve the Java classpath (fat jar) — env `PARITY_CP` or the first existing candidate.
fn resolve_parity_cp() -> String {
    std::env::var("PARITY_CP").unwrap_or_else(|_| {
        let candidates = [
            r"C:\Users\Admin\niels\ffb\ffb\ffb-ai\target\ffb-ai-jar-with-dependencies.jar",
            "../../ffb/ffb/ffb-ai/target/ffb-ai-jar-with-dependencies.jar",
            "../../ffb/ffb-ai/target/ffb-ai-jar-with-dependencies.jar",
            "../ffb/ffb-ai/target/ffb-ai-jar-with-dependencies.jar",
            "ffb-ai/target/ffb-ai-jar-with-dependencies.jar",
        ];
        for c in &candidates {
            if std::path::Path::new(c).exists() { return c.to_string(); }
        }
        "ffb-ai-jar-with-dependencies.jar".to_string()
    })
}

/// Resolve the ffb-server directory — env `FFB_SERVER_DIR` or the first existing candidate.
fn resolve_server_dir() -> String {
    std::env::var("FFB_SERVER_DIR").unwrap_or_else(|_| {
        let candidates = [
            r"C:\Users\Admin\niels\ffb\ffb\ffb-server",
            "../../ffb/ffb/ffb-server",
            "../../ffb/ffb-server",
            "../ffb/ffb-server",
            "ffb-server",
        ];
        for c in &candidates {
            if std::path::Path::new(c).exists() { return c.to_string(); }
        }
        "ffb-server".to_string()
    })
}

/// BATCH variant of [`run_java_headless`]: run the whole seed range `[seed_start, seed_end]` in a
/// SINGLE JVM invocation via ParityRunner's `--seed-end` batch mode, writing one JSONL per seed to
/// the same paths [`java_log_path_for`] produces. This amortizes JVM start-up, fat-jar
/// class-loading and server construction across every seed (they were paid per-seed before).
pub fn run_java_headless_range(
    seed_start: u64, seed_end: u64,
    home_team_id: &str, away_team_id: &str, home_race: &str, away_race: &str, tier: u8,
    edition: &str, agent_spec: AgentSpec,
) {
    let dir = crate::log_format::matchup_dir(edition, home_race, away_race);
    std::fs::create_dir_all(&dir).ok();
    // Path template: ParityRunner substitutes {seed} per seed → matches java_log_path_for exactly.
    let output_template = format!("{dir}/seed_{{seed}}_java.jsonl");

    let cp = resolve_parity_cp();
    let server_dir = resolve_server_dir();

    let mut args: Vec<String> = vec!["-cp".into(), cp];
    if let Some(opt) = jvm_core_opt() { args.push(opt); }
    // `java.util.Collections`' one-arg `shuffle(List)` uses a private shared `Random` seeded from
    // system entropy — non-deterministic within Java itself, and reachable in bb2020 via
    // `StepApplyKickoffResult.handleCheeringFans` (which picks a Prayer to Nuffle). ParityRunner
    // seeds that field by reflection so the engine becomes reproducible; JDK 17 needs the module
    // opened for it. Rust reproduces the identical permutation via
    // `ffb_model::util::java_random::collections_shuffle`.
    args.push("--add-opens".into());
    args.push("java.base/java.util=ALL-UNNAMED".into());

    if std::env::var_os("FFB_DICE_TRACE").is_some() { args.push("-Dffb.diceTrace=true".into()); }
    if std::env::var_os("FFB_TRACE").is_some() { args.push("-Dffb.parityDebug=true".into()); }
    if std::env::var_os("FFB_DICE_DEEP").is_some() { args.push("-Dffb.parityDebugDeep=true".into()); }
    args.extend([
        "com.fumbbl.ffb.ai.parity.ParityRunner".into(),
        server_dir,
        home_team_id.into(),
        away_team_id.into(),
        seed_start.to_string(),
        output_template,
    ]);
    // Batch always implies the tier-3 CLI shape (matrix runs are tier 3).
    args.push("--tier".into());
    args.push(tier.to_string());
    args.push("--seed-end".into());
    args.push(seed_end.to_string());
    if let Some(rs) = java_ruleset_arg(edition) {
        args.push("--ruleset".into());
        args.push(rs.into());
    }
    // The heuristic ladder: BOTH engines must be given the same scale and the same class mask, or
    // they disagree about which side answers a prompt -- a divergence produced by the harness
    // rather than by either engine. See AGENT_CONTRACT_HEURISTIC.md.
    if let AgentSpec::Random { multimove } = agent_spec {
        if multimove > 1 {
            args.push("--multimove".into());
            args.push(multimove.to_string());
        }
    }
    if let AgentSpec::Heuristic { temp_scale, classes, .. } = agent_spec {
        args.push("--agent".into());
        args.push("heuristic".into());
        args.push("--heur-scale".into());
        args.push(temp_scale.to_string());
        args.push("--heur-classes".into());
        args.push(classes.to_spec());
    }

    match Command::new("java").args(&args).status() {
        Ok(s) if s.success() => {}
        Ok(s) => log::warn!("Java parity runner (batch {seed_start}-{seed_end}) exited with status {s}"),
        Err(e) => log::warn!("Could not launch Java parity runner (batch {seed_start}-{seed_end}): {e}"),
    }
}

// ── Java log reuse ────────────────────────────────────────────────────────────
//
// Roughly 98% of a matrix gate's wall-clock is the JVM: the Rust engine plays a full game in
// milliseconds, Java takes seconds. But an iteration that only edits the Rust engine cannot
// change a single byte of what Java produced, so re-running it re-derives an identical file.
// `--reuse-java` skips the JVM when the logs on disk provably came from the same inputs.
//
// "Provably" is the whole point — a stale Java log silently turns a red into a green, which is
// worse than a slow gate. The fingerprint below covers every input that can change Java's
// output: the jar itself, the Java-side team/roster XMLs (regenerated by
// gen_java_parity_data.py after any data change), the team ids, the tier, the edition, and the
// trace flags (which alter what ParityRunner records). Anything not matching → run the JVM.

/// One recorded Java batch, written next to the logs it produced.
fn java_fingerprint(
    home_team_id: &str, away_team_id: &str, tier: u8, edition: &str, agent_spec: AgentSpec,
) -> serde_json::Value {
    // Which agent drove the Java side. Without this a heuristic-arm log could be served to a
    // random-arm gate (or the reverse) as if it were current -- the exact shape of silent-stale
    // failure that turned a 100/100 gate into 30/100 on 2026-08-27.
    let (agent, agent_scale, agent_classes) = match agent_spec {
        AgentSpec::Random { multimove } => ("random", format!("mm{multimove}"), String::new()),
        AgentSpec::Heuristic { temp_scale, mode, classes } => {
            ("heuristic", format!("{temp_scale}:{mode:?}"), classes.to_spec())
        }
    };
    let cp = resolve_parity_cp();
    let server_dir = resolve_server_dir();
    serde_json::json!({
        "version": 1,
        "jar": file_stamp(std::path::Path::new(&cp)),
        "server_data": dir_stamp(std::path::Path::new(&server_dir)),
        "home_team_id": home_team_id,
        "away_team_id": away_team_id,
        "tier": tier,
        "edition": edition,
        // ParityRunner records extra lines under these, so a log captured with tracing on is
        // not interchangeable with one captured without it.
        "dice_trace": std::env::var_os("FFB_DICE_TRACE").is_some(),
        "trace": std::env::var_os("FFB_TRACE").is_some(),
        "dice_deep": std::env::var_os("FFB_DICE_DEEP").is_some(),
        "agent": agent,
        "agent_scale": agent_scale,
        "agent_classes": agent_classes,
    })
}

/// `len:mtime_nanos` for one file, or `"missing"`. Cheap and change-sensitive; a rebuilt jar
/// always moves at least one of the two.
fn file_stamp(path: &std::path::Path) -> String {
    match std::fs::metadata(path) {
        Ok(m) => {
            let mtime = m.modified().ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_nanos()).unwrap_or(0);
            format!("{}:{}", m.len(), mtime)
        }
        Err(_) => "missing".to_string(),
    }
}

/// Stamp of the Java server's parity DATA — the `teams/` and `rosters/` XMLs that
/// `scripts/gen_java_parity_data.py` writes. A roster edit that is not mirrored into these
/// files is exactly the kind of change that must invalidate a cached Java log.
fn dir_stamp(server_dir: &std::path::Path) -> String {
    let mut parts: Vec<String> = Vec::new();
    for sub in ["teams", "rosters"] {
        let dir = server_dir.join(sub);
        let mut entries: Vec<_> = match std::fs::read_dir(&dir) {
            Ok(rd) => rd.filter_map(|e| e.ok()).map(|e| e.path()).collect(),
            Err(_) => continue,
        };
        entries.sort();
        for p in entries {
            let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            parts.push(format!("{sub}/{name}={}", file_stamp(&p)));
        }
    }
    format!("{:016x}", ffb_model::util::state_hash::fnv1a64(parts.join("|").as_bytes()))
}

fn manifest_path(edition: &str, home_race: &str, away_race: &str) -> String {
    format!("{}/java_manifest.json", crate::log_format::matchup_dir(edition, home_race, away_race))
}

/// True when every Java log for `[seed_start, seed_end]` is already on disk AND was produced
/// by the same inputs. Refuses (returns false) on any doubt — a missing manifest, a changed
/// jar, a missing or empty seed log, or a batch that covered a narrower seed range.
#[allow(clippy::too_many_arguments)]
pub fn java_logs_reusable(
    seed_start: u64, seed_end: u64,
    home_team_id: &str, away_team_id: &str, home_race: &str, away_race: &str, tier: u8,
    edition: &str, agent_spec: AgentSpec,
) -> Result<(), String> {
    let path = manifest_path(edition, home_race, away_race);
    let raw = std::fs::read_to_string(&path)
        .map_err(|_| format!("no cached Java batch at {path}"))?;
    let recorded: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| format!("unreadable manifest {path}: {e}"))?;
    let want = java_fingerprint(home_team_id, away_team_id, tier, edition, agent_spec);
    for key in ["version", "jar", "server_data", "home_team_id", "away_team_id", "tier",
                "edition", "dice_trace", "trace", "dice_deep",
                "agent", "agent_scale", "agent_classes"] {
        if recorded.get(key) != want.get(key) {
            return Err(format!("cached Java batch differs on `{key}`"));
        }
    }
    for seed in seed_start..=seed_end {
        let log = crate::log_format::java_log_path_for(seed, edition, home_race, away_race);
        match std::fs::metadata(&log) {
            Ok(m) if m.len() > 0 => {}
            _ => return Err(format!("cached Java batch is missing {log}")),
        }
    }
    Ok(())
}

/// Record the fingerprint of the batch just produced, so a later `--reuse-java` can trust it.
pub fn write_java_manifest(
    home_team_id: &str, away_team_id: &str, home_race: &str, away_race: &str, tier: u8,
    edition: &str, agent_spec: AgentSpec,
) {
    let path = manifest_path(edition, home_race, away_race);
    let fp = java_fingerprint(home_team_id, away_team_id, tier, edition, agent_spec);
    if let Ok(s) = serde_json::to_string_pretty(&fp) {
        let _ = std::fs::write(&path, s);
    }
}

#[cfg(test)]
mod reuse_tests {
    use super::*;

    /// A reuse decision must FAIL CLOSED. A cached Java log that is silently trusted when it
    /// should not be turns a red into a green, which is strictly worse than a slow gate, so
    /// anything short of a matching manifest plus every seed log present has to decline.
    #[test]
    fn reuse_declines_without_a_cached_batch() {
        let err = java_logs_reusable(
            1, 5, "teamA", "teamB", "no_such_roster", "no_such_roster", 3,
            "no_such_edition", AgentSpec::Random { multimove: 0 },
        ).unwrap_err();
        assert!(err.contains("no cached Java batch"), "unexpected reason: {err}");
    }

    /// The fingerprint has to move when any input Java reads moves. Team ids are the cheapest
    /// input to check without touching the filesystem.
    #[test]
    fn fingerprint_distinguishes_inputs() {
        let base = java_fingerprint("teamA", "teamB", 3, "bb2025", AgentSpec::Random { multimove: 0 });
        assert_ne!(base, java_fingerprint("teamZ", "teamB", 3, "bb2025", AgentSpec::Random { multimove: 0 }));
        assert_ne!(base, java_fingerprint("teamA", "teamB", 2, "bb2025", AgentSpec::Random { multimove: 0 }));
        assert_ne!(base, java_fingerprint("teamA", "teamB", 3, "bb2020", AgentSpec::Random { multimove: 0 }));
        // The agent arm changes what Java produces, so serving a random-arm log to a heuristic
        // gate (or the reverse) must be refused. Without this, `--reuse-java` would silently
        // compare two different experiments -- the failure mode that turned a 100/100 gate into
        // 30/100 on 2026-08-27.
        let heur = AgentSpec::Heuristic {
            temp_scale: 0.0,
            mode: ffb_engine::agent::Mode::Wide,
            classes: ffb_engine::agent::ClassMask::NONE,
        };
        assert_ne!(base, java_fingerprint("teamA", "teamB", 3, "bb2025", heur));
        let heur_more = AgentSpec::Heuristic {
            temp_scale: 0.0,
            mode: ffb_engine::agent::Mode::Wide,
            classes: ffb_engine::agent::ClassMask::NONE.with(ffb_engine::agent::PromptClass::CoinChoice),
        };
        assert_ne!(
            java_fingerprint("teamA", "teamB", 3, "bb2025", heur),
            java_fingerprint("teamA", "teamB", 3, "bb2025", heur_more),
            "a different class mask must invalidate the cache"
        );
    }
}

/// Run the Rust headless engine and write a JSONL parity log. Returns the log lines plus
/// all GameEvents emitted during the run (for coverage analysis).
///
/// Uses `RandomAgent::new_parity(seed)` — Xoshiro256StarStar seeded with
/// `seed ^ 0xDEAD_BEEF_CAFE_0001`, matching Java's decisionRng exactly.
/// `tier` selects the agent behavior and step-logging granularity:
///   2 — T2 agent (`act_parity_v1`: 1 decisionRng pick then EndTurn), one log step per
///       turn boundary (Phase-1 INIT_SELECTING with no acting player). Matches the
///       historical T2 Java logs.
///   3 — T3 Phase 2 agent (`act`: real activations), one log step per ActivatePlayer,
///       matching Java's per-activation recordStep().
/// Which agent drives the Rust side of a parity run.
///
/// `Random` is the historical parity driver whose RNG consumption is byte-matched against
/// `ParityRunner.java` (`AGENT_CONTRACT.md`). `Heuristic` is the Java-port ladder
/// (`AGENT_CONTRACT_HEURISTIC.md`): it scores only the prompt classes in `classes` and answers
/// everything else through an embedded `RandomAgent::new_parity`, so an empty mask is
/// indistinguishable from `Random` and the gate is green before any Java exists.
#[derive(Clone, Copy, Debug)]
pub enum AgentSpec {
    /// `multimove`: 0/1 = the historical one-square-per-activation behaviour. Greater than 1 is
    /// the spike from `docs/PARITY_HEURISTIC_CAMPAIGN.md` -- submit a planned path of up to N
    /// squares in ONE Move, mirrored by `ParityRunner --multimove N`, to test whether the two
    /// engines consume a multi-square move stack identically before the heuristic agent's scorer
    /// is ported to Java.
    Random { multimove: usize },
    Heuristic { temp_scale: f32, mode: ffb_engine::agent::Mode, classes: ffb_engine::agent::ClassMask },
}

/// The parity run's decision-maker. An enum rather than `Box<dyn Agent>` because tier 2 needs
/// `RandomAgent::pick_t2_activation`, which is not part of the `Agent` trait and has no meaning
/// for the heuristic agent.
enum Driver {
    Random(RandomAgent),
    Heuristic(Box<ffb_engine::agent::HeuristicAgent>),
}

impl Driver {
    fn new(spec: AgentSpec, seed: u64) -> Driver {
        match spec {
            AgentSpec::Random { multimove } => {
                let mut a = RandomAgent::new_parity(seed);
                a.multimove = multimove;
                Driver::Random(a)
            }
            AgentSpec::Heuristic { temp_scale, mode, classes } => Driver::Heuristic(Box::new(
                ffb_engine::agent::HeuristicAgent::with_classes(seed, temp_scale, mode, classes),
            )),
        }
    }

    fn act(&mut self, gs: &GameState) -> ffb_engine::action::Action {
        use ffb_engine::agent::Agent as _;
        match self {
            Driver::Random(a) => a.act(gs),
            Driver::Heuristic(a) => a.act(gs),
        }
    }

    /// Tier 2 only: consume the single decisionRng draw Java's tier-2 path spends on the player
    /// pick, then end the turn. The heuristic agent has no tier-2 contract, so pairing it with
    /// `--tier 2` is a caller error rather than something to paper over.
    fn pick_t2_activation(&mut self, n: usize) {
        match self {
            Driver::Random(a) => {
                a.pick_t2_activation(n);
            }
            Driver::Heuristic(_) => {
                panic!("--agent heuristic requires --tier 3 or higher; tier 2 has no heuristic contract")
            }
        }
    }
}

pub fn run_rust_headless(seed: u64, home_roster: &str, away_roster: &str, edition: &str, verbose: bool, tier: u8, agent_spec: AgentSpec) -> (Vec<LogLine>, Vec<GameEvent>, i32, i32) {
    let rust_path = rust_log_path_for(seed, edition, home_roster, away_roster);
    let dir = std::path::Path::new(&rust_path).parent().unwrap_or(std::path::Path::new("parity"));
    std::fs::create_dir_all(dir).ok();

    let rules = edition_to_rules(edition);
    let home = make_team(home_roster, "home", edition);
    let away = make_team(away_roster, "away", edition);

    let mut engine = GameState::new_with_options(home, away, rules, seed, BASELINE_SETUP_OPTIONS);
    // ONE shared agent for both teams, mirroring ParityRunner's single-object shape.
    // `new_parity` seeds both decision and action RNGs to match Java's decisionRng and actionRng
    // exactly; the heuristic agent seeds its own single stream and embeds a parity RandomAgent for
    // every class outside its mask. The two-agent home/away split used by `run_heuristic_game`
    // exists only for head-to-head A/B and must NOT be used here -- see
    // AGENT_CONTRACT_HEURISTIC.md section 7.
    let mut agent = Driver::new(agent_spec, seed);

    // GameStart hash = the fresh game BEFORE any roll (Java logs it pre-pregame). The engine
    // snapshots it during construction, since `new` runs the pregame to the first prompt.
    let initial_hash = engine.initial_state_hash().to_string();
    let mut lines: Vec<LogLine> = Vec::new();
    let mut all_events: Vec<GameEvent> = Vec::new();
    lines.push(LogLine::GameStart {
        i: 0,
        home: home_roster.to_string(),
        away: away_roster.to_string(),
        seed,
        state_hash: initial_hash,
    });

    let max_iters = 100_000usize;
    let mut step_index = 1u64;
    let mut pending_steps: Vec<PendingStep> = Vec::new();
    // No-progress guard: a healthy loop always advances something (state hash, dice
    // count, or the prompt kind). If none change for many consecutive iterations the
    // engine and agent are stuck re-prompting each other (e.g. a response that
    // silently no-ops against a missing pending) — abort with a diagnostic instead
    // of burning to max_iters.
    let mut stall: (String, u64, Option<std::mem::Discriminant<AgentPrompt>>) =
        (String::new(), 0, None);
    let mut stall_count = 0u32;

    for _ in 0..max_iters {
        if engine.is_finished() { break; }
        if engine.current_prompt().is_none() { break; }

        // Capture state BEFORE the agent acts.
        // Tier 2 logs one step per genuine Phase-1 turn boundary: first ActivatePlayer of a
        // new turn (eligible≠[] AND no player currently active). This excludes Blitz block
        // re-offers (eligible≠[] but acting_player still active) and Phase 2 (eligible=[]).
        let is_turn_boundary = match engine.current_prompt() {
            Some(AgentPrompt::ActivatePlayer { eligible_players })
                if !eligible_players.is_empty() && !engine.game.acting_player.is_active() => true,
            _ => false,
        };
        let turn_nr = if engine.game.home_playing {
            engine.game.turn_data_home.turn_nr
        } else {
            engine.game.turn_data_away.turn_nr
        };
        let half = engine.game.half;
        let pre_hash = state_hash(&engine.game);
        let active_str = if engine.game.home_playing { "home" } else { "away" };
        let pre_rng = engine.rng_call_count();
        let pre_state_str = if verbose || std::env::var_os("FFB_TRACE").is_some() {
            Some(state_string(&engine.game))
        } else {
            None
        };
        // FFB_IDSTATE: full-board coordinate + state dump for EVERY player (both teams, all nr),
        // keyed by parity step index — the Rust mirror of ParityRunner's JIDSTATE. Diff the two by
        // `i=` to see any player (including hash-blind nr>11) whose coord/state diverges.
        let pre_idstate = if std::env::var_os("FFB_IDSTATE").is_some() {
            let g = &engine.game;
            let mut s = String::new();
            for team in [&g.team_home, &g.team_away] {
                for p in &team.players {
                    let c = g.field_model.player_coordinate(&p.id);
                    let st = g.field_model.player_state(&p.id);
                    s.push_str(&format!(
                        "{}={}/{} ",
                        p.id,
                        c.map(|c| format!("{},{}", c.x, c.y)).unwrap_or_else(|| "?".into()),
                        st.map(|st| format!("{:x}", st.base())).unwrap_or_else(|| "?".into())
                    ));
                }
            }
            Some(s)
        } else {
            None
        };
        let probe = (
            pre_hash.clone(),
            engine.rng_call_count(),
            engine.current_prompt().map(std::mem::discriminant),
        );
        if probe == stall {
            stall_count += 1;
            if stall_count >= 50 {
                eprintln!(
                    "NO_PROGRESS seed={seed} half={half} turn={turn_nr} active={active_str}: \
                     50 iterations with unchanged hash={} rng_calls={} prompt={:?} — aborting game",
                    probe.0, probe.1, engine.current_prompt()
                );
                break;
            }
        } else {
            stall = probe;
            stall_count = 0;
        }

        let side = engine.active_side();
        let action = if tier >= 3 {
            agent.act(&engine)
        } else {
            // T2: consume exactly 1 decisionRng for the player pick (matching Java T2's
            // pick-one-then-deselect-then-EndTurn pattern), then return EndTurn.
            // Pre-activation prompts (coin, receive, kick) still go through the agent.
            match engine.current_prompt() {
                Some(AgentPrompt::ActivatePlayer { eligible_players }) => {
                    agent.pick_t2_activation(eligible_players.len());
                    ffb_engine::action::Action::EndTurn
                }
                _ => agent.act(&engine),
            }
        };
        let chosen = action_label(&action);
        let is_activation = matches!(action, ffb_engine::action::Action::ActivatePlayer { .. });

        match engine.apply(side, action) {
            Ok(evs) => all_events.extend(evs),
            Err(e) => {
                log::warn!("engine error at seed {seed}: {e}");
                break;
            }
        }

        if ffb_engine::parity_trace_enabled() {
            eprintln!("LOOP applied={chosen} prompt_after={:?} finished={}",
                engine.current_prompt(), engine.is_finished());
        }

        // Tier 2: one step line per INIT_SELECTING turn boundary (historical T2 format).
        // Tier 3: one step line per player activation (Phase 1 and Phase 2), matching
        // Java's per-activation recordStep().
        let log_step = if tier >= 3 { is_activation } else { is_turn_boundary };
        if log_step && turn_nr >= 1 && std::env::var_os("FFB_TRACE").is_some() {
            eprintln!("RUST_STEP i={} rng_calls={} turn={} half={} active={} chosen={} state={}", step_index, pre_rng, turn_nr, half, active_str, chosen, pre_state_str.as_deref().unwrap_or("?"));
        }
        if log_step && turn_nr >= 1 {
            if let Some(ref ids) = pre_idstate {
                eprintln!("RIDSTATE i={} {}", step_index, ids);
            }
            pending_steps.push(PendingStep {
                i: step_index,
                turn: turn_nr,
                half,
                active: active_str.to_string(),
                hash: pre_hash,
                chosen,
                state: pre_state_str,
            });
            step_index += 1;
        }
    }

    // Fill post_hashes retroactively
    let end_hash = state_hash(&engine.game);
    if std::env::var_os("FFB_TRACE").is_some() {
        eprintln!("RUST_END state={}", ffb_model::util::state_hash::state_string(&engine.game));
    }
    for i in 0..pending_steps.len() {
        let post_hash = if i + 1 < pending_steps.len() {
            pending_steps[i + 1].hash.clone()
        } else {
            end_hash.clone()
        };
        let s = &pending_steps[i];
        lines.push(LogLine::Step {
            i: s.i,
            turn: s.turn,
            half: s.half,
            active: s.active.clone(),
            dialog: "None".to_string(),
            state_hash: s.hash.clone(),
            actions: vec!["EndTurn".to_string()],
            chosen: s.chosen.clone(),
            dice: vec![],
            post_hash,
            state: s.state.clone(),
        });
    }

    let score_home = engine.game.game_result.home.score;
    let score_away = engine.game.game_result.away.score;
    // Counterpart to ParityRunner's `JAVA_END state=`. A divergence in the resolution of the LAST
    // logged step has no following step to diff, so without this the end-of-game state can only be
    // compared as an opaque hash.
    if std::env::var_os("FFB_TRACE").is_some() {
        eprintln!("RUST_END state={}", state_string(&engine.game));
    }
    lines.push(LogLine::GameEnd {
        i: step_index,
        home_score: score_home,
        away_score: score_away,
        state_hash: end_hash,
    });

    // Write step log to disk
    let log = GameLog {
        seed,
        home_roster: home_roster.to_string(),
        away_roster: away_roster.to_string(),
        lines: lines.clone(),
    };
    if let Err(e) = log.write_to_file(&rust_path) {
        log::warn!("Could not write Rust log for seed {seed}: {e}");
    }

    // Write events log to disk (one JSON line per GameEvent).
    let events_path = rust_events_path_for(seed, edition, home_roster, away_roster);
    if let Ok(mut f) = std::fs::File::create(&events_path) {
        use std::io::Write;
        for ev in &all_events {
            if let Ok(line) = serde_json::to_string(ev) {
                let _ = writeln!(f, "{}", line);
            }
        }
    }

    (lines, all_events, score_home, score_away)
}

// ── Private helpers ───────────────────────────────────────────────────────────

struct PendingStep {
    i: u64,
    turn: i32,
    half: i32,
    active: String,
    hash: String,
    chosen: String,
    state: Option<String>,
}

/// Build a team. `roster_name` is "lineman" (generic) or a race name like "human".
/// `side` is "home" or "away" (used for player IDs). `edition` is "bb2016"/"bb2020"/"bb2025".
///
/// Preference order:
///   1. the hand-drafted team spec `data/teams/<edition>/team_<race>.json` (rule-legal
///      drafts, see docs/TEAM_DRAFTS_*.md) — mirrored to Java by gen_java_parity_data.py
///   2. the legacy first-11-by-(quantity,cost) builder (kept for bb2020 and ad-hoc rosters)
pub(crate) fn make_team(roster_name: &str, side: &str, edition: &str) -> Team {
    if roster_name == "lineman"
        || roster_name == "teamLinemanParityHome"
        || roster_name == "teamLinemanParityAway"
    {
        return make_lineman_team(side, roster_name);
    }
    match make_team_from_file(roster_name, side, edition) {
        Ok(team) => return team,
        Err(e) => log::debug!("No team file for '{roster_name}' ({edition}): {e}; using legacy builder"),
    }
    make_team_from_roster(roster_name, side, edition)
        .unwrap_or_else(|e| {
            log::warn!("Could not load roster '{roster_name}': {e}; falling back to lineman");
            make_lineman_team(side, roster_name)
        })
}

/// Deserialized `data/teams/<edition>/team_<race>.json` (hand-drafted team spec).
#[derive(serde::Deserialize)]
pub(crate) struct TeamFileJson {
    pub race: String,
    pub roster_id: String,
    pub rerolls: i32,
    pub apothecaries: i32,
    pub dedicated_fans: i32,
    pub fan_factor: i32,
    pub treasury: i32,
    pub team_value: i32,
    #[serde(default)]
    pub special_rules: Vec<String>,
    pub players: Vec<TeamFilePlayer>,
    /// Star players fielded as extra rostered players (parity tier §9). Each entry names a
    /// star from data/star_players/all_editions.json by id; gen_java_parity_data.py emits the
    /// identical star into the Java roster/team XMLs, so both engines field the same player.
    #[serde(default)]
    pub stars: Vec<TeamFileStar>,
}

#[derive(serde::Deserialize)]
pub(crate) struct TeamFilePlayer {
    pub nr: i32,
    pub position_id: String,
}

#[derive(serde::Deserialize)]
pub(crate) struct TeamFileStar {
    pub nr: i32,
    pub star_id: String,
}

/// Locate `data/teams/<edition>/team_<race>.json` (env `FFB_TEAMS_DIR` overrides the root).
fn team_file_path(roster_name: &str, edition: &str) -> Option<std::path::PathBuf> {
    let file = format!("{edition}/team_{roster_name}.json");
    if let Ok(root) = std::env::var("FFB_TEAMS_DIR") {
        let p = std::path::Path::new(&root).join(&file);
        return p.exists().then_some(p);
    }
    let candidates = [
        "data/teams",
        "../data/teams",
        "../../data/teams",
        r"C:\Users\Admin\niels\ffb-rust\ffb-rust\data\teams",
    ];
    candidates.iter()
        .map(|c| std::path::Path::new(c).join(&file))
        .find(|p| p.exists())
}

/// Build a team from its hand-drafted spec file. The Java engine loads the XML mirror of
/// the SAME spec (team<Race>Parity{25,16}{Home,Away} referencing roster `<race>.<edition>`),
/// so both engines see an identical team.
pub fn make_team_from_file(roster_name: &str, side: &str, edition: &str) -> Result<Team, String> {
    let path = team_file_path(roster_name, edition)
        .ok_or_else(|| format!("no team file data/teams/{edition}/team_{roster_name}.json"))?;
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let spec: TeamFileJson = serde_json::from_str(&text)
        .map_err(|e| format!("{}: {e}", path.display()))?;

    let rosters = match edition {
        "bb2016" => bb2016_rosters(),
        "bb2020" => bb2020_rosters(),
        "bb2025" | _ => bb2025_rosters(),
    };
    let roster_json = rosters.into_iter()
        .find(|r| r.id == spec.roster_id)
        .ok_or_else(|| format!("roster id '{}' not found in edition '{}'", spec.roster_id, edition))?;

    let mut players: Vec<Player> = Vec::new();
    for pl in &spec.players {
        let pos_json = roster_json.positions.iter()
            .find(|p| p.id == pl.position_id)
            .ok_or_else(|| format!("position '{}' not in roster '{}'", pl.position_id, spec.roster_id))?;
        let rp = position_json_to_roster_position(pos_json, &roster_json.id, roster_json.undead, edition_to_rules(edition));
        players.push(Player::from_position(
            format!("{side}_{:02}", pl.nr),
            format!("{} {} {}", side, rp.name, pl.nr),
            pl.nr,
            &rp,
        ));
    }

    // Star players drafted by the spec's `stars` list — fielded as ordinary rostered players
    // (no inducement phase; the Java sheets get the identical player from the SAME star data
    // via gen_java_parity_data.py, so the two engines stay in lockstep).
    for star_entry in &spec.stars {
        let star = ffb_model::data::STAR_PLAYERS.star_players.iter()
            .find(|s| s.id == star_entry.star_id)
            .ok_or_else(|| format!("star id '{}' not found in data/star_players/all_editions.json",
                star_entry.star_id))?;
        let pos_json = ffb_model::data::roster_json::PositionJson {
            id: star.id.clone(),
            name: star.name.clone(),
            display_name: star.display_name.clone(),
            player_type: star.player_type.clone(),
            quantity: star.quantity.unwrap_or(1),
            cost: star.cost,
            ma: star.ma,
            st: star.st,
            ag: star.ag,
            pa: star.pa,
            av: star.av,
            skills: star.skills.clone(),
            skill_categories: Default::default(),
            keywords: vec![],
        };
        let rp = position_json_to_roster_position(&pos_json, &roster_json.id, roster_json.undead, edition_to_rules(edition));
        players.push(Player::from_position(
            format!("{side}_{:02}", star_entry.nr),
            format!("{} {} {}", side, rp.name, star_entry.nr),
            star_entry.nr,
            &rp,
        ));
    }
    // Keep the roster in squad-nr order regardless of spec-file order: Java's player list is
    // nr-ordered, and the harness activation snapshots index into the list (idx % N), so an
    // out-of-order list silently pairs the same idx with different players (dwarf bb2016 star
    // pilot, seed 1 half 2: pick=8 N=10 gave Java nr 12 and Rust nr 13). gen_java_parity_data.py
    // applies the same sort to the emitted team XML.
    players.sort_by_key(|p| p.nr);

    Ok(Team {
        id: format!("{side}_{}", roster_json.id),
        name: format!("{} {}", side, roster_json.name),
        race: roster_json.name.clone(),
        roster_id: roster_json.id.clone(),
        coach: format!("Coach_{side}"),
        rerolls: spec.rerolls,
        apothecaries: spec.apothecaries,
        bribes: 0,
        master_chefs: 0,
        prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
        cheerleaders: 0,
        assistant_coaches: 0,
        fan_factor: spec.fan_factor,
        dedicated_fans: spec.dedicated_fans,
        team_value: spec.team_value,
        treasury: spec.treasury,
        special_rules: spec.special_rules.clone(),
        players,
        vampire_lord: roster_json.has_vampire_lord(),
        necromancer: roster_json.has_necromancer(),
    })
}

fn make_lineman_team(side: &str, roster_id: &str) -> Team {
    let players: Vec<Player> = (1..=11).map(|nr| Player {
        id: format!("{side}_{nr:02}"),
        name: format!("{side} Player {nr}"),
        nr,
        position_id: "lineman".to_string(),
        player_type: PlayerType::Regular,
        gender: PlayerGender::Male,
        movement: 6,
        strength: 3,
        agility: 3,
        passing: 4,
        armour: 8,
        starting_skills: vec![],
        extra_skills: vec![],
        temporary_skills: vec![],
        used_skills: HashSet::new(),
        // Mirrors the Java twin of this fixture, ffb-server/rosters/roster_lineman_parity.xml:
        //     <skillCategoryList>
        //       <normal>General</normal>
        //       <double>Agility</double> <double>Strength</double> <double>Passing</double>
        //     </skillCategoryList>
        // Only the NORMAL list is modelled, because that is the one Java reads
        // (`getSkillCategories(false)`) when the Intensive Training prayer builds its skill list.
        // Without it the offered list is empty and the prayer is silently wasted, while Java grants
        // a skill (lineman bb2020 seed 50: Java's Home2 gets Block and so does not fall on Both Down).
        skill_categories_normal: vec![SkillCategory::General],
        niggling_injuries: 0,
        stat_injuries: vec![],
        current_spps: 0,
        career_spps: 0,
        race: None,
        ..Default::default()
    }).collect();

    Team {
        id: format!("{side}_{roster_id}"),
        name: format!("{} ({})", side, roster_id),
        race: roster_id.to_string(),
        roster_id: roster_id.to_string(),
        coach: format!("Coach_{side}"),
        rerolls: 3,
        apothecaries: 1,
        bribes: 0,
        master_chefs: 0,
        prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
        cheerleaders: 0,
        assistant_coaches: 0,
        // Java's parity team sheets carry <fanFactor>5 for BOTH lineman teams
        // (`ffb-server/teams/team_lineman_parity_{home,away}.xml`); Rust's synthetic twin hard-coded
        // 0. Fan factor feeds the spectator count, which sets FAME, which decides the kickoff
        // extra-re-roll contest — so Rust computed a different winner (bb2016 lineman seed 65: Java
        // spectators 8000/11000 → fameA=1 → a 3-3 tie and BOTH teams gain; Rust 3000/6000 → fameA=2
        // → away only). Invisible until re-rolls entered the state hash.
        fan_factor: 5,
        dedicated_fans: 0,
        team_value: 1_000_000,
        treasury: 0,
        special_rules: vec![],
        players,
        vampire_lord: false,
        necromancer: false,
    }
}

/// Build a 11-player team from a named roster (e.g. "human", "orc") for the given edition.
/// Positions are filled in roster order up to 11 players, respecting each position's max quantity.
pub fn make_team_from_roster(roster_name: &str, side: &str, edition: &str) -> Result<Team, String> {
    let rosters = match edition {
        "bb2016" => bb2016_rosters(),
        "bb2020" => bb2020_rosters(),
        "bb2025" | _ => bb2025_rosters(),
    };

    // Explicit aliases for rosters whose JSON id/name don't match the race short-name.
    let roster_name = match roster_name.to_ascii_lowercase().as_str() {
        "renegades" => "chaos renegade",
        "chaos_chosen" => "chaos",
        // FUMBBL league-import rosters carry a numeric FUMBBL roster `id` (e.g. "4959") and a
        // generic race `name` ("Dark Elf") that collides with the standard roster's name, so
        // neither matches the CLI key by name. Alias each to its unique numeric roster id so the
        // find() below matches on `norm(id)`. Without this, make_team_from_roster returned Err and
        // make_team() silently fell back to an all-generic-lineman team (AG3, no skills), diverging
        // from Java's real roster at the first dodge (dark_elf_league_fumbbl seed 1 step 9).
        "dark_elf_league_fumbbl" => "4959",
        "khemri_fumbbl" => "55051",
        "slann_fumbbl" => "744258",
        _ => roster_name,
    };
    // Normalize: lowercase, strip non-alphanumeric (collapses "Chaos Dwarf" = "chaos_dwarf" = "chaosdwarf")
    let norm = |s: &str| s.chars().filter(|c| c.is_alphanumeric()).collect::<String>().to_lowercase();
    let roster_norm = norm(roster_name);
    let roster_json = rosters
        .into_iter()
        .find(|r| {
            norm(&r.name) == roster_norm
                || norm(&r.id) == roster_norm
                || r.id.to_ascii_lowercase().starts_with(&format!("{}.", roster_name.to_ascii_lowercase()))
                || norm(&r.id).starts_with(&roster_norm)
        })
        .ok_or_else(|| format!("roster '{}' not found in edition '{}'", roster_name, edition))?;

    // Sort positions by (quantity ASC, cost DESC) — premium/limited positions first,
    // cheap/abundant filler (linemen) last. Matches gen_java_teams.py's sort order so
    // both engines build the exact same 11-player composition.
    let mut non_star: Vec<&PositionJson> = roster_json.positions.iter()
        .filter(|p| p.player_type != "Star" && p.player_type != "Infamous Staff" && p.quantity > 0)
        .collect();
    non_star.sort_by_key(|p| (p.quantity, -(p.cost)));

    let mut players: Vec<Player> = Vec::new();
    let mut nr = 1i32;

    'outer: for pos_json in &non_star {
        let rp = position_json_to_roster_position(pos_json, &roster_json.id, roster_json.undead, edition_to_rules(edition));
        let max_this = pos_json.quantity.min(11 - players.len() as i32);
        for _ in 0..max_this {
            if players.len() >= 11 {
                break 'outer;
            }
            let player = Player::from_position(
                format!("{side}_{nr:02}"),
                format!("{} {} {nr}", side, rp.name),
                nr,
                &rp,
            );
            players.push(player);
            nr += 1;
        }
    }

    if players.is_empty() {
        return Err(format!("roster '{}' has no non-star positions", roster_name));
    }

    let team_value: i32 = players.iter()
        .zip(non_star.iter().flat_map(|p| std::iter::repeat(p.cost).take(p.quantity as usize)))
        .map(|(_, cost)| cost)
        .sum();

    Ok(Team {
        id: format!("{side}_{}", roster_json.id),
        name: format!("{} {}", side, roster_json.name),
        race: roster_json.name.clone(),
        roster_id: roster_json.id.clone(),
        coach: format!("Coach_{side}"),
        rerolls: 3,
        apothecaries: if roster_json.apothecary { 1 } else { 0 },
        bribes: 0,
        master_chefs: 0,
        prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
        cheerleaders: 0,
        assistant_coaches: 0,
        fan_factor: 0,
        dedicated_fans: 0,
        team_value,
        treasury: 0,
        special_rules: roster_json.special_rules.clone(),
        players,
        vampire_lord: roster_json.has_vampire_lord(),
        necromancer: roster_json.has_necromancer(),
    })
}


pub(crate) fn edition_to_rules(edition: &str) -> Rules {
    match edition {
        "bb2016" => Rules::Bb2016,
        "bb2020" => Rules::Bb2020,
        _ => Rules::Bb2025,
    }
}

/// Every non-star roster name available for the given edition (used by `--uniform` to
/// sweep the full roster catalog rather than a single hardcoded matchup).
pub fn roster_names_for_edition(edition: &str) -> Vec<String> {
    let rosters = match edition {
        "bb2016" => bb2016_rosters(),
        "bb2020" => bb2020_rosters(),
        _ => bb2025_rosters(),
    };
    rosters.into_iter().map(|r| r.name).collect()
}

/// Map a snake_case race name + side + edition to the Java server parity team ID.
/// Uses PascalCase conversion matching gen_java_parity_data.py exactly.
/// e.g. "dark_elf" + "home" + "bb2025" → "teamDarkElfParity25Home".
/// The synthetic "lineman" fixture keeps the legacy "Parity" id
/// (team_lineman_parity_<side>.xml, generated by the old gen_java_teams.py); every real
/// roster uses the per-edition ids emitted by gen_java_parity_data.py.
pub fn java_team_id(race_name: &str, side: &str, edition: &str) -> String {
    let side_suffix = if side == "away" { "Away" } else { "Home" };
    let parity = if race_name == "lineman" {
        "Parity"
    } else {
        match edition {
            "bb2025" => "Parity25",
            "bb2020" => "Parity20",
            "bb2016" => "Parity16",
            _ => "Parity",
        }
    };
    let suffix = format!("{parity}{side_suffix}");
    let pascal: String = race_name
        .split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect();
    format!("team{pascal}{suffix}")
}


pub(crate) fn action_label(action: &ffb_engine::action::Action) -> String {
    use ffb_engine::action::Action;
    match action {
        Action::CoinChoice { heads } => if *heads { "Heads".into() } else { "Tails".into() },
        Action::ReceiveChoice { receive } => if *receive { "Receive".into() } else { "Kick".into() },
        Action::KickBall { coord } => format!("Kick({},{})", coord.x, coord.y),
        Action::Touchback { player_id } => format!("Touchback({player_id})"),
        Action::ActivatePlayer { player_id, player_action, .. } => format!("Activate({player_id},{player_action:?})"),
        Action::EndPlayerAction => "EndPlayerAction".into(),
        Action::EndTurn => "EndTurn".into(),
        Action::Move { path } => path.last().map(|c| format!("Move→({},{})", c.x, c.y)).unwrap_or("Move".into()),
        Action::Block { defender_id } => format!("Block→{defender_id}"),
        Action::Stab { defender_id } => format!("Stab→{defender_id}"),
        Action::BlockChoice { die_index, .. } => format!("BlockChoice({die_index})"),
        Action::PushTo { coord } => format!("Push({},{})", coord.x, coord.y),
        Action::FollowUp { follow_up } => if *follow_up { "FollowUp".into() } else { "NoFollowUp".into() },
        Action::Pass { coord } => format!("Pass({},{})", coord.x, coord.y),
        Action::HandOff { receiver_id } => format!("HandOff→{receiver_id}"),
        Action::Foul { target_id } => format!("Foul→{target_id}"),
        Action::UseReRoll { use_reroll } => if *use_reroll { "UseReRoll".into() } else { "NoReRoll".into() },
        Action::UseSkill { skill_id, use_skill } => if *use_skill { format!("UseSkill({skill_id:?})") } else { format!("NoSkill({skill_id:?})") },
        Action::SelectPlayer { player_id } => format!("Select({player_id})"),
        Action::UseApothecary { use_apothecary, .. } => if *use_apothecary { "Heal".into() } else { "AcceptInjury".into() },
        _ => format!("{action:?}"),
    }
}

// ── Visual snapshot types ─────────────────────────────────────────────────────

#[derive(serde::Serialize)]
pub struct PlayerSnap {
    pub id: String,
    pub nr: i32,
    pub nm: String,
    pub h: bool,
    pub x: i32,
    pub y: i32,
    pub st: i32,
    pub ma: i32,
    pub ag: i32,
    pub pa: i32,
    pub av: i32,
    pub pos: String,
    pub skls: Vec<String>,
    pub bs: u32,
    pub act: bool,
    pub cur: bool,
}

/// How many options a dumped decision keeps. The agent considers every reachable square, which is
/// over two thousand on an open pitch; a viewer needs the shape of the distribution, not all of it.
const DUMP_OPTION_CAP: usize = 40;

/// One decision, with the board it was made on and the full distribution behind it.
///
/// Written only when `FFB_HEUR_DUMP=<path>` is set. The point of carrying every option and its
/// probability -- rather than just the action taken -- is that the interesting thing to look at is
/// what the agent *considered*: a pick tells you nothing about whether it was forced or a coin flip.
#[derive(serde::Serialize)]
pub struct DecisionRec {
    pub i: usize,
    pub side: String,
    pub prompt: String,
    pub chosen: usize,
    /// The most likely options only - see DUMP_OPTION_CAP. A full activation now offers over two
    /// thousand, and serialising every one of them for every prompt would make the file unusable.
    pub options: Vec<ffb_engine::agent::ScoredOption>,
    /// How many options there really were, and how much probability the kept ones account for.
    pub n_options: usize,
    pub p_shown: f32,
    /// What was actually sent to the engine. Recorded even when `options` is empty: a prompt
    /// answered from the activation plan still DID something, and a reader needs to see what.
    pub taken: String,
    pub snap: GameSnap,
}

#[derive(serde::Serialize)]
pub struct GameSnap {
    pub i: usize,
    pub l: String,
    pub hl: i32,
    pub t: i32,
    pub hp: bool,
    pub hs: i32,
    pub aw: i32,
    pub hr: i32,
    pub ar: i32,
    pub hb: i32,
    pub ab: i32,
    pub bx: Option<i32>,
    pub by: Option<i32>,
    pub bp: bool,
    /// Player who currently holds the ball (on-pitch ball carrier), if any
    pub ball_carrier: Option<String>,
    /// Currently acting player ID + move spent/max for MA display
    pub act_id: Option<String>,
    pub act_ma_spent: i32,
    pub act_ma_max: i32,
    pub ps: Vec<PlayerSnap>,
    pub evs: Vec<ffb_model::events::GameEvent>,
}

fn snap(engine: &GameState, step: usize, label: String,
        evs: Vec<ffb_model::events::GameEvent>) -> GameSnap {
    use ffb_model::enums::PlayerState;
    let g = &engine.game;
    let acting_id: Option<&str> = g.acting_player.player_id.as_deref();
    let turn_nr = if g.home_playing { g.turn_data_home.turn_nr } else { g.turn_data_away.turn_nr };

    // Find ball carrier: player on the ball's square
    let ball_carrier = g.field_model.ball_coordinate
        .filter(|c| c.is_on_pitch() && g.field_model.ball_in_play)
        .and_then(|c| g.field_model.player_at(c))
        .map(|id| id.to_string());

    // Acting player MA info
    let (act_id, act_ma_spent, act_ma_max) = if let Some(pid) = acting_id {
        let ma = g.team_home.player(pid).or_else(|| g.team_away.player(pid))
            .map(|p| p.movement).unwrap_or(0);
        (Some(pid.to_string()), g.acting_player.current_move, ma)
    } else {
        (None, 0, 0)
    };

    let mut ps = Vec::new();
    for is_home in [true, false] {
        let team = if is_home { &g.team_home } else { &g.team_away };
        for p in &team.players {
            let coord = g.field_model.player_coordinates.get(&p.id).copied();
            let state = g.field_model.player_states.get(&p.id).copied().unwrap_or(PlayerState(0));
            let dx = if is_home { -1 } else { 30 };
            let skls: Vec<String> = p.all_skill_ids().map(|s| format!("{s:?}")).collect();
            ps.push(PlayerSnap {
                id: p.id.clone(), nr: p.nr, nm: p.name.clone(), h: is_home,
                x: coord.map(|c| c.x).unwrap_or(dx),
                y: coord.map(|c| c.y).unwrap_or(7),
                st: p.strength, ma: p.movement, ag: p.agility, pa: p.passing, av: p.armour,
                pos: p.position_id.clone(), skls,
                bs: state.base(), act: state.is_active(),
                cur: acting_id == Some(p.id.as_str()),
            });
        }
    }
    GameSnap {
        i: step, l: label, hl: g.half, t: turn_nr, hp: g.home_playing,
        hs: g.game_result.home.score, aw: g.game_result.away.score,
        hr: g.turn_data_home.rerolls, ar: g.turn_data_away.rerolls,
        hb: g.team_home.bribes, ab: g.team_away.bribes,
        bx: g.field_model.ball_coordinate.map(|c| c.x),
        by: g.field_model.ball_coordinate.map(|c| c.y),
        bp: g.field_model.ball_in_play,
        ball_carrier, act_id, act_ma_spent, act_ma_max,
        ps, evs,
    }
}

/// Run a complete game using RandomAgent for both sides, collecting snapshots after
/// every engine action for HTML replay generation.
pub fn run_visual_game(
    seed: u64,
    home_roster: &str,
    away_roster: &str,
    edition: &str,
) -> Vec<GameSnap> {
    use ffb_engine::agent::Agent;

    let rules = edition_to_rules(edition);
    let home = make_team(home_roster, "home", edition);
    let away = make_team(away_roster, "away", edition);
    let mut engine = GameState::new_with_options(home, away, rules, seed, BASELINE_SETUP_OPTIONS);

    // Separate agents for home and away so they don't share RNG state.
    let mut home_agent = ffb_engine::agent::RandomAgent::new(seed);
    let mut away_agent = ffb_engine::agent::RandomAgent::new(seed ^ 0xFFFF_FFFF);

    let mut snaps: Vec<GameSnap> = Vec::new();
    let mut idx = 0usize;

    snaps.push(snap(&engine, idx, "Game Start".into(), vec![]));
    idx += 1;

    for _ in 0..200_000 {
        if engine.is_finished() { break; }
        if engine.current_prompt().is_none() { break; }
        let side = engine.active_side();
        let action = if matches!(side, TeamSide::Home) {
            home_agent.act(&engine)
        } else {
            away_agent.act(&engine)
        };
        let label = action_label(&action);
        let evs = engine.apply(side, action).unwrap_or_default();
        snaps.push(snap(&engine, idx, label, evs));
        idx += 1;
    }

    snaps
}

/// Run a complete game using RandomAgent, collecting all GameEvents for coverage reporting.
pub fn run_coverage_game(
    seed: u64,
    home_roster: &str,
    away_roster: &str,
    edition: &str,
) -> (Vec<GameEvent>, i32, i32) {
    use ffb_engine::agent::Agent;

    let rules = edition_to_rules(edition);
    let home = make_team(home_roster, "home", edition);
    let away = make_team(away_roster, "away", edition);
    let mut engine = GameState::new_with_options(home, away, rules, seed, BASELINE_SETUP_OPTIONS);

    let mut home_agent = ffb_engine::agent::RandomAgent::new(seed);
    let mut away_agent = ffb_engine::agent::RandomAgent::new(seed ^ 0xFFFF_FFFF);

    let mut all_events: Vec<GameEvent> = Vec::new();

    for _ in 0..200_000 {
        if engine.is_finished() { break; }
        if engine.current_prompt().is_none() { break; }
        let side = engine.active_side();
        let action = if matches!(side, TeamSide::Home) {
            home_agent.act(&engine)
        } else {
            away_agent.act(&engine)
        };
        match engine.apply(side, action) {
            Ok(evs) => all_events.extend(evs),
            Err(e) => { eprintln!("engine error seed {seed}: {e}"); break; }
        }
    }

    let score_home = engine.game.game_result.home.score;
    let score_away = engine.game.game_result.away.score;
    (all_events, score_home, score_away)
}

/// Run a complete game using `UniformAgent` (uniform sampling over every legal action,
/// including inducement purchases — unlike `RandomAgent`, which is fixed-response on
/// several pre-game prompts to preserve Java-parity RNG-stream sync). Collects all
/// `GameEvent`s for coverage reporting plus the names of any prompts `UniformAgent`
/// couldn't turn into a real choice (see `UniformAgent::last_unhandled_prompt`).
pub fn run_uniform_game(
    seed: u64,
    home_roster: &str,
    away_roster: &str,
    edition: &str,
) -> (Vec<GameEvent>, i32, i32, Vec<String>) {
    use ffb_engine::agent::{Agent, UniformAgent};

    let rules = edition_to_rules(edition);
    let home = make_team(home_roster, "home", edition);
    let away = make_team(away_roster, "away", edition);
    // `GameState::new`/`new_with_options` drive the flattened pregame
    // (`sequences::start_game_sequence()`) that the Java-parity RNG contract depends on —
    // it skips PettyCash/BuyInducements entirely. `new_full_pregame` uses the real,
    // edition-aware generator sequence so PettyCash/BuyInducements actually run; combined
    // with enabling INDUCEMENTS (every option starts disabled — no ruleset-loader hook at
    // this synchronous layer), `StepBuyInducements` now actually fires instead of
    // auto-skipping to DONE. This is the whole point of a "uniform sampling including
    // inducements" run.
    let mut options: Vec<(&str, &str)> = vec![(INDUCEMENTS, "true")];
    options.extend_from_slice(BASELINE_SETUP_OPTIONS);
    let mut engine = GameState::new_full_pregame(home, away, rules, seed, &options);

    let mut home_agent = UniformAgent::new(seed);
    let mut away_agent = UniformAgent::new(seed ^ 0xFFFF_FFFF);

    let mut all_events: Vec<GameEvent> = Vec::new();
    let mut unhandled_prompts: Vec<String> = Vec::new();

    for _ in 0..200_000 {
        if engine.is_finished() { break; }
        if engine.current_prompt().is_none() { break; }
        let side = engine.active_side();
        let action = if matches!(side, TeamSide::Home) {
            let a = home_agent.act(&engine);
            if let Some(p) = home_agent.last_unhandled_prompt.take() { unhandled_prompts.push(p); }
            a
        } else {
            let a = away_agent.act(&engine);
            if let Some(p) = away_agent.last_unhandled_prompt.take() { unhandled_prompts.push(p); }
            a
        };
        match engine.apply(side, action) {
            Ok(evs) => all_events.extend(evs),
            Err(e) => { eprintln!("engine error seed {seed}: {e}"); break; }
        }
    }

    let score_home = engine.game.game_result.home.score;
    let score_away = engine.game.game_result.away.score;
    (all_events, score_home, score_away, unhandled_prompts)
}

/// One-line description of the agent's answer, for the FFB_SEQ decision trace.
fn describe_action(a: &ffb_engine::action::Action) -> String {
    use ffb_engine::action::Action as A;
    match a {
        A::ActivatePlayer { player_id, player_action, block_defender_id } => format!(
            "Activate({player_id}, {player_action:?}{})",
            block_defender_id.as_ref().map(|t| format!(", target={t}")).unwrap_or_default()),
        A::Move { path } => format!(
            "Move({} squares -> {})", path.len(),
            path.last().map(|c| format!("{},{}", c.x, c.y)).unwrap_or_default()),
        A::BlockChoice { die_index, .. } => format!("BlockChoice(die {die_index})"),
        A::PushTo { coord } => format!("PushTo({},{})", coord.x, coord.y),
        A::FollowUp { follow_up } => format!("FollowUp({follow_up})"),
        A::UseReRoll { use_reroll } => format!("UseReRoll({use_reroll})"),
        A::SelectPlayer { player_id } => format!("SelectPlayer({player_id})"),
        A::EndPlayerAction => "EndPlayerAction".into(),
        A::EndTurn => "EndTurn".into(),
        A::PlacePlayer { player_id, coord } => {
            format!("PlacePlayer({player_id} -> {},{})", coord.x, coord.y)
        }
        A::ConfirmSetup => "ConfirmSetup".into(),
        A::KickBall { coord } => format!("KickBall({},{})", coord.x, coord.y),
        A::Touchback { player_id } => format!("Touchback({player_id})"),
        A::Block { defender_id } => format!("Block({defender_id})"),
        A::Stab { defender_id } => format!("Stab({defender_id})"),
        A::Foul { target_id } => format!("Foul({target_id})"),
        A::HandOff { receiver_id } => format!("HandOff({receiver_id})"),
        A::Pass { coord } => format!("Pass(-> {},{})", coord.x, coord.y),
        A::Intercept { attempt } => format!("Intercept({attempt})"),
        A::UseSkill { skill_id, use_skill } => format!("UseSkill({skill_id:?}, {use_skill})"),
        A::CoinChoice { heads } => {
            format!("CoinChoice({})", if *heads { "heads" } else { "tails" })
        }
        A::ReceiveChoice { receive } => {
            format!("ReceiveChoice({})", if *receive { "receive" } else { "kick" })
        }
        A::RaidingPartyTarget { coord } => {
            format!("RaidingPartyTarget({},{})", coord.x, coord.y)
        }
        A::HitAndRun { coord } => match coord {
            Some(c) => format!("HitAndRun({},{})", c.x, c.y),
            None => "HitAndRun(decline)".into(),
        },
        A::Acknowledge => "Acknowledge".into(),
        // Generous, and only a backstop: the arms above cover everything these games produce.
        // The old 48-char cut was a SEQ-trace nicety that silently truncated the dump mid-word.
        other => format!("{other:?}").chars().take(200).collect(),
    }
}

/// The block die faces by name. `Action::BlockChoice` carries only an index, so the face has to
/// come from the prompt that offered it.
fn die_face(d: i32) -> &'static str {
    match d {
        1 => "Skull",
        2 => "Both Down",
        3 | 4 => "Push",
        5 => "Defender Stumbles",
        6 => "Defender Down",
        _ => "?",
    }
}

/// Prompt-variant name, for the per-class timing breakdown.
fn prompt_class(p: &AgentPrompt) -> &'static str {
    match p {
        AgentPrompt::ActivatePlayer { .. } => "ActivatePlayer",
        AgentPrompt::Move { .. } => "Move",
        AgentPrompt::BlockChoice { .. } => "BlockChoice",
        AgentPrompt::BlockChoiceProperties { .. } => "BlockChoiceProperties",
        AgentPrompt::BlitzTarget { .. } => "BlitzTarget",
        AgentPrompt::BlockTarget { .. } => "BlockTarget",
        AgentPrompt::Pushback { .. } => "Pushback",
        AgentPrompt::FollowUp { .. } => "FollowUp",
        AgentPrompt::ReRollOffer { .. } => "ReRollOffer",
        AgentPrompt::SkillUse { .. } => "SkillUse",
        AgentPrompt::Interception { .. } => "Interception",
        AgentPrompt::KickBall => "KickBall",
        AgentPrompt::Touchback { .. } => "Touchback",
        AgentPrompt::CoinChoice { .. } => "CoinChoice",
        AgentPrompt::ReceiveChoice { .. } => "ReceiveChoice",
        AgentPrompt::TeamSetup { .. } => "TeamSetup",
        AgentPrompt::ApothecaryChoice { .. } => "ApothecaryChoice",
        AgentPrompt::PlayerChoice { .. } => "PlayerChoice",
        AgentPrompt::KickoffEventPlacement { .. } => "KickoffEventPlacement",
        _ => "other",
    }
}

/// Run a complete game with `HeuristicAgent` on BOTH sides at the given temperature scale.
///
/// `temp_scale = 1.0` is the §8 table. A very large scale makes every softmax uniform over the
/// **identical** option set, which is the control arm of the §16 experiment: same enumeration,
/// same code path, only the sampling differs.
pub fn run_heuristic_game(
    seed: u64,
    home_roster: &str,
    away_roster: &str,
    edition: &str,
    temp_scale: f32,
    away_scale: Option<f32>,
    mode: ffb_engine::agent::Mode,
    mode_away: ffb_engine::agent::Mode,
) -> (Vec<GameEvent>, i32, i32) {
    use ffb_engine::agent::{Agent, HeuristicAgent};

    let rules = edition_to_rules(edition);
    let home = make_team(home_roster, "home", edition);
    let away = make_team(away_roster, "away", edition);
    let mut options: Vec<(&str, &str)> = vec![(INDUCEMENTS, "true")];
    options.extend_from_slice(BASELINE_SETUP_OPTIONS);
    let mut engine = GameState::new_full_pregame(home, away, rules, seed, &options);

    let mut home_agent = HeuristicAgent::with_mode(seed, temp_scale, mode);
    let mut away_agent =
        HeuristicAgent::with_mode(seed ^ 0xFFFF_FFFF, away_scale.unwrap_or(temp_scale), mode_away);

    let mut all_events: Vec<GameEvent> = Vec::new();
    // `FFB_HEUR_TIME=1` separates AGENT time from ENGINE time. Wall-clock per game confounds the
    // two badly here: the sharper arms produce ~70% more events per game, so they do more engine
    // work for reasons that have nothing to do with how long scoring takes.
    let timed = std::env::var_os("FFB_HEUR_TIME").is_some();
    let dump_path = std::env::var("FFB_HEUR_DUMP").ok();
    let mut dump: Vec<DecisionRec> = Vec::new();
    let mut agent_ns: u128 = 0;
    let mut engine_ns: u128 = 0;
    let mut decisions: u64 = 0;
    // Per-prompt-class accounting. The arms score identically, so a difference in the AVERAGE
    // cost per decision can only come from a different MIX of decisions -- this is what proves
    // that rather than asserting it. Also a whole-loop timer, so `agent + engine + residual`
    // has to add up to the wall clock and nothing can hide outside the two brackets.
    let mut per_class: std::collections::BTreeMap<&'static str, (u64, u128)> = Default::default();
    // FFB_GIVE_TRACE: was the give the ONLY way that touchdown could happen?
    //
    // A touchdown after a hand-off or a pass is not evidence the give earned it - the carrier may
    // have been able to walk it in himself. The counterfactual that matters is the thrower's
    // position AT THE START OF HIS TEAM'S TURN: if the endzone was already within his own
    // movement plus two rushes, running was available and the give was a stylistic choice. If it
    // was not, the give created a touchdown that no amount of running could have produced.
    //
    // The event stream cannot answer this - it carries no coordinates for the thrower - so the
    // reach has to be snapshotted at every turn change, before any movement happens, and looked
    // up when the give resolves.
    let give_trace = std::env::var_os("FFB_GIVE_TRACE").is_some();
    let mut turn_reach: std::collections::HashMap<String, (i32, i32)> = Default::default();
    let mut turn_key: Option<(String, i32)> = None;
    let loop_start = std::time::Instant::now();
    for _ in 0..200_000 {
        if engine.is_finished() { break; }
        if engine.current_prompt().is_none() { break; }
        let side = engine.active_side();
        let class: &'static str = if timed {
            match engine.current_prompt() {
                Some(p) => prompt_class(p),
                None => "none",
            }
        } else {
            ""
        };
        let seq = std::env::var_os("FFB_SEQ").is_some();
        let seq_class: &'static str = if seq {
            engine.current_prompt().map(prompt_class).unwrap_or("none")
        } else {
            ""
        };
        // The snapshot has to be taken BEFORE the action is applied: the picture must show the
        // board the agent was looking at when it scored, not the board its choice produced.
        let pre_snap = if dump_path.is_some() {
            Some(snap(&engine, dump.len(), String::new(), Vec::new()))
        } else {
            None
        };
        let dump_prompt: String = if dump_path.is_some() {
            engine.current_prompt().map(prompt_class).unwrap_or("none").to_string()
        } else {
            String::new()
        };
        let dump_dice: Option<Vec<i32>> = if dump_path.is_some() {
            match engine.current_prompt() {
                Some(AgentPrompt::BlockChoice { dice, .. }) => Some(dice.clone()),
                _ => None,
            }
        } else {
            None
        };
        if give_trace {
            let g = &engine.game;
            let home_turn = matches!(side, TeamSide::Home);
            let td = if home_turn { &g.turn_data_home } else { &g.turn_data_away };
            let key = (format!("{:?}", side), td.turn_nr);
            if turn_key.as_ref() != Some(&key) {
                // New team turn: record every player's distance to the endzone he is attacking and
                // his own reach, BEFORE he has moved a square this turn.
                turn_key = Some(key);
                turn_reach.clear();
                for (pid, c) in g.field_model.player_coordinates.iter() {
                    if let Some(pl) = g.player(pid) {
                        let ez = if home_turn { 25 } else { 0 };
                        let d = (c.x - ez).abs();
                        let reach = pl.movement_with_modifiers() + 2;
                        turn_reach.insert(pid.clone(), (d, reach));
                    }
                }
            }
        }
        let t0 = if timed { Some(std::time::Instant::now()) } else { None };
        let action = if matches!(side, TeamSide::Home) {
            home_agent.act(&engine)
        } else {
            away_agent.act(&engine)
        };
        if let Some(sn) = pre_snap {
            let ag = if matches!(side, TeamSide::Home) { &home_agent } else { &away_agent };
            // Keep the most likely options and the one actually taken; report the rest as a total.
            let mut idx: Vec<usize> = (0..ag.last_options.len()).collect();
            idx.sort_by(|&a, &b| {
                ag.last_options[b].p
                    .partial_cmp(&ag.last_options[a].p)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then(a.cmp(&b))
            });
            idx.truncate(DUMP_OPTION_CAP);
            if !idx.contains(&ag.last_chosen) && ag.last_chosen < ag.last_options.len() {
                idx.pop();
                idx.push(ag.last_chosen);
            }
            let kept_chosen = idx.iter().position(|&j| j == ag.last_chosen).unwrap_or(0);
            let kept: Vec<_> = idx.iter().map(|&j| ag.last_options[j].clone()).collect();
            let p_shown: f32 = kept.iter().map(|o| o.p).sum();
            dump.push(DecisionRec {
                i: dump.len(),
                side: format!("{:?}", side).to_lowercase(),
                prompt: dump_prompt,
                chosen: kept_chosen,
                options: kept,
                n_options: ag.last_options.len(),
                p_shown,
                taken: {
                    let mut t = describe_action(&action);
                    if let (Some(dice), ffb_engine::action::Action::BlockChoice { die_index, .. }) =
                        (&dump_dice, &action)
                    {
                        if let Some(d) = dice.get(*die_index) {
                            t = format!("BlockChoice(die {die_index} = {})", die_face(*d));
                        }
                    }
                    t
                },
                snap: sn,
            });
        }
        if seq {
            eprintln!("SEQ {:>6} {:<22} -> {}", format!("{:?}", side), seq_class,
                      describe_action(&action));
        }
        if let Some(t) = t0 {
            let d = t.elapsed().as_nanos();
            agent_ns += d;
            decisions += 1;
            let e = per_class.entry(class).or_insert((0, 0));
            e.0 += 1;
            e.1 += d;
        }
        let t1 = if timed { Some(std::time::Instant::now()) } else { None };
        let r = engine.apply(side, action);
        if let Some(t) = t1 { engine_ns += t.elapsed().as_nanos(); }
        match r {
            Ok(evs) => {
                if give_trace {
                    for e in &evs {
                        let (kind, thrower) = match e {
                            GameEvent::HandOver { from_id, .. } => ("handoff", Some(from_id.clone())),
                            GameEvent::PassRoll { player_id, .. } => ("pass", Some(player_id.clone())),
                            _ => ("", None),
                        };
                        // Touchdown and turn-end markers, so the correlation can be done from this
                        // one stream: a give "converted" if a touchdown by the giving side follows
                        // it before that side's turn ends.
                        match e {
                            GameEvent::Touchdown { player_id, .. } =>
                                eprintln!("GTD scorer={player_id}"),
                            GameEvent::TurnEnd { team_id, turn_nr } =>
                                eprintln!("GEND team={team_id} turn={turn_nr}"),
                            _ => {}
                        }
                        if let Some(tid) = thrower {
                            let (d, reach) = turn_reach.get(&tid).copied().unwrap_or((-1, -1));
                            // in_range: the thrower could have reached the endzone himself, from
                            // where he stood when the turn began.
                            let in_range = d >= 0 && d <= reach;
                            eprintln!(
                                "GIVE kind={kind} thrower={tid} d_turn_start={d} reach={reach} in_range={in_range}"
                            );
                        }
                    }
                }
                all_events.extend(evs)
            }
            Err(e) => { eprintln!("engine error seed {seed}: {e}"); break; }
        }
    }
    if let Some(path) = &dump_path {
        let mut out = String::new();
        for r in &dump {
            out.push_str(&serde_json::to_string(r).unwrap_or_default());
            out.push('\n');
        }
        std::fs::write(path, out).expect("write heuristic dump");
        eprintln!("HDUMP {} decisions -> {}", dump.len(), path);
    }
    if timed {
        let total_ns = loop_start.elapsed().as_nanos();
        let residual = total_ns.saturating_sub(agent_ns + engine_ns);
        eprintln!(
            "HTOTAL seed={seed} loop_ms={:.1} agent_ms={:.1} engine_ms={:.1}              residual_ms={:.1} residual_pct={:.2}%",
            total_ns as f64 / 1e6,
            agent_ns as f64 / 1e6,
            engine_ns as f64 / 1e6,
            residual as f64 / 1e6,
            100.0 * residual as f64 / total_ns.max(1) as f64,
        );
        for (k, (n, ns)) in &per_class {
            eprintln!(
                "HCLASS seed={seed} class={k} n={n} total_ms={:.2} us_each={:.1}",
                *ns as f64 / 1e6,
                *ns as f64 / 1e3 / (*n).max(1) as f64,
            );
        }
        eprintln!(
            "HTIME seed={seed} decisions={decisions} agent_ms={:.1} engine_ms={:.1}              agent_us_per_decision={:.1} agent_share={:.1}%",
            agent_ns as f64 / 1e6,
            engine_ns as f64 / 1e6,
            agent_ns as f64 / 1e3 / decisions.max(1) as f64,
            100.0 * agent_ns as f64 / (agent_ns + engine_ns).max(1) as f64,
        );
    }

    let score_home = engine.game.game_result.home.score;
    let score_away = engine.game.game_result.away.score;
    (all_events, score_home, score_away)
}

#[cfg(test)]
mod baseline_option_tests {
    use super::BASELINE_SETUP_OPTIONS;
    use ffb_model::option::game_option_id::MB_STACKS_AGAINST_CHAINSAW;

    /// Regression (goblin seed 99): the parity game must enable mbStacksAgainstChainsaw to match
    /// Java's runtime (a knocked-down Chainsaw wielder's own chainsaw AND the opponent's Mighty Blow
    /// both apply to its fall armour). Dropping this reverts a fallen Looney's KO to a prone-only
    /// result and desyncs the shared dice stream.
    #[test]
    fn baseline_enables_mb_stacks_against_chainsaw() {
        assert!(
            BASELINE_SETUP_OPTIONS.iter().any(|(k, v)| *k == MB_STACKS_AGAINST_CHAINSAW && *v == "true"),
            "BASELINE_SETUP_OPTIONS must set mbStacksAgainstChainsaw=true to match Java parity",
        );
    }
}

#[cfg(test)]
mod team_file_tests {
    use super::*;

    fn teams_root() -> std::path::PathBuf {
        for c in ["data/teams", "../data/teams", "../../data/teams"] {
            let p = std::path::Path::new(c);
            if p.exists() { return p.to_path_buf(); }
        }
        panic!("data/teams not found from test cwd");
    }

    /// Validate every hand-drafted team spec (data/teams/) against the drafting rules and
    /// heuristics documented in docs/TEAM_DRAFTS_*.md: 11-16 players within position quantity
    /// caps, >=2 rerolls <= roster max, spend <= 1,100,000, team_value per UtilTeamValue, and
    /// both sides materialize through make_team_from_file.
    #[test]
    fn all_hand_drafted_team_files_are_legal() {
        for edition in ["bb2016", "bb2025"] {
            let rosters = match edition {
                "bb2016" => bb2016_rosters(),
                _ => bb2025_rosters(),
            };
            let dir = teams_root().join(edition);
            let mut checked = 0;
            for entry in std::fs::read_dir(&dir).unwrap() {
                let path = entry.unwrap().path();
                if path.extension().and_then(|e| e.to_str()) != Some("json") { continue; }
                let race = path.file_stem().unwrap().to_str().unwrap()
                    .strip_prefix("team_").unwrap().to_string();
                let spec: TeamFileJson = serde_json::from_str(
                    &std::fs::read_to_string(&path).unwrap()).unwrap();
                let roster = rosters.iter().find(|r| r.id == spec.roster_id)
                    .unwrap_or_else(|| panic!("{edition}/{race}: roster id {} missing", spec.roster_id));

                let n = spec.players.len();
                assert!((11..=16).contains(&n), "{edition}/{race}: {n} players");
                assert!(spec.rerolls >= 2, "{edition}/{race}: fewer than 2 rerolls");
                assert!(spec.rerolls <= roster.max_rerolls, "{edition}/{race}: rerolls over roster max");

                let mut players_cost = 0i32;
                let mut counts: std::collections::HashMap<&str, i32> = Default::default();
                for pl in &spec.players {
                    let pos = roster.positions.iter().find(|p| p.id == pl.position_id)
                        .unwrap_or_else(|| panic!("{edition}/{race}: unknown position {}", pl.position_id));
                    players_cost += pos.cost;
                    *counts.entry(pos.id.as_str()).or_default() += 1;
                }
                for (pid, cnt) in &counts {
                    let q = roster.positions.iter().find(|p| p.id == *pid).unwrap().quantity;
                    assert!(*cnt <= q, "{edition}/{race}: {cnt}x {pid} exceeds quantity cap {q}");
                }

                let apo_cost = spec.apothecaries * 50_000;
                let fans_cost = if edition == "bb2025" {
                    (spec.dedicated_fans - 1).max(0) * 5_000
                } else {
                    spec.fan_factor * 10_000
                };
                let spend = players_cost + spec.rerolls * roster.reroll_cost + apo_cost + fans_cost;
                assert!(spend <= 1_100_000, "{edition}/{race}: spend {spend} over 1.1M budget");
                assert_eq!(spend + spec.treasury, 1_100_000,
                    "{edition}/{race}: spend {spend} + treasury {} != 1.1M", spec.treasury);

                // Java UtilTeamValue: rerolls*rrCost + fanFactor*10k + coaches/cheerleaders (0)
                // + apo*50k + player position costs. Dedicated Fans are NOT part of TV.
                let tv = spec.rerolls * roster.reroll_cost + spec.fan_factor * 10_000
                    + apo_cost + players_cost;
                assert_eq!(spec.team_value, tv, "{edition}/{race}: team_value mismatch");

                for side in ["home", "away"] {
                    let team = make_team_from_file(&race, side, edition)
                        .unwrap_or_else(|e| panic!("{edition}/{race}/{side}: {e}"));
                    // Stars ride on top of the hand-drafted spend: they model an INDUCED star
                    // player (bought from petty cash in a real game), so they are deliberately
                    // outside the 1.1M budget/treasury identity checked above.
                    assert_eq!(team.players.len(), n + spec.stars.len(),
                        "{edition}/{race}/{side}: player count");
                    assert_eq!(team.rerolls, spec.rerolls);
                    assert_eq!(team.dedicated_fans, spec.dedicated_fans);
                }
                checked += 1;
            }
            assert_eq!(checked, 29, "{edition}: expected 29 team files, found {checked}");
        }
    }
}

#[cfg(test)]
mod fumbbl_roster_tests {
    use super::make_team;
    use ffb_model::enums::SkillId;

    /// Regression (dark_elf_league_fumbbl seed 1 step 9): the FUMBBL league-import rosters have a
    /// numeric FUMBBL `id` and a generic race `name` that collides with the standard roster, so the
    /// CLI key ("dark_elf_league_fumbbl") matched neither. make_team_from_roster returned Err and
    /// make_team fell back to an all-generic-lineman team (AG3, no skills), diverging from Java's
    /// real roster at the first dodge. The id alias must build the real roster: the away team's
    /// first player is a Witch Elf (AG2, Dodge/Frenzy/Jump Up), never a bare AG3 lineman.
    #[test]
    fn fumbbl_dark_elf_builds_real_roster_not_lineman_fallback() {
        let team = make_team("dark_elf_league_fumbbl", "away", "bb2025");
        let p = &team.players[0];
        assert_eq!(p.agility, 2, "first fumbbl player must be the AG2 Witch Elf, not the AG3 lineman fallback");
        assert!(
            p.starting_skills.iter().any(|s| s.skill_id == SkillId::Dodge),
            "first fumbbl player must carry the Witch Elf's Dodge skill (fallback lineman has none)",
        );
        assert_eq!(team.players.len(), 11);
    }

    #[test]
    fn fumbbl_khemri_and_slann_build_real_rosters() {
        // Both share the same numeric-id lookup gap; a fallback would make every player an
        // identical AG3 lineman, so assert the built teams are not uniformly the lineman default.
        for roster in ["khemri_fumbbl", "slann_fumbbl"] {
            let team = make_team(roster, "home", "bb2025");
            // Hand-drafted specs (data/teams/) field 11-16 players; the point here is only
            // that the REAL roster loaded (not the generic-lineman fallback).
            assert!((11..=16).contains(&team.players.len()), "{roster} must build a full team");
            let all_bare_linemen = team.players.iter()
                .all(|p| p.agility == 3 && p.movement == 6 && p.armour == 8 && p.starting_skills.is_empty());
            assert!(!all_bare_linemen, "{roster} must not fall back to a uniform generic-lineman team");
        }
    }

    /// Regression (slann_fumbbl seed 1 step 9): the FUMBBL slann roster (744258) spells the Kroxigor's
    /// trait "Bone-head" (hyphen). Java's bb2025 SkillFactory keys the skill by its canonical name
    /// "Bone Head" (space) and `forName` does an exact case-insensitive match — so "bone-head" resolves
    /// to null and the Kroxigor gets NO Bone Head. Rust's lenient resolver used to keep it, adding a
    /// per-activation negatrait d6 Java never rolled, desyncing the dice stream (the Kroxigor's dodge
    /// then failed → turnover). bb2025 must therefore build the Kroxigor WITHOUT Bone Head to match Java.
    /// Java's parity team sheets (`ffb-server/teams/team_lineman_parity_{home,away}.xml`) carry
    /// `<fanFactor>5`; Rust's synthetic twin hard-coded 0. Fan factor feeds the spectator count,
    /// which sets FAME, which decides the kickoff extra-re-roll contest — so the two engines picked
    /// different winners (bb2016 lineman seed 65: Java 8000/11000 spectators → fameA=1 → a 3-3 tie
    /// where BOTH teams gain; Rust 3000/6000 → fameA=2 → away only). Invisible until re-roll counts
    /// entered the state hash.
    #[test]
    fn lineman_parity_team_matches_the_java_sheet_fan_factor() {
        for side in ["home", "away"] {
            let team = make_team("lineman", side, "bb2016");
            assert_eq!(team.fan_factor, 5, "{side} must match <fanFactor>5 in the Java team sheet");
            assert_eq!(team.rerolls, 3, "{side} must match <reRolls>3 in the Java team sheet");
        }
    }

    /// §9 star drafting: a team spec's `stars` list fields the star as an ordinary rostered
    /// player, resolved from data/star_players/all_editions.json (the Java sheets get the
    /// identical player via gen_java_parity_data.py). The list must come out nr-SORTED even
    /// though stars are appended after the regular players: the harness activation snapshots
    /// index by position (idx % N), so an out-of-order list pairs the same idx with different
    /// players in the two engines (dwarf bb2016 pilot seed 1, half 2: pick=8 N=10 gave Java
    /// nr 12 and Rust nr 13 — 5/10 seeds red until the sort).
    #[test]
    fn star_drafting_injects_the_star_nr_sorted() {
        let team = make_team("dwarf", "home", "bb2016");
        let barik = team.players.iter().find(|p| p.nr == 11)
            .expect("bb2016 dwarf spec drafts dwarf.Farblast at nr 11");
        assert!(barik.name.contains("Barik"), "nr 11 must be the star, got {}", barik.name);
        assert!(
            barik.starting_skills.iter().any(|s| s.skill_id == SkillId::HailMaryPass),
            "Barik must carry Hail Mary Pass",
        );
        let nrs: Vec<i32> = team.players.iter().map(|p| p.nr).collect();
        let mut sorted = nrs.clone();
        sorted.sort();
        assert_eq!(nrs, sorted, "players must be nr-sorted");
    }

    #[test]
    fn fumbbl_slann_kroxigor_has_no_bonehead_in_bb2025() {
        let team = make_team("slann_fumbbl", "away", "bb2025");
        let kroxigor = &team.players[0];
        assert_eq!(kroxigor.strength, 5, "first slann_fumbbl player must be the ST5 Kroxigor");
        assert!(
            !kroxigor.starting_skills.iter().any(|s| s.skill_id == SkillId::BoneHead),
            "bb2025 Kroxigor must NOT carry Bone Head — the roster's hyphen spelling \"Bone-head\" \
             does not resolve against Java's canonical \"Bone Head\"",
        );
        // Sanity: the rest of the Kroxigor's traits (spelled with spaces) still resolve.
        assert!(
            kroxigor.starting_skills.iter().any(|s| s.skill_id == SkillId::PrehensileTail),
            "Kroxigor's other traits (Prehensile Tail etc.) must still be present",
        );
    }
}
