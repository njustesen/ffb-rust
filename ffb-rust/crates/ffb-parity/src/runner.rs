use std::collections::HashSet;
use std::process::Command;
use ffb_engine::agent::{RandomAgent, Agent};
use ffb_engine::step::GameState;
use ffb_engine::legal_actions::TeamSide;
use ffb_model::events::GameEvent;
use ffb_model::data::roster_json::{PositionJson, SkillEntry};
use ffb_model::data::{bb2016_rosters, bb2020_rosters, bb2025_rosters};
use ffb_model::enums::{PlayerGender, PlayerType, Rules, SkillCategory};
use ffb_model::enums::SkillId;
use ffb_model::model::player::Player;
use ffb_model::model::roster_position::RosterPosition;
use ffb_model::model::skill_def::SkillWithValue;
use ffb_model::model::team::Team;
use ffb_model::prompts::AgentPrompt;
use crate::log_format::{GameLog, LogLine, java_log_path_for, rust_log_path_for, rust_events_path_for};
use crate::state_hash::state_hash;
use ffb_model::util::state_hash::state_string;

/// Invoke the Java parity runner as a subprocess.
///
/// Uses `java -cp <classpath> com.fumbbl.ffb.ai.parity.ParityRunner`
/// Env vars:
///   PARITY_CP  — Java classpath (default: scans for ffb-ai fat jar)
///   FFB_SERVER_DIR — path to the ffb-server directory
/// Writes `parity/seed_{seed}_java.jsonl`.
/// `home_team_id` / `away_team_id` are Java server team IDs (e.g. "teamHumanKalimar").
/// `home_race` / `away_race` are the Rust race names used for the output directory.
pub fn run_java_headless(seed: u64, home_team_id: &str, away_team_id: &str, home_race: &str, away_race: &str, tier: u8) {
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

    let mut args: Vec<String> = vec!["-cp".into(), cp.clone()];
    // Mirror the Rust-side trace env vars onto the Java process.
    if std::env::var_os("FFB_DICE_TRACE").is_some() {
        args.push("-Dffb.diceTrace=true".into());
    }
    if std::env::var_os("FFB_TRACE").is_some() {
        args.push("-Dffb.parityDebug=true".into());
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
    let status = Command::new("java").args(&args).status();

    match status {
        Ok(s) if s.success() => {}
        Ok(s) => log::warn!("Java parity runner exited with status {s} for seed {seed}"),
        Err(e) => log::warn!("Could not launch Java parity runner for seed {seed}: {e}"),
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
pub fn run_rust_headless(seed: u64, home_roster: &str, away_roster: &str, edition: &str, verbose: bool, tier: u8) -> (Vec<LogLine>, Vec<GameEvent>, i32, i32) {
    let rust_path = rust_log_path_for(seed, home_roster, away_roster);
    let dir = std::path::Path::new(&rust_path).parent().unwrap_or(std::path::Path::new("parity"));
    std::fs::create_dir_all(dir).ok();

    let rules = edition_to_rules(edition);
    let home = make_team(home_roster, "home", edition);
    let away = make_team(away_roster, "away", edition);

    let mut engine = GameState::new(home, away, rules, seed);
    // One shared agent for both teams. new_parity seeds both decision and action RNGs
    // to match Java's decisionRng and actionRng exactly.
    let mut agent = RandomAgent::new_parity(seed);

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
    let events_path = rust_events_path_for(seed, home_roster, away_roster);
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
pub(crate) fn make_team(roster_name: &str, side: &str, edition: &str) -> Team {
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

    // Explicit aliases for rosters whose JSON id/name don't match the race short-name.
    let roster_name = match roster_name.to_ascii_lowercase().as_str() {
        "renegades" => "chaos renegade",
        "chaos_chosen" => "chaos",
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

pub(crate) fn edition_to_rules(edition: &str) -> Rules {
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


pub(crate) fn action_label(action: &ffb_engine::action::Action) -> String {
    use ffb_engine::action::Action;
    match action {
        Action::CoinChoice { heads } => if *heads { "Heads".into() } else { "Tails".into() },
        Action::ReceiveChoice { receive } => if *receive { "Receive".into() } else { "Kick".into() },
        Action::KickBall { coord } => format!("Kick({},{})", coord.x, coord.y),
        Action::Touchback { player_id } => format!("Touchback({player_id})"),
        Action::ActivatePlayer { player_id, player_action, .. } => format!("Activate({player_id},{player_action:?})"),
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
    let mut engine = GameState::new(home, away, rules, seed);

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
    let mut engine = GameState::new(home, away, rules, seed);

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
