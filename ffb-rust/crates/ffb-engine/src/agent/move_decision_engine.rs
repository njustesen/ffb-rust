/// Translated from `ffb-ai/MoveDecisionEngine.java` + `ActionScore.java` + `PolicySampler.java`.
///
/// Provides stateless scoring helpers used by a strategic agent to select players
/// and move targets based on PathProbabilityFinder output.
///
/// Usage: call [`MoveDecisionEngine::select_player`] and [`MoveDecisionEngine::select_move`]
/// with read-only access to the game state.  For an end-to-end game loop, use
/// [`run_game_with_mde`] which passes the engine state into the decision functions.

use rand_chacha::ChaCha8Rng;
use rand::{SeedableRng, Rng};

use ffb_model::model::game::Game;
use ffb_model::model::player::PlayerId;
use ffb_model::types::FieldCoordinate;
use ffb_model::enums::{PS_STANDING, PS_PRONE, PlayerAction};
use ffb_mechanics::mechanics::path_probability::{
    find_all_paths, OpponentOnField, PathContext, PathEntry, PlayerMoveContext,
};
use ffb_mechanics::skills::SkillId;

use crate::engine::GameEngine;
use crate::legal_actions::TeamSide;

// ── Softmax temperatures ───────────────────────────────────────────────────

pub const T_PLAYER: f64  = 0.50;
pub const T_MOVE:   f64  = 0.60;

// ── ActionScore ────────────────────────────────────────────────────────────

/// A scored AI decision: probability × value × confidence.
#[derive(Debug, Clone, Copy)]
pub struct ActionScore {
    pub probability: f64,  // [0, 1]
    pub value:       f64,  // [-1, 1]
    pub confidence:  f64,  // [0, 1]
}

impl ActionScore {
    pub fn new(probability: f64, value: f64, confidence: f64) -> Self {
        ActionScore {
            probability: probability.clamp(0.0, 1.0),
            value:       value.clamp(-1.0, 1.0),
            confidence:  confidence.clamp(0.0, 1.0),
        }
    }

    /// Raw product p × v × c ∈ [-1, 1].
    pub fn score(&self) -> f64 {
        self.probability * self.value * self.confidence
    }

    /// Shifted to [0, 2] for softmax input: 1.0 + score().
    pub fn softmax_score(&self) -> f64 {
        1.0 + self.score()
    }
}

// ── PolicySampler ──────────────────────────────────────────────────────────

/// Convert raw scores to a softmax probability distribution.
pub fn softmax(scores: &[f64], temperature: f64) -> Vec<f64> {
    if scores.is_empty() { return vec![]; }
    let max = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mut probs: Vec<f64> = scores.iter().map(|&s| ((s - max) / temperature).exp()).collect();
    let sum: f64 = probs.iter().sum();
    probs.iter_mut().for_each(|p| *p /= sum);
    probs
}

/// Index of the highest score.
pub fn argmax(scores: &[f64]) -> usize {
    scores.iter().enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(0)
}

/// Sample an index from the softmax distribution.
pub fn sample(scores: &[f64], temperature: f64, rng: &mut ChaCha8Rng) -> usize {
    let probs = softmax(scores, temperature);
    let r: f64 = rng.gen();
    let mut cumulative = 0.0;
    for (i, &p) in probs.iter().enumerate() {
        cumulative += p;
        if r < cumulative { return i; }
    }
    probs.len() - 1
}

// ── Result types ───────────────────────────────────────────────────────────

/// Result of [`MoveDecisionEngine::select_player`].
/// `player_id == None` means end the turn.
pub struct PlayerDecision {
    pub player_id: Option<PlayerId>,
    pub action: Option<PlayerAction>,
    pub raw_scores: Vec<f64>,
}

impl PlayerDecision {
    pub fn is_end_turn(&self) -> bool { self.player_id.is_none() }
}

/// Result of [`MoveDecisionEngine::select_move`].
/// `target == None` means end the activation.
pub struct MoveDecision {
    pub target: Option<FieldCoordinate>,
    pub path_entry: Option<PathEntry>,
    pub raw_scores: Vec<f64>,
    pub is_ball_carrier: bool,
    pub is_retriever: bool,
    pub is_receiver: bool,
}

impl MoveDecision {
    pub fn is_end_action(&self) -> bool { self.target.is_none() }

