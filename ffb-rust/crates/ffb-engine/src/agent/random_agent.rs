use std::collections::HashSet;
use rand_xoshiro::Xoshiro256StarStar;
use rand_core::{RngCore, SeedableRng};
use ffb_model::prompts::{AgentPrompt, AgentResponse};
use ffb_model::types::FieldCoordinate;
use ffb_model::enums::PlayerAction;
use ffb_mechanics::skills::{SkillId, SKILL_TABLE};
use crate::action::{Action, PlayerActionChoice};
use crate::legal_actions::{TeamSide, legal_block_targets, legal_foul_targets, legal_move_targets};
use crate::engine::GameEngine;
use super::Agent;

/// Canonical random agent for all automated game runs — parity testing, coverage, and replay.
///
/// Uses Xoshiro256StarStar with `next_u64() % n` for all picks, matching Java's
/// `Long.remainderUnsigned(rng.nextLong(), n)` pattern so both engines consume
/// the decision RNG in the same order given the same seed.
///
/// ## Usage
/// ```no_run
/// # let seed: u64 = 1;
/// # use ffb_engine::agent::RandomAgent;
/// // Parity: one agent for both sides, seed XOR applied here to stay distinct from game dice.
/// let mut agent = RandomAgent::new(seed ^ 0xDEAD_BEEF_CAFE_0001);
///
/// // Coverage / replay: separate agents per side.
/// let mut home_agent = RandomAgent::new(seed);
/// let mut away_agent = RandomAgent::new(seed ^ 0xFFFF_FFFF);
/// ```
///
/// ## Java counterpart
/// `com.fumbbl.ffb.ai.parity.ParityRunner` implements the same `act()` algorithm:
/// same turn-tracking, same Phase 2 player injection, same follow-up action selection.
pub struct RandomAgent {
    /// Decision RNG — synced with Java's `decisionRng` when seeded with `seed ^ XOR`.
    /// Used for: CoinChoice, ReceiveChoice, KickBall, player selection at activation.
    rng: Xoshiro256StarStar,
    /// Action diversity RNG — independent from Java's decisions.
    /// Used for: Move paths, Block/Foul targets, Pass coords.
    action_rng: Xoshiro256StarStar,
    /// Count of pick_action calls — used for debugging RNG sync.
    pub action_rng_calls: u64,
    /// Eligible players captured at the start of each team turn (Phase 1 ActivatePlayer).
    /// Stored as Vec to preserve roster order (same order Java uses), so no sort is needed.
    eligible_this_turn: Vec<(String, Vec<PlayerAction>)>,
    /// Players already activated this turn — used to filter the eligible list at Phase 2.
    used_this_turn: HashSet<String>,
    /// (half, turn_nr, home_playing) — detects genuine new turns to reset tracking.
    last_turn_key: (i32, i32, bool),
    /// When Some: the next act() call must return this follow-up action (move target,
    /// block target, etc.) for the player just activated. Cleared after use.
    pending_follow_up: Option<(String, PlayerAction)>,
}

impl RandomAgent {
    /// Construct a random agent seeded with `seed`.
    ///
    /// For coverage/visual runs (no Java sync required). Both RNGs derive from `seed`.
    pub fn new(seed: u64) -> Self {
        RandomAgent {
            rng: Xoshiro256StarStar::seed_from_u64(seed),
            action_rng: Xoshiro256StarStar::seed_from_u64(seed ^ 0xC0FFEE_ACE0_0001),
            action_rng_calls: 0,
            eligible_this_turn: Vec::new(),
            used_this_turn: HashSet::new(),
            last_turn_key: (-1, -1, true),
            pending_follow_up: None,
        }
    }

    /// Construct a random agent for Java parity runs, given the raw game seed.
    ///
    /// Matches Java's ParityRunner exactly:
    /// - `decisionRng` seeded with `game_seed ^ 0xDEAD_BEEF_CAFE_0001`
    /// - `actionRng`   seeded with `game_seed ^ 0xC0FFEE_ACE0_0001`
    pub fn new_parity(game_seed: u64) -> Self {
        RandomAgent {
            rng: Xoshiro256StarStar::seed_from_u64(game_seed ^ 0xDEAD_BEEF_CAFE_0001),
            action_rng: Xoshiro256StarStar::seed_from_u64(game_seed ^ 0xC0FFEE_ACE0_0001),
            action_rng_calls: 0,
            eligible_this_turn: Vec::new(),
            used_this_turn: HashSet::new(),
            last_turn_key: (-1, -1, true),
            pending_follow_up: None,
        }
    }

