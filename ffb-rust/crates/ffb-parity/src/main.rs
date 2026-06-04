mod runner;
mod log_format;
mod comparator;
mod update_progress;
mod network_test;
mod state_hash;
mod coverage_report;

#[allow(dead_code)] mod debug_rng;

/// Parsed CLI arguments for the parity runner.
struct ParityArgs {
    network: bool,
    home: String,
    home_java: String,
    away: String,
    away_java: String,
    edition: String,
    seed_start: u64,
    seed_end: u64,
    no_abort: bool,
    verbose: bool,
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
        let mut no_abort = false;
        let mut verbose = false;

        let mut i = 0;
        while i < raw.len() {
            match raw[i].as_str() {
                "--network" => network = true,
                "--no-abort" => no_abort = true,
                "--verbose" => verbose = true,
                "--home" if i + 1 < raw.len() => { home = raw[i + 1].clone(); i += 1; }
                "--away" if i + 1 < raw.len() => { away = raw[i + 1].clone(); i += 1; }
                "--edition" if i + 1 < raw.len() => { edition = raw[i + 1].clone(); i += 1; }
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

        let home_java = runner::java_team_id(&home, "home");
        let away_java = runner::java_team_id(&away, "away");

        ParityArgs { network, home, home_java, away, away_java, edition, seed_start, seed_end, no_abort, verbose }
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

    let total = args.seed_end - args.seed_start + 1;
    let mut passed = 0u64;
    let mut failed = 0u64;
    let mut coverage = coverage_report::CoverageReport::default();
    coverage.matchups.push(coverage_report::MatchupSummary {
        home: args.home.clone(),
        away: args.away.clone(),
        seeds: total as u32,
        home_wins: 0, away_wins: 0, draws: 0,
        touchdowns_home: 0, touchdowns_away: 0,
    });

    for seed in args.seed_start..=args.seed_end {
        println!("Seed {seed}: {} vs {} ({})", args.home, args.away, args.edition);

        runner::run_java_headless(seed, &args.home_java, &args.away_java, &args.home, &args.away);
        let (_, events, home_score, away_score) = runner::run_rust_headless(seed, &args.home, &args.away, &args.edition, args.verbose);
        let result = comparator::compare_logs(seed, &args.home, &args.away);
        update_progress::update(seed, &args.home, &args.away, &result);

        // Accumulate coverage from this parity run
        for ev in &events { coverage.tally(ev); }
        coverage.games += 1;
        coverage.touchdowns_home += home_score as u32;
        coverage.touchdowns_away += away_score as u32;
        if let Some(m) = coverage.matchups.last_mut() {
            m.touchdowns_home += home_score as u32;
            m.touchdowns_away += away_score as u32;
            if home_score > away_score { coverage.home_wins += 1; m.home_wins += 1; }
            else if away_score > home_score { coverage.away_wins += 1; m.away_wins += 1; }
            else { coverage.draws += 1; m.draws += 1; }
        }

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
                eprintln!("→ Enter TDD loop: write Java test, Rust test, fix, restart from seed 1");
                std::process::exit(1);
            }
        }
    }

    if failed == 0 {
        println!("PARITY: {passed}/{total} games match.");
    } else {
        eprintln!("PARITY: {passed}/{total} passed, {failed} FAILED.");
        std::process::exit(1);
    }

    coverage.skill_names = coverage_report::build_skill_names();
    let json = serde_json::to_string(&coverage).expect("coverage serialization failed");
    let html = coverage_report::generate_html(&json);
    std::fs::write("coverage.html", &html).expect("failed to write coverage.html");
    println!("Coverage report written to coverage.html ({} games)", coverage.games);
}

/// Diagnostic: run seed=1 (BB2025) and print state string at each step boundary.
/// Compare against Java parity log to identify divergence source.
#[cfg(test)]
mod diagnostic {
    use ffb_engine::engine::GameEngine;
    use ffb_engine::agent::response_to_action_pub;
    use ffb_engine::legal_actions::TeamSide;
    use ffb_model::enums::Rules;
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::prompts::{AgentPrompt, AgentResponse};
    use ffb_model::util::state_string;
    use ffb_model::types::FieldCoordinate;
    use rand_xoshiro::Xoshiro256StarStar;
    use rand_core::{RngCore, SeedableRng};

    fn make_team(side: &str) -> Team {
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
            used_skills: Default::default(),
            niggling_injuries: 0,
            stat_injuries: vec![],
            current_spps: 0,
            career_spps: 0,
            race: None,
        }).collect();
        Team {
            id: format!("{side}_lineman"),
            name: format!("{side} Linemen"),
            race: "lineman".into(),
            roster_id: "lineman".into(),
            coach: format!("Coach_{side}"),
            rerolls: 3, apothecaries: 1, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 5, dedicated_fans: 5,
            team_value: 1_000_000, treasury: 0,
            special_rules: vec![],
            players,
        }
    }

    struct SimpleAgent {
        rng: Xoshiro256StarStar,
    }

    impl SimpleAgent {
        fn new(seed: u64) -> Self {
            SimpleAgent { rng: Xoshiro256StarStar::seed_from_u64(seed ^ 0xDEAD_BEEF_CAFE_0001) }
        }

        fn respond(&mut self, prompt: &AgentPrompt, active_side: TeamSide) -> AgentResponse {
            match prompt {
                AgentPrompt::CoinChoice { .. } =>
                    AgentResponse::CoinChoice { heads: self.rng.next_u64() % 2 == 0 },
                AgentPrompt::ReceiveChoice { .. } =>
                    AgentResponse::ReceiveChoice { receive: self.rng.next_u64() % 2 == 0 },
                AgentPrompt::KickBall => {
                    let x_raw = (self.rng.next_u64() % 13) as i32;
                    let y_raw = (self.rng.next_u64() % 13) as i32;
                    let x = if active_side == TeamSide::Home { x_raw + 13 } else { x_raw };
                    let y = y_raw + 1;
                    AgentResponse::KickBall { coord: FieldCoordinate::new(x, y) }
                }
                AgentPrompt::TeamSetup { .. } =>
                    AgentResponse::TeamSetup { placements: vec![] },
                AgentPrompt::Touchback { eligible_players } => {
                    let pid = eligible_players.iter()
                        .min_by_key(|(_, c)| {
                            let dx = c.x as i32 - 13;
                            let dy = c.y as i32 - 8;
                            dx * dx + dy * dy
                        })
                        .map(|(id, _)| id.clone())
                        .unwrap_or_default();
                    AgentResponse::Touchback { player_id: pid }
                }
                _ => AgentResponse::Confirm,
            }
        }
    }

    #[test]
    fn print_step1_state_string_seed2() {
        let seed = 2u64;
        let home = make_team("home");
        let away = make_team("away");
        let mut engine = GameEngine::new(home, away, Rules::Bb2025, seed);
        let mut agent = SimpleAgent::new(seed);
        let mut step_count = 0;
        for _ in 0..50_000 {
            if engine.is_finished() { break; }
            let prompt = match engine.current_prompt() { Some(p) => p.clone(), None => break };
            let side = engine.active_side();
            let is_turn_boundary = matches!(prompt, AgentPrompt::ActivatePlayer { .. });
            if is_turn_boundary {
                let pre = state_string(&engine.game);
                let turn = if engine.game.home_playing { engine.game.turn_data_home.turn_nr } else { engine.game.turn_data_away.turn_nr };
                let active = if engine.game.home_playing { "home" } else { "away" };
                println!("PRE STEP seed2 (half={} turn={turn} active={active}): {pre}", engine.game.half);
            }
            let response = agent.respond(&prompt, side);
            let action = response_to_action_pub(response, Some(&prompt));
            if engine.apply(side, action).is_err() { break; }
            if is_turn_boundary {
                step_count += 1;
                if step_count >= 2 { break; }
            }
        }
    }

    #[test]
    fn print_step1_state_string_seed1() {
        let seed = 1u64;
        let home = make_team("home");
        let away = make_team("away");
        let mut engine = GameEngine::new(home, away, Rules::Bb2025, seed);
        let mut agent = SimpleAgent::new(seed);

        let initial = state_string(&engine.game);
        println!("INITIAL: {initial}");

        let mut step_count = 0;
        for _ in 0..50_000 {
            if engine.is_finished() { break; }
            let prompt = match engine.current_prompt() {
                Some(p) => p.clone(),
                None => break,
            };
            let side = engine.active_side();
            let is_turn_boundary = matches!(prompt, AgentPrompt::ActivatePlayer { .. });
            if is_turn_boundary {
                // Print state string BEFORE applying (matches runner.rs pre_hash)
                let pre = state_string(&engine.game);
                let turn = if engine.game.home_playing {
                    engine.game.turn_data_home.turn_nr
                } else {
                    engine.game.turn_data_away.turn_nr
                };
                let active = if engine.game.home_playing { "home" } else { "away" };
                println!("PRE STEP (half={} turn={turn} active={active}): {pre}", engine.game.half);
            }
            let response = agent.respond(&prompt, side);
            let action = response_to_action_pub(response, Some(&prompt));
            if engine.apply(side, action).is_err() { break; }

            if is_turn_boundary {
                step_count += 1;
                let s = state_string(&engine.game);
                let turn = if engine.game.home_playing {
                    engine.game.turn_data_home.turn_nr
                } else {
                    engine.game.turn_data_away.turn_nr
                };
                let active = if engine.game.home_playing { "home" } else { "away" };
                println!("STEP {step_count} (half={} turn={turn} active={active}): {s}",
                    engine.game.half);
                if step_count >= 3 { break; } // only first 3 steps for diagnosis
            }
        }
    }
}