    pub fn role(&self) -> &'static str {
        if self.is_ball_carrier { "ball carrier" }
        else if self.is_retriever { "retriever" }
        else if self.is_receiver { "receiver" }
        else { "support" }
    }
}

// ── MoveDecisionEngine ─────────────────────────────────────────────────────

pub struct MoveDecisionEngine {
    pub rng: ChaCha8Rng,
    /// true = argmax (deterministic); false = sample from softmax distribution.
    pub argmax: bool,
}

impl MoveDecisionEngine {
    pub fn new(seed: u64) -> Self {
        MoveDecisionEngine { rng: ChaCha8Rng::seed_from_u64(seed), argmax: false }
    }

    pub fn deterministic(seed: u64) -> Self {
        MoveDecisionEngine { rng: ChaCha8Rng::seed_from_u64(seed), argmax: true }
    }

    // ── Player selection ───────────────────────────────────────────────────

    /// Score and select which player to activate (and which action) for `side`.
    pub fn select_player(
        &mut self,
        game: &Game,
        side: TeamSide,
        allow_block: bool,
    ) -> PlayerDecision {
        let is_home = matches!(side, TeamSide::Home);
        let my_team    = if is_home { &game.team_home } else { &game.team_away };
        let opp_team   = if is_home { &game.team_away } else { &game.team_home };

        let ball_coord = game.field_model.ball_coordinate;
        let ball_carrier_id: Option<PlayerId> = ball_coord.and_then(|bc| {
            game.field_model.player_at(bc).cloned()
        });
        let opponent_has_ball = ball_carrier_id.as_ref()
            .map(|id| opp_team.has_player(id))
            .unwrap_or(false);
        let ball_is_loose = ball_coord.is_some() && ball_carrier_id.is_none();

        let remaining = my_team.players.iter().filter(|p| {
            game.field_model.player_state(&p.id)
                .map(|s| s.is_standing() && s.is_active())
                .unwrap_or(false)
        }).count() as i32;

        let mut cand_ids:     Vec<Option<PlayerId>>   = Vec::new();
        let mut cand_actions: Vec<Option<PlayerAction>> = Vec::new();
        let mut cand_scores:  Vec<f64>                = Vec::new();

        for p in &my_team.players {
            let coord = match game.field_model.player_coordinate(&p.id) { Some(c) => c, None => continue };
            let state = match game.field_model.player_state(&p.id) { Some(s) => s, None => continue };
            if !state.is_active() { continue; }
            let standing = state.base() == PS_STANDING;
            let prone    = state.base() == PS_PRONE;
            if !standing && !prone { continue; }

            if prone {
                // Prone player: stand up (costs 3 MA)
                let score = 0.5 * 0.75; // discounted
                cand_ids.push(Some(p.id.clone()));
                cand_actions.push(Some(PlayerAction::Move));
                cand_scores.push(1.0 + score);
                continue;
            }

            // Ball carrier → MOVE
            if ball_coord.map(|bc| bc == coord).unwrap_or(false) {
                let score = best_activation_score(game, p.id.clone(), p.movement, p.agility,
                    ball_coord, coord, true, false, false, is_home, game.rules);
                cand_ids.push(Some(p.id.clone()));
                cand_actions.push(Some(PlayerAction::Move));
                cand_scores.push(score);
                continue;
            }

            // Opponent has ball — consider blitz/block on carrier
            if opponent_has_ball {
                if let Some(ref bc_id) = ball_carrier_id {
                    if let Some(bc_coord) = game.field_model.player_coordinate(bc_id) {
                        let dist = chebyshev(coord, bc_coord);
                        let bp = block_probability_coords(p.strength, opp_team.player(bc_id)
                            .map(|q| q.strength).unwrap_or(3), 0, 0);
                        if allow_block && dist > 1 && dist <= p.movement + 1 {
                            cand_ids.push(Some(p.id.clone()));
                            cand_actions.push(Some(PlayerAction::Blitz));
                            cand_scores.push(ActionScore::new(bp, 0.80, 0.75).softmax_score());
                        }
                        if allow_block && dist == 1 && state.has_tacklezones() {
                            cand_ids.push(Some(p.id.clone()));
                            cand_actions.push(Some(PlayerAction::Block));
                            cand_scores.push(ActionScore::new(bp, 0.75, 0.70).softmax_score());
                        }
                    }
                }
            }

            // Nearest mover to loose ball
            if ball_is_loose {
                if let Some(bc) = ball_coord {
                    if chebyshev(coord, bc) <= p.movement {
                        let score = best_activation_score(game, p.id.clone(), p.movement, p.agility,
                            ball_coord, coord, false, true, false, is_home, game.rules);
                        cand_ids.push(Some(p.id.clone()));
                        cand_actions.push(Some(PlayerAction::Move));
                        cand_scores.push(score);
                        continue;
                    }
                }
            }

            // Block adjacent opponent
            if allow_block && state.has_tacklezones() {
                let has_adj_opp = opp_team.players.iter().any(|q| {
                    game.field_model.player_coordinate(&q.id)
                        .map(|qc| qc.is_adjacent(coord))
                        .unwrap_or(false)
                    && game.field_model.player_state(&q.id)
                        .map(|qs| qs.can_be_blocked())
                        .unwrap_or(false)
                });
                if has_adj_opp {
                    let best_bp = opp_team.players.iter()
                        .filter(|q| game.field_model.player_coordinate(&q.id)
                            .map(|qc| qc.is_adjacent(coord)).unwrap_or(false))
                        .map(|q| block_probability_coords(p.strength, q.strength, 0, 0))
                        .fold(0.0_f64, f64::max);
                    cand_ids.push(Some(p.id.clone()));
                    cand_actions.push(Some(PlayerAction::Block));
                    cand_scores.push(ActionScore::new(best_bp, 0.50, 0.65).softmax_score());
                }
            }

            // Receiver (has Catch skill)
            let is_receiver = p.has_skill(SkillId::Catch);
            if is_receiver {
                let score = best_activation_score(game, p.id.clone(), p.movement, p.agility,
                    ball_coord, coord, false, false, true, is_home, game.rules);
                cand_ids.push(Some(p.id.clone()));
                cand_actions.push(Some(PlayerAction::Move));
                cand_scores.push(score);
                continue;
            }

            // Support → MOVE
            let score = best_activation_score(game, p.id.clone(), p.movement, p.agility,
                ball_coord, coord, false, false, false, is_home, game.rules);
            cand_ids.push(Some(p.id.clone()));
            cand_actions.push(Some(PlayerAction::Move));
            cand_scores.push(score);
        }

        // End turn
        let end_turn_value = -1.0 + 1.0 / remaining.max(1) as f64;
        cand_ids.push(None);
        cand_actions.push(None);
        cand_scores.push(ActionScore::new(1.0, end_turn_value, 0.30).softmax_score());

        let idx = if self.argmax || cand_scores.is_empty() {
            argmax(&cand_scores)
        } else {
            sample(&cand_scores, T_PLAYER, &mut self.rng)
        };

        PlayerDecision {
            player_id: cand_ids[idx].clone(),
            action: cand_actions[idx],
            raw_scores: cand_scores,
        }
    }