    /// Pick a uniform random index in `[0, len)` using the decision RNG.
    /// Matches Java's `Long.remainderUnsigned(decisionRng.nextLong(), len)`.
    fn pick(&mut self, len: usize) -> usize {
        if len == 0 { 0 } else { (self.rng.next_u64() as usize) % len }
    }

    /// Pick a uniform random index using the action diversity RNG (independent from Java sync).
    fn pick_action(&mut self, len: usize) -> usize {
        self.action_rng_calls += 1;
        if len == 0 { 0 } else { (self.action_rng.next_u64() as usize) % len }
    }

    fn pick_bool(&mut self) -> bool {
        self.rng.next_u64() % 2 == 0
    }

    fn pick_bool_action(&mut self) -> bool {
        self.action_rng.next_u64() % 2 == 0
    }

    /// After a player is activated, compute the concrete follow-up action (move path,
    /// block target, pass coord, etc.) using simple deterministic selection so Java
    /// can match with the same RNG calls.
    ///
    /// Only takes real actions in Regular turn mode. For kickoff events (Blitz!, QuickSnap,
    /// etc.) this returns EndTurn, matching Java's "activate then immediately deselect" behavior.
    ///
    /// For Move: picks from legal_move_targets() sorted by coordinate, 1-step path.
    /// For Block/Foul: picks from legal_*_targets() sorted by player_id.
    /// For Pass: picks a teammate on pitch sorted by coordinate; random coord if none.
    fn compute_follow_up(&mut self, engine: &GameEngine, pid: &str, pa: PlayerAction) -> Action {
        use ffb_model::enums::TurnMode;
        // Non-Regular modes (kickoff Blitz!, QuickSnap, etc.): no concrete action.
        // This matches Java's ParityRunner which activates then immediately deselects.
        if engine.game.turn_mode != TurnMode::Regular {
            return Action::EndTurn;
        }
        let side = engine.active_side();
        let game = &engine.game;
        match pa {
            PlayerAction::Move | PlayerAction::BlitzMove | PlayerAction::PassMove
            | PlayerAction::FoulMove | PlayerAction::HandOverMove
            | PlayerAction::ThrowTeamMateMove | PlayerAction::KickTeamMateMove
            | PlayerAction::PuntMove | PlayerAction::KickTeamMate => {
                let mut targets = legal_move_targets(game, pid);
                if targets.is_empty() {
                    return Action::EndTurn;
                }
                targets.sort_by_key(|c| (c.x, c.y));
                let arng_before = self.action_rng_calls;
                let i = self.pick_action(targets.len());
                if crate::parity_trace_enabled() {
                    let targets_str: Vec<String> = targets.iter().map(|c| format!("({},{})", c.x, c.y)).collect();
                    eprintln!("MOVE_PICK pid={} arng_before={} N={} i={} t=({},{}) all={}", pid, arng_before, targets.len(), i, targets[i].x, targets[i].y, targets_str.join(","));
                }
                Action::Move { path: vec![targets[i]] }
            }
            PlayerAction::StandUp => {
                let mut targets = legal_move_targets(game, pid);
                targets.sort_by_key(|c| (c.x, c.y));
                if targets.is_empty() {
                    Action::EndTurn
                } else {
                    let i = self.pick_action(targets.len());
                    Action::Move { path: vec![targets[i]] }
                }
            }
            PlayerAction::Block | PlayerAction::Blitz | PlayerAction::StandUpBlitz => {
                let mut targets = legal_block_targets(game, pid, side);
                targets.sort();
                if targets.is_empty() {
                    Action::EndTurn
                } else {
                    let i = self.pick_action(targets.len());
                    Action::Block { defender_id: targets[i].clone() }
                }
            }
            PlayerAction::Stab | PlayerAction::Chainsaw => {
                let mut targets = legal_block_targets(game, pid, side);
                targets.sort();
                if targets.is_empty() {
                    Action::EndTurn
                } else {
                    let i = self.pick_action(targets.len());
                    Action::Stab { defender_id: targets[i].clone() }
                }
            }
            PlayerAction::Foul => {
                let mut targets = legal_foul_targets(game, pid, side);
                targets.sort();
                if targets.is_empty() {
                    Action::EndTurn
                } else {
                    let i = self.pick_action(targets.len());
                    Action::Foul { target_id: targets[i].clone() }
                }
            }
            PlayerAction::Pass | PlayerAction::HailMaryPass | PlayerAction::DumpOff => {
                let same_team = if matches!(side, TeamSide::Home) { &game.team_home } else { &game.team_away };
                let mut teammates: Vec<FieldCoordinate> = same_team.players.iter()
                    .filter(|p| p.id != pid)
                    .filter_map(|p| game.field_model.player_coordinate(&p.id))
                    .filter(|c| c.is_on_pitch())
                    .collect();
                teammates.sort_by_key(|c| (c.x, c.y));
                let coord = if !teammates.is_empty() {
                    let i = self.pick_action(teammates.len());
                    teammates[i]
                } else {
                    let x = (self.rng.next_u64() % 26) as i32;
                    let y = (self.rng.next_u64() % 14 + 1) as i32;
                    FieldCoordinate::new(x, y)
                };
                Action::Pass { coord }
            }
            PlayerAction::HandOver => {
                let my_coord = game.field_model.player_coordinate(pid);
                let same_team = if matches!(side, TeamSide::Home) { &game.team_home } else { &game.team_away };
                let mut receivers: Vec<String> = same_team.players.iter()
                    .filter(|p| p.id != pid)
                    .filter(|p| {
                        game.field_model.player_coordinate(&p.id)
                            .zip(my_coord)
                            .map(|(c, mc)| c.is_adjacent(mc))
                            .unwrap_or(false)
                    })
                    .map(|p| p.id.clone())
                    .collect();
                receivers.sort();
                if receivers.is_empty() {
                    Action::EndTurn
                } else {
                    let i = self.pick_action(receivers.len());
                    Action::HandOff { receiver_id: receivers[i].clone() }
                }
            }
            PlayerAction::ThrowTeamMate => {
                let my_coord = game.field_model.player_coordinate(pid);
                let same_team = if matches!(side, TeamSide::Home) { &game.team_home } else { &game.team_away };
                let mut throwable: Vec<String> = same_team.players.iter()
                    .filter(|p| p.id != pid)
                    .filter(|p| p.has_skill(SkillId::RightStuff) || p.has_skill(SkillId::Stunty))
                    .filter(|p| {
                        game.field_model.player_coordinate(&p.id)
                            .zip(my_coord)
                            .map(|(c, mc)| c.is_adjacent(mc))
                            .unwrap_or(false)
                    })
                    .map(|p| p.id.clone())
                    .collect();
                throwable.sort();
                if throwable.is_empty() {
                    Action::EndTurn
                } else {
                    let i = self.pick_action(throwable.len());
                    let x = (self.rng.next_u64() % 26) as i32;
                    let y = (self.rng.next_u64() % 14 + 1) as i32;
                    Action::ThrowTeamMate { player_id: throwable[i].clone(), coord: FieldCoordinate::new(x, y) }
                }
            }
            PlayerAction::KickTeamMate => {
                // Same selection as ThrowTeamMate but using KickTeamMate action
                let my_coord = game.field_model.player_coordinate(pid);
                let same_team = if matches!(side, TeamSide::Home) { &game.team_home } else { &game.team_away };
                let mut throwable: Vec<String> = same_team.players.iter()
                    .filter(|p| p.id != pid)
                    .filter(|p| p.has_skill(SkillId::RightStuff) || p.has_skill(SkillId::Stunty))
                    .filter(|p| {
                        game.field_model.player_coordinate(&p.id)
                            .zip(my_coord)
                            .map(|(c, mc)| c.is_adjacent(mc))
                            .unwrap_or(false)
                    })
                    .map(|p| p.id.clone())
                    .collect();
                throwable.sort();
                if throwable.is_empty() {
                    Action::EndTurn
                } else {
                    let i = self.pick_action(throwable.len());
                    let x = (self.rng.next_u64() % 26) as i32;
                    let y = (self.rng.next_u64() % 14 + 1) as i32;
                    Action::KickTeamMate { player_id: throwable[i].clone(), coord: FieldCoordinate::new(x, y) }
                }
            }
            PlayerAction::Gaze | PlayerAction::GazeSelect | PlayerAction::GazeMove
            | PlayerAction::AutoGazeZoat => {
                let opp = if matches!(side, TeamSide::Home) { &game.team_away } else { &game.team_home };
                let mut candidates: Vec<String> = opp.players.iter()
                    .filter(|p| game.field_model.player_coordinate(&p.id).is_some())
                    .filter(|p| game.field_model.player_state(&p.id).map(|s| s.has_tacklezones()).unwrap_or(false))
                    .map(|p| p.id.clone())
                    .collect();
                candidates.sort();
                if candidates.is_empty() {
                    Action::EndTurn
                } else {
                    let i = self.pick_action(candidates.len());
                    Action::HypnoticGaze { target_id: candidates[i].clone() }
                }
            }
            PlayerAction::BreatheFire => {
                let mut targets = legal_block_targets(game, pid, side);
                targets.sort();
                if targets.is_empty() {
                    Action::EndTurn
                } else {
                    let i = self.pick_action(targets.len());
                    Action::BreatheFire { target_id: targets[i].clone() }
                }
            }
            PlayerAction::ProjectileVomit => {
                let mut targets = legal_block_targets(game, pid, side);
                targets.sort();
                if targets.is_empty() {
                    Action::EndTurn
                } else {
                    let i = self.pick_action(targets.len());
                    Action::ProjectileVomit { target_id: targets[i].clone() }
                }
            }
            _ => Action::EndTurn,
        }
    }