#[cfg(test)]
mod rng_test {
    use rand_xoshiro::Xoshiro256StarStar;
    use rand_core::{RngCore, SeedableRng};
    #[test]
    fn print_seed1_decisions() {
        let seed: u64 = 1;
        let mut rng = Xoshiro256StarStar::seed_from_u64(seed ^ 0xDEAD_BEEF_CAFE_0001);
        for i in 1..=5 {
            let v = rng.next_u64();
            println!("call{i}={:#018x} mod2={}", v, v % 2);
        }
    }
}

#[cfg(test)]
mod rng_test2 {
    use rand_xoshiro::Xoshiro256StarStar;
    use rand_core::{RngCore, SeedableRng};
    #[test]
    fn print_dice_rng_seed1() {
        let seed: u64 = 1;
        // Dice RNG: seeded with plain seed
        let mut dice = Xoshiro256StarStar::seed_from_u64(seed);
        // Decision RNG: seeded with seed ^ constant  
        let mut dec = Xoshiro256StarStar::seed_from_u64(seed ^ 0xDEAD_BEEF_CAFE_0001);
        
        // Java throwCoin() = getDieRoll(2) using dice RNG
        // getDieRoll uses rejection sampling, but for sides=2 threshold is huge
        // so it's just: v = nextLong(); result = v % 2 + 1  
        let coin_v = dice.next_u64();
        let coin_result = coin_v % 2 + 1; // 1 or 2
        println!("dice_rng call1 = {:#018x}, coin_result={} (1=heads)", coin_v, coin_result);
        
        // Decision: home guesses heads = next_u64() % 2 == 0
        let guess_v = dec.next_u64();
        let home_guesses_heads = guess_v % 2 == 0;
        println!("dec_rng call1 = {:#018x}, home_guesses_heads={}", guess_v, home_guesses_heads);
        
        let coin_is_heads = coin_result == 1;
        let home_wins = home_guesses_heads == coin_is_heads;
        println!("coin_is_heads={}, home_wins={}", coin_is_heads, home_wins);
        
        // Decision: receive choice  
        let recv_v = dec.next_u64();
        let receive = recv_v % 2 == 0;
        println!("dec_rng call2 = {:#018x}, receive={}", recv_v, receive);
        
        // Determine who plays first:
        // If home won: receive=true → home receives → home plays first
        //              receive=false → away receives → away plays first
        // If away won: receive=true → away receives → away plays first  
        //              receive=false → home receives → home plays first
        let home_plays_first = if home_wins { receive } else { !receive };
        println!("home_plays_first={}", home_plays_first);
    }
}

#[cfg(test)]
mod game_rng_test {
    use ffb_model::util::GameRng;
    #[test]
    fn print_game_rng_seed1_dice_sequence() {
        let mut rng = GameRng::new(1);
        // Print first 30 dice rolls to trace through H1 and H2 kickoffs
        for i in 1..=30 {
            let d6 = rng.d6();
            println!("game_rng d6 call{i} = {d6}");
        }
    }

    #[test]
    fn print_game_rng_seed1_h2_trace() {
        use ffb_model::enums::Direction;
        use ffb_mechanics::mechanics::scatter_coordinate;

        let mut rng = GameRng::new(1);
        // H1 start: spectators (d3x2) + weather (d6x2)
        let s1 = rng.d3(); let s2 = rng.d3(); let w1 = rng.d6(); let w2 = rng.d6();
        println!("H1 start: spec={s1},{s2} weather={w1},{w2}");
        // Coin flip
        let coin = rng.bool();
        println!("Coin: is_heads={coin}");
        // H1 scatter
        let sd = rng.d8(); let sdist = rng.d6();
        println!("H1 scatter: dir(d8)={sd} dist(d6)={sdist}");
        // H1 kickoff event
        let ke1 = rng.d6(); let ke2 = rng.d6();
        println!("H1 kickoff event: {ke1}+{ke2}={}", ke1+ke2);
        // H1 kickoff event extra dice (CheeringFans=6 or BrilliantCoaching=7 add 2 extra d6)
        let event_h1 = ke1 + ke2;
        if event_h1 == 6 || event_h1 == 7 {
            let ex1 = rng.d6(); let ex2 = rng.d6();
            println!("H1 kickoff extra: {ex1},{ex2}");
        }
        // H2 start: spectators (d3x2) + weather (d6x2)
        let s1h2 = rng.d3(); let s2h2 = rng.d3(); let w1h2 = rng.d6(); let w2h2 = rng.d6();
        println!("H2 start: spec={s1h2},{s2h2} weather={w1h2},{w2h2}");
        // H2 scatter
        let sd2 = rng.d8(); let sdist2 = rng.d6();
        println!("H2 scatter: dir(d8)={sd2} dist(d6)={sdist2}");
        // H2 kickoff event
        let ke1h2 = rng.d6(); let ke2h2 = rng.d6();
        println!("H2 kickoff event: {ke1h2}+{ke2h2}={}", ke1h2+ke2h2);
        if ke1h2 + ke2h2 == 6 || ke1h2 + ke2h2 == 7 {
            let ex1 = rng.d6(); let ex2 = rng.d6();
            println!("H2 kickoff extra: {ex1},{ex2}");
        }

        // Compute H1 scatter from (21,9) home kicks:
        if let Some(dir1) = Direction::for_roll(sd) {
            let (nx, ny) = scatter_coordinate(21, 9, dir1, sdist);
            println!("H1 ball scatter: dir={dir1:?} dist={sdist} → ({nx},{ny})");
        }
        // Compute H2 scatter from (2,7) away kicks (c5%13=2, c6%13=6, y=7):
        // dec RNG: c5=2/13=0, c6=? Let's also compute from decision rng
        use rand_core::{RngCore, SeedableRng};
        let mut dec = rand_xoshiro::Xoshiro256StarStar::seed_from_u64(1u64 ^ 0xDEAD_BEEF_CAFE_0001);
        let c1 = dec.next_u64(); let c2 = dec.next_u64();
        let c3 = dec.next_u64(); let c4 = dec.next_u64();
        let c5 = dec.next_u64(); let c6 = dec.next_u64();
        let h2_x = (c5 % 13) as i32;
        let h2_y = (c6 % 13) as i32 + 1;
        println!("H2 decision RNG: c5={c5} x_raw={h2_x} c6={c6} y_raw={}", c6%13);
        println!("H2 kick coord (away kicks): x={h2_x} y={h2_y}");
        if let Some(dir2) = Direction::for_roll(sd2) {
            let (nx, ny) = scatter_coordinate(h2_x, h2_y, dir2, sdist2);
            println!("H2 ball scatter end: dir={dir2:?} dist={sdist2} → ({nx},{ny})");
            // lastValid
            let mut last_valid = (h2_x, h2_y);
            for step in 1..=sdist2 {
                let (tx, ty) = scatter_coordinate(h2_x, h2_y, dir2, step);
                if tx >= 0 && tx <= 25 && ty >= 0 && ty <= 14 {
                    last_valid = (tx, ty);
                } else {
                    break;
                }
            }
            println!("H2 lastValid: ({},{})", last_valid.0, last_valid.1);
            let off_pitch = nx < 0 || nx > 25 || ny < 0 || ny > 14;
            let in_half_home = nx >= 0 && nx <= 12 && ny >= 0 && ny <= 14;
            println!("H2 off_pitch={off_pitch} in_half_home={in_half_home} touchback_java={}", !in_half_home || off_pitch);
        }
    }

    #[test]
    #[test]
    fn trace_seed28_goblin_kickoff_dice() {
        // Goblin seed 28: H1 away kicks to (5,3). DICE_TRACE shows scatter NW dist=1 → (4,2), then bounce N → (4,1).
        // Pitch Invasion (event=12) with stun_count=2 (d3=2).
        // Java: Ball and Chain player stunned → 2 extra injury dice at pos 14,15 before second stun.
        let mut rng = GameRng::new(28);
        // Pre-game: d3+d3+d6+d6+bool
        let f_h = rng.d3(); let f_a = rng.d3();
        let w1 = rng.d6(); let w2 = rng.d6();
        let coin = rng.bool();
        println!("Pre-game: fan_h={f_h} fan_a={f_a} weather={}+{}={} coin={coin}", w1, w2, w1+w2);
        // H1 scatter
        let sd = rng.d8(); let dist = rng.d6();
        println!("H1 scatter: d8={sd} dist={dist}");
        // H1 event roll
        let e1 = rng.d6(); let e2 = rng.d6();
        println!("H1 event: {}+{}={}", e1, e2, e1+e2);
        // Pitch Invasion fan rolls + stun
        let pi_h = rng.d6(); let pi_a = rng.d6();
        let stun = rng.d3();
        println!("PI: home_roll={pi_h} away_roll={pi_a} stun_count={stun}");
        // First stun player selection
        let s1 = rng.die(11);
        println!("Stun1: die(11)={s1} → index {}", s1-1);
        // Java has 2 Ball and Chain dice here, Rust skips them
        // If we DON'T skip: pos 13=s1, pos 14=stun2, pos 15=CSTI_bounce
        let s2_no_bac = rng.die(10);
        let csti_no_bac = rng.d8();
        println!("Without BaC: stun2=die(10)={s2_no_bac} CSTI=d8={csti_no_bac}");
        // Now print what Java would use (skipping the d6+d6 BaC injury)
        println!("Java stun2 at pos 16 would be a different value than at pos 13");
    }

