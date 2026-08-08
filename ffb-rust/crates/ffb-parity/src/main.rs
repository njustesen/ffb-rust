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
        let mut tier = 2u8;

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

        ParityArgs { network, coverage, uniform, all_rosters, all_editions, home, home_java, away, away_java, edition, seed_start, seed_end, no_abort, verbose, visualize, tier }
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
        let dir = format!("parity/{}_{}_vs_{}", args.edition, args.home, args.away);
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
    if args.coverage {
        println!("Coverage run: {} vs {} ({}) — {} seeds", args.home, args.away, args.edition, total);
        let mut cov = coverage_report::CoverageReport::default();
        cov.matchups.push(coverage_report::MatchupSummary {
            home: args.home.clone(),
            away: args.away.clone(),
            seeds: total as u32,
            home_wins: 0, away_wins: 0, draws: 0,
            touchdowns_home: 0, touchdowns_away: 0,
        });
        for seed in args.seed_start..=args.seed_end {
            let (events, home_score, away_score) =
                runner::run_coverage_game(seed, &args.home, &args.away, &args.edition);
            for ev in &events { cov.tally(ev); }
            cov.games += 1;
            cov.touchdowns_home += home_score as u32;
            cov.touchdowns_away += away_score as u32;
            if let Some(m) = cov.matchups.last_mut() {
                m.touchdowns_home += home_score as u32;
                m.touchdowns_away += away_score as u32;
                if home_score > away_score { cov.home_wins += 1; m.home_wins += 1; }
                else if away_score > home_score { cov.away_wins += 1; m.away_wins += 1; }
                else { cov.draws += 1; m.draws += 1; }
            }
            println!("  seed {seed} done ({home_score}-{away_score})");
        }
        cov.skill_names = coverage_report::build_skill_names();
        let json = serde_json::to_string(&cov).expect("coverage serialization failed");
        let html = coverage_report::generate_html(&json);
        std::fs::write("coverage.html", &html).expect("failed to write coverage.html");
        println!("Coverage report written to coverage.html ({} games)", cov.games);
        return;
    }

    // ── Parity mode (default) ────────────────────────────────────────────────────
    let mut passed = 0u64;
    let mut failed = 0u64;
    // Tier 3: aggregate the Rust engine's GameEvents across all seeds for the
    // coverage checklist (the events are a faithful proxy for both engines once
    // the per-activation hashes match).
    let mut t3_cov = (args.tier >= 3).then(coverage_report::CoverageReport::default);

    // Amortize JVM start-up + fat-jar class-loading + server construction (perf reasons #1/#2/#4):
    // run EVERY Java game in ONE JVM via ParityRunner's batch mode, instead of a fresh JVM per seed.
    // Then run the Rust engine per seed. Report per-engine wall-clock at the end.
    let java_t0 = std::time::Instant::now();
    runner::run_java_headless_range(args.seed_start, args.seed_end, &args.home_java, &args.away_java, &args.home, &args.away, args.tier, &args.edition);
    let java_total = java_t0.elapsed();
    println!("TIMING java_total={:.3}s (batched JVM, {total} seeds)", java_total.as_secs_f64());
    let mut rust_total = std::time::Duration::ZERO;

    for seed in args.seed_start..=args.seed_end {
        println!("Seed {seed}: {} vs {} ({})", args.home, args.away, args.edition);

        let rust_t0 = std::time::Instant::now();
        let (_, events, _home_score, _away_score) = runner::run_rust_headless(seed, &args.home, &args.away, &args.edition, args.verbose, args.tier);
        rust_total += rust_t0.elapsed();
        if let Some(cov) = t3_cov.as_mut() {
            for ev in &events { cov.tally(ev); }
            cov.games += 1;
        }
        let result = comparator::compare_logs(seed, &args.home, &args.away);
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

    if failed == 0 && checklist_ok {
        println!("PARITY: {passed}/{total} games match.");
    } else if failed == 0 {
        eprintln!("PARITY: {passed}/{total} games match, but required coverage items are MISSING.");
        std::process::exit(1);
    } else {
        eprintln!("PARITY: {passed}/{total} passed, {failed} FAILED.");
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