    /// Internal respond for non-activation prompts. Returns AgentResponse for conversion.
    ///
    /// Decision-RNG prompts (synced with Java's decisionRng): CoinChoice, ReceiveChoice.
    /// KickBall is handled directly in act() for side-awareness.
    /// All other prompts use action_rng or deterministic choices to avoid disturbing Java sync.
    ///
    /// Touchback uses nearest-to-center (deterministic, 0 RNG) matching Java's ParityRunner.
    #[allow(dead_code)]
    pub(crate) fn respond(&mut self, prompt: &AgentPrompt) -> AgentResponse {
        match prompt {
            // ── Decision-RNG prompts (match Java's decisionRng consumption) ──────────
            AgentPrompt::CoinChoice { .. } =>
                AgentResponse::CoinChoice { heads: self.pick_bool() },

            AgentPrompt::ReceiveChoice { .. } =>
                AgentResponse::ReceiveChoice { receive: self.pick_bool() },

            // ── Deterministic action choices — match Java's ParityRunner defaults ─────
            // These are fixed to avoid shifting action_rng before the concrete action
            // target selection (move square, block target, etc.) in compute_follow_up.
            // Java's handleDialog() uses the same fixed values.
            AgentPrompt::ReRollOffer { .. } =>
                AgentResponse::UseReRoll { use_reroll: false },

            // Always use the skill — matches Java "always USE the skill" handler.
            AgentPrompt::SkillUse { .. } | AgentPrompt::PilingOn { .. } =>
                AgentResponse::UseSkill { use_skill: true },

            AgentPrompt::FollowUp { .. } =>
                AgentResponse::FollowUp { follow_up: false },

            AgentPrompt::HitAndRun { .. } =>
                AgentResponse::Pushback { coord: FieldCoordinate::new(0, 0) },

            AgentPrompt::BlockChoice { .. } =>
                AgentResponse::BlockChoice { index: 0 },

            AgentPrompt::Pushback { squares, .. } => {
                let coord = squares.first().copied().unwrap_or(FieldCoordinate::new(0, 0));
                AgentResponse::Pushback { coord }
            }

            AgentPrompt::ApothecaryChoice { .. } =>
                AgentResponse::ApothecaryChoice { heal: false },

            AgentPrompt::UseApothecary { .. } =>
                AgentResponse::ApothecaryChoice { heal: false },

            AgentPrompt::SelectSkill { available, .. } => {
                if available.is_empty() { return AgentResponse::Confirm; }
                let (_, skills) = &available[0];
                if skills.is_empty() { return AgentResponse::Confirm; }
                AgentResponse::SelectSkill { skill_id: skills[0] }
            }

            AgentPrompt::ArgueTheCall { .. } =>
                AgentResponse::UseReRoll { use_reroll: false },

            AgentPrompt::BriberyAndCorruption { .. } =>
                AgentResponse::UseBribe { use_bribe: false },

            AgentPrompt::KickoffReturn { .. } =>
                AgentResponse::PlayerChoice { player_id: String::new() },

            AgentPrompt::PlayerChoice { .. } =>
                AgentResponse::PlayerChoice { player_id: String::new() },

            // ── Deterministic (0 RNG calls, matches Java's defaults) ─────────────────
            // Touchback: nearest player to center (13, 8) — same as Java's ParityRunner.
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

            AgentPrompt::TeamSetup { .. } =>
                AgentResponse::TeamSetup { placements: vec![] },

            AgentPrompt::BuyInducements { .. } | AgentPrompt::BuyPrayersAndInducements { .. } =>
                AgentResponse::BuyInducements { purchases: vec![] },

            AgentPrompt::ConcedeGame { .. } =>
                AgentResponse::UseReRoll { use_reroll: false },

            _ => AgentResponse::Confirm,
        }
    }
}