    #[test]
    fn dump_renegades_team_jerseys() {
        let team = crate::runner::make_team_from_roster("renegades", "away", "bb2025").unwrap();
        for p in &team.players {
            println!("jersey {} ag={} name={}", p.nr, p.agility, p.name);
        }
    }

    fn trace_seed67_norse_kickoff_dice() {
        // Norse seed 67: H1 away kicks to (0,5). H2 home kicks to (15,6).
        // Pre-game dice: d3(fan_h)+d3(fan_a)+d6(weather)+d6(weather)+bool(coin) = 5 dice
        // H1 kickoff (Rust order: scatter before event):
        //   scatter(d8,d6)+event(2d6)+CSTI_bounce(d8) = 5 dice
        // H2 kickoff: scatter(d8,d6)+event(2d6)+BrilliantCoaching(2d6)+CSTI_catch(d6) = 7 dice
        let mut rng = GameRng::new(67);
        // Pre-game
        let fan_h = rng.d3(); let fan_a = rng.d3();
        let w1 = rng.d6(); let w2 = rng.d6();
        let coin = rng.bool();
        println!("Pre-game: fan_h={fan_h} fan_a={fan_a} weather={}+{}={} coin={coin}", w1, w2, w1+w2);
        // H1 kickoff: scatter(d8,d6)+event_roll(2d6)+CheeringFans_extra(2d6)+CSTI_bounce(d8) = 7 dice
        let h1_sd = rng.d8();
        let h1_dist = rng.d6();
        let h1_ev1 = rng.d6();
        let h1_ev2 = rng.d6();
        // Cheering Fans (sum=4): 2 extra dice
        let cf_home = rng.d6(); let cf_away = rng.d6();
        let h1_csti_bounce = rng.d8();
        println!("H1: scatter_dir={h1_sd} dist={h1_dist} event={}+{}={} cf_h={cf_home} cf_a={cf_away} bounce={h1_csti_bounce}",
            h1_ev1, h1_ev2, h1_ev1+h1_ev2);
        // H2 kickoff: scatter(d8,d6)+event_roll(2d6)+BrilliantCoaching(2d6)+CSTI_catch(d6)+CSTI_bounce(d8)
        let h2_sd = rng.d8();
        let h2_dist = rng.d6();
        let h2_ev1 = rng.d6();
        let h2_ev2 = rng.d6();
        let h2_coach_h = rng.d6();
        let h2_coach_a = rng.d6();
        let h2_catch = rng.d6();
        let h2_bounce = rng.d8();
        println!("H2: scatter_dir={h2_sd} dist={h2_dist} event={}+{}={} coach_h={h2_coach_h} coach_a={h2_coach_a}",
            h2_ev1, h2_ev2, h2_ev1+h2_ev2);
        println!("H2 CSTI: catch_roll={h2_catch} (if bounces: bounce_dir={h2_bounce})");
        println!("Expected scatter W(7) dist=2, event=Brilliant(7)");
    }
}

#[cfg(test)]
mod game_rng_d8_test {
    use ffb_model::util::GameRng;
    #[test]
    fn print_game_rng_seed1_kickoff_scatter_sequence() {
        let mut rng = GameRng::new(1);
        // Simulate: kickoff_roll (2xd6) + BrilliantCoaching (2xd6) + scatter (d8 + d6)
        let k1 = rng.d6();
        let k2 = rng.d6();
        println!("kickoff roll: {k1} + {k2} = {}", k1+k2);
        let h = rng.d6();
        let a = rng.d6();
        println!("BrilliantCoaching: home={h} away={a}");
        let sdir = rng.d8();
        let sdist = rng.d6();
        println!("scatter: d8={sdir} d6={sdist}");
    }
}

#[cfg(test)]
mod ball_position_diagnostic {
    use rand_xoshiro::Xoshiro256StarStar;
    use rand_core::{RngCore, SeedableRng};
    use ffb_model::util::GameRng;
    use ffb_model::enums::Direction;
    use ffb_mechanics::mechanics::scatter_coordinate;

    #[test]
    fn trace_seed1_ball_position() {
        let seed: u64 = 1;
        let mut dice = GameRng::new(seed);
        let mut dec = Xoshiro256StarStar::seed_from_u64(seed ^ 0xDEAD_BEEF_CAFE_0001);

        let coin_bool = dice.bool();
        println!("Dice[1] coin_is_heads={}", coin_bool);

        let guess_v = dec.next_u64();
        let home_guesses_heads = guess_v % 2 == 0;
        println!("Dec[1]={:018x} home_guesses_heads={}", guess_v, home_guesses_heads);

        let recv_v = dec.next_u64();
        let receive = recv_v % 2 == 0;
        println!("Dec[2]={:018x} receive={}", recv_v, receive);

        let home_wins = home_guesses_heads == coin_bool;
        let home_first_offense = if home_wins { receive } else { !receive };
        let home_kicks = !home_first_offense;
        println!("home_wins={} receive={} home_first_offense={} home_kicks={}", home_wins, receive, home_first_offense, home_kicks);

        let x_raw_v = dec.next_u64();
        let x_raw = (x_raw_v % 13) as i32;
        let y_raw_v = dec.next_u64();
        let y_raw = (y_raw_v % 13) as i32;
        let x_kick = if home_kicks { x_raw + 13 } else { x_raw };
        let y_kick = y_raw + 1;
        println!("Dec[3]={:018x} x_raw={}", x_raw_v, x_raw);
        println!("Dec[4]={:018x} y_raw={}", y_raw_v, y_raw);
        println!("Kick coord: ({}, {})", x_kick, y_kick);

        let scatter_dir = dice.d8();
        let scatter_dist = dice.d6();
        println!("Dice[2] scatter_dir(d8)={}", scatter_dir);
        println!("Dice[3] scatter_dist(d6)={}", scatter_dist);

        let kof1 = dice.d6();
        let kof2 = dice.d6();
        println!("Dice[4,5] kickoff_event={}+{}={}", kof1, kof2, kof1 + kof2);

        if let Some(dir) = Direction::for_roll(scatter_dir) {
            let (nx, ny) = scatter_coordinate(x_kick, y_kick, dir, scatter_dist);
            println!("Scatter dir={:?} dist={}", dir, scatter_dist);
            println!("Ball lands at: ({}, {})", nx, ny);
            let on_pitch = nx >= 0 && nx <= 25 && ny >= 0 && ny <= 14;
            let in_wrong_half = if home_first_offense {
                nx >= 13
            } else {
                nx < 13
            };
            println!("on_pitch={} in_wrong_half={} touchback={}", on_pitch, in_wrong_half, !on_pitch || in_wrong_half);
        } else {
            println!("Invalid scatter direction: {}", scatter_dir);
        }
    }
}

#[cfg(test)]
mod parity_seed1_check {
    use ffb_engine::agent::response_to_action_pub;
    use ffb_engine::engine::GameEngine;
    use ffb_engine::legal_actions::TeamSide;
    use ffb_model::enums::{Rules, PlayerType, PlayerGender};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;
    use ffb_model::prompts::{AgentPrompt, AgentResponse};
    use ffb_model::util::{state_string, state_hash};
    use ffb_model::types::FieldCoordinate;
    use rand_xoshiro::Xoshiro256StarStar;
    use rand_core::{RngCore, SeedableRng};

