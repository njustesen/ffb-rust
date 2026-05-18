/// Parity runner: runs a single seeded game and emits a full JSONL decision log.
///
/// Usage (parity mode):
///   parity_runner parity <home_race> <away_race> <seed> [output.jsonl]
///
/// Usage (legacy multi-game mode):
///   parity_runner <n_games> <seed_start>
///
/// In parity mode the output is one JSON object per line:
///   {"i":0,"type":"game_start",...}
///   {"i":1,"type":"step",...}
///   ...
///   {"i":N,"type":"game_end",...}
use std::io::{BufWriter, Write};
use std::sync::Arc;

use ffb_core::model::game_state::GameState;
use ffb_core::rng::GameRng;
use ffb_core::types::TeamId;
use ffb_sim::canonical_strategy::CanonicalStrategy;
use ffb_sim::parity_log::{
    canonical_string, finalize_steps, state_hash, LoggingStrategy, ParityGameEnd, ParityGameStart,
};
use ffb_sim::roster::make_team;
use ffb_sim::simulation::SimulationLoop;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.get(1).map(String::as_str) == Some("parity") {
        run_parity_mode(&args);
    } else {
        run_legacy_mode(&args);
    }
}

fn run_parity_mode(args: &[String]) {
    if args.len() < 5 {
        eprintln!("Usage: parity_runner parity <home_race> <away_race> <seed> [output.jsonl]");
        std::process::exit(1);
    }

    let home_race = &args[2];
    let away_race = &args[3];
    let seed: u64 = args[4].parse().expect("seed must be a u64");
    let output_path = args.get(5);

    let mut out: Box<dyn Write> = match output_path {
        Some(path) => {
            let f = std::fs::File::create(path).expect("cannot create output file");
            Box::new(BufWriter::new(f))
        }
        None => Box::new(BufWriter::new(std::io::stdout())),
    };

    let home = make_team(home_race, TeamId::Home, "parity_home", "Home", 3)
        .unwrap_or_else(|_| panic!("unknown home race: {home_race}"));
    let away = make_team(away_race, TeamId::Away, "parity_away", "Away", 2)
        .unwrap_or_else(|_| panic!("unknown away race: {away_race}"));

    let mut state = GameState::new(home.clone(), away.clone());
    // home_is_offense=false: away kicks, home receives, home acts first.
    // Matches Java ParityRunner (game.setHomePlaying(true) → home receives, home acts first).
    state.home_is_offense = false;
    let initial_hash = state_hash(&state);

    // game_start line
    let start = ParityGameStart {
        i: 0,
        step_type: "game_start",
        home: home.name.clone(),
        away: away.name.clone(),
        seed,
        state_hash: initial_hash,
    };
    writeln!(out, "{}", serde_json::to_string(&start).unwrap()).unwrap();

    // Run with logging strategy on both sides so away turns are also logged
    let home_logger = LoggingStrategy::new(CanonicalStrategy);
    let shared = home_logger.steps_handle();
    let away_logger = LoggingStrategy::new_shared(CanonicalStrategy, Arc::clone(&shared));
    let mut rng = GameRng::new_from_xoshiro(seed);

    // Consume pre-game dice to synchronize with Java's initialization sequence:
    //   StepSpectators:  fan_factor home (d3), fan_factor away (d3)
    //   StepWeather:     weather die 1 (d6), weather die 2 (d6)
    //   StepCoinChoice:  coin toss (d2)
    rng.roll(3); // fan factor home
    rng.roll(3); // fan factor away
    let w1 = rng.roll(6) as u8;
    let w2 = rng.roll(6) as u8;
    state.field.weather = ffb_core::types::Weather::from_kickoff_roll(w1 + w2);
    rng.roll(2); // coin toss

    let final_state = SimulationLoop::run(state, &home_logger, &away_logger, &mut rng);

    let game_end_hash = state_hash(&final_state);
    // Both loggers share `shared`; extract after dropping loggers.
    drop(home_logger);
    drop(away_logger);
    let pending = Arc::try_unwrap(shared).unwrap().into_inner().unwrap();
    let steps = finalize_steps(pending, &game_end_hash);

    let n_steps = steps.len();
    for step in steps {
        writeln!(out, "{}", serde_json::to_string(&step).unwrap()).unwrap();
    }

    // game_end line
    let end = ParityGameEnd {
        i: n_steps + 1,
        step_type: "game_end",
        home_score: final_state.home.score,
        away_score: final_state.away.score,
        state_hash: game_end_hash,
    };
    writeln!(out, "{}", serde_json::to_string(&end).unwrap()).unwrap();
    out.flush().unwrap();
}

fn run_legacy_mode(args: &[String]) {
    if args.len() != 3 {
        eprintln!("Usage: parity_runner <n_games> <seed_start>");
        eprintln!("       parity_runner parity <home_race> <away_race> <seed> [output.jsonl]");
        std::process::exit(1);
    }

    let n_games: u64 = args[1].parse().expect("n_games must be a u64");
    let seed_start: u64 = args[2].parse().expect("seed_start must be a u64");

    for i in 0..n_games {
        let seed = seed_start + i;
        let mut rng = GameRng::new_from_xoshiro(seed);
        let state = make_default_game_for_parity();
        let final_state = SimulationLoop::run(state, &CanonicalStrategy, &CanonicalStrategy, &mut rng);
        println!("{}", to_outcome_json(&final_state, seed));
    }
}

fn make_default_game_for_parity() -> GameState {
    let home = make_team("human", TeamId::Home, "parity_home", "Reavers", 3)
        .expect("human roster");
    let away = make_team("orc", TeamId::Away, "parity_away", "Grudgebearers", 2)
        .expect("orc roster");
    GameState::new(home, away)
}

fn count_turns(state: &GameState) -> u32 {
    state.turn_data_home.turn_number as u32 + state.turn_data_away.turn_number as u32
}

fn to_outcome_json(state: &GameState, seed: u64) -> String {
    let score_home = state.team(TeamId::Home).score;
    let score_away = state.team(TeamId::Away).score;
    let finished = state.result.finished;
    let turns_played = count_turns(state);
    format!(
        r#"{{"seed":{seed},"score_home":{score_home},"score_away":{score_away},"finished":{finished},"turns_played":{turns_played}}}"#
    )
}