impl RandomAgent {
    /// T3 Phase 1 parity behavior: consume 1 decision-RNG call (to stay in sync with Java's
    /// `decisionRng.nextLong() % n` pick) but return EndTurn instead of applying the activation.
    ///
    /// Used by `run_rust_headless` (Java parity comparison) until Java's T3 Phase 2 is complete.
    /// Coverage/visual runs use the full `act()` method instead.
    pub fn act_parity_v1(&mut self, engine: &GameEngine) -> Action {
        match engine.current_prompt() {
            Some(AgentPrompt::ActivatePlayer { eligible_players }) if !eligible_players.is_empty() => {
                // Consume 1 rng call to stay synced with Java's `decisionRng.nextLong() % n`.
                let _ = self.pick(eligible_players.len());
                Action::EndTurn
            }
            _ => self.act(engine),
        }
    }
}

impl Agent for RandomAgent {
    /// Choose an action given the current engine state.
    ///
    /// On each call, resolves in order:
    /// 1. A pending follow-up from the previous ActivatePlayer (move path, block target, etc.).
    /// 2. Phase 1 ActivatePlayer (non-empty eligible): save eligible list, pick a player,
    ///    set pending_follow_up for next call.
    /// 3. Phase 2 ActivatePlayer (empty eligible): inject the next unused eligible player
    ///    from the saved list, or EndTurn if all are used.
    /// 4. All other prompts: random response converted to Action.
    /// 5. No prompt: EndTurn.
    fn act(&mut self, engine: &GameEngine) -> Action {
        // 1. Pending follow-up from previous ActivatePlayer
        if let Some((pid, pa)) = self.pending_follow_up.take() {
            let action = self.compute_follow_up(engine, &pid, pa);
            if engine.game.acting_player.player_id.as_deref() == Some(pid.as_str()) {
                return action;
            }
            // Player no longer acting (e.g. WildAnimal cleared acting_player).
            // action_rng was consumed above to match Java's pre-negatrait actionRng call. Fall through.
            if crate::parity_trace_enabled() {
                eprintln!("WILDANIMAL_FALLTHROUGH pid={} acting={:?} half={} turn={} home={}", pid, engine.game.acting_player.player_id, engine.game.half, if engine.game.home_playing { engine.game.turn_data_home.turn_nr } else { engine.game.turn_data_away.turn_nr }, engine.game.home_playing);
            }
        }

        match engine.current_prompt() {
            Some(AgentPrompt::ActivatePlayer { eligible_players }) => {
                if !eligible_players.is_empty() {
                    // Phase 1: record eligible list for this turn, pick a player.
                    let turn_nr = if engine.game.home_playing {
                        engine.game.turn_data_home.turn_nr
                    } else {
                        engine.game.turn_data_away.turn_nr
                    };
                    let turn_key = (engine.game.half, turn_nr, engine.game.home_playing);
                    if turn_key != self.last_turn_key {
                        // New turn: reset and capture the full eligible list.
                        self.last_turn_key = turn_key;
                        self.used_this_turn.clear();
                        self.eligible_this_turn.clear();
                        for (pid, acts) in eligible_players.iter() {
                            self.eligible_this_turn.push((pid.clone(), acts.clone()));
                        }
                        // Fall through to the skip-loop below.
                    } else if eligible_players.len() == 1
                        && self.used_this_turn.contains(eligible_players[0].0.as_str())
                    {
                        // Blitz block step: single already-used player (the blitzing player returning for block).
                        let pi = self.pick(eligible_players.len());
                        let (pid, acts) = &eligible_players[pi];
                        let pa = acts.get(0).copied().unwrap_or(PlayerAction::Move);
                        self.used_this_turn.insert(pid.clone());
                        self.pending_follow_up = Some((pid.clone(), pa));
                        return Action::ActivatePlayer { player_id: pid.clone(), player_action: player_action_to_choice(pa) };
                    }
                    // Fall through to skip-loop. Handles WildAnimal recovery (engine's eligible list
                    // may include the inactive player; skip-loop uses eligible_this_turn.filter(!used_this_turn)).
                    // New-turn Phase 1: pick from eligible_this_turn with inactive-skip loop.
                    // Java's server rejects just-unstunned players (isActive()=false → SKIP_STEP),
                    // but still consumes a decisionRng call for the rejected pick. Mirror that here.
                    loop {
                        let remaining: Vec<(String, Vec<PlayerAction>)> = self.eligible_this_turn.iter()
                            .filter(|(pid, _)| !self.used_this_turn.contains(pid.as_str()))
                            .map(|(pid, acts)| (pid.clone(), acts.clone()))
                            .collect();
                        if remaining.is_empty() { return Action::EndTurn; }
                        let pi = self.pick(remaining.len());
                        let (pid, acts) = remaining[pi].clone();
                        self.used_this_turn.insert(pid.clone());
                        let just_unstunned = engine.game.field_model.player_state(&pid)
                            .map(|s| !s.is_active())
                            .unwrap_or(false);
                        if just_unstunned { continue; }
                        let pa = acts.get(0).copied().unwrap_or(PlayerAction::Move);
                        self.pending_follow_up = Some((pid.clone(), pa));
                        return Action::ActivatePlayer { player_id: pid, player_action: player_action_to_choice(pa) };
                    }
                } else {
                    // Phase 2: inject next unused eligible player (in roster order), or EndTurn.
                    // In non-Regular turn modes (kickoff Blitz!, QuickSnap, etc.),
                    // end turn immediately — matching Java's "one activation then EndTurn" behavior.
                    // Also end immediately on turnover (ball scatter on pickup, etc.) — matching
                    // Java's ParityRunner which re-computes eligible players after each activation
                    // and stops when the turn has ended.
                    use ffb_model::enums::TurnMode;
                    if engine.game.turn_mode != TurnMode::Regular {
                        return Action::EndTurn;
                    }
                    // Pick with inactive-skip loop: just-unstunned players (active=false) are
                    // rejected by Java's server (SKIP_STEP). Consume the pick call and retry.
                    loop {
                        let remaining: Vec<(String, Vec<PlayerAction>)> = self.eligible_this_turn.iter()
                            .filter(|(pid, acts)| !self.used_this_turn.contains(pid.as_str()) && !acts.is_empty())
                            .map(|(pid, acts)| (pid.clone(), acts.clone()))
                            .collect();
                        if remaining.is_empty() { return Action::EndTurn; }
                        let i = self.pick(remaining.len());
                        let (pid, acts) = remaining[i].clone();
                        self.used_this_turn.insert(pid.clone());
                        let just_unstunned = engine.game.field_model.player_state(&pid)
                            .map(|s| !s.is_active())
                            .unwrap_or(false);
                        if just_unstunned { continue; }
                        let pa = acts.get(0).copied().unwrap_or(PlayerAction::Move);
                        self.pending_follow_up = Some((pid.clone(), pa));
                        return Action::ActivatePlayer { player_id: pid, player_action: player_action_to_choice(pa) };
                    }
                }
            }

            Some(AgentPrompt::KickBall) => {
                // Side-aware kick: kicking team sends to opponent's half
                let x_raw = (self.rng.next_u64() % 13) as i32;
                let y_raw = (self.rng.next_u64() % 13) as i32;
                let x = if engine.game.home_playing { x_raw + 13 } else { x_raw };
                Action::KickBall { coord: FieldCoordinate::new(x, y_raw + 1) }
            }

            Some(prompt) => {
                let prompt = prompt.clone();
                let response = self.respond(&prompt);
                response_to_action(response, Some(&prompt))
            }

            None => Action::EndTurn,
        }
    }
}