    fn make_team(side: &str) -> Team {
        let players: Vec<Player> = (1..=11).map(|nr| Player {
            id: format!("{side}_{nr:02}"),
            name: format!("{side} Player {nr}"),
            nr,
            position_id: "lineman".to_string(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
        }).collect();
        Team {
            id: format!("{side}_lineman"),
            name: format!("{side} Linemen"),
            race: "lineman".into(), roster_id: "lineman".into(),
            coach: format!("Coach_{side}"),
            rerolls: 3, apothecaries: 1, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0,
            team_value: 1_000_000, treasury: 0, special_rules: vec![], players,
        }
    }

    struct ParityAgent { rng: Xoshiro256StarStar }
    impl ParityAgent {
        fn new(seed: u64) -> Self { ParityAgent { rng: Xoshiro256StarStar::seed_from_u64(seed ^ 0xDEAD_BEEF_CAFE_0001) } }
        fn respond(&mut self, prompt: &AgentPrompt, active_side: TeamSide) -> AgentResponse {
            match prompt {
                AgentPrompt::CoinChoice { .. } => AgentResponse::CoinChoice { heads: self.rng.next_u64() % 2 == 0 },
                AgentPrompt::ReceiveChoice { .. } => AgentResponse::ReceiveChoice { receive: self.rng.next_u64() % 2 == 0 },
                AgentPrompt::KickBall => {
                    let x_raw = (self.rng.next_u64() % 13) as i32;
                    let y_raw = (self.rng.next_u64() % 13) as i32;
                    let x = if active_side == TeamSide::Home { x_raw + 13 } else { x_raw };
                    AgentResponse::KickBall { coord: FieldCoordinate::new(x, y_raw + 1) }
                }
                AgentPrompt::TeamSetup { .. } => AgentResponse::TeamSetup { placements: vec![] },
                AgentPrompt::Touchback { eligible_players } => {
                    let pid = eligible_players.iter()
                        .min_by_key(|(_, c)| { let dx = c.x as i32 - 13; let dy = c.y as i32 - 8; dx*dx + dy*dy })
                        .map(|(id, _)| id.clone()).unwrap_or_default();
                    AgentResponse::Touchback { player_id: pid }
                }
                _ => AgentResponse::Confirm,
            }
        }
    }

    #[test]
    fn print_rust_seed1_full_state_string() {
        let seed = 1u64;
        let home = make_team("home");
        let away = make_team("away");
        let mut engine = GameEngine::new(home, away, Rules::Bb2025, seed);
        let mut agent = ParityAgent::new(seed);

        println!("INITIAL hash={}", state_hash(&engine.game));

        let mut steps = 0u32;
        for _ in 0..100_000 {
            if engine.is_finished() { break; }
            let prompt = match engine.current_prompt() { Some(p) => p.clone(), None => break };
            let side = engine.active_side();
            let is_boundary = matches!(prompt, AgentPrompt::ActivatePlayer { .. });
            let turn_nr = if engine.game.home_playing { engine.game.turn_data_home.turn_nr } else { engine.game.turn_data_away.turn_nr };
            if is_boundary && turn_nr >= 1 {
                let h = state_hash(&engine.game);
                let ss = state_string(&engine.game);
                let active = if engine.game.home_playing { "home" } else { "away" };
                steps += 1;
                println!("STEP{steps} half={} turn={turn_nr} active={active} hash={h}", engine.game.half);
                if steps == 17 { println!("  state={ss}"); }
            }
            let response = agent.respond(&prompt, side);
            let action = response_to_action_pub(response, Some(&prompt));
            if engine.apply(side, action).is_err() { break; }
        }
        let end_hash = state_hash(&engine.game);
        let end_state = ffb_model::util::state_string(&engine.game);
        println!("GAME END hash={end_hash} steps={steps}");
        println!("GAME END state={end_state}");
        // Compute alternative state strings to identify what Java's game_end state is:
        let player_suffix = &end_state[end_state.find(" b").unwrap()..];
        let target = "d9ac6d7bb0e9faf7";
        fn fnv(s: &str) -> String {
            let mut h: u64 = 0xcbf29ce484222325;
            for &b in s.as_bytes() { h ^= b as u64; h = h.wrapping_mul(1_099_511_628_211); }
            format!("{h:016x}")
        }
        for variant in &["h2t88aaway", "h2t89aaway", "h2t88ahome", "h2t98ahome", "h2t99ahome", "h2t89ahome"] {
            let s = format!("{}s0,0{}", variant, player_suffix);
            let h = fnv(&s);
            println!("  variant={variant} hash={h} match={}", h == target);
        }
        // Check ball_in_play=false variant
        let player_suffix_false = player_suffix.replace("b12,8,true", "b12,8,false");
        for variant in &["h2t88aaway", "h2t88ahome"] {
            let s = format!("{}s0,0{}", variant, player_suffix_false);
            let h = fnv(&s);
            println!("  variant={variant}+ball_false hash={h} match={}", h == target);
        }
        // Check ball at (-1,-1) false (ball removed at game end?)
        let no_ball_suffix = player_suffix.replace("b12,8,true", "b-1,-1,false");
        for variant in &["h2t88aaway", "h2t88ahome"] {
            let s = format!("{}s0,0{}", variant, no_ball_suffix);
            let h = fnv(&s);
            println!("  variant={variant}+no_ball hash={h} match={}", h == target);
        }
        println!("Expected: 32 steps, end hash=d9ac6d7bb0e9faf7");
        assert_eq!(steps, 32, "Expected 32 step boundaries (16 per half)");
    }
}

#[cfg(test)]
mod game_rng_seed2_test {
    use ffb_model::util::GameRng;
    #[test]
    fn print_game_rng_seed2_kickoff_sequence() {
        let mut rng = GameRng::new(2);
        let fan_home = rng.d3();
        let fan_away = rng.d3();
        let w1 = rng.d6(); let w2 = rng.d6();
        println!("fan_home={fan_home} fan_away={fan_away} weather={w1}+{w2}={}", w1+w2);
        let coin = rng.bool();
        println!("coin={coin}");
        let ko1 = rng.d6(); let ko2 = rng.d6();
        println!("kickoff_roll={ko1}+{ko2}={}", ko1+ko2);
        // Print next 8 dice to see what kickoff result consumes
        for i in 1..=8 {
            let d = rng.d6();
            println!("extra_d6[{i}]={d}");
        }
        println!("--- scatter candidates ---");
        for i in 1..=4 {
            let d8 = rng.d8();
            let d6 = rng.d6();
            println!("scatter[{i}]: d8={d8} d6={d6}");
        }
    }
}

#[cfg(test)]
mod game_rng_seed2_actual_sequence {
    use ffb_model::util::GameRng;
    #[test]
    fn trace_seed2_actual_kickball_order() {
        let mut rng = GameRng::new(2);
        // Pre-kickoff: spectators + weather + coin
        let fan_home = rng.d3(); let fan_away = rng.d3();
        let w1 = rng.d6(); let w2 = rng.d6();
        let coin = rng.bool();
        println!("setup: fan_home={fan_home} fan_away={fan_away} weather={w1}+{w2} coin={coin}");

        // KickBall action in Rust: scatter (d8+d6) BEFORE kickoff roll (2d6)
        let scatter_d8 = rng.d8();
        let scatter_d6 = rng.d6();
        let kick_r1 = rng.d6(); let kick_r2 = rng.d6();
        let kickoff_total = kick_r1 + kick_r2;
        println!("scatter d8={scatter_d8} d6={scatter_d6}  kickoff_roll={kick_r1}+{kick_r2}={kickoff_total}");

        // CheeringFans (roll=6): 2 extra d6
        let cf_home = rng.d6(); let cf_away = rng.d6();
        println!("CheeringFans home={cf_home} away={cf_away}");

        // bounce if needed
        let bounce_d8 = rng.d8();
        println!("bounce_d8={bounce_d8}");
    }
}

#[cfg(test)]
mod seed1_full_trace {
    use ffb_model::util::GameRng;
    use ffb_model::enums::Direction;
    use ffb_mechanics::mechanics::scatter_coordinate;
    #[test]
    fn trace_seed1_kickoff_full_dice() {
        let mut rng = GameRng::new(1);
        let fan_home = rng.d3(); let fan_away = rng.d3();
        let w1 = rng.d6(); let w2 = rng.d6();
        let coin = rng.bool();
        println!("fan_home={fan_home} fan_away={fan_away} weather={w1}+{w2} coin={coin}");
        // scatter first
        let scatter_d8 = rng.d8();
        let scatter_d6 = rng.d6();
        let k1 = rng.d6(); let k2 = rng.d6();
        println!("scatter d8={scatter_d8} d6={scatter_d6}  kickoff={k1}+{k2}={}", k1+k2);
        let e1 = rng.d6(); let e2 = rng.d6();
        println!("event extra d6: {e1} {e2}");
        let bounce = rng.d8();
        println!("bounce d8={bounce}");
        // Java kick coord: home kicks to away half, x=xRaw+13, y=yRaw+1
        // Decision RNG: seed ^ constant, call3=xRaw, call4=yRaw
        // From test output: x_raw=8, y_raw=8 → kick=(21,9)
        let kick_x = 21i32; let kick_y = 9i32;
        if let Some(dir) = Direction::for_roll(scatter_d8) {
            let (nx, ny) = scatter_coordinate(kick_x, kick_y, dir, scatter_d6);
            println!("scatter dir={:?} dist={scatter_d6} from ({kick_x},{kick_y}) to ({nx},{ny})", dir);
            if let Some(bd) = Direction::for_roll(bounce) {
                let (bx, by) = scatter_coordinate(nx, ny, bd, 1);
                println!("bounce dir={bd:?} from ({nx},{ny}) to ({bx},{by})");
            }
        }
    }
}

#[cfg(test)]
mod seed1_h2_trace {
    use ffb_model::util::GameRng;
    use ffb_model::enums::Direction;
    use ffb_mechanics::mechanics::scatter_coordinate;