    // ── Move target selection ──────────────────────────────────────────────

    /// Score all reachable squares and select a move target for the acting player.
    pub fn select_move(
        &mut self,
        game: &Game,
        player_id: &str,
        current_move: i32,
        side: TeamSide,
    ) -> MoveDecision {
        let is_home = matches!(side, TeamSide::Home);
        let my_team  = if is_home { &game.team_home } else { &game.team_away };
        let opp_team = if is_home { &game.team_away } else { &game.team_home };
        let rules = game.rules;

        let player = match my_team.player(player_id) { Some(p) => p, None => {
            return MoveDecision { target: None, path_entry: None, raw_scores: vec![], is_ball_carrier: false, is_retriever: false, is_receiver: false };
        }};
        let coord = match game.field_model.player_coordinate(player_id) { Some(c) => c, None => {
            return MoveDecision { target: None, path_entry: None, raw_scores: vec![], is_ball_carrier: false, is_retriever: false, is_receiver: false };
        }};
        let ball_coord = game.field_model.ball_coordinate;

        let is_ball_carrier = ball_coord.map(|bc| bc == coord).unwrap_or(false);
        let is_receiver = !is_ball_carrier && player.has_skill(SkillId::Catch);

        // Build opponent list for PathContext
        let occupied: std::collections::HashSet<FieldCoordinate> = {
            let mut h = std::collections::HashSet::new();
            for pl in my_team.players.iter().chain(opp_team.players.iter()) {
                if pl.id != player_id {
                    if let Some(c) = game.field_model.player_coordinate(&pl.id) { h.insert(c); }
                }
            }
            h
        };
        let opponents: Vec<OpponentOnField> = opp_team.players.iter().filter_map(|q| {
            let qc = game.field_model.player_coordinate(&q.id)?;
            let qs = game.field_model.player_state(&q.id)?;
            Some(OpponentOnField {
                coord: qc,
                has_tackle_zones: qs.has_tacklezones(),
                has_diving_tackle: q.has_skill(SkillId::DivingTackle),
                has_prehensile_tail: q.has_skill(SkillId::PrehensileTail),
                has_disturbing_presence: q.has_skill(SkillId::DisturbingPresence),
                is_titchy: q.has_skill(SkillId::Titchy),
            })
        }).collect();

        let ignore_tz = player.has_skill(SkillId::Incorporeal)
            || player.has_skill(SkillId::TwoHeads); // TwoHeads doesn't truly ignore but simplify
        let player_ctx = PlayerMoveContext {
            start: coord,
            movement_allowance: player.movement,
            current_move,
            agility: player.agility,
            strength: player.strength,
            rules,
            has_two_heads: player.has_skill(SkillId::TwoHeads),
            ignore_tackle_zones: ignore_tz && !player.has_skill(SkillId::TwoHeads),
            has_break_tackle: player.has_skill(SkillId::BreakTackle),
            gfi_modifier_total: 0,
            extra_gfi: if player.has_skill(SkillId::Sprint) { 1 } else { 0 },
        };
        let field_ctx = PathContext { occupied, opponents };
        let path_map = find_all_paths(&player_ctx, &field_ctx);

        let is_retriever = !is_ball_carrier && !is_receiver
            && ball_coord.map(|bc| path_map.contains_key(&bc)).unwrap_or(false);

        let failure_cost = role_dice_failure_cost(is_ball_carrier, is_retriever);

        let can_end_now = current_move > 0;

        let mut candidates: Vec<FieldCoordinate> = path_map.keys().cloned().collect();
        let n = candidates.len();
        let total = if can_end_now { n + 1 } else { n };
        let mut scores = vec![0.0_f64; total];
        let mut entries: Vec<Option<&PathEntry>> = vec![None; total];

        for (i, &cand) in candidates.iter().enumerate() {
            let entry = &path_map[&cand];
            let base = move_base_score(cand, player.movement, current_move,
                is_ball_carrier, is_retriever, is_receiver, ball_coord, coord, is_home);
            let effective = entry.probability * base.value * base.confidence
                - (1.0 - entry.probability) * failure_cost;
            scores[i] = 1.0 + effective;
            entries[i] = Some(entry);
        }

        if can_end_now {
            let end_base = move_base_score(coord, player.movement, current_move,
                is_ball_carrier, is_retriever, is_receiver, ball_coord, coord, is_home);
            scores[n] = 1.0 + end_base.value * end_base.confidence * 0.9;
        }

        if total == 0 {
            return MoveDecision {
                target: None, path_entry: None, raw_scores: scores,
                is_ball_carrier, is_retriever, is_receiver,
            };
        }

        let idx = if self.argmax { argmax(&scores) } else { sample(&scores, T_MOVE, &mut self.rng) };

        if can_end_now && idx == n {
            MoveDecision {
                target: None, path_entry: None, raw_scores: scores,
                is_ball_carrier, is_retriever, is_receiver,
            }
        } else {
            let target = candidates[idx];
            MoveDecision {
                target: Some(target),
                path_entry: entries[idx].cloned(),
                raw_scores: scores,
                is_ball_carrier, is_retriever, is_receiver,
            }
        }
    }
}