pub(super) fn player_action_to_choice(action: PlayerAction) -> PlayerActionChoice {
    match action {
        PlayerAction::Move | PlayerAction::BlitzMove | PlayerAction::PassMove
        | PlayerAction::FoulMove | PlayerAction::HandOverMove | PlayerAction::ThrowTeamMateMove
        | PlayerAction::StandUp | PlayerAction::PuntMove | PlayerAction::KickTeamMateMove => PlayerActionChoice::Move,
        PlayerAction::Block => PlayerActionChoice::Block,
        PlayerAction::Stab => PlayerActionChoice::Stab,
        PlayerAction::Blitz | PlayerAction::BlitzSelect | PlayerAction::StandUpBlitz => PlayerActionChoice::Blitz,
        PlayerAction::Pass | PlayerAction::HailMaryPass | PlayerAction::DumpOff => PlayerActionChoice::Pass,
        PlayerAction::HandOver => PlayerActionChoice::HandOff,
        PlayerAction::Foul => PlayerActionChoice::Foul,
        PlayerAction::ThrowTeamMate => PlayerActionChoice::ThrowTeamMate,
        PlayerAction::KickTeamMate => PlayerActionChoice::KickTeamMate,
        PlayerAction::Gaze | PlayerAction::GazeSelect | PlayerAction::GazeMove => PlayerActionChoice::HypnoticGaze,
        PlayerAction::ThrowBomb | PlayerAction::HailMaryBomb => PlayerActionChoice::ThrowBomb,
        PlayerAction::Swoop => PlayerActionChoice::Swoop,
        PlayerAction::Punt => PlayerActionChoice::Punt,
        _ => PlayerActionChoice::Move,
    }
}