    /// Traces all dice + decisions from game-start through H2 kickoff for seed 1.
    #[test]
    fn trace_seed1_full_game_dice() {
        use rand_xoshiro::Xoshiro256StarStar;
        use rand_core::{RngCore, SeedableRng};
        let seed: u64 = 1;
        let mut dec = Xoshiro256StarStar::seed_from_u64(seed ^ 0xDEAD_BEEF_CAFE_0001);
        let c1 = dec.next_u64(); let c2 = dec.next_u64();
        let c3 = dec.next_u64(); let c4 = dec.next_u64();
        // H1: home kicks (home_first_offense=false → home_playing=!false=true at kickoff)
        // Wait: home_first_offense = if home_wins { receive } else { !receive }
        let home_wins_coin = (c1 % 2 == 0); // CoinChoice: heads = true
        let wants_receive = c2 % 2 == 0;     // ReceiveChoice: receive = true
        // home_playing after ReceiveChoice = !home_receives (kicker goes first in setup)
        // home_first_offense = if home_wins_coin { wants_receive } else { !wants_receive }
        let home_first_offense = if home_wins_coin { wants_receive } else { !wants_receive };
        let h1_home_kicks = !home_first_offense;
        let h1_x_raw = (c3 % 13) as i32;
        let h1_y_raw = (c4 % 13) as i32;
        let h1_x = if h1_home_kicks { h1_x_raw + 13 } else { h1_x_raw };
        let h1_y = h1_y_raw + 1;
        println!("Dec c1={c1:#018x} coin_heads={home_wins_coin}");
        println!("Dec c2={c2:#018x} receive={wants_receive}");
        println!("home_first_offense={home_first_offense} h1_home_kicks={h1_home_kicks}");
        println!("H1 kick: x_raw={h1_x_raw} y_raw={h1_y_raw} → ({h1_x},{h1_y})");

        // H2: home_first_offense flipped → h2_home_first_offense = !home_first_offense
        let h2_home_first_offense = !home_first_offense;
        let h2_home_kicks = !h2_home_first_offense;
        let c5 = dec.next_u64(); let c6 = dec.next_u64();
        let h2_x_raw = (c5 % 13) as i32;
        let h2_y_raw = (c6 % 13) as i32;
        let h2_x = if h2_home_kicks { h2_x_raw + 13 } else { h2_x_raw };
        let h2_y = h2_y_raw + 1;
        println!("h2_home_first_offense={h2_home_first_offense} h2_home_kicks={h2_home_kicks}");
        println!("Dec c5={c5:#018x} h2_x_raw={h2_x_raw}");
        println!("Dec c6={c6:#018x} h2_y_raw={h2_y_raw}");
        println!("H2 kick coord: ({h2_x},{h2_y})");
    }

    #[test]
    fn trace_seed1_h2_ball_from_dice() {
        let mut rng = GameRng::new(1);

        // H1 pre-kickoff
        let fan_home = rng.d3(); let fan_away = rng.d3();
        let w1 = rng.d6(); let w2 = rng.d6();
        let coin = rng.bool();
        println!("H1 pre: fan_home={fan_home} fan_away={fan_away} weather={w1}+{w2}={} coin={coin}", w1+w2);

        // H1 kickoff scatter (first in Rust order)
        let h1_scatter_d8 = rng.d8();
        let h1_scatter_d6 = rng.d6();
        println!("H1 scatter: d8={h1_scatter_d8} d6={h1_scatter_d6}");

        // H1 kickoff event
        let h1_k1 = rng.d6(); let h1_k2 = rng.d6();
        let h1_kof = h1_k1 + h1_k2;
        println!("H1 kickoff event roll: {h1_k1}+{h1_k2}={h1_kof} (BB2025: 6=CheeringFans)");

        // CheeringFans: 2 extra d6
        let h1_cf_home = rng.d6(); let h1_cf_away = rng.d6();
        println!("H1 CheeringFans: home={h1_cf_home} away={h1_cf_away}");

        // H1 bounce
        let h1_bounce = rng.d8();
        println!("H1 bounce d8={h1_bounce}");

        // ── Half-time ────────────────────────────────────────────────────────
        // (16 turns pass with no dice)

        // H2 spectators + weather
        let h2_fan_home = rng.d3(); let h2_fan_away = rng.d3();
        let h2_w1 = rng.d6(); let h2_w2 = rng.d6();
        println!("H2 pre: fan_home={h2_fan_home} fan_away={h2_fan_away} weather={h2_w1}+{h2_w2}={}", h2_w1+h2_w2);

        // H2 kickoff scatter (home kicks in H2 for seed 1? or away?)
        // home_first_offense was false in H1 (away received), so H2 home_first_offense = true (home receives)
        // H2 kicker = away (away kicks to home's half, x_raw → x = x_raw, y = y_raw+1)
        // Decision RNG call5 (0-indexed): x_raw, call6: y_raw
        // From debug_rng: the decision rng produces specific values for these calls

        let h2_scatter_d8 = rng.d8();
        let h2_scatter_d6 = rng.d6();
        println!("H2 scatter: d8={h2_scatter_d8} d6={h2_scatter_d6}");

        // H2 kickoff event
        let h2_k1 = rng.d6(); let h2_k2 = rng.d6();
        let h2_kof = h2_k1 + h2_k2;
        println!("H2 kickoff event roll: {h2_k1}+{h2_k2}={h2_kof}");

        // Print what the H2 event is and any extra dice based on the roll
        // Just print 4 extra d6 so we can see them
        let extra1 = rng.d6(); let extra2 = rng.d6();
        let extra3 = rng.d6(); let extra4 = rng.d6();
        println!("H2 extra dice (if event): d6={extra1} {extra2} {extra3} {extra4}");
        let bounce_d8 = rng.d8();
        println!("H2 bounce d8={bounce_d8}");
    }
}

#[cfg(test)]
mod seed3_debug {
    use ffb_engine::engine::GameEngine;
    use ffb_engine::agent::response_to_action_pub;
    use ffb_engine::legal_actions::TeamSide;
    use ffb_model::enums::{Rules, PlayerType, PlayerGender};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;
    use ffb_model::prompts::{AgentPrompt, AgentResponse};
    use ffb_model::util::state_string;
    use ffb_model::types::FieldCoordinate;
    use rand_xoshiro::Xoshiro256StarStar;
    use rand_core::{RngCore, SeedableRng};