// ── Public scoring helpers ─────────────────────────────────────────────────

/// Block probability from attacker/defender effective strength.
///
/// Probabilities match Java's table:
/// 2× ST → 0.70, 1×ST advantage → 0.56, equal → 0.33, 2× underdog → 0.04
pub fn block_probability_coords(att_str: i32, def_str: i32, off_assists: i32, def_assists: i32) -> f64 {
    let eff_att = (att_str + off_assists).max(1);
    let eff_def = (def_str + def_assists).max(1);
    if      eff_att >= 2 * eff_def { 0.70 }
    else if eff_att >      eff_def { 0.56 }
    else if eff_att ==     eff_def { 0.33 }
    else if eff_att * 2 >= eff_def { 0.11 }
    else                           { 0.04 }
}

/// Normalized advance score toward opponent endzone. 1.0 = opponent endzone x=25 (home).
pub fn advance_score(coord: FieldCoordinate, is_home: bool) -> f64 {
    if is_home { coord.x as f64 / 25.0 } else { (25 - coord.x) as f64 / 25.0 }
}

/// Chebyshev (king) distance between two squares.
pub fn chebyshev(a: FieldCoordinate, b: FieldCoordinate) -> i32 {
    (a.x - b.x).abs().max((a.y - b.y).abs())
}

