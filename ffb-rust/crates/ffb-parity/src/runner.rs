use std::process::Command;
use rand_xoshiro::Xoshiro256StarStar;
use rand_core::{RngCore, SeedableRng};
use ffb_engine::agent::response_to_action_pub;
use ffb_engine::engine::GameEngine;
use ffb_engine::legal_actions::TeamSide;
use ffb_model::enums::Rules;
use ffb_model::model::team::Team;
use ffb_model::prompts::{AgentPrompt, AgentResponse};
use ffb_model::types::FieldCoordinate;
use crate::log_format::{GameLog, LogLine, rust_log_path};
use crate::state_hash::state_hash;
use ffb_model::util::state_hash::state_string;

/// Deterministic parity agent: mirrors Java ParityRunner's decision logic exactly.
/// Uses Xoshiro256StarStar seeded with `seed ^ 0xDEAD_BEEF_CAFE_0001` — same as Java's
/// `new Xoshiro256StarStar(seed ^ 0xDEADBEEFCAFE0001L)`.
struct ParityAgent {
    rng: Xoshiro256StarStar,
    rng_advances: u32,
    debug_seed: u64,
}

impl ParityAgent {
    fn new(seed: u64) -> Self {
        ParityAgent { rng: Xoshiro256StarStar::seed_from_u64(seed ^ 0xDEAD_BEEF_CAFE_0001), rng_advances: 0, debug_seed: seed }
    }

    fn respond(&mut self, prompt: &AgentPrompt, active_side: TeamSide) -> AgentResponse {
        match prompt {
            AgentPrompt::CoinChoice { .. } => {
                self.rng_advances += 1;
                AgentResponse::CoinChoice { heads: self.rng.next_u64() % 2 == 0 }
            }
            AgentPrompt::ReceiveChoice { .. } => {
                self.rng_advances += 1;
                AgentResponse::ReceiveChoice { receive: self.rng.next_u64() % 2 == 0 }
            }
            AgentPrompt::KickBall => {
                // Home kicks to away's half (x 13..25), away kicks to home's half (x 0..12).
                // Matches Java: Long.remainderUnsigned(decisionRng.nextLong(), 13) for x and y.
                let x_raw = (self.rng.next_u64() % 13) as i32;
                let y_raw = (self.rng.next_u64() % 13) as i32;
                self.rng_advances += 2;
                let x = if active_side == TeamSide::Home { x_raw + 13 } else { x_raw };
                let y = y_raw + 1;
                if self.debug_seed == 57 {
                    eprintln!("RUST_KICK seed=57 rng_advances_before={} x_raw={x_raw} y_raw={y_raw} x={x} y={y}", self.rng_advances - 2);
                }
                AgentResponse::KickBall { coord: FieldCoordinate::new(x, y) }
            }
            // TeamSetup: return empty placements → Action::ConfirmSetup → engine calls
            // place_team_canonical() which places players at the canonical squares.
            AgentPrompt::TeamSetup { .. } => {
                AgentResponse::TeamSetup { placements: vec![] }
            }
            // Touchback: pick the player nearest to (13,8) — matches Java's ParityRunner.
            AgentPrompt::Touchback { eligible_players } => {
                let ref_x = 13i32;
                let ref_y = 8i32;
                let pid = eligible_players.iter()
                    .min_by_key(|(_, c)| {
                        let dx = c.x as i32 - ref_x;
                        let dy = c.y as i32 - ref_y;
                        dx * dx + dy * dy
                    })
                    .map(|(id, _)| id.clone())
                    .unwrap_or_default();
                AgentResponse::Touchback { player_id: pid }
            }
            // PlayerChoice (SolidDefence, Charge, HighKick, etc.): always decline by passing
            // empty player_id. Matches Java's ParityRunner which skips optional player selection.
            AgentPrompt::PlayerChoice { .. } => AgentResponse::PlayerChoice { player_id: String::new() },
            AgentPrompt::KickoffReturn { .. } => AgentResponse::PlayerChoice { player_id: String::new() },
            _ => AgentResponse::Confirm,
        }
    }
}

/// Invoke the Java parity runner as a subprocess.
///
/// Uses `java -cp <classpath> com.fumbbl.ffb.ai.parity.ParityRunner`
/// Env vars:
///   PARITY_CP  — Java classpath (default: scans for ffb-ai fat jar)
///   FFB_SERVER_DIR — path to the ffb-server directory
/// Writes `parity/seed_{seed}_java.jsonl`.
pub fn run_java_headless(seed: u64, home_team_id: &str, away_team_id: &str) {
    std::fs::create_dir_all("parity").ok();

    let output_path = format!("parity/seed_{seed}_java.jsonl");

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

    let status = Command::new("java")
        .args([
            "-cp", &cp,
            "com.fumbbl.ffb.ai.parity.ParityRunner",
            &server_dir,
            home_team_id,
            away_team_id,
            &seed.to_string(),
            &output_path,
        ])
        .status();

    match status {
        Ok(s) if s.success() => {}
        Ok(s) => log::warn!("Java parity runner exited with status {s} for seed {seed}"),
        Err(e) => log::warn!("Could not launch Java parity runner for seed {seed}: {e}"),
    }
}

