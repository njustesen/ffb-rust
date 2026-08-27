mod runner;
mod log_format;
mod comparator;
mod update_progress;
mod network_test;
mod state_hash;
mod coverage_report;
mod t3_checklist;
mod visual;

#[allow(dead_code)] mod debug_rng;

/// Parsed CLI arguments for the parity runner.
struct ParityArgs {
    network: bool,
    coverage: bool,
    /// `--uniform`: drive full games with `UniformAgent` (samples uniformly over every
    /// legal action, including inducement purchases) instead of `RandomAgent`, tallying
    /// into the same `CoverageReport` plus the data-driven `full_mechanic_items` checklist.
    uniform: bool,
    /// `--all-rosters`: sweep every roster in the edition (mirror matchups, roster vs
    /// itself) instead of just `--home`/`--away`. Only consulted by `--uniform`.
    all_rosters: bool,
    /// `--all-editions`: sweep bb2016/bb2020/bb2025 instead of just `--edition`. Only
    /// consulted by `--uniform`.
    all_editions: bool,
    home: String,
    home_java: String,
    away: String,
    away_java: String,
    edition: String,
    seed_start: u64,
    seed_end: u64,
    no_abort: bool,
    verbose: bool,
    visualize: bool,
    /// Parity tier: 2 = T2 trivial agent (1 decisionRng pick + EndTurn, the 26-race
    /// regression suite), 3 = T3 Phase 2 agent (real activations). Default 2.
    tier: u8,
    /// `--reuse-java`: skip the JVM entirely and compare against the Java logs already on
    /// disk, when those logs were produced by the same jar, the same Java server data and
    /// the same invocation. ~98% of a gate's wall-clock is JVM time, and an iteration that
    /// only touched Rust cannot change what Java produced. The reuse is refused (and the
    /// JVM runs) unless `runner::java_logs_reusable` confirms the recorded fingerprint.
    reuse_java: bool,
    /// `--heuristic <scale>`: run BOTH sides with `HeuristicAgent` at the given temperature
    /// scale and dump one JSONL events file per seed. `1.0` = the docs/HEURISTIC_AGENT.md §8
    /// table; a large scale (e.g. `1e6`) makes every softmax uniform over the IDENTICAL option
    /// set, which is the control arm — same enumeration, same code path, only the sampling
    /// differs. `None` disables the mode.
    heuristic: Option<f32>,
    /// `--heuristic-away <scale>`: give the AWAY side a different temperature scale, turning the
    /// run into a genuine head-to-head instead of self-play.
    heuristic_away: Option<f32>,
    /// `--mode wide|deep`: how the heuristic agent searches (see agent::Mode).
    agent_mode: String,
    /// `--mode-away <wide|deep>`: give the away agent a different search shape, for head-to-head.
    agent_mode_away: Option<String>,
    /// `--out <dir>`: where `--heuristic` writes its per-seed events files.
    out_dir: String,
    /// `--agent random|heuristic`: which agent drives the Rust side of a PARITY run.
    ///
    /// Orthogonal to `--heuristic <scale>`, which is a Rust-only self-play EXPERIMENT that never
    /// launches a JVM. This flag instead swaps the driver inside the real parity comparison, so
    /// the heuristic agent can be gated against Java one prompt class at a time. Default
    /// `random`, which is the historical behaviour byte-for-byte.
    agent: String,
    /// `--heur-scale <f32>`: temperature scale for `--agent heuristic`. 0 = argmax and consumes
    /// NO agent RNG at all, which is the cleanest first rung: any divergence is then the scorer
    /// or the engine, never the sampler.
    heur_scale: f32,
    /// `--heur-classes <all|none|csv>`: which prompt classes the heuristic scores; the rest are
    /// answered by the embedded parity `RandomAgent`. `none` (rung 0) must reproduce the
    /// random-agent gate exactly. See `agent::PromptClass::name`.
    heur_classes: String,
}