/// Distance to the opponent's endzone (0 = at endzone).
pub fn endzone_distance(coord: FieldCoordinate, is_home: bool) -> i32 {
    if is_home { 25 - coord.x } else { coord.x }
}

/// Dice failure cost by role: higher penalty for risky positions.
pub fn role_dice_failure_cost(is_ball_carrier: bool, is_retriever: bool) -> f64 {
    if is_ball_carrier { 0.50 } else if is_retriever { 0.40 } else { 0.25 }
}

// ── Private helpers ────────────────────────────────────────────────────────

/// Score for a move target square by role.
fn move_base_score(
    coord: FieldCoordinate,
    ma: i32,
    current_move: i32,
    is_ball_carrier: bool,
    is_retriever: bool,
    is_receiver: bool,
    ball_coord: Option<FieldCoordinate>,
    player_coord: FieldCoordinate,
    is_home: bool,
) -> ActionScore {
    if is_ball_carrier {
        if endzone_distance(coord, is_home) == 0 {
            return ActionScore::new(1.0, 1.0, 1.0);
        }
        let adv = advance_score(coord, is_home);
        let advancing = advance_score(coord, is_home) > advance_score(player_coord, is_home);
        return if advancing {
            ActionScore::new(1.0, adv * 0.6, 0.50)
        } else {
            ActionScore::new(1.0, adv * 0.3 - 0.1, 0.30)
        };
    }
    if is_retriever {
        if let Some(bc) = ball_coord {
            let dist = chebyshev(coord, bc);
            return match dist {
                0 => ActionScore::new(1.0, 1.0,  0.95),
                1 => ActionScore::new(1.0, 0.80, 0.80),
                _ => ActionScore::new(1.0, (0.7 - 0.1 * dist as f64).max(0.05), 0.55),
            };
        }
        return ActionScore::new(1.0, 0.0, 0.30);
    }
    if is_receiver {
        let moves_remaining = ma - current_move;
        if endzone_distance(coord, is_home) <= moves_remaining {
            return ActionScore::new(1.0, 0.70, 0.75);
        }
        return ActionScore::new(1.0, advance_score(coord, is_home) * 0.5, 0.50);
    }
    // Support
    let curr_edz = endzone_distance(player_coord, is_home);
    let advancing = endzone_distance(coord, is_home) <= curr_edz;
    if advancing { ActionScore::new(1.0, 0.10, 0.30) } else { ActionScore::new(1.0, 0.0, 0.30) }
}

