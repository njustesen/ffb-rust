use std::collections::HashSet;
use std::process::Command;
use rand_xoshiro::Xoshiro256StarStar;
use rand_core::{RngCore, SeedableRng};
use ffb_engine::agent::response_to_action_pub;
use ffb_engine::engine::GameEngine;
use ffb_engine::legal_actions::TeamSide;
use ffb_model::data::roster_json::{PositionJson, SkillEntry};
use ffb_model::data::{bb2016_rosters, bb2020_rosters, bb2025_rosters};
use ffb_model::enums::{PlayerGender, PlayerType, Rules, SkillCategory};
use ffb_model::enums::SkillId;
use ffb_model::model::player::Player;
use ffb_model::model::roster_position::RosterPosition;
use ffb_model::model::skill_def::SkillWithValue;
use ffb_model::model::team::Team;
use ffb_model::prompts::{AgentPrompt, AgentResponse};
use ffb_model::types::FieldCoordinate;
use crate::log_format::{GameLog, LogLine, java_log_path_for, rust_log_path_for};
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

            // ── Inducements ─────────────────────────────────────────────────────
            // Java RandomStrategy sends empty inducement set for all buy prompts.
            AgentPrompt::BuyInducements { .. } | AgentPrompt::BuyPrayersAndInducements { .. } => {
                AgentResponse::BuyInducements { purchases: vec![] }
            }
            // PettyCash / Journeymen: just confirm (accept without consuming RNG).
            AgentPrompt::PettyCash { .. } | AgentPrompt::Journeymen { .. } => AgentResponse::Confirm,
            // UseInducement: always confirm (use it). Java returns null = decline.
            AgentPrompt::UseInducement { .. } => AgentResponse::Confirm,
            // WizardSpell: decline by confirming (Java also declines).
            AgentPrompt::WizardSpell { .. } => AgentResponse::Confirm,
            // ArgueTheCall: decline. Java uses RANDOM but we mirror the Java ParityRunner
            // default, which passes null → decline.
            AgentPrompt::ArgueTheCall { .. } => AgentResponse::UseReRoll { use_reroll: false },
            // BriberyAndCorruption: decline bribe. Java returns null = decline.
            AgentPrompt::BriberyAndCorruption { .. } => AgentResponse::UseBribe { use_bribe: false },
            // ConcedeGame: never concede.
            AgentPrompt::ConcedeGame { .. } => AgentResponse::UseReRoll { use_reroll: false },

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
/// `home_team_id` / `away_team_id` are Java server team IDs (e.g. "teamHumanKalimar").
/// `home_race` / `away_race` are the Rust race names used for the output directory.
pub fn run_java_headless(seed: u64, home_team_id: &str, away_team_id: &str, home_race: &str, away_race: &str) {
    let output_path = java_log_path_for(seed, home_race, away_race);
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
/// `home_roster` / `away_roster` are race names (e.g. "human") or "lineman" for the generic team.
/// `edition` is "bb2016", "bb2020", or "bb2025".
/// `verbose`: when true, each Step entry includes the full `state` string for per-player diagnosis.
pub fn run_rust_headless(seed: u64, home_roster: &str, away_roster: &str, edition: &str, verbose: bool) -> Vec<LogLine> {
    let rust_path = rust_log_path_for(seed, home_roster, away_roster);
    let dir = std::path::Path::new(&rust_path).parent().unwrap_or(std::path::Path::new("parity"));
    std::fs::create_dir_all(dir).ok();

    let rules = edition_to_rules(edition);
    let home = make_team(home_roster, "home", edition);
    let away = make_team(away_roster, "away", edition);

    let mut engine = GameEngine::new(home, away, rules, seed);
    let mut agent = ParityAgent::new(seed);

    let initial_hash = state_hash(&engine.game);
    let mut lines: Vec<LogLine> = Vec::new();
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

        // Capture state string if verbose or if debug seed
        let pre_state_str = if is_turn_boundary && turn_nr >= 1 {
            if verbose || seed == 57 || seed == 88 {
                Some(state_string(&engine.game))
            } else {
                None
            }
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

        if (seed == 57 || seed == 88) && is_turn_boundary && turn_nr >= 1 {
            eprintln!("RUST_STATE_STR step={step_index} half={half} turn={turn_nr} active={active_str} hash={pre_hash}");
            if let Some(ref s) = pre_state_str { eprintln!("  {s}"); }
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
                state: pre_state_str,
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
            state: s.state.clone(),
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
    let log = GameLog {
        seed,
        home_roster: home_roster.to_string(),
        away_roster: away_roster.to_string(),
        lines: lines.clone(),
    };
    if let Err(e) = log.write_to_file(&rust_path) {
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
    state: Option<String>,
}

/// Build a team. `roster_name` is "lineman" (generic) or a race name like "human".
/// `side` is "home" or "away" (used for player IDs). `edition` is "bb2016"/"bb2020"/"bb2025".
fn make_team(roster_name: &str, side: &str, edition: &str) -> Team {
    if roster_name == "lineman"
        || roster_name == "teamLinemanParityHome"
        || roster_name == "teamLinemanParityAway"
    {
        return make_lineman_team(side, roster_name);
    }
    make_team_from_roster(roster_name, side, edition)
        .unwrap_or_else(|e| {
            log::warn!("Could not load roster '{roster_name}': {e}; falling back to lineman");
            make_lineman_team(side, roster_name)
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

/// Build a 11-player team from a named roster (e.g. "human", "orc") for the given edition.
/// Positions are filled in roster order up to 11 players, respecting each position's max quantity.
pub fn make_team_from_roster(roster_name: &str, side: &str, edition: &str) -> Result<Team, String> {
    let rosters = match edition {
        "bb2016" => bb2016_rosters(),
        "bb2020" => bb2020_rosters(),
        "bb2025" | _ => bb2025_rosters(),
    };

    let roster_json = rosters
        .into_iter()
        .find(|r| {
            r.name.to_ascii_lowercase() == roster_name.to_ascii_lowercase()
                || r.id.to_ascii_lowercase().starts_with(&format!("{}.", roster_name.to_ascii_lowercase()))
                || r.id.to_ascii_lowercase() == roster_name.to_ascii_lowercase()
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
        let rp = position_json_to_roster_position(pos_json, &roster_json.id, roster_json.undead);
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
    })
}

fn position_json_to_roster_position(pos: &PositionJson, roster_id: &str, is_undead: bool) -> RosterPosition {
    let skills: Vec<SkillWithValue> = pos.skills.iter()
        .filter_map(|e| skill_entry_to_skill_with_value(e))
        .collect();
    let cats_normal: Vec<SkillCategory> = pos.skill_categories.normal.iter()
        .filter_map(|s| SkillCategory::from_name(s))
        .collect();
    let cats_double: Vec<SkillCategory> = pos.skill_categories.double.iter()
        .filter_map(|s| SkillCategory::from_name(s))
        .collect();
    let player_type = PlayerType::from_name(&pos.player_type)
        .unwrap_or(PlayerType::Regular);
    let is_big_guy = player_type == PlayerType::BigGuy
        || pos.keywords.iter().any(|k| k == "Big Guy");

    RosterPosition {
        id: pos.id.clone(),
        name: pos.name.clone(),
        display_name: pos.display_name.clone(),
        shorthand: None,
        player_type,
        gender: PlayerGender::Male,
        quantity: pos.quantity,
        cost: pos.cost,
        movement: pos.ma,
        strength: pos.st,
        agility: pos.ag,
        passing: pos.pa,
        armour: pos.av,
        skills,
        skill_categories_normal: cats_normal,
        skill_categories_double: cats_double,
        keywords: pos.keywords.clone(),
        is_big_guy,
        is_undead,
        is_thrall: false,
        race: Some(roster_id.to_string()),
        replaces_position: None,
    }
}

fn skill_entry_to_skill_with_value(entry: &SkillEntry) -> Option<SkillWithValue> {
    let skill_id = SkillId::from_class_name(entry.name())?;
    let value = match entry {
        SkillEntry::WithValue { value, .. } => Some(value.to_string()),
        SkillEntry::Simple(_) => None,
    };
    Some(SkillWithValue { skill_id, value })
}

fn edition_to_rules(edition: &str) -> Rules {
    match edition {
        "bb2016" => Rules::Bb2016,
        "bb2020" => Rules::Bb2020,
        _ => Rules::Bb2025,
    }
}

/// Map a snake_case race name + side to the Java server parity team ID.
/// Uses PascalCase conversion matching gen_java_teams.py exactly.
/// e.g. "dark_elf_league_fumbbl" + "home" → "teamDarkElfLeagueFumbblParityHome"
pub fn java_team_id(race_name: &str, side: &str) -> String {
    let suffix = if side == "away" { "Away" } else { "Home" };
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
    format!("team{pascal}Parity{suffix}")
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