impl ParityArgs {
    fn parse() -> Self {
        let raw: Vec<String> = std::env::args().skip(1).collect();
        let mut home = "lineman".to_string();
        let mut away = "lineman".to_string();
        let mut edition = "bb2025".to_string();
        let mut seed_start = 1u64;
        let mut seed_end = 100u64;
        let mut network = false;
        let mut coverage = false;
        let mut uniform = false;
        let mut all_rosters = false;
        let mut all_editions = false;
        let mut no_abort = false;
        let mut verbose = false;
        let mut visualize = false;
        let mut agent_mode = String::from("wide");
        let mut agent_mode_away: Option<String> = None;
        let mut tier = 2u8;
        let mut reuse_java = false;
        let mut heuristic: Option<f32> = None;
        let mut heuristic_away: Option<f32> = None;
        let mut out_dir = "parity/heuristic".to_string();
        let mut agent = "random".to_string();
        let mut heur_scale = 0.0f32;
        let mut heur_classes = "none".to_string();

        let mut i = 0;
        while i < raw.len() {
            match raw[i].as_str() {
                "--network" => network = true,
                "--coverage" => coverage = true,
                "--uniform" => uniform = true,
                "--all-rosters" => all_rosters = true,
                "--all-editions" => all_editions = true,
                "--no-abort" => no_abort = true,
                "--verbose" => verbose = true,
                "--visualize" => visualize = true,
                "--reuse-java" => reuse_java = true,
                "--heuristic" if i + 1 < raw.len() => {
                    heuristic = raw[i + 1].parse().ok(); i += 1;
                }
                "--heuristic-away" if i + 1 < raw.len() => {
                    heuristic_away = raw[i + 1].parse().ok(); i += 1;
                }
                "--out" if i + 1 < raw.len() => { out_dir = raw[i + 1].clone(); i += 1; }
                "--agent" if i + 1 < raw.len() => { agent = raw[i + 1].clone(); i += 1; }
                "--heur-scale" if i + 1 < raw.len() => {
                    heur_scale = raw[i + 1].parse().unwrap_or(0.0); i += 1;
                }
                "--heur-classes" if i + 1 < raw.len() => { heur_classes = raw[i + 1].clone(); i += 1; }
                "--mode" if i + 1 < raw.len() => { agent_mode = raw[i + 1].clone(); i += 1; }
                "--mode-away" if i + 1 < raw.len() => { agent_mode_away = Some(raw[i + 1].clone()); i += 1; }
                "--home" if i + 1 < raw.len() => { home = raw[i + 1].clone(); i += 1; }
                "--away" if i + 1 < raw.len() => { away = raw[i + 1].clone(); i += 1; }
                "--edition" if i + 1 < raw.len() => { edition = raw[i + 1].clone(); i += 1; }
                "--tier" if i + 1 < raw.len() => { tier = raw[i + 1].parse().unwrap_or(2); i += 1; }
                "--seeds" if i + 1 < raw.len() => {
                    let s = &raw[i + 1];
                    if let Some(dash) = s.find('-') {
                        seed_start = s[..dash].parse().unwrap_or(1);
                        seed_end   = s[dash+1..].parse().unwrap_or(100);
                    } else {
                        seed_end = s.parse().unwrap_or(100);
                    }
                    i += 1;
                }
                _ => {}
            }
            i += 1;
        }

        let home_java = runner::java_team_id(&home, "home", &edition);
        let away_java = runner::java_team_id(&away, "away", &edition);

        ParityArgs { network, coverage, uniform, all_rosters, all_editions, home, home_java, away, away_java, edition, seed_start, seed_end, no_abort, verbose, visualize, tier, reuse_java, heuristic, heuristic_away, out_dir, agent_mode, agent_mode_away, agent, heur_scale, heur_classes }
    }
}