pub(super) fn skill_id_from_u16(idx: u16) -> ffb_mechanics::skills::SkillId {
    SKILL_TABLE.get(idx as usize).map(|s| s.id).unwrap_or(ffb_mechanics::skills::SkillId::Block)
}

/// Convert an AgentResponse to an engine Action. Used internally by act() and for testing.
pub(crate) fn response_to_action(response: AgentResponse, prompt: Option<&AgentPrompt>) -> Action {
    match response {
        AgentResponse::CoinChoice { heads } => Action::CoinChoice { heads },
        AgentResponse::ReceiveChoice { receive } => Action::ReceiveChoice { receive },
        AgentResponse::UseReRoll { use_reroll } => {
            match prompt {
                Some(AgentPrompt::ArgueTheCall { .. }) => Action::ArgueTheCall { argue: use_reroll },
                _ => Action::UseReRoll { use_reroll },
            }
        }
        AgentResponse::UseSkill { use_skill } => {
            let skill_id = match prompt {
                Some(AgentPrompt::SkillUse { skill_id, .. }) => skill_id_from_u16(*skill_id),
                Some(AgentPrompt::PilingOn { .. }) => ffb_mechanics::skills::SkillId::PilingOn,
                _ => ffb_mechanics::skills::SkillId::Block,
            };
            Action::UseSkill { skill_id, use_skill }
        }
        AgentResponse::FollowUp { follow_up } => Action::FollowUp { follow_up },
        AgentResponse::BlockChoice { index } => Action::BlockChoice { die_index: index as usize },
        AgentResponse::Pushback { coord } => {
            match prompt {
                Some(AgentPrompt::HitAndRun { squares, .. }) => {
                    let chosen = if squares.contains(&coord) { Some(coord) } else { None };
                    Action::HitAndRun { coord: chosen }
                }
                _ => Action::PushTo { coord },
            }
        }
        AgentResponse::ActivatePlayer { player_id, action } => {
            let player_action = player_action_to_choice(action);
            Action::ActivatePlayer { player_id, player_action }
        }
        AgentResponse::Touchback { player_id } => Action::Touchback { player_id },
        AgentResponse::KickBall { coord } => Action::KickBall { coord },
        AgentResponse::PlayerChoice { player_id } => Action::SelectPlayer { player_id },
        AgentResponse::TeamSetup { .. } => Action::ConfirmSetup,
        AgentResponse::ApothecaryChoice { heal } => {
            let player_id = match prompt {
                Some(AgentPrompt::UseApothecary { player_id, .. }) => player_id.clone(),
                Some(AgentPrompt::ApothecaryChoice { player_id, .. }) => player_id.clone(),
                _ => String::new(),
            };
            Action::UseApothecary { player_id, use_apothecary: heal }
        }
        AgentResponse::UseBribe { use_bribe } => Action::UseBribe { use_bribe },
        AgentResponse::BuyInducements { purchases } => {
            use crate::action::InducementPurchase;
            Action::BuyInducements {
                purchases: purchases.iter()
                    .map(|(id, count)| InducementPurchase { id: id.clone(), count: *count as u32 })
                    .collect()
            }
        }
        AgentResponse::SelectSkill { skill_id } => {
            Action::SelectSkill { skill_id: skill_id_from_u16(skill_id) }
        }
        AgentResponse::Confirm => Action::EndTurn,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::prompts::{AgentPrompt, AgentResponse};
    use ffb_mechanics::skills::SkillId;
    use ffb_model::enums::ReRollSource;
    use crate::action::Action;

    #[test]
    fn respond_reroll_offer() {
        let prompt = AgentPrompt::ReRollOffer {
            source: ReRollSource::new("TeamReRoll"),
            action: "Dodge".into(),
            team_id: "home".into(),
        };
        let resp = RandomAgent::new(42).respond(&prompt);
        assert!(matches!(resp, AgentResponse::UseReRoll { .. }));
    }

    #[test]
    fn respond_follow_up() {
        let prompt = AgentPrompt::FollowUp {
            attacker_id: "att".into(),
            target_coord: FieldCoordinate::new(10, 7),
        };
        let resp = RandomAgent::new(42).respond(&prompt);
        assert!(matches!(resp, AgentResponse::FollowUp { .. }));
    }

    #[test]
    fn respond_activate_player() {
        let prompt = AgentPrompt::ActivatePlayer {
            eligible_players: vec![
                ("p1".into(), vec![PlayerAction::Move]),
                ("p2".into(), vec![PlayerAction::Move, PlayerAction::Block]),
            ],
        };
        let resp = RandomAgent::new(42).respond(&prompt);
        assert!(matches!(resp, AgentResponse::Confirm),
            "respond() returns Confirm for ActivatePlayer (use act() instead)");
    }

    #[test]
    fn respond_block_choice() {
        let prompt = AgentPrompt::BlockChoice {
            attacker_id: "att".into(),
            defender_id: "def".into(),
            dice: vec![1, 3, 5],
            own_choice: true,
            nr_of_dice: 2,
        };
        let resp = RandomAgent::new(42).respond(&prompt);
        assert!(matches!(resp, AgentResponse::BlockChoice { .. }));
    }

    #[test]
    fn use_skill_uses_skill_id_from_prompt() {
        let dodge_idx = SKILL_TABLE.iter().position(|s| s.id == SkillId::Dodge).unwrap_or(0) as u16;
        let prompt = AgentPrompt::SkillUse {
            player_id: "p1".into(),
            skill_id: dodge_idx,
            skill_name: "Dodge".into(),
        };
        let response = AgentResponse::UseSkill { use_skill: true };
        let action = response_to_action(response, Some(&prompt));
        assert!(matches!(action, Action::UseSkill { skill_id: SkillId::Dodge, use_skill: true }));
    }

    #[test]
    fn use_apothecary_uses_player_id_from_prompt() {
        let prompt = AgentPrompt::UseApothecary {
            player_id: "injured_player".into(),
            apothecary_type: "standard".into(),
        };
        let response = AgentResponse::ApothecaryChoice { heal: true };
        let action = response_to_action(response, Some(&prompt));
        assert!(matches!(action, Action::UseApothecary { ref player_id, use_apothecary: true } if player_id == "injured_player"));
    }
}