/// Run the Rust headless engine with two RandomAgents and write a JSONL parity log.
///
/// The log format matches Java's ParityRunner output:
///   - game_start line with initial state_hash
///   - one step line per INIT_SELECTING decision point (turn >= 1)
///   - game_end line with final scores and state_hash
pub fn run_rust_headless(seed: u64, home_roster: &str, away_roster: &str) -> Vec<LogLine> {
    std::fs::create_dir_all("parity").ok();

    let home = make_team("home", home_roster);
    let away = make_team("away", away_roster);

    let mut engine = GameEngine::new(home, away, Rules::Bb2025, seed);
    let mut agent = ParityAgent::new(seed);

    let initial_hash = state_hash(&engine.game);
    let mut lines: Vec<LogLine> = Vec::new();
    lines.push(LogLine::GameStart {
        i: 0,
        home: format!("home_{home_roster}"),
        away: format!("away_{away_roster}"),
        seed,
        state_hash: initial_hash,
    });

    let max_iters = 100_000usize;
    let mut step_index = 1u64;
    let mut pending_steps: Vec<PendingStep> = Vec::new();

    for _ in 0..max_iters {
        if engine.is_finished() {
            break;
        }
        let prompt = match engine.current_prompt() {
            Some(p) => p.clone(),
            None => break,
        };

        let side = engine.active_side();
        let response = agent.respond(&prompt, side);

        // Capture state BEFORE applying the action
        let is_turn_boundary = matches!(prompt, AgentPrompt::ActivatePlayer { .. });
        let turn_nr = if engine.game.home_playing {
            engine.game.turn_data_home.turn_nr
        } else {
            engine.game.turn_data_away.turn_nr
        };
        let half = engine.game.half;
        let pre_hash = state_hash(&engine.game);
        let active_str = if engine.game.home_playing { "home" } else { "away" };
        let chosen = response_chosen_str(&response);

        // Debug: print pre-action state string for seed 57 and seed 88
        let pre_string = if (seed == 57 || seed == 88) && is_turn_boundary && turn_nr >= 1 {
            Some(state_string(&engine.game))
        } else {
            None
        };

        let action = response_to_action_pub(response, Some(&prompt));
        match engine.apply(side, action) {
            Ok(_) => {}
            Err(e) => {
                log::warn!("engine error at seed {seed}: {e}");
                break;
            }
        }

        if let Some(s) = pre_string {
            eprintln!("RUST_STATE_STR step={step_index} half={half} turn={turn_nr} active={active_str} hash={pre_hash}");
            eprintln!("  {s}");
        }

        // Log one step line per INIT_SELECTING decision (turn >= 1)
        if is_turn_boundary && turn_nr >= 1 {
            pending_steps.push(PendingStep {
                i: step_index,
                turn: turn_nr,
                half,
                active: active_str.to_string(),
                hash: pre_hash,
                chosen,
            });
            step_index += 1;
        }
    }

    // Fill post_hashes retroactively
    let end_hash = state_hash(&engine.game);
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
        });
    }

    let score_home = engine.game.game_result.home.score;
    let score_away = engine.game.game_result.away.score;
    lines.push(LogLine::GameEnd {
        i: step_index,
        home_score: score_home,
        away_score: score_away,
        state_hash: end_hash,
    });

    // Write to disk
    let path = rust_log_path(seed);
    let log = GameLog {
        seed,
        home_roster: home_roster.to_string(),
        away_roster: away_roster.to_string(),
        lines: lines.clone(),
    };
    if let Err(e) = log.write_to_file(&path) {
        log::warn!("Could not write Rust log for seed {seed}: {e}");
    }

    lines
}

// ── Private helpers ───────────────────────────────────────────────────────────

struct PendingStep {
    i: u64,
    turn: i32,
    half: i32,
    active: String,
    hash: String,
    chosen: String,
}

fn make_team(side: &str, roster_id: &str) -> Team {
    use std::collections::HashSet;
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};

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
        niggling_injuries: 0,
        stat_injuries: vec![],
        current_spps: 0,
        career_spps: 0,
        race: None,
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
        fan_factor: 0,
        dedicated_fans: 0,
        team_value: 1_000_000,
        treasury: 0,
        special_rules: vec![],
        players,
    }
}

fn response_chosen_str(response: &AgentResponse) -> String {
    match response {
        AgentResponse::CoinChoice { heads } =>
            if *heads { "Heads".into() } else { "Tails".into() },
        AgentResponse::ReceiveChoice { receive } =>
            if *receive { "Receive".into() } else { "Kick".into() },
        AgentResponse::UseReRoll { use_reroll } =>
            if *use_reroll { "UseReRoll".into() } else { "NoReRoll".into() },
        AgentResponse::UseSkill { use_skill } =>
            if *use_skill { "UseSkill".into() } else { "NoSkill".into() },
        AgentResponse::FollowUp { follow_up } =>
            if *follow_up { "FollowUp".into() } else { "NoFollowUp".into() },
        AgentResponse::BlockChoice { index } => format!("BlockChoice({index})"),
        AgentResponse::Pushback { coord } => format!("Pushback({},{})", coord.x, coord.y),
        AgentResponse::ActivatePlayer { player_id, action } =>
            format!("Activate({player_id},{action:?})"),
        AgentResponse::Touchback { player_id } => format!("Touchback({player_id})"),
        AgentResponse::KickBall { coord } => format!("Kick({},{})", coord.x, coord.y),
        AgentResponse::PlayerChoice { player_id } => format!("Player({player_id})"),
        AgentResponse::ApothecaryChoice { heal } =>
            if *heal { "Heal".into() } else { "AcceptInjury".into() },
        AgentResponse::UseBribe { use_bribe } =>
            if *use_bribe { "UseBribe".into() } else { "DeclineBribe".into() },
        AgentResponse::BuyInducements { .. } => "BuyInducements".into(),
        AgentResponse::SelectSkill { skill_id } => format!("Skill({skill_id:?})"),
        AgentResponse::TeamSetup { .. } => "TeamSetup".into(),
        AgentResponse::Confirm => "EndTurn".into(),
    }
}