    fn make_team(side: &str, roster_id: &str) -> Team {
        let players: Vec<Player> = (1..=11).map(|nr| Player {
            id: format!("{side}_{nr:02}"),
            name: format!("{side} Player {nr}"),
            nr,
            position_id: "lineman".to_string(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
        }).collect();
        Team {
            id: format!("{side}_{roster_id}"),
            name: format!("{} ({})", side, roster_id),
            race: roster_id.to_string(), roster_id: roster_id.to_string(),
            coach: format!("Coach_{side}"),
            rerolls: 3, apothecaries: 1, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0,
            team_value: 1_000_000, treasury: 0, special_rules: vec![], players,
        }
    }

    struct ParityAgent3 { rng: Xoshiro256StarStar }
    impl ParityAgent3 {
        fn new(seed: u64) -> Self { ParityAgent3 { rng: Xoshiro256StarStar::seed_from_u64(seed ^ 0xDEAD_BEEF_CAFE_0001) } }
        fn respond(&mut self, prompt: &AgentPrompt, side: TeamSide) -> AgentResponse {
            match prompt {
                AgentPrompt::CoinChoice { .. } => AgentResponse::CoinChoice { heads: self.rng.next_u64() % 2 == 0 },
                AgentPrompt::ReceiveChoice { .. } => AgentResponse::ReceiveChoice { receive: self.rng.next_u64() % 2 == 0 },
                AgentPrompt::KickBall => {
                    let x_raw = (self.rng.next_u64() % 13) as i32;
                    let y_raw = (self.rng.next_u64() % 13) as i32;
                    let x = if side == TeamSide::Home { x_raw + 13 } else { x_raw };
                    AgentResponse::KickBall { coord: FieldCoordinate::new(x, y_raw + 1) }
                }
                AgentPrompt::TeamSetup { .. } => AgentResponse::TeamSetup { placements: vec![] },
                AgentPrompt::Touchback { eligible_players } => {
                    let pid = eligible_players.iter()
                        .min_by_key(|(_, c)| { let dx = c.x as i32 - 13; let dy = c.y as i32 - 8; dx*dx + dy*dy })
                        .map(|(id, _)| id.clone()).unwrap_or_default();
                    AgentResponse::Touchback { player_id: pid }
                }
                _ => AgentResponse::Confirm,
            }
        }
    }

    #[test]
    fn trace_seed18_dice() {
        use ffb_model::util::GameRng;
        use ffb_model::enums::Direction;
        use ffb_mechanics::mechanics::scatter_coordinate;
        let seed = 18u64;
        let mut dice = GameRng::new(seed);
        let mut dec = rand_xoshiro::Xoshiro256StarStar::seed_from_u64(seed ^ 0xDEAD_BEEF_CAFE_0001);

        let fh = dice.d3(); let fa = dice.d3();
        let w1 = dice.d6(); let w2 = dice.d6();
        let coin = dice.bool();
        let d1 = dec.next_u64(); let d2 = dec.next_u64();
        let home_wins = (d1 % 2 == 0) == coin;
        let receives = d2 % 2 == 0;
        let home_first_offense = if home_wins { receives } else { !receives };
        let home_kicks_h1 = !home_first_offense;
        println!("H1 pre: fan={fh},{fa} weather={w1}+{w2} coin={coin} home_kicks={home_kicks_h1}");

        let d3 = dec.next_u64(); let d4 = dec.next_u64();
        let h1_xr = (d3 % 13) as i32;
        let h1_yr = (d4 % 13) as i32;
        let h1_x = if home_kicks_h1 { h1_xr + 13 } else { h1_xr };
        let h1_y = h1_yr + 1;
        println!("H1 kick: ({h1_x},{h1_y})");

        let h1_sd = dice.d8(); let h1_sdist = dice.d6();
        let h1_k1 = dice.d6(); let h1_k2 = dice.d6();
        let h1_kof = h1_k1 + h1_k2;
        println!("H1 scatter d8={h1_sd} d6={h1_sdist}  kickoff={h1_k1}+{h1_k2}={h1_kof}");

        // Consume extra dice for known events
        if h1_kof == 6 || h1_kof == 7 { let e1 = dice.d6(); let e2 = dice.d6(); println!("H1 BrilliantCoaching/CheeringFans extra: {e1},{e2}"); }
        else if h1_kof == 8 { // WeatherChange
            let ww1 = dice.d6(); let ww2 = dice.d6();
            let new_weather = ww1 + ww2;
            println!("H1 WeatherChange: {ww1}+{ww2}={new_weather}");
            // Nice weather = 4-10: 3 extra d8 scatter
            if new_weather >= 4 && new_weather <= 10 {
                let bd1 = dice.d8(); let bd2 = dice.d8(); let bd3 = dice.d8();
                println!("H1 WeatherChange NICE scatter d8x3: {bd1},{bd2},{bd3}");
            }
        }
        else if h1_kof == 9 { let qs = dice.d3(); println!("H1 QuickSnap d3={qs}"); }
        else if h1_kof == 12 { // PitchInvasion
            let pi1 = dice.d6(); let pi2 = dice.d6(); let pi3 = dice.d3();
            println!("H1 PitchInvasion: {pi1},{pi2},{pi3}");
        }

        if let Some(dir1) = Direction::for_roll(h1_sd) {
            let mut lv = (h1_x, h1_y); let mut end = (h1_x, h1_y);
            for step in 1..=h1_sdist {
                let (nx, ny) = scatter_coordinate(h1_x, h1_y, dir1, step);
                end = (nx, ny);
                if nx >= 0 && nx <= 25 && ny >= 0 && ny <= 14 { lv = (nx, ny); } else { break; }
            }
            let y_ok = end.1 >= 0 && end.1 <= 14;
            let in_recv = y_ok && if home_first_offense { end.0 >= 0 && end.0 < 13 } else { end.0 >= 13 && end.0 <= 25 };
            let tb = !in_recv;
            println!("H1 ball: final={lv:?} touchback={tb}");
            if !tb { let h1_b = dice.d8(); println!("H1 bounce d8={h1_b}"); }
            if tb { let _dt = dec.next_u64(); }
        }

        println!("--- H2 ---");
        let h2_home_first = !home_first_offense;
        let h2_home_kicks = !h2_home_first;
        let d5 = dec.next_u64(); let d6v = dec.next_u64();
        let h2_xr = (d5 % 13) as i32; let h2_yr = (d6v % 13) as i32;
        let h2_x = if h2_home_kicks { h2_xr + 13 } else { h2_xr };
        let h2_y = h2_yr + 1;
        println!("H2 kick: ({h2_x},{h2_y}) home_kicks={h2_home_kicks}");

        let h2_sd = dice.d8(); let h2_sdist = dice.d6();
        let h2_k1 = dice.d6(); let h2_k2 = dice.d6();
        let h2_kof = h2_k1 + h2_k2;
        println!("H2 scatter d8={h2_sd} d6={h2_sdist}  kickoff={h2_k1}+{h2_k2}={h2_kof}");
        println!("H2 kickoff event check: roll={h2_kof}");
    }

    #[test]
    fn trace_seed3_dice() {
        use ffb_model::util::GameRng;
        use ffb_model::enums::Direction;
        use ffb_mechanics::mechanics::scatter_coordinate;
        let seed = 3u64;
        let mut dice = GameRng::new(seed);
        let mut dec = rand_xoshiro::Xoshiro256StarStar::seed_from_u64(seed ^ 0xDEAD_BEEF_CAFE_0001);

        let fh = dice.d3(); let fa = dice.d3();
        let w1 = dice.d6(); let w2 = dice.d6();
        let coin = dice.bool();
        let d1 = dec.next_u64(); let d2 = dec.next_u64();
        let home_wins = (d1 % 2 == 0) == coin;
        let receives = d2 % 2 == 0;
        let home_first_offense = if home_wins { receives } else { !receives };
        let home_kicks_h1 = !home_first_offense;
        println!("H1 pre: fan={fh},{fa} weather={w1}+{w2} coin={coin} home_kicks={home_kicks_h1}");

        let d3 = dec.next_u64(); let d4 = dec.next_u64();
        let h1_xr = (d3 % 13) as i32;
        let h1_yr = (d4 % 13) as i32;
        let h1_x = if home_kicks_h1 { h1_xr + 13 } else { h1_xr };
        let h1_y = h1_yr + 1;
        println!("H1 kick: ({h1_x},{h1_y}) xr={h1_xr} yr={h1_yr}");

        let h1_sd = dice.d8(); let h1_sdist = dice.d6();
        let h1_k1 = dice.d6(); let h1_k2 = dice.d6();
        let h1_kof = h1_k1 + h1_k2;
        println!("H1 scatter d8={h1_sd} d6={h1_sdist}  kickoff={h1_k1}+{h1_k2}={h1_kof}");

        if h1_kof == 6 || h1_kof == 7 || h1_kof == 8 {
            let ex1 = dice.d6(); let ex2 = dice.d6();
            println!("H1 event extra d6x2: {ex1} {ex2}");
        } else if h1_kof == 9 {
            // BB2025 QuickSnap: APPLY_KICKOFF_RESULT rolls rollDice(3) = d3 for nrOfPlayersAllowed
            let qs_roll = dice.d3();
            println!("H1 QuickSnap extra d3={qs_roll} (allowed={} players)", qs_roll + 3);
        }

        if let Some(dir1) = Direction::for_roll(h1_sd) {
            let mut lv = (h1_x, h1_y); let mut end = (h1_x, h1_y);
            for step in 1..=h1_sdist {
                let (nx, ny) = scatter_coordinate(h1_x, h1_y, dir1, step);
                end = (nx, ny);
                if nx >= 0 && nx <= 25 && ny >= 0 && ny <= 14 { lv = (nx, ny); } else { break; }
            }
            let y_ok = end.1 >= 0 && end.1 <= 14;
            let in_recv = y_ok && if home_first_offense { end.0 >= 0 && end.0 < 13 } else { end.0 >= 13 && end.0 <= 25 };
            let tb = !in_recv;
            println!("H1 ball: final={lv:?} end={end:?} dir={dir1:?} touchback={tb}");
            if !tb {
                let h1_b = dice.d8();
                if let Some(bd) = Direction::for_roll(h1_b) {
                    let (bx,by) = scatter_coordinate(lv.0,lv.1,bd,1);
                    println!("H1 bounce d8={h1_b} dir={bd:?} →({bx},{by})");
                }
            }
            if tb { let _dt = dec.next_u64(); println!("H1 touchback: consumed 1 dec call"); }
        }

        println!("--- H2 ---");
        let h2_home_first = !home_first_offense;
        let h2_home_kicks = !h2_home_first;
        let d5 = dec.next_u64(); let d6v = dec.next_u64();
        let h2_xr = (d5 % 13) as i32;
        let h2_yr = (d6v % 13) as i32;
        let h2_x = if h2_home_kicks { h2_xr + 13 } else { h2_xr };
        let h2_y = h2_yr + 1;
        println!("H2 kick: ({h2_x},{h2_y}) xr={h2_xr} yr={h2_yr} home_kicks={h2_home_kicks}");

        let h2_sd = dice.d8(); let h2_sdist = dice.d6();
        let h2_k1 = dice.d6(); let h2_k2 = dice.d6();
        let h2_kof = h2_k1 + h2_k2;
        println!("H2 scatter d8={h2_sd} d6={h2_sdist}  kickoff={h2_k1}+{h2_k2}={h2_kof}");

        if h2_kof == 6 || h2_kof == 7 || h2_kof == 8 {
            let ex1 = dice.d6(); let ex2 = dice.d6();
            println!("H2 event extra: {ex1} {ex2}");
        }

        if let Some(dir2) = Direction::for_roll(h2_sd) {
            let mut lv = (h2_x, h2_y); let mut end = (h2_x, h2_y);
            for step in 1..=h2_sdist {
                let (nx, ny) = scatter_coordinate(h2_x, h2_y, dir2, step);
                end = (nx, ny);
                if nx >= 0 && nx <= 25 && ny >= 0 && ny <= 14 { lv = (nx, ny); } else { break; }
            }
            let y_ok = end.1 >= 0 && end.1 <= 14;
            let in_recv = y_ok && if h2_home_first { end.0 >= 0 && end.0 < 13 } else { end.0 >= 13 && end.0 <= 25 };
            let tb = !in_recv;
            println!("H2 ball: final={lv:?} end={end:?} dir={dir2:?} touchback={tb}");
            if !tb {
                let h2_b = dice.d8();
                if let Some(bd) = Direction::for_roll(h2_b) {
                    let (bx,by) = scatter_coordinate(lv.0,lv.1,bd,1);
                    println!("H2 bounce d8={h2_b} dir={bd:?} →({bx},{by})");
                }
            }
        }
    }

    #[test]
    fn trace_seed3_h2() {
        let seed = 3u64;
        let home = make_team("home", "teamLinemanParityHome");
        let away = make_team("away", "teamLinemanParityAway");
        let mut engine = GameEngine::new(home, away, Rules::Bb2025, seed);
        let mut agent = ParityAgent3::new(seed);

        let mut step = 0u32;
        for _ in 0..100_000 {
            if engine.is_finished() { break; }
            let prompt = match engine.current_prompt() { Some(p) => p.clone(), None => break };
            let side = engine.active_side();
            let is_boundary = matches!(prompt, AgentPrompt::ActivatePlayer { .. });
            let turn_nr = if engine.game.home_playing { engine.game.turn_data_home.turn_nr } else { engine.game.turn_data_away.turn_nr };
            if is_boundary && turn_nr >= 1 {
                step += 1;
                if step == 17 {
                    println!("STEP17 state={}", state_string(&engine.game));
                }
                if step > 17 { break; }
            }
            let response = agent.respond(&prompt, side);
            let action = response_to_action_pub(response, Some(&prompt));
            if engine.apply(side, action).is_err() { break; }
        }
    }
}

#[cfg(test)]
mod seed57_engine_trace {
    use ffb_engine::engine::GameEngine;
    use ffb_engine::agent::response_to_action_pub;
    use ffb_engine::legal_actions::TeamSide;
    use ffb_model::enums::{Rules, PlayerType, PlayerGender};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;
    use ffb_model::prompts::{AgentPrompt, AgentResponse};
    use ffb_model::util::state_string;
    use ffb_model::types::FieldCoordinate;
    use rand_xoshiro::Xoshiro256StarStar;
    use rand_core::{RngCore, SeedableRng};