/// Compute the best activation softmax score by scanning all reachable squares.
fn best_activation_score(
    game: &Game,
    player_id: PlayerId,
    ma: i32,
    agility: i32,
    ball_coord: Option<FieldCoordinate>,
    player_coord: FieldCoordinate,
    is_ball_carrier: bool,
    is_retriever: bool,
    is_receiver: bool,
    is_home: bool,
    rules: ffb_model::enums::Rules,
) -> f64 {
    let my_team  = if is_home { &game.team_home } else { &game.team_away };
    let opp_team = if is_home { &game.team_away } else { &game.team_home };

    let occupied: std::collections::HashSet<FieldCoordinate> = {
        let mut h = std::collections::HashSet::new();
        for pl in my_team.players.iter().chain(opp_team.players.iter()) {
            if pl.id != player_id {
                if let Some(c) = game.field_model.player_coordinate(&pl.id) { h.insert(c); }
            }
        }
        h
    };
    let opponents: Vec<OpponentOnField> = opp_team.players.iter().filter_map(|q| {
        let qc = game.field_model.player_coordinate(&q.id)?;
        let qs = game.field_model.player_state(&q.id)?;
        Some(OpponentOnField {
            coord: qc,
            has_tackle_zones: qs.has_tacklezones(),
            has_diving_tackle: q.has_skill(SkillId::DivingTackle),
            has_prehensile_tail: q.has_skill(SkillId::PrehensileTail),
            has_disturbing_presence: q.has_skill(SkillId::DisturbingPresence),
            is_titchy: q.has_skill(SkillId::Titchy),
        })
    }).collect();

    let player_ctx = PlayerMoveContext {
        start: player_coord,
        movement_allowance: ma,
        current_move: 0,
        agility,
        strength: 3,
        rules,
        has_two_heads: false,
        ignore_tackle_zones: false,
        has_break_tackle: false,
        gfi_modifier_total: 0,
        extra_gfi: 0,
    };
    let field_ctx = PathContext { occupied, opponents };
    let path_map = find_all_paths(&player_ctx, &field_ctx);

    if path_map.is_empty() { return 0.0; }

    let failure_cost = role_dice_failure_cost(is_ball_carrier, is_retriever);
    let best: f64 = path_map.iter().map(|(&cand, entry)| {
        let base = move_base_score(cand, ma, 0, is_ball_carrier, is_retriever, is_receiver,
            ball_coord, player_coord, is_home);
        entry.probability * base.value * base.confidence - (1.0 - entry.probability) * failure_cost
    }).fold(0.0_f64, f64::max);

    1.0 + best
}

// ── Game loop integration ─────────────────────────────────────────────────