fn main() {
    env_logger::init();

    let args = ParityArgs::parse();

    if args.network {
        println!("Running network integration test...");
        network_test::run();
        return;
    }

    if let Some(scale) = args.heuristic {
        let label = if scale <= 0.0 {
            "argmax"
        } else if scale >= 1000.0 {
            "uniform"
        } else {
            "heuristic"
        };
        let parse_mode = |s: &str| match s {
            "deep" => ffb_engine::agent::Mode::Deep,
            "wide-noball" => ffb_engine::agent::Mode::WideNoBall,
            "wide-nopass" => ffb_engine::agent::Mode::WideNoPass,
            "wide-nohandoff" => ffb_engine::agent::Mode::WideNoHandOff,
            _ => ffb_engine::agent::Mode::Wide,
        };
        let mode = parse_mode(&args.agent_mode);
        let mode_away = args.agent_mode_away.as_deref().map(parse_mode).unwrap_or(mode);
        let quiet = std::env::var_os("FFB_QUIET").is_some();
        if !quiet { std::fs::create_dir_all(&args.out_dir).ok(); }
        eprintln!("Running {} seeds with HeuristicAgent (temp_scale={scale}, arm={label}, mode={:?})                    {} vs {} {}", args.seed_end - args.seed_start + 1, mode, args.home, args.away, args.edition);
        let t0 = std::time::Instant::now();
        for seed in args.seed_start..=args.seed_end {
            let (events, sh, sa) = runner::run_heuristic_game(
                seed, &args.home, &args.away, &args.edition, scale, args.heuristic_away, mode, mode_away);
            if quiet {
                // FFB_QUIET: no event dump, no per-seed line. For runtime measurement only.
                std::hint::black_box((&events, sh, sa));
            } else {
                let mut out = String::new();
                for e in &events {
                    out.push_str(&serde_json::to_string(e).unwrap_or_default());
                    out.push('\n');
                }
                let path = format!("{}/seed_{}_{}_events.jsonl", args.out_dir, seed, label);
                std::fs::write(&path, out).expect("write events");
                println!("seed {seed}: score {sh}-{sa}, {} events", events.len());
            }
        }
        eprintln!("done in {:.1}s", t0.elapsed().as_secs_f32());
        return;
    }

    if args.visualize {
        if args.seed_start != args.seed_end {
            eprintln!("--visualize requires a single seed, e.g. --seeds 1");
            std::process::exit(1);
        }
        let seed = args.seed_start;
        println!("Running full game for seed {seed} ({} vs {}, {})...",
            args.home, args.away, args.edition);
        let snaps = runner::run_visual_game(seed, &args.home, &args.away, &args.edition);
        println!("  {} snapshots captured", snaps.len());
        let html = visual::generate_html(seed, &args.home, &args.away, &args.edition, &snaps);
        let dir = log_format::matchup_dir(&args.edition, &args.home, &args.away);
        std::fs::create_dir_all(&dir).ok();
        let path = format!("{dir}/seed_{seed}_visual.html");
        std::fs::write(&path, &html).expect("Failed to write visual HTML");
        println!("Visual replay written to: {path}");
        return;
    }

    let total = args.seed_end - args.seed_start + 1;

    // ── Uniform mode ─────────────────────────────────────────────────────────────
    // Uses `UniformAgent` (samples uniformly over every legal action, including
    // inducement purchases) instead of `RandomAgent`. Sweeps every roster (mirror
    // matchups) and/or every edition when `--all-rosters`/`--all-editions` are given.
    // No Java invocation or parity comparison — this is a "how much of the mechanic
    // surface does random play exercise" run, the "parity match run" harness.
    if args.uniform {
        let editions: Vec<String> = if args.all_editions {
            vec!["bb2016".into(), "bb2020".into(), "bb2025".into()]
        } else {
            vec![args.edition.clone()]
        };

        let mut cov = coverage_report::CoverageReport::default();
        let n_threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).max(1);

        for edition in &editions {
            let matchups: Vec<(String, String)> = if args.all_rosters {
                runner::roster_names_for_edition(edition).into_iter().map(|r| (r.clone(), r)).collect()
            } else {
                vec![(args.home.clone(), args.away.clone())]
            };

            for (home, away) in &matchups {
                println!("Uniform run: {home} vs {away} ({edition}) — {total} seeds");
                let mut summary = coverage_report::MatchupSummary {
                    home: home.clone(), away: away.clone(), seeds: total as u32,
                    home_wins: 0, away_wins: 0, draws: 0,
                    touchdowns_home: 0, touchdowns_away: 0,
                };

                let seeds: Vec<u64> = (args.seed_start..=args.seed_end).collect();
                for chunk in seeds.chunks(n_threads.max(1)) {
                    let results: Vec<(Vec<ffb_model::events::GameEvent>, i32, i32, Vec<String>)> =
                        std::thread::scope(|scope| {
                            let handles: Vec<_> = chunk.iter().map(|&seed| {
                                let home = home.clone();
                                let away = away.clone();
                                let edition = edition.clone();
                                scope.spawn(move || runner::run_uniform_game(seed, &home, &away, &edition))
                            }).collect();
                            handles.into_iter().map(|h| h.join().expect("uniform game thread panicked")).collect()
                        });

                    for (events, home_score, away_score, unhandled) in results {
                        for ev in &events { cov.tally(&ev); }
                        for prompt in &unhandled { cov.record_unhandled_prompt(prompt); }
                        cov.games += 1;
                        cov.touchdowns_home += home_score as u32;
                        cov.touchdowns_away += away_score as u32;
                        summary.touchdowns_home += home_score as u32;
                        summary.touchdowns_away += away_score as u32;
                        if home_score > away_score { cov.home_wins += 1; summary.home_wins += 1; }
                        else if away_score > home_score { cov.away_wins += 1; summary.away_wins += 1; }
                        else { cov.draws += 1; summary.draws += 1; }
                    }
                }
                println!("  {} vs {} done ({}-{} home wins over {} seeds)",
                    home, away, summary.home_wins, summary.away_wins, summary.seeds);
                cov.matchups.push(summary);
            }
        }

        cov.skill_names = coverage_report::build_skill_names();
        let checklist_edition = if args.all_editions { "bb2025".to_string() } else { args.edition.clone() };
        let (md, ok) = t3_checklist::render_full_mechanic_markdown(&cov, cov.games, &checklist_edition);
        std::fs::write("FULL_MECHANIC_COVERAGE.md", &md).ok();
        let json = serde_json::to_string(&cov).expect("coverage serialization failed");
        std::fs::write("uniform_coverage.html", coverage_report::generate_html(&json)).ok();
        std::fs::write("coverage_uniform.json", &json).ok();
        println!("\n{md}");
        println!("Coverage written to FULL_MECHANIC_COVERAGE.md and uniform_coverage.html ({} games)", cov.games);
        if !ok {
            eprintln!("UNIFORM RUN: required coverage items are MISSING (see FULL_MECHANIC_COVERAGE.md).");
            std::process::exit(1);
        }
        return;
    }

    // ── Coverage mode ────────────────────────────────────────────────────────────
    // Uses the full RandomAgent (players activate and take real actions) to collect
    // the broadest possible event coverage. No Java invocation or parity comparison.
    // `--all-rosters` / `--all-editions` sweep here exactly as they do for `--uniform`, so the
    // parity agent's reachable surface can be measured in ONE run instead of 90 shell-outs into a
    // fixed `coverage.html` path. A per-edition JSON is written alongside the combined one: the
    // editions do NOT cover the same mechanics (BB2016 has no Prayers, BB2020/25 have no
    // apothecary-on-KO, …), so a single merged total hides exactly the gaps worth seeing.
    if args.coverage {
        let editions: Vec<String> = if args.all_editions {
            vec!["bb2016".into(), "bb2020".into(), "bb2025".into()]
        } else {
            vec![args.edition.clone()]
        };
        let n_threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).max(1);
        let mut combined = coverage_report::CoverageReport::default();

        for edition in &editions {
            let matchups: Vec<(String, String)> = if args.all_rosters {
                runner::roster_names_for_edition(edition).into_iter().map(|r| (r.clone(), r)).collect()
            } else {
                vec![(args.home.clone(), args.away.clone())]
            };
            let mut per_edition = coverage_report::CoverageReport::default();

            for (home, away) in &matchups {
                println!("Coverage run: {home} vs {away} ({edition}) — {total} seeds");
                let mut summary = coverage_report::MatchupSummary {
                    home: home.clone(), away: away.clone(), seeds: total as u32,
                    home_wins: 0, away_wins: 0, draws: 0,
                    touchdowns_home: 0, touchdowns_away: 0,
                };
                let seeds: Vec<u64> = (args.seed_start..=args.seed_end).collect();
                for chunk in seeds.chunks(n_threads.max(1)) {
                    let results: Vec<(Vec<ffb_model::events::GameEvent>, i32, i32)> =
                        std::thread::scope(|scope| {
                            let handles: Vec<_> = chunk.iter().map(|&seed| {
                                let home = home.clone();
                                let away = away.clone();
                                let edition = edition.clone();
                                scope.spawn(move || runner::run_coverage_game(seed, &home, &away, &edition))
                            }).collect();
                            handles.into_iter().map(|h| h.join().expect("coverage game thread panicked")).collect()
                        });
                    for (events, home_score, away_score) in results {
                        for ev in &events { per_edition.tally(ev); combined.tally(ev); }
                        for c in [&mut per_edition, &mut combined] {
                            c.games += 1;
                            c.touchdowns_home += home_score as u32;
                            c.touchdowns_away += away_score as u32;
                            if home_score > away_score { c.home_wins += 1; }
                            else if away_score > home_score { c.away_wins += 1; }
                            else { c.draws += 1; }
                        }
                        summary.touchdowns_home += home_score as u32;
                        summary.touchdowns_away += away_score as u32;
                        if home_score > away_score { summary.home_wins += 1; }
                        else if away_score > home_score { summary.away_wins += 1; }
                        else { summary.draws += 1; }
                    }
                }
                per_edition.matchups.push(summary.clone());
                combined.matchups.push(summary);
            }

            per_edition.skill_names = coverage_report::build_skill_names();
            let json = serde_json::to_string(&per_edition).expect("coverage serialization failed");
            std::fs::write(format!("coverage_{edition}.json"), &json).ok();
            println!("  wrote coverage_{edition}.json ({} games)", per_edition.games);
        }

        combined.skill_names = coverage_report::build_skill_names();
        let json = serde_json::to_string(&combined).expect("coverage serialization failed");
        std::fs::write("coverage.json", &json).ok();
        std::fs::write("coverage.html", coverage_report::generate_html(&json))
            .expect("failed to write coverage.html");
        println!("Coverage report written to coverage.html / coverage.json ({} games)", combined.games);
        return;
    }

    // ── Parity mode (default) ────────────────────────────────────────────────────
    //
    // Which agent drives the Rust side. `random` is the historical parity driver; `heuristic` is
    // the Java-port ladder, which scores only the named prompt classes and answers the rest
    // through an embedded parity RandomAgent (AGENT_CONTRACT_HEURISTIC.md).
    let agent_spec = match args.agent.as_str() {
        "random" => runner::AgentSpec::Random,
        "heuristic" => {
            let classes = match ffb_engine::agent::ClassMask::parse(&args.heur_classes) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("--heur-classes: {e}");
                    std::process::exit(2);
                }
            };
            let mode = match args.agent_mode.as_str() {
                "deep" => ffb_engine::agent::Mode::Deep,
                "wide-noball" => ffb_engine::agent::Mode::WideNoBall,
                "wide-nopass" => ffb_engine::agent::Mode::WideNoPass,
                "wide-nohandoff" => ffb_engine::agent::Mode::WideNoHandOff,
                _ => ffb_engine::agent::Mode::Wide,
            };
            if args.tier < 3 {
                eprintln!("--agent heuristic requires --tier 3 or higher");
                std::process::exit(2);
            }
            eprintln!(
                "Rust driver: HeuristicAgent (scale={}, mode={:?}, classes={})",
                args.heur_scale, mode, args.heur_classes
            );
            runner::AgentSpec::Heuristic { temp_scale: args.heur_scale, mode, classes }
        }
        other => {
            eprintln!("--agent must be 'random' or 'heuristic', got '{other}'");
            std::process::exit(2);
        }
    };

    let mut passed = 0u64;
    let mut failed = 0u64;
    // Tier 3: aggregate the Rust engine's GameEvents across all seeds for the
    // coverage checklist (the events are a faithful proxy for both engines once
    // the per-activation hashes match).
    let mut t3_cov = (args.tier >= 3).then(coverage_report::CoverageReport::default);

    // Amortize JVM start-up + fat-jar class-loading + server construction (perf reasons #1/#2/#4):
    // run EVERY Java game in ONE JVM via ParityRunner's batch mode, instead of a fresh JVM per seed.
    // Then run the Rust engine per seed. Report per-engine wall-clock at the end.
    //
    // `--reuse-java` short-circuits that when the logs on disk provably came from the same jar,
    // the same Java server data and the same invocation (see runner::java_logs_reusable). The
    // reason for the decision is always printed: a silently-reused stale log would turn a red
    // into a green, so the run must say which of the two it did and why.
    let java_t0 = std::time::Instant::now();
    let reused = if args.reuse_java {
        match runner::java_logs_reusable(args.seed_start, args.seed_end, &args.home_java,
            &args.away_java, &args.home, &args.away, args.tier, &args.edition) {
            Ok(()) => {
                println!("REUSE java logs for {} vs {} ({}) — cached batch matches",
                    args.home, args.away, args.edition);
                true
            }
            Err(why) => {
                println!("REUSE declined ({why}) — running the JVM");
                false
            }
        }
    } else {
        false
    };
    if !reused {
        runner::run_java_headless_range(args.seed_start, args.seed_end, &args.home_java, &args.away_java, &args.home, &args.away, args.tier, &args.edition);
        runner::write_java_manifest(&args.home_java, &args.away_java, &args.home, &args.away, args.tier, &args.edition);
    }
    let java_total = java_t0.elapsed();
    println!("TIMING java_total={:.3}s (batched JVM, {total} seeds)", java_total.as_secs_f64());
    let mut rust_total = std::time::Duration::ZERO;
    let mut rust_panics = 0usize;

    for seed in args.seed_start..=args.seed_end {
        println!("Seed {seed}: {} vs {} ({})", args.home, args.away, args.edition);

        let rust_t0 = std::time::Instant::now();
        // A Rust panic IS a parity divergence — Java played the game, Rust could not. Catch it and
        // record the seed as a FAILURE instead of letting the process abort.
        //
        // This guard exists because an aborting process is indistinguishable from a clean sweep to
        // anything that counts "PARITY FAIL" lines: on 2026-08-14 a whole bb2020 matrix was reported
        // green while the engine panicked on game 1 of every roster (exit 101, zero games compared,
        // zero failure lines to count). Now the run keeps going, the seed is counted, and the
        // summary reports the truth.
        let home = args.home.clone();
        let away = args.away.clone();
        let edition = args.edition.clone();
        let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            runner::run_rust_headless(seed, &home, &away, &edition, args.verbose, args.tier, agent_spec)
        }));
        rust_total += rust_t0.elapsed();

        let result = match panic_result {
            Ok((_, events, _home_score, _away_score)) => {
                if let Some(cov) = t3_cov.as_mut() {
                    for ev in &events { cov.tally(ev); }
                    cov.games += 1;
                }
                comparator::compare_logs(seed, &args.edition, &args.home, &args.away)
            }
            Err(payload) => {
                let msg = payload
                    .downcast_ref::<&str>().map(|s| s.to_string())
                    .or_else(|| payload.downcast_ref::<String>().cloned())
                    .unwrap_or_else(|| "<non-string panic payload>".to_string());
                rust_panics += 1;
                eprintln!("RUST PANIC seed={seed} ({} vs {}): {msg}", args.home, args.away);
                comparator::CompareResult::rust_panic(msg)
            }
        };
        update_progress::update(seed, &args.home, &args.away, &result);

        if result.matches {
            passed += 1;
            println!("✓ seed {seed} ({} vs {}) — {passed}/{total}", args.home, args.away);
        } else {
            failed += 1;
            eprintln!(
                "PARITY FAIL seed={seed} ({} vs {}), step {}: java={:?} rust={:?}",
                args.home, args.away,
                result.divergence_index,
                result.java_event,
                result.rust_event,
            );
            eprintln!("  java_hash={}", result.java_hash);
            eprintln!("  rust_hash={}", result.rust_hash);
            if !args.no_abort {
                println!("TIMING java_total={:.3}s rust_total={:.3}s (aborted at seed {seed})",
                    java_total.as_secs_f64(), rust_total.as_secs_f64());
                eprintln!("→ Enter TDD loop: write Java test, Rust test, fix, restart from seed 1");
                std::process::exit(1);
            }
        }
    }

    println!("TIMING java_total={:.3}s rust_total={:.3}s ({total} seeds; batched JVM)",
        java_total.as_secs_f64(), rust_total.as_secs_f64());

    // Tier-3 coverage checklist: write T3_COVERAGE.md + t3_coverage.html and print
    // the verdict. A missing required item fails the run even when parity passes.
    let mut checklist_ok = true;
    if let Some(cov) = t3_cov.as_mut() {
        cov.skill_names = coverage_report::build_skill_names();
        let (md, ok) = t3_checklist::render_markdown(cov, cov.games);
        // Only enforce coverage on suite-sized runs — a single debug seed can't be
        // expected to roll every mechanic, but it should still print the table.
        checklist_ok = ok || cov.games < 50;
        std::fs::write("T3_COVERAGE.md", &md).ok();
        if let Ok(json) = serde_json::to_string(&*cov) {
            std::fs::write("t3_coverage.html", coverage_report::generate_html(&json)).ok();
        }
        println!("\n{md}");
        println!("Coverage written to T3_COVERAGE.md and t3_coverage.html");
    }

    // ALWAYS emit a verdict line naming how many games were actually COMPARED, and always name
    // panics separately. Counting the absence of "PARITY FAIL" lines cannot distinguish "all
    // passed" from "nothing ran"; this line can, because `passed + failed` is the games compared.
    let panic_note = if rust_panics > 0 {
        format!(" [{rust_panics} RUST PANIC(S) — counted as failures]")
    } else {
        String::new()
    };

    if failed == 0 && checklist_ok {
        println!("PARITY: {passed}/{total} games match.{panic_note}");
    } else if failed == 0 {
        eprintln!("PARITY: {passed}/{total} games match, but required coverage items are MISSING.{panic_note}");
        std::process::exit(1);
    } else {
        eprintln!("PARITY: {passed}/{total} passed, {failed} FAILED.{panic_note}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod roster_name_tests {
    use crate::runner::make_team_from_roster;

    // These tests verify that the roster name normalization introduced in session 39
    // correctly resolves multi-word race names (space vs underscore) and special aliases.

    #[test]
    fn chaos_dwarf_resolves_to_actual_roster() {
        let team = make_team_from_roster("chaos_dwarf", "home", "bb2025")
            .expect("chaos_dwarf must resolve to the Chaos Dwarf roster");
        assert_eq!(team.players.len(), 11, "must build 11-player team");
        // Jersey 1 = Minotaur (qty=1, cost=150k) — wrong roster gives a generic lineman (ag=3)
        let j1 = &team.players[0];
        assert_ne!(j1.agility, 3, "jersey 1 must not be a generic lineman (ag=3)");
    }

    #[test]
    fn dark_elf_resolves_to_actual_roster() {
        make_team_from_roster("dark_elf", "home", "bb2025")
            .expect("dark_elf must resolve to the Dark Elf roster");
    }

    #[test]
    fn high_elf_resolves_to_actual_roster() {
        let team = make_team_from_roster("high_elf", "home", "bb2025")
            .expect("high_elf must resolve to the High Elf roster");
        assert_eq!(team.players.len(), 11);
    }

    #[test]
    fn chaos_pact_resolves_to_actual_roster() {
        let team = make_team_from_roster("chaos_pact", "home", "bb2025")
            .expect("chaos_pact must resolve to the Chaos Pact roster");
        assert_eq!(team.players.len(), 11);
        // Chaos Pact includes positions with agility < 3 (e.g. Goblin ag=2)
        let has_low_ag = team.players.iter().any(|p| p.agility < 3);
        assert!(has_low_ag, "Chaos Pact team must contain at least one low-agility position");
    }

    #[test]
    fn wood_elf_resolves_to_actual_roster() {
        let team = make_team_from_roster("wood_elf", "home", "bb2025")
            .expect("wood_elf must resolve to the Wood Elf roster");
        assert_eq!(team.players.len(), 11);
    }

    #[test]
    fn renegades_resolves_via_alias() {
        // "renegades" uses the explicit alias -> "chaos renegade" roster (id="1050157")
        let team = make_team_from_roster("renegades", "home", "bb2025")
            .expect("renegades must resolve to Chaos Renegade roster via alias");
        assert_eq!(team.players.len(), 11);
        // Renegade Rat Ogre is highest-cost qty=1 position -> jersey 1, ag=4
        let j1 = &team.players[0];
        assert_eq!(j1.agility, 4, "Renegade Rat Ogre (jersey 1) must have ag=4");
    }

    #[test]
    fn single_word_races_still_resolve() {
        for race in ["amazon", "chaos", "dwarf", "goblin", "nurgle", "norse"] {
            make_team_from_roster(race, "home", "bb2025")
                .unwrap_or_else(|e| panic!("{race} must resolve without fallback: {e}"));
        }
    }
}