    fn make_team(side: &str, roster_id: &str) -> Team {
        let players: Vec<Player> = (1..=11).map(|nr| Player {
            id: format!("{side}_{nr:02}"),
            name: format!("{side} Player {nr}"),
            nr,
            position_id: "lineman".to_string(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
        }).collect();
        Team {
            id: format!("{side}_{roster_id}"),
            name: format!("{} ({})", side, roster_id),
            race: roster_id.to_string(), roster_id: roster_id.to_string(),
            coach: format!("Coach_{side}"),
            rerolls: 3, apothecaries: 1, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0,
            team_value: 1_000_000, treasury: 0, special_rules: vec![], players,
        }
    }

    struct ParityAgent57 { rng: Xoshiro256StarStar }
    impl ParityAgent57 {
        fn new(seed: u64) -> Self { ParityAgent57 { rng: Xoshiro256StarStar::seed_from_u64(seed ^ 0xDEAD_BEEF_CAFE_0001) } }
        fn respond(&mut self, prompt: &AgentPrompt, side: TeamSide) -> AgentResponse {
            match prompt {
                AgentPrompt::CoinChoice { .. } => AgentResponse::CoinChoice { heads: self.rng.next_u64() % 2 == 0 },
                AgentPrompt::ReceiveChoice { .. } => AgentResponse::ReceiveChoice { receive: self.rng.next_u64() % 2 == 0 },
                AgentPrompt::KickBall => {
                    let x_raw = (self.rng.next_u64() % 13) as i32;
                    let y_raw = (self.rng.next_u64() % 13) as i32;
                    let x = if side == TeamSide::Home { x_raw + 13 } else { x_raw };
                    AgentResponse::KickBall { coord: FieldCoordinate::new(x, y_raw + 1) }
                }
                AgentPrompt::TeamSetup { .. } => AgentResponse::TeamSetup { placements: vec![] },
                AgentPrompt::Touchback { eligible_players } => {
                    let pid = eligible_players.iter()
                        .min_by_key(|(_, c)| { let dx = c.x as i32 - 13; let dy = c.y as i32 - 8; dx*dx + dy*dy })
                        .map(|(id, _)| id.clone()).unwrap_or_default();
                    AgentResponse::Touchback { player_id: pid }
                }
                AgentPrompt::PlayerChoice { .. } => AgentResponse::PlayerChoice { player_id: String::new() },
                AgentPrompt::KickoffReturn { .. } => AgentResponse::PlayerChoice { player_id: String::new() },
                _ => AgentResponse::Confirm,
            }
        }
    }

    #[test]
    fn trace_seed57_state_at_h2t1() {
        let seed = 57u64;
        let home = make_team("home", "teamLinemanParityHome");
        let away = make_team("away", "teamLinemanParityAway");
        let mut engine = GameEngine::new(home, away, Rules::Bb2025, seed);
        let mut agent = ParityAgent57::new(seed);

        let mut step = 0u32;
        for _ in 0..100_000 {
            if engine.is_finished() { break; }
            let prompt = match engine.current_prompt() { Some(p) => p.clone(), None => break };
            let side = engine.active_side();
            let is_boundary = matches!(prompt, AgentPrompt::ActivatePlayer { .. });
            let turn_nr = if engine.game.home_playing { engine.game.turn_data_home.turn_nr } else { engine.game.turn_data_away.turn_nr };
            if is_boundary && turn_nr >= 1 {
                step += 1;
                let ss = state_string(&engine.game);
                let hash = ffb_model::util::state_hash(&engine.game);
                let active = if engine.game.home_playing { "home" } else { "away" };
                println!("STEP{step:02} half={} turn={turn_nr} active={active} hash={hash}", engine.game.half);
                if step >= 16 && step <= 18 {
                    println!("  state={ss}");
                }
                if step >= 20 { break; }
            }
            let response = agent.respond(&prompt, side);
            let action = response_to_action_pub(response, Some(&prompt));
            if engine.apply(side, action).is_err() { break; }
        }
    }
}

#[cfg(test)]
mod seed57_rng_trace {
    use ffb_model::util::rng::GameRng;

    #[test]
    fn trace_seed57_dice_sequence() {
        let mut rng = GameRng::new(57);
        // H1 game start: 2 fan rolls (d3 each)
        let fan_home = rng.d3();
        let fan_away = rng.d3();
        println!("fans: home={fan_home} away={fan_away}");
        // H1 kickoff scatter
        let sc_dir = rng.d8();
        let sc_dist = rng.d6();
        println!("H1 scatter: dir={sc_dir} dist={sc_dist}");
        // H1 kickoff event 2d6
        let ev1 = rng.d6();
        let ev2 = rng.d6();
        println!("H1 event: {ev1}+{ev2}={}", ev1+ev2);
        // H1 Charge d3
        let charge_d3 = rng.d3();
        println!("H1 charge d3: {charge_d3}");
        // H1 CSTI bounce d8
        let csti = rng.d8();
        println!("H1 CSTI: {csti}");
        // H2 kickoff scatter
        let h2_dir = rng.d8();
        let h2_dist = rng.d6();
        let h2_ev1 = rng.d6();
        let h2_ev2 = rng.d6();
        println!("H2 scatter: dir={h2_dir} dist={h2_dist} event={}+{}={}", h2_ev1, h2_ev2, h2_ev1+h2_ev2);
    }
}

#[cfg(test)]
mod seed57_full_trace {
    use ffb_model::util::rng::GameRng;

    #[test]
    fn trace_seed57_full_sequence() {
        let mut rng = GameRng::new(57);
        // Game start (CoinChoice):
        let fan_home = rng.d3();
        let fan_away = rng.d3();
        let weather1 = rng.d6();
        let weather2 = rng.d6();
        let coin = rng.die(2); // bool = range(2)
        println!("fans: home={fan_home} away={fan_away}  weather: {weather1}+{weather2}={}  coin: {coin}", weather1+weather2);
        // H1 kickoff scatter:
        let h1_dir = rng.d8();
        let h1_dist = rng.d6();
        println!("H1 scatter: dir={h1_dir} dist={h1_dist}");
        // H1 kickoff event 2d6:
        let ev1 = rng.d6();
        let ev2 = rng.d6();
        println!("H1 event: {ev1}+{ev2}={}", ev1+ev2);
        // H1 Charge d3 (kickoff_roll=10):
        let charge_d3 = rng.d3();
        println!("H1 charge d3: {charge_d3}");
        // H1 CSTI bounce d8:
        let csti = rng.d8();
        println!("H1 CSTI: {csti}");
        // H2 kickoff scatter:
        let h2_dir = rng.d8();
        let h2_dist = rng.d6();
        let h2_ev1 = rng.d6();
        let h2_ev2 = rng.d6();
        println!("H2 scatter: dir={h2_dir} dist={h2_dist}  event: {h2_ev1}+{h2_ev2}={}", h2_ev1+h2_ev2);
    }
}

#[cfg(test)]
mod seed57_coin_start {
    use ffb_model::util::rng::GameRng;

    #[test]
    fn trace_if_java_starts_from_coin() {
        let mut rng = GameRng::new(57);
        // Hypothesis: Java starts seeded RNG from coin flip, not fans/weather
        let coin = rng.die(2);
        println!("coin: {coin}");
        // Then H1 scatter (no fans/weather dice consumed):
        let h1_dir = rng.d8();
        let h1_dist = rng.d6();
        println!("H1 scatter (if no fans/weather): dir={h1_dir} dist={h1_dist}");
        // But H1 should give scatter SW 3 → ball at (22,13)
        // With only coin before scatter: dir=? dist=?

        // Let me also test hypothesis: Java starts from H1 scatter only (no coin/fans/weather)
        let mut rng2 = GameRng::new(57);
        let h1_dir2 = rng2.d8();
        let h1_dist2 = rng2.d6();
        println!("H1 scatter (if no preamble): dir={h1_dir2} dist={h1_dist2}");
    }
}

#[cfg(test)]
mod seed57_decision_rng_trace {
    use rand_xoshiro::Xoshiro256StarStar;
    use rand_core::{RngCore, SeedableRng};