/// Run a full game using `MoveDecisionEngine` for both teams.
///
/// The engine provides game state access that `MoveDecisionEngine` needs for
/// scoring decisions. Returns all events emitted during the game.
pub fn run_game_with_mde(
    engine: &mut GameEngine,
    home_mde: &mut MoveDecisionEngine,
    away_mde:  &mut MoveDecisionEngine,
) -> Vec<ffb_model::events::GameEvent> {
    use ffb_model::prompts::{AgentPrompt, AgentResponse};
    use crate::agent::{response_to_action_pub, Agent, RandomAgent};

    // Fall back to RandomAgent for complex prompts MDE doesn't yet handle specifically.
    let mut fallback_home = RandomAgent::new(home_mde.rng.gen());
    let mut fallback_away = RandomAgent::new(away_mde.rng.gen());

    let mut all_events = Vec::new();
    let max_steps = 10_000;
    let mut steps = 0;

    while !engine.is_finished() && steps < max_steps {
        let prompt = match engine.current_prompt() { Some(p) => p.clone(), None => break };
        let side = engine.active_side();
        let is_home = matches!(side, TeamSide::Home);
        let mde = if is_home { &mut *home_mde } else { &mut *away_mde };
        let fallback = if is_home { &mut fallback_home } else { &mut fallback_away };

        let response = match &prompt {
            AgentPrompt::ActivatePlayer { eligible_players } => {
                // Use MDE to pick among eligible players
                let decision = mde.select_player(&engine.game, side, true);
                if decision.is_end_turn() {
                    AgentResponse::ActivatePlayer {
                        player_id: eligible_players.first()
                            .map(|(id, _)| id.clone())
                            .unwrap_or_default(),
                        action: ffb_model::enums::PlayerAction::Move,
                    }
                } else {
                    let pid = decision.player_id.unwrap_or_default();
                    // Find the player in eligible list; fall back to first if not found
                    let (found_id, actions) = eligible_players.iter()
                        .find(|(id, _)| *id == pid)
                        .or_else(|| eligible_players.first())
                        .map(|(id, acts)| (id.clone(), acts.clone()))
                        .unwrap_or_default();
                    let action = actions.first().cloned().unwrap_or(ffb_model::enums::PlayerAction::Move);
                    AgentResponse::ActivatePlayer { player_id: found_id, action }
                }
            }
            AgentPrompt::HitAndRun { squares, .. } => {
                if squares.is_empty() {
                    AgentResponse::Pushback { coord: ffb_model::types::FieldCoordinate::new(0, 0) }
                } else {
                    // For MDE, end the HitAndRun (pass None to stop moving)
                    fallback.respond(&prompt)
                }
            }
            _ => fallback.respond(&prompt),
        };

        let action = response_to_action_pub(response, Some(&prompt));
        match engine.apply(side, action) {
            Ok(events) => all_events.extend(events),
            Err(_) => break,
        }
        steps += 1;
    }

    all_events
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::types::FieldCoordinate;

    // ── ActionScore ────────────────────────────────────────────────────────

    #[test]
    fn action_score_product_is_correct() {
        let s = ActionScore::new(0.5, 0.8, 0.6);
        let expected = 0.5 * 0.8 * 0.6;
        assert!((s.score() - expected).abs() < 1e-10, "score={} expected={}", s.score(), expected);
        assert!((s.softmax_score() - (1.0 + expected)).abs() < 1e-10);
    }

    #[test]
    fn action_score_clamps_inputs() {
        let s = ActionScore::new(2.0, -5.0, -1.0);
        assert_eq!(s.probability, 1.0);
        assert_eq!(s.value, -1.0);
        assert_eq!(s.confidence, 0.0);
    }

    // ── PolicySampler ──────────────────────────────────────────────────────

    #[test]
    fn softmax_sums_to_one() {
        let scores = [1.0, 2.0, 0.5, 1.5];
        let probs = softmax(&scores, 0.5);
        assert!((probs.iter().sum::<f64>() - 1.0).abs() < 1e-10);
        assert!(probs.iter().all(|&p| p >= 0.0));
    }

    #[test]
    fn argmax_finds_largest() {
        let scores = [0.5, 1.5, 1.2, 0.8];
        assert_eq!(argmax(&scores), 1);
    }

    #[test]
    fn argmax_empty_returns_zero() {
        assert_eq!(argmax(&[]), 0);
    }

    // ── Scoring helpers ────────────────────────────────────────────────────

    #[test]
    fn block_prob_equal_strength_is_third() {
        assert!((block_probability_coords(3, 3, 0, 0) - 0.33).abs() < 1e-9);
    }

    #[test]
    fn block_prob_double_strength_is_seventy_percent() {
        assert!((block_probability_coords(6, 3, 0, 0) - 0.70).abs() < 1e-9);
    }

    #[test]
    fn advance_score_home_endzone_is_one() {
        let sq = FieldCoordinate::new(25, 7);
        assert!((advance_score(sq, true) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn advance_score_away_endzone_is_one() {
        let sq = FieldCoordinate::new(0, 7);
        assert!((advance_score(sq, false) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn endzone_distance_home_at_endzone_is_zero() {
        assert_eq!(endzone_distance(FieldCoordinate::new(25, 7), true), 0);
    }

    #[test]
    fn chebyshev_adjacent_is_one() {
        assert_eq!(chebyshev(FieldCoordinate::new(5, 5), FieldCoordinate::new(6, 6)), 1);
    }

    // ── MoveDecisionEngine integration ─────────────────────────────────────

    #[test]
    fn run_game_with_mde_terminates() {
        use crate::engine::GameEngine;
        use ffb_model::enums::Rules;
        use ffb_model::model::{team::Team, player::Player};
        use ffb_model::enums::{PlayerGender, PlayerType};

        fn make_team(name: &str, n: usize) -> Team {
            let players = (0..n).map(|i| Player {
                id: format!("{name}{i}"), name: format!("{name}{i}"), nr: i as i32,
                position_id: String::new(), player_type: PlayerType::Regular,
                gender: PlayerGender::Neutral, movement: 6, strength: 3, agility: 3,
                passing: 4, armour: 8, starting_skills: vec![], extra_skills: vec![],
                temporary_skills: vec![], used_skills: Default::default(),
                niggling_injuries: 0, stat_injuries: vec![], current_spps: 0,
                career_spps: 0, race: None,
            }).collect();
            Team { id: name.into(), name: name.into(), race: String::new(),
                roster_id: String::new(), coach: String::new(), rerolls: 2,
                apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0,
                bloodweiser_kegs: 0, riotous_rookies: 0, fan_factor: 5,
                assistant_coaches: 0, cheerleaders: 0, dedicated_fans: 5,
                treasury: 0, team_value: 0, players, special_rules: vec![] }
        }

        let mut engine = GameEngine::new(make_team("h", 11), make_team("a", 11), Rules::Bb2020, 42);
        let mut home_mde = MoveDecisionEngine::new(1);
        let mut away_mde = MoveDecisionEngine::new(2);
        let events = run_game_with_mde(&mut engine, &mut home_mde, &mut away_mde);
        assert!(events.len() > 0, "game must produce events");
    }
}