    #[test]
    fn trace_decision_rng_calls() {
        let seed = 57u64;
        let mut rng = Xoshiro256StarStar::seed_from_u64(seed ^ 0xDEAD_BEEF_CAFE_0001);
        for i in 1..=8 {
            let raw = rng.next_u64();
            let mod13 = (raw % 13) as i32;
            let mod2 = (raw % 2) as i32;
            println!("call {i}: raw=0x{raw:016x} mod2={mod2} mod13={mod13} as_kick_x_home={} as_kick_x_away={mod13} as_kick_y={}", mod13 + 13, mod13 + 1);
        }
    }
}

#[cfg(test)]
mod seed69_trace {
    use ffb_model::util::rng::GameRng;
    use ffb_model::enums::Weather;

    #[test]
    fn trace_seed69_rng_sequence() {
        // Java debug output confirms: JAVA_KICK half=1 home=false xRaw=11 yRaw=6 x=11 y=7 rng_adv=4
        //                             JAVA_KICK half=2 home=true xRaw=4  yRaw=4 x=17 y=5 rng_adv=6
        // Decision RNG advances: coin=1, receive=1, H1kick(2), H2kick(2) → 6 total

        // Test with coin as game RNG die(2) call (position 5 in sequence)
        println!("=== WITH coin as game RNG ===");
        {
            let mut rng = GameRng::new(69);
            let fan_home = rng.d3();
            let fan_away = rng.d3();
            let w1 = rng.d6(); let w2 = rng.d6();
            let weather = Weather::for_roll(w1 + w2);
            let coin = rng.die(2); // coin in game RNG?
            println!("fans={fan_home},{fan_away} weather={w1}+{w2}={} ({weather:?}) coin={coin}", w1+w2);
            let h1_dir = rng.d8(); let h1_dist = rng.d6();
            let ev1 = rng.d6(); let ev2 = rng.d6();
            println!("H1 scatter dir={h1_dir} dist={h1_dist}  event={ev1}+{ev2}={}=WeatherChange?", ev1+ev2);
            let new_w1 = rng.d6(); let new_w2 = rng.d6();
            let new_w = Weather::for_roll(new_w1 + new_w2);
            println!("  new_weather={new_w1}+{new_w2}={} ({new_w:?})", new_w1+new_w2);
            if new_w == Weather::Nice {
                let s1=rng.d8(); let s2=rng.d8(); let s3=rng.d8();
                println!("  Nice scatter: d8s={s1},{s2},{s3}");
            }
            let h1_csti = rng.d8();
            println!("H1 CSTI bounce d8={h1_csti}");
            let h2_dir = rng.d8(); let h2_dist = rng.d6();
            let h2_ev1 = rng.d6(); let h2_ev2 = rng.d6();
            println!("H2 scatter dir={h2_dir} dist={h2_dist}  event={h2_ev1}+{h2_ev2}={}=Charge?", h2_ev1+h2_ev2);
            let charge_d3 = rng.d3(); // Charge event rolls D3 for player count
            println!("  Charge D3={charge_d3}");
            let h2_csti = rng.d8();
            println!("H2 CSTI bounce d8={h2_csti}");
        }

        // Test without coin as game RNG
        println!("=== WITHOUT coin as game RNG ===");
        {
            let mut rng = GameRng::new(69);
            let fan_home = rng.d3(); let fan_away = rng.d3();
            let w1 = rng.d6(); let w2 = rng.d6();
            let weather = Weather::for_roll(w1 + w2);
            println!("fans={fan_home},{fan_away} weather={w1}+{w2}={} ({weather:?})", w1+w2);
            let h1_dir = rng.d8(); let h1_dist = rng.d6();
            let ev1 = rng.d6(); let ev2 = rng.d6();
            println!("H1 scatter dir={h1_dir} dist={h1_dist}  event={ev1}+{ev2}={}=WeatherChange?", ev1+ev2);
            let new_w1 = rng.d6(); let new_w2 = rng.d6();
            let new_w = Weather::for_roll(new_w1 + new_w2);
            println!("  new_weather={new_w1}+{new_w2}={} ({new_w:?})", new_w1+new_w2);
            if new_w == Weather::Nice {
                let s1=rng.d8(); let s2=rng.d8(); let s3=rng.d8();
                println!("  Nice scatter: d8s={s1},{s2},{s3}");
            }
            let h1_csti = rng.d8();
            println!("H1 CSTI bounce d8={h1_csti}");
            let h2_dir = rng.d8(); let h2_dist = rng.d6();
            let h2_ev1 = rng.d6(); let h2_ev2 = rng.d6();
            println!("H2 scatter dir={h2_dir} dist={h2_dist}  event={h2_ev1}+{h2_ev2}={}=Charge?", h2_ev1+h2_ev2);
            let charge_d3 = rng.d3();
            println!("  Charge D3={charge_d3}");
            let h2_csti = rng.d8();
            println!("H2 CSTI bounce d8={h2_csti}");
        }
    }
}

#[cfg(test)]
mod seed69_engine_trace {
    use ffb_engine::engine::GameEngine;
    use ffb_engine::agent::response_to_action_pub;
    use ffb_engine::legal_actions::TeamSide;
    use ffb_model::enums::{Rules, PlayerType, PlayerGender};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;
    use ffb_model::prompts::{AgentPrompt, AgentResponse};
    use ffb_model::util::state_string;
    use ffb_model::types::FieldCoordinate;
    use rand_xoshiro::Xoshiro256StarStar;
    use rand_core::{RngCore, SeedableRng};

    fn make_team(side: &str, roster_id: &str) -> Team {
        let players: Vec<Player> = (1..=11).map(|nr| Player {
            id: format!("{side}_{nr:02}"),
            name: format!("{side} Player {nr}"),
            nr,
            position_id: "lineman".to_string(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
        }).collect();
        Team {
            id: format!("{side}_{roster_id}"),
            name: format!("{} ({})", side, roster_id),
            race: roster_id.to_string(), roster_id: roster_id.to_string(),
            coach: format!("Coach_{side}"),
            rerolls: 3, apothecaries: 1, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0,
            team_value: 1_000_000, treasury: 0, special_rules: vec![], players,
        }
    }

    struct ParityAgent { rng: Xoshiro256StarStar }
    impl ParityAgent {
        fn new(seed: u64) -> Self { ParityAgent { rng: Xoshiro256StarStar::seed_from_u64(seed ^ 0xDEAD_BEEF_CAFE_0001) } }
        fn respond(&mut self, prompt: &AgentPrompt, side: TeamSide) -> AgentResponse {
            match prompt {
                AgentPrompt::CoinChoice { .. } => AgentResponse::CoinChoice { heads: self.rng.next_u64() % 2 == 0 },
                AgentPrompt::ReceiveChoice { .. } => AgentResponse::ReceiveChoice { receive: self.rng.next_u64() % 2 == 0 },
                AgentPrompt::KickBall => {
                    let x_raw = (self.rng.next_u64() % 13) as i32;
                    let y_raw = (self.rng.next_u64() % 13) as i32;
                    let x = if side == TeamSide::Home { x_raw + 13 } else { x_raw };
                    AgentResponse::KickBall { coord: FieldCoordinate::new(x, y_raw + 1) }
                }
                AgentPrompt::TeamSetup { .. } => AgentResponse::TeamSetup { placements: vec![] },
                AgentPrompt::Touchback { eligible_players } => {
                    let pid = eligible_players.iter()
                        .min_by_key(|(_, c)| { let dx = c.x as i32 - 13; let dy = c.y as i32 - 8; dx*dx + dy*dy })
                        .map(|(id, _)| id.clone()).unwrap_or_default();
                    AgentResponse::Touchback { player_id: pid }
                }
                AgentPrompt::PlayerChoice { .. } => AgentResponse::PlayerChoice { player_id: String::new() },
                AgentPrompt::KickoffReturn { .. } => AgentResponse::PlayerChoice { player_id: String::new() },
                _ => AgentResponse::Confirm,
            }
        }
    }

    #[test]
    fn trace_seed69_state_at_h2t1() {
        let seed = 69u64;
        let home = make_team("home", "teamLinemanParityHome");
        let away = make_team("away", "teamLinemanParityAway");
        let mut engine = GameEngine::new(home, away, Rules::Bb2025, seed);
        let mut agent = ParityAgent::new(seed);

        let mut step = 0u32;
        for _ in 0..100_000 {
            if engine.is_finished() { break; }
            let prompt = match engine.current_prompt() { Some(p) => p.clone(), None => break };
            let side = engine.active_side();
            let is_boundary = matches!(prompt, AgentPrompt::ActivatePlayer { .. });
            let turn_nr = if engine.game.home_playing { engine.game.turn_data_home.turn_nr } else { engine.game.turn_data_away.turn_nr };
            if is_boundary && turn_nr >= 1 {
                step += 1;
                let ss = state_string(&engine.game);
                let hash = ffb_model::util::state_hash(&engine.game);
                let active = if engine.game.home_playing { "home" } else { "away" };
                eprintln!("STEP{step:02} half={} turn={turn_nr} active={active} hash={hash}", engine.game.half);
                if step >= 16 && step <= 18 {
                    eprintln!("  state={ss}");
                }
                if step >= 20 { break; }
            }
            let response = agent.respond(&prompt, side);
            let action = response_to_action_pub(response, Some(&prompt));
            if engine.apply(side, action).is_err() { break; }
        }
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
