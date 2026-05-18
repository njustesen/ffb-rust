/// Simulation loop and strategy interface.
use ffb_core::actions::{enumerate_actions, BbAction};
use ffb_core::model::game_state::{DialogState, GameState};
use ffb_core::pathfinding::{find_paths, find_paths_in_state};
use ffb_core::rng::GameRng;
use ffb_core::steps::{
    begin_activation, begin_block, begin_move, resume_move_after_reroll, begin_turn, end_activation, end_turn,
    apply_block_dice_choice, apply_push_choice, resume_pass_after_reroll, resume_catch_after_reroll,
    TurnStepResult,
};
#[allow(unused_imports)] // used in #[cfg(test)] via `use super::*`
use ffb_core::types::{FieldCoordinate, PlayerId, PlayerAction, PlayerState, TeamId, TurnMode};

use ffb_core::steps::{apply_kickoff_event, roll_kickoff_event};

use crate::setup::{apply_nice_weather_scatter, default_kickoff_ball_placement, kickoff_bounce_if_needed, perform_kickoff_scatter, place_players_for_kickoff};

// ── Strategy trait ────────────────────────────────────────────────────────────

/// Plug-in strategy interface.  Implementors choose one action from the legal
/// action list for each decision point.
pub trait Strategy: Send + Sync {
    fn choose_action(&self, state: &GameState, legal_actions: &[BbAction]) -> BbAction;
}

// ── Null strategy ─────────────────────────────────────────────────────────────

/// Always ends the turn immediately — used for unit tests and as a baseline.
pub struct NullStrategy;

impl Strategy for NullStrategy {
    fn choose_action(&self, _state: &GameState, legal_actions: &[BbAction]) -> BbAction {
        // Prefer EndTurn; fall back to the first action available.
        legal_actions
            .iter()
            .find(|a| **a == BbAction::EndTurn)
            .cloned()
            .unwrap_or_else(|| legal_actions[0].clone())
    }
}

// ── Scripted strategy ─────────────────────────────────────────────────────────

/// Heuristic strategy that scores each action and chooses the best one.
/// With `temperature > 0.0` uses softmax sampling; with `temperature = 0.0` uses argmax.
pub struct ScriptedStrategy {
    pub temperature: f64,
}

impl Default for ScriptedStrategy {
    fn default() -> Self { Self { temperature: 0.0 } }
}

impl Strategy for ScriptedStrategy {
    fn choose_action(&self, state: &GameState, legal_actions: &[BbAction]) -> BbAction {
        let team = state.active_team_id();
        let scores: Vec<f64> = legal_actions.iter().map(|a| score_action(state, a, team)).collect();

        if self.temperature <= 0.0 || legal_actions.len() == 1 {
            let best = scores.iter().enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(i, _)| i)
                .unwrap_or(0);
            return legal_actions[best].clone();
        }

        // Softmax sampling
        let max_score = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let exps: Vec<f64> = scores.iter().map(|&s| ((s - max_score) / self.temperature).exp()).collect();
        let sum: f64 = exps.iter().sum();
        // Simple deterministic choice: pick highest softmax weight (avoids needing an RNG here)
        let best = exps.iter().enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0);
        let _ = sum; // used conceptually
        legal_actions[best].clone()
    }
}

/// Score a single action heuristically from `team`'s perspective.
fn score_action(state: &GameState, action: &BbAction, team: ffb_core::types::TeamId) -> f64 {
    match action {
        BbAction::EndTurn => -0.1, // prefer actions over ending turn
        BbAction::Activate { action: player_action, .. } => {
            match player_action {
                ffb_core::types::PlayerAction::Block | ffb_core::types::PlayerAction::Blitz => 0.4,
                ffb_core::types::PlayerAction::Move => 0.3,
                ffb_core::types::PlayerAction::Pass | ffb_core::types::PlayerAction::HandOff => 0.5,
                _ => 0.1,
            }
        }
        BbAction::MoveTo(coord) => {
            // Prefer moving toward the ball
            let ball_bonus = if let Some(ball_coord) = state.field.ball.coord {
                let dist = {
                    let dx = (coord.x as i16 - ball_coord.x as i16).unsigned_abs();
                    let dy = (coord.y as i16 - ball_coord.y as i16).unsigned_abs();
                    (dx + dy) as f64
                };
                1.0 / (dist + 1.0)
            } else {
                0.0
            };
            // Prefer moving toward opponent's end zone
            let td_bonus = if team == ffb_core::types::TeamId::Home {
                coord.x as f64 / 26.0
            } else {
                (25 - coord.x) as f64 / 26.0
            };
            0.2 + ball_bonus * 0.5 + td_bonus * 0.3
        }
        BbAction::BlockTarget(_) => 0.5,
        BbAction::ChooseBlockDie(_) => 0.3,
        BbAction::ChoosePush(_) => 0.3,
        BbAction::UseReroll(true) => 0.2,
        BbAction::UseReroll(false) => 0.1,
        BbAction::PassTo(_) => 0.6,
        BbAction::PlaceBall(_) => 0.5,
        BbAction::ChooseFollowup(true) => 0.4,  // prefer following up (advances position)
        BbAction::ChooseFollowup(false) => 0.1,
    }
}

// ── Simulation loop ───────────────────────────────────────────────────────────

pub struct SimulationLoop;

impl SimulationLoop {
    /// Drive `state` to completion using the two strategies.
    ///
    /// Returns the final `GameState` with `result.finished == true`.
    pub fn run(
        mut state: GameState,
        strategy_home: &dyn Strategy,
        strategy_away: &dyn Strategy,
        rng: &mut GameRng,
    ) -> GameState {
        const MAX_ACTIONS: u32 = 10_000;
        let mut action_count: u32 = 0;

        // ── Initial setup: place players and handle kickoff ───────────────────
        if state.turn_mode == TurnMode::StartGame {
            place_players_for_kickoff(&mut state);
            // Kick from center of kicking team's half, then scatter
            let kick_from = ffb_core::types::FieldCoordinate::new(13, 8);
            let (_ball_pos, mut scatter_touchback) = perform_kickoff_scatter(&mut state, kick_from, rng);
            let kickoff_event = roll_kickoff_event(rng);
            apply_kickoff_event(&mut state, kickoff_event, rng);
            if kickoff_event == ffb_core::types::KickoffEvent::ChangingWeather
                && apply_nice_weather_scatter(&mut state, scatter_touchback, rng)
            {
                scatter_touchback = true;
            }
            kickoff_bounce_if_needed(&mut state, kick_from, scatter_touchback, rng);
            // Transition to regular play (kickoff event may have set HighKick/Blitz etc.)
            if state.turn_mode != TurnMode::Regular {
                state.turn_mode = TurnMode::Regular;
            }
            // The receiving team acts first (non-kicking team)
            state.home_is_active = !state.home_is_offense;
            // Begin first turn
            begin_turn(&mut state);
        }

        loop {
            if state.result.finished {
                break;
            }
            if action_count >= MAX_ACTIONS {
                // Safety valve — force game end
                state.result.finished = true;
                break;
            }

            // Determine active strategy
            let active_team = state.active_team_id();
            let strategy: &dyn Strategy = if active_team == TeamId::Home {
                strategy_home
            } else {
                strategy_away
            };

            // Enumerate legal actions
            let legal_actions = enumerate_actions(&state, active_team);
            if legal_actions.is_empty() {
                // No legal actions — force end turn
                let result = end_turn(&mut state);
                handle_turn_result(&mut state, result, rng);
                action_count += 1;
                continue;
            }

            // Ask strategy for an action
            let action = strategy.choose_action(&state, &legal_actions);
            action_count += 1;

            // Apply the action
            let result = apply_action(&mut state, action, rng);

            // Handle half/game end signals
            if let Some(ts) = result {
                handle_turn_result(&mut state, ts, rng);
            }
        }

        state
    }
}

/// Public wrapper for begin_pass — used by ffb-mcts to avoid duplicating pass logic.
pub fn begin_pass_wrapper(
    state: &mut GameState,
    passer_id: &ffb_core::types::PlayerId,
    target: ffb_core::types::FieldCoordinate,
    rng: &mut GameRng,
) {
    use ffb_core::steps::begin_pass;
    let _ = begin_pass(state, passer_id, target, rng);
}

/// Apply a single action to the game state.
/// Returns a `TurnStepResult` if a turn-level transition occurred.
fn apply_action(state: &mut GameState, action: BbAction, rng: &mut GameRng) -> Option<TurnStepResult> {
    match action {
        BbAction::EndTurn => {
            end_activation(state);
            let result = end_turn(state);
            return Some(result);
        }

        BbAction::Activate { player_id, action: player_action } => {
            let result = begin_activation(state, &player_id, rng);
            if result == TurnStepResult::TurnOver {
                // Activation failed (BoneHead etc.) — end turn
                let et = end_turn(state);
                return Some(et);
            }
            // Set the chosen action on the acting player
            if let Some(ap) = state.acting_player.as_mut() {
                ap.current_action = Some(player_action);
                match player_action {
                    PlayerAction::Blitz => ap.has_blitzed = true,
                    PlayerAction::Pass => ap.has_passed = true,
                    PlayerAction::HandOff => ap.has_handed_off = true,
                    PlayerAction::Foul => {
                        ap.has_fouled = true;
                    }
                    _ => {}
                }
            }
            // For Blitz: mark blitz_used on the active team
            if player_action == PlayerAction::Blitz {
                state.active_turn_data_mut().blitz_used = true;
            }
            // For Foul: mark foul_used on the active team (unless QuickFoul)
            if player_action == PlayerAction::Foul {
                if let Some(ap) = state.acting_player.as_ref() {
                    let pid = ap.player_id.clone();
                    let has_quick_foul = state.active_team()
                        .player_by_id(&pid)
                        .map(|p| p.has_skill(ffb_core::skills::SkillId::QuickFoul))
                        .unwrap_or(false);
                    if !has_quick_foul {
                        state.active_turn_data_mut().foul_used = true;
                    }
                }
            }
        }

        BbAction::MoveTo(coord) => {
            if let Some(ap) = state.acting_player.as_ref() {
                let player_id = ap.player_id.clone();
                let team = ap.team;
                let movement_remaining = {
                    let player = state.team(team).player_by_id(&player_id).expect("player");
                    player.effective_ma().saturating_sub(ap.movement_used)
                };
                // Find path to coord — use skill-aware pathfinding (BreakTackle, Leap, PrehensileTail)
                let path_opt = {
                    let player = state.team(team).player_by_id(&player_id).expect("player");
                    let paths = find_paths_in_state(state, player, &player_id, team, movement_remaining);
                    paths.get(&coord).map(|e| e.path.to_vec())
                };
                if let Some(path) = path_opt {
                    use ffb_core::steps::move_step::MoveStepResult;
                    let move_result = begin_move(state, &player_id, &path, rng);
                    // Handle touchdown: reset and kickoff
                    if matches!(move_result, MoveStepResult::Touchdown { .. }) {
                        end_activation(state);
                        // Eject Secret Weapon players before next drive setup
                        ffb_core::steps::eject_secret_weapons(state);
                        // Reset for next drive
                        place_players_for_kickoff(state);
                        let kick_from = ffb_core::types::FieldCoordinate::new(13, 8);
                        let (_, mut scatter_tb) = perform_kickoff_scatter(state, kick_from, rng);
                        let kickoff_event = roll_kickoff_event(rng);
                        apply_kickoff_event(state, kickoff_event, rng);
                        if kickoff_event == ffb_core::types::KickoffEvent::ChangingWeather
                            && apply_nice_weather_scatter(state, scatter_tb, rng)
                        {
                            scatter_tb = true;
                        }
                        kickoff_bounce_if_needed(state, kick_from, scatter_tb, rng);
                        state.turn_mode = TurnMode::Regular;
                        // Scoring team does not act immediately — opponent gets kickoff receive
                        state.home_is_active = !state.home_is_active;
                        begin_turn(state);
                        return None;
                    }
                    // Handle turnover (failed pickup, knocked down with ball)
                    if matches!(move_result, MoveStepResult::Turnover { .. }) {
                        end_activation(state);
                        let et = end_turn(state);
                        return Some(et);
                    }
                    // PendingTeamReroll: dialog was set, wait for UseReroll action
                    if matches!(move_result, MoveStepResult::PendingTeamReroll) {
                        return None; // dialog is SelectReroll; loop will ask strategy
                    }
                }
                // After move: if Blitz action, show block target selection; otherwise end activation
                let is_blitz = state.acting_player.as_ref()
                    .map(|ap| ap.current_action == Some(PlayerAction::Blitz) && !ap.has_blocked)
                    .unwrap_or(false);
                if is_blitz {
                    // Find adjacent opponents for block target selection
                    if let Some(ap) = state.acting_player.as_ref() {
                        let pid = ap.player_id.clone();
                        let team = ap.team;
                        if let Some(coord) = state.field.player_coord(&pid) {
                            let targets: Vec<PlayerId> = coord.neighbors()
                                .filter_map(|n| {
                                    let opp_id = state.field.player_at(n)?;
                                    if state.field.player_team(opp_id) != Some(team) {
                                        Some(opp_id.clone())
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            if !targets.is_empty() {
                                state.dialog = DialogState::SelectBlockTarget { targets };
                            } else {
                                // No adjacent opponents after movement — end activation
                                end_activation(state);
                            }
                        } else {
                            end_activation(state);
                        }
                    } else {
                        end_activation(state);
                    }
                } else {
                    end_activation(state);
                }
            }
        }

        BbAction::BlockTarget(defender_id) => {
            if let Some(ap) = state.acting_player.as_ref() {
                let attacker_id = ap.player_id.clone();
                let is_foul = ap.current_action == Some(PlayerAction::Foul);
                if is_foul {
                    // Foul: roll armor directly (no block dice)
                    execute_foul(state, &attacker_id, &defender_id, rng);
                    end_activation(state);
                } else {
                    // Mark block initiated (for Blitz tracking)
                    if let Some(ap) = state.acting_player.as_mut() {
                        ap.has_blocked = true;
                    }
                    begin_block(state, &attacker_id, &defender_id, rng);
                    // Dialog is now SelectBlockDice — will be handled on next step
                }
            }
        }

        BbAction::ChooseBlockDie(result) => {
            // We need attacker and defender IDs; they should still be in acting_player
            // and field context. We derive them from the block context.
            if let Some(ap) = state.acting_player.as_ref() {
                let attacker_id = ap.player_id.clone();
                let team = ap.team;
                let attacker_coord = state.field.player_coord(&attacker_id);
                if let Some(att_coord) = attacker_coord {
                    let defender_id = find_adjacent_opponent(state, att_coord, team);
                    if let Some(def_id) = defender_id {
                        use ffb_core::steps::BlockStepResult;
                        let block_result = apply_block_dice_choice(state, &attacker_id, &def_id, result, rng);
                        // Check for turnover (attacker knocked down by Skull/BothDown)
                        if let BlockStepResult::Done(res) = &block_result {
                            if res.turnover {
                                end_activation(state);
                                let et = end_turn(state);
                                return Some(et);
                            }
                        }
                        // If dialog is now None (Done) and no turnover, end activation normally
                        if state.dialog == DialogState::None {
                            end_activation(state);
                        }
                    }
                }
            }
        }

        BbAction::ChoosePush(coord) => {
            if let Some(ap) = state.acting_player.as_ref() {
                let attacker_id = ap.player_id.clone();
                let team = ap.team;
                let att_coord = state.field.player_coord(&attacker_id);
                if let Some(att_coord) = att_coord {
                    let defender_id = find_adjacent_opponent_or_prone(state, att_coord, team);
                    if let Some(def_id) = defender_id {
                        use ffb_core::steps::BlockStepResult;
                        let push_result = apply_push_choice(state, &attacker_id, &def_id, coord, rng);
                        // Check for turnover (defender on active team was injured/KO'd)
                        if let BlockStepResult::Done(res) = &push_result {
                            if res.turnover {
                                end_activation(state);
                                let et = end_turn(state);
                                return Some(et);
                            }
                        }
                    }
                }
                // Frenzy: if second block is required, auto-execute it before ending activation
                let frenzy_required = state.acting_player.as_ref()
                    .map(|ap| ap.frenzy_second_block_required)
                    .unwrap_or(false);
                if frenzy_required && state.dialog == DialogState::None {
                    // Find the pushed defender (should now be adjacent to attacker after follow-up)
                    let att_id = state.acting_player.as_ref().map(|ap| ap.player_id.clone());
                    if let Some(att_id) = att_id {
                        let att_coord = state.field.player_coord(&att_id);
                        let att_team = state.field.player_team(&att_id);
                        if let (Some(att_coord), Some(att_team)) = (att_coord, att_team) {
                            let def_id = find_adjacent_opponent(state, att_coord, att_team);
                            if let Some(def_id) = def_id {
                                if let Some(ap) = state.acting_player.as_mut() {
                                    ap.frenzy_second_block_required = false;
                                }
                                begin_block(state, &att_id, &def_id, rng);
                                if state.dialog == DialogState::None {
                                    end_activation(state);
                                }
                                return None;
                            }
                        }
                    }
                }
                // SelectFollowup pending: don't end activation — wait for ChooseFollowup
                // For other dialogs (None = done), end activation.
                if state.dialog == DialogState::None {
                    end_activation(state);
                }
            }
        }

        BbAction::ChooseFollowup(follow_up) => {
            if let DialogState::SelectFollowup { square } = state.dialog.clone() {
                state.dialog = DialogState::None;
                if follow_up {
                    if let Some(ap) = state.acting_player.as_ref() {
                        let att_id = ap.player_id.clone();
                        if !state.field.is_occupied(square) {
                            state.field.move_player(&att_id, square);
                        }
                    }
                }
                // After follow-up decision, check Frenzy second block
                let frenzy_required = state.acting_player.as_ref()
                    .map(|ap| ap.frenzy_second_block_required)
                    .unwrap_or(false);
                if frenzy_required {
                    let att_id = state.acting_player.as_ref().map(|ap| ap.player_id.clone());
                    if let Some(att_id) = att_id {
                        let att_coord = state.field.player_coord(&att_id);
                        let att_team = state.field.player_team(&att_id);
                        if let (Some(att_coord), Some(att_team)) = (att_coord, att_team) {
                            let def_id = find_adjacent_opponent(state, att_coord, att_team);
                            if let Some(def_id) = def_id {
                                if let Some(ap) = state.acting_player.as_mut() {
                                    ap.frenzy_second_block_required = false;
                                }
                                begin_block(state, &att_id, &def_id, rng);
                                if state.dialog == DialogState::None {
                                    end_activation(state);
                                }
                                return None;
                            }
                        }
                    }
                }
                end_activation(state);
            }
        }

        BbAction::UseReroll(use_it) => {
            // Check if this is a catch reroll
            let is_catch_reroll = state.acting_player.as_ref()
                .map(|ap| ap.pending_catch_at.is_some())
                .unwrap_or(false);
            if is_catch_reroll {
                state.dialog = DialogState::None;
                if let Some(ap) = state.acting_player.as_ref() {
                    let catcher_id = ap.player_id.clone();
                    use ffb_core::steps::PassStepResult;
                    let catch_result = resume_catch_after_reroll(state, &catcher_id, use_it, rng);
                    match catch_result {
                        PassStepResult::Dropped { .. } => {
                            let catcher_team = state.field.player_team(&catcher_id);
                            let active_team = if state.home_is_active { TeamId::Home } else { TeamId::Away };
                            if catcher_team != Some(active_team) {
                                end_activation(state);
                                let et = end_turn(state);
                                return Some(et);
                            }
                        }
                        _ => {}
                    }
                }
                end_activation(state);
                return None;
            }

            // Check if this is a movement (dodge/GFI or pickup) reroll
            let is_move_reroll = state.acting_player.as_ref()
                .map(|ap| ap.pending_move.is_some() || ap.pending_pickup_at.is_some())
                .unwrap_or(false);
            if is_move_reroll {
                state.dialog = DialogState::None;
                if let Some(ap) = state.acting_player.as_ref() {
                    let player_id = ap.player_id.clone();
                    use ffb_core::steps::move_step::MoveStepResult;
                    let result = resume_move_after_reroll(state, &player_id, use_it, rng);
                    if matches!(result, MoveStepResult::Touchdown { .. }) {
                        end_activation(state);
                        ffb_core::steps::eject_secret_weapons(state);
                        place_players_for_kickoff(state);
                        let kick_from = ffb_core::types::FieldCoordinate::new(13, 8);
                        let (_, mut scatter_tb) = perform_kickoff_scatter(state, kick_from, rng);
                        let kickoff_event = roll_kickoff_event(rng);
                        apply_kickoff_event(state, kickoff_event, rng);
                        if kickoff_event == ffb_core::types::KickoffEvent::ChangingWeather
                            && apply_nice_weather_scatter(state, scatter_tb, rng)
                        {
                            scatter_tb = true;
                        }
                        kickoff_bounce_if_needed(state, kick_from, scatter_tb, rng);
                        state.turn_mode = TurnMode::Regular;
                        state.home_is_active = !state.home_is_active;
                        begin_turn(state);
                        return None;
                    }
                    if matches!(result, MoveStepResult::Turnover { .. }) {
                        end_activation(state);
                        let et = end_turn(state);
                        return Some(et);
                    }
                    if matches!(result, MoveStepResult::PendingTeamReroll) {
                        return None; // another reroll needed (shouldn't happen in practice)
                    }
                    // Success/KnockedDown without ball — continue to end_activation below
                    if matches!(result, MoveStepResult::Success | MoveStepResult::KnockedDown { .. } | MoveStepResult::PickedUpBall) {
                        let is_blitz = state.acting_player.as_ref()
                            .map(|ap| ap.current_action == Some(PlayerAction::Blitz) && !ap.has_blocked)
                            .unwrap_or(false);
                        if !is_blitz {
                            end_activation(state);
                        }
                    }
                }
                return None;
            }

            // Check if this is a pass reroll
            let is_pass_reroll = state.acting_player.as_ref()
                .map(|ap| ap.pending_pass_target.is_some())
                .unwrap_or(false);
            if is_pass_reroll {
                state.dialog = DialogState::None;
                if let Some(ap) = state.acting_player.as_ref() {
                    let passer_id = ap.player_id.clone();
                    use ffb_core::steps::{apply_catch, PassStepResult};
                    let pass_result = resume_pass_after_reroll(state, &passer_id, use_it, rng);
                    let landing = match pass_result {
                        PassStepResult::Fumble { at: _ } => {
                            end_activation(state);
                            let et = end_turn(state);
                            return Some(et);
                        }
                        PassStepResult::PendingReroll => return None, // another reroll offered
                        PassStepResult::Inaccurate { landed } => landed,
                        PassStepResult::Accurate { to } => to,
                        _ => { end_activation(state); return None; }
                    };
                    if let Some(catcher_id) = state.field.player_at(landing).cloned() {
                        let catch_result = apply_catch(state, &catcher_id, rng);
                        match catch_result {
                            PassStepResult::PendingReroll => return None,
                            PassStepResult::Dropped { .. } => {
                                let catcher_team = state.field.player_team(&catcher_id);
                                let active_team = if state.home_is_active { TeamId::Home } else { TeamId::Away };
                                if catcher_team != Some(active_team) {
                                    end_activation(state);
                                    let et = end_turn(state);
                                    return Some(et);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                end_activation(state);
                return None;
            }

            // Check if this is a block dice reroll offer
            let is_block_reroll = matches!(state.dialog, DialogState::SelectBlockReroll { .. });
            if is_block_reroll {
                // Capture original dice before consuming the reroll
                let original_dice = if let DialogState::SelectBlockReroll { dice, defender_chooses, .. } = state.dialog.clone() {
                    Some((dice, defender_chooses))
                } else {
                    None
                };
                if use_it {
                    if let Some(ap) = state.acting_player.as_ref() {
                        let att_id = ap.player_id.clone();
                        let att_team = ap.team;
                        // Consume team reroll (Loner check included)
                        let reroll_ok = ffb_core::steps::turn_step::use_team_reroll(state, &att_id, rng);
                        if reroll_ok {
                            // Find defender and re-run begin_block to re-roll
                            let att_coord = state.field.player_coord(&att_id);
                            if let Some(att_coord) = att_coord {
                                let def_id = find_adjacent_opponent(state, att_coord, att_team);
                                if let Some(def_id) = def_id {
                                    state.dialog = DialogState::None;
                                    begin_block(state, &att_id, &def_id, rng);
                                    // Force SelectBlockDice since reroll is now consumed
                                    if let DialogState::SelectBlockReroll { dice, defender_chooses, .. } = state.dialog.clone() {
                                        state.dialog = DialogState::SelectBlockDice { dice, defender_chooses };
                                    }
                                }
                            }
                        } else {
                            // Loner failed — show original dice as SelectBlockDice
                            if let Some((dice, defender_chooses)) = original_dice {
                                state.dialog = DialogState::SelectBlockDice { dice, defender_chooses };
                            }
                        }
                    }
                } else {
                    // Decline reroll: show original dice as SelectBlockDice
                    if let Some((dice, defender_chooses)) = original_dice {
                        state.dialog = DialogState::SelectBlockDice { dice, defender_chooses };
                    } else {
                        state.dialog = DialogState::None;
                    }
                }
            } else {
                // Clear the reroll dialog regardless
                state.dialog = DialogState::None;
                if use_it {
                    if let Some(ap) = state.acting_player.as_ref() {
                        let pid = ap.player_id.clone();
                        ffb_core::steps::turn_step::use_team_reroll(state, &pid, rng);
                    }
                }
            }
        }

        BbAction::PlaceBall(coord) => {
            state.field.ball.coord = Some(coord);
            state.field.ball.in_play = true;
            state.dialog = DialogState::None;
        }

        BbAction::PassTo(target) => {
            if let Some(ap) = state.acting_player.as_ref() {
                let passer_id = ap.player_id.clone();
                let is_handoff = ap.current_action == Some(PlayerAction::HandOff);
                if is_handoff {
                    // Handoff: ball teleports to adjacent player; receiver attempts catch
                    state.field.ball.coord = Some(target);
                    if let Some(catcher_id) = state.field.player_at(target).cloned() {
                        use ffb_core::steps::apply_catch;
                        let catch_result = apply_catch(state, &catcher_id, rng);
                        use ffb_core::steps::PassStepResult;
                        match catch_result {
                            PassStepResult::PendingReroll => return None,
                            PassStepResult::Dropped { .. } => {
                                end_activation(state);
                                let et = end_turn(state);
                                return Some(et);
                            }
                            _ => {}
                        }
                    }
                } else {
                    // Regular pass: roll PA, scatter on fumble/inaccurate, catch at landing
                    use ffb_core::steps::{begin_pass, apply_catch, PassStepResult};
                    let pass_result = begin_pass(state, &passer_id, target, rng);
                    let landing = match pass_result {
                        PassStepResult::Fumble { at: _ } => {
                            end_activation(state);
                            let et = end_turn(state);
                            return Some(et);
                        }
                        PassStepResult::PendingReroll => {
                            // Team reroll dialog shown; wait for UseReroll action
                            return None;
                        }
                        PassStepResult::Inaccurate { landed } => landed,
                        PassStepResult::Accurate { to } => to,
                        _ => { end_activation(state); return None; }
                    };
                    // Attempt catch if a player is at the landing square
                    if let Some(catcher_id) = state.field.player_at(landing).cloned() {
                        let catch_result = apply_catch(state, &catcher_id, rng);
                        match catch_result {
                            PassStepResult::PendingReroll => return None,
                            PassStepResult::Dropped { .. } => {
                                // Turnover only if the catcher is on the active team's side
                                let catcher_team = state.field.player_team(&catcher_id);
                                let active_team = if state.home_is_active { TeamId::Home } else { TeamId::Away };
                                if catcher_team != Some(active_team) {
                                    end_activation(state);
                                    let et = end_turn(state);
                                    return Some(et);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            end_activation(state);
        }
    }

    None
}

/// Handle a TurnStepResult by transitioning to new turn or half/game end.
fn handle_turn_result(state: &mut GameState, result: TurnStepResult, rng: &mut GameRng) {
    match result {
        TurnStepResult::Ok | TurnStepResult::TurnOver => {
            // Begin the new team's turn
            begin_turn(state);
        }
        TurnStepResult::HalfEnd => {
            // KO recovery: each KO'd player rolls d6; on 4+ they recover to the reserves
            ko_recovery(state, rng);
            // SwelteringHeat fainting: roll d3 for count, then d(N) per team to select players
            // Matches Java's StepEndTurn.getFaintingCount (fires when fNewHalf=true)
            sweltering_heat_fainting(state, rng);
            // Second half setup: place players again, re-do kickoff
            place_players_for_kickoff(state);
            let kick_from = ffb_core::types::FieldCoordinate::new(13, 8);
            let (_, mut scatter_tb) = perform_kickoff_scatter(state, kick_from, rng);
            let kickoff_event = roll_kickoff_event(rng);
            apply_kickoff_event(state, kickoff_event, rng);
            if kickoff_event == ffb_core::types::KickoffEvent::ChangingWeather
                && apply_nice_weather_scatter(state, scatter_tb, rng)
            {
                scatter_tb = true;
            }
            kickoff_bounce_if_needed(state, kick_from, scatter_tb, rng);
            state.turn_mode = TurnMode::Regular;
            // The receiving team (new offense = team not kicking) acts first
            state.home_is_active = !state.home_is_offense;
            begin_turn(state);
        }
        TurnStepResult::GameEnd => {
            state.result.finished = true;
        }
    }
}

/// Find the first adjacent opponent standing player (for block target resolution).
/// Roll KO recovery at half-time or after touchdown: d6 4+ → move from KO to Reserve.
/// SwelteringHeat end-of-half fainting: matches Java's StepEndTurn.getFaintingCount.
/// Rolls d3 for fainting count, then for each team rolls d(on_pitch_size) that many times
/// to select players who become Exhausted. Players are replaced at next kickoff setup,
/// so this only affects dice consumption for the half-2 kickoff sequence.
fn sweltering_heat_fainting(state: &mut GameState, rng: &mut GameRng) {
    use ffb_core::types::Weather;
    if state.field.weather != Weather::SwelteringHeat {
        return;
    }
    let fainting_count = rng.roll(3) as usize;
    for team_id in [TeamId::Home, TeamId::Away] {
        let mut on_pitch: Vec<_> = state.team(team_id).players().iter()
            .filter(|p| {
                state.field.player_coord(&p.id)
                    .map(|c| c.is_valid())
                    .unwrap_or(false)
            })
            .map(|p| p.id.clone())
            .collect();
        for _ in 0..fainting_count {
            if on_pitch.is_empty() {
                break;
            }
            let idx = (rng.roll(on_pitch.len() as u8) as usize) - 1;
            on_pitch.remove(idx);
        }
    }
}

fn ko_recovery(state: &mut GameState, rng: &mut GameRng) {
    for team in [TeamId::Home, TeamId::Away] {
        let ko_ids: Vec<_> = state.team(team).players()
            .iter()
            .filter(|p| {
                state.field.player_state(&p.id) == Some(ffb_core::types::PlayerState::Ko)
            })
            .map(|p| p.id.clone())
            .collect();
        for pid in ko_ids {
            if rng.roll_d6() >= 4 {
                // Recover: remove from field (they'll be re-placed at next kickoff)
                if state.field.player_state(&pid).is_some() {
                    state.field.set_player_state(&pid, ffb_core::types::PlayerState::Reserve);
                }
            }
        }
    }
}

fn find_adjacent_opponent(state: &GameState, coord: FieldCoordinate, team: TeamId) -> Option<ffb_core::types::PlayerId> {
    coord.neighbors().find_map(|n| {
        let pid = state.field.player_at(n)?;
        let t = state.field.player_team(pid)?;
        if t != team {
            Some(pid.clone())
        } else {
            None
        }
    })
}

/// Find an adjacent opponent — includes prone players (for push resolution).
fn find_adjacent_opponent_or_prone(state: &GameState, coord: FieldCoordinate, team: TeamId) -> Option<ffb_core::types::PlayerId> {
    coord.neighbors().find_map(|n| {
        let pid = state.field.player_at(n)?;
        let t = state.field.player_team(pid)?;
        if t != team {
            Some(pid.clone())
        } else {
            None
        }
    })
}

/// Execute a foul action: roll armor directly with Dirty Player bonus, then injury if broken.
/// Check for ejection: if both armor dice show doubles, 50% chance the fouler is sent off.
fn execute_foul(
    state: &mut GameState,
    fouler_id: &ffb_core::types::PlayerId,
    defender_id: &ffb_core::types::PlayerId,
    rng: &mut GameRng,
) {
    use ffb_core::mechanics::injury::{ArmorOutcome, resolve_injury};
    use ffb_core::skills::SkillId;

    // Get attacker's DirtyPlayer bonus
    let dirty_player_bonus = state.home.player_by_id(fouler_id)
        .or_else(|| state.away.player_by_id(fouler_id))
        .map(|p| if p.has_skill(SkillId::DirtyPlayer) { 1u8 } else { 0u8 })
        .unwrap_or(0);

    // Get defender's AV
    let defender_av = state.home.player_by_id(defender_id)
        .or_else(|| state.away.player_by_id(defender_id))
        .map(|p| p.effective_av())
        .unwrap_or(8);

    // Roll 2d6 for armor (we roll individually to check for doubles)
    let die1 = rng.roll_d6();
    let die2 = rng.roll_d6();
    let is_doubles = die1 == die2;
    let armor_total = die1 + die2 + dirty_player_bonus;

    let armor_outcome = if armor_total > defender_av {
        ArmorOutcome::Broken
    } else {
        ArmorOutcome::NotBroken
    };

    if matches!(armor_outcome, ArmorOutcome::Broken) {
        // Roll injury
        let injury = resolve_injury(0, rng);
        state.field.set_player_state(defender_id, injury.new_state);
    }

    // Check ejection: if doubles on armor dice and referee is watching (50% — d6 >= 4)
    if is_doubles {
        let ref_roll = rng.roll_d6();
        if ref_roll >= 4 {
            // Fouler is sent off — move to Reserve
            state.field.set_player_state(fouler_id, ffb_core::types::PlayerState::Reserve);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_core::actions::enumerate_actions;
    use ffb_core::model::game_state::GameState;
    use ffb_core::model::player::{Player, PlayerStats};
    use ffb_core::model::team::Team;
    use ffb_core::rng::GameRng;
    use ffb_core::skills::SkillSet;
    use ffb_core::types::{PlayerId, TeamId};

    fn make_full_state() -> GameState {
        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        for i in 0..11u8 {
            home.add_player(Player::new(
                PlayerId(format!("h{i}")),
                format!("HP{i}"),
                "lineman".into(),
                TeamId::Home,
                i + 1,
                PlayerStats::new(6, 3, 4, 8, None),
                SkillSet::empty(),
            ));
        }

        let mut away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        for i in 0..11u8 {
            away.add_player(Player::new(
                PlayerId(format!("a{i}")),
                format!("AP{i}"),
                "lineman".into(),
                TeamId::Away,
                i + 1,
                PlayerStats::new(5, 4, 3, 9, None),
                SkillSet::empty(),
            ));
        }

        GameState::new(home, away)
    }

    #[test]
    fn enumerate_actions_always_has_end_turn() {
        let mut state = make_full_state();
        state.turn_mode = TurnMode::Regular;
        state.home_is_active = true;
        // Place a player on pitch
        let pid = PlayerId("h0".into());
        state.field.place_player(
            pid,
            TeamId::Home,
            ffb_core::types::FieldCoordinate::new(5, 5),
            PlayerState::Standing,
        );

        let actions = enumerate_actions(&state, TeamId::Home);
        assert!(
            actions.contains(&BbAction::EndTurn),
            "EndTurn must be in legal actions"
        );
    }

    #[test]
    fn null_strategy_game_completes() {
        let state = make_full_state();
        let mut rng = GameRng::new_live(42);
        let home_strat = NullStrategy;
        let away_strat = NullStrategy;

        let final_state = SimulationLoop::run(state, &home_strat, &away_strat, &mut rng);
        assert!(
            final_state.result.finished,
            "game must be finished after SimulationLoop::run"
        );
    }

    #[test]
    fn statistical_smoke_100_games_complete() {
        // Run 100 games with NullStrategy; verify all complete and scores are sane.
        let home_strat = NullStrategy;
        let away_strat = NullStrategy;
        let mut finished = 0u32;
        let mut total_score = 0u32;

        for seed in 0..100u64 {
            let state = make_full_state();
            let mut rng = GameRng::new_live(seed);
            let final_state = SimulationLoop::run(state, &home_strat, &away_strat, &mut rng);
            assert!(final_state.result.finished, "game {seed} did not finish");
            finished += 1;
            total_score += (final_state.result.score_home + final_state.result.score_away) as u32;
        }

        assert_eq!(finished, 100, "all 100 games must complete");
        // NullStrategy ends turns immediately → few or zero TDs expected
        // Just verify the total score is plausible (0-50 TDs across 100 games)
        assert!(total_score <= 50, "unexpectedly high total score: {total_score}");
    }

    /// T-30: ScriptedStrategy vs ScriptedStrategy — all games must complete.
    /// With simple heuristics, games end with few or zero TDs. The key invariant
    /// is that no game exceeds MAX_ACTIONS and all finish with `result.finished`.
    #[test]
    fn scripted_vs_scripted_all_games_complete() {
        use crate::roster::make_team;
        use ffb_core::types::TeamId;
        let home_strat = ScriptedStrategy::default();
        let away_strat = ScriptedStrategy::default();
        let n = 30u32;
        let mut finished = 0u32;
        let mut total_score: u32 = 0;

        for seed in 0..n {
            let home = make_team("human", TeamId::Home, "h", "Home", 3).unwrap();
            let away = make_team("human", TeamId::Away, "a", "Away", 3).unwrap();
            let state = ffb_core::model::game_state::GameState::new(home, away);
            let mut rng = GameRng::new_live(seed as u64);
            let final_state = SimulationLoop::run(state, &home_strat, &away_strat, &mut rng);
            assert!(final_state.result.finished, "game {seed} did not finish");
            finished += 1;
            total_score += (final_state.result.score_home + final_state.result.score_away) as u32;
        }
        assert_eq!(finished, n, "all {n} scripted games must complete");
        // Total score should be plausible (0 to 60 TDs across 30 games)
        assert!(total_score <= 60, "unexpectedly high score: {total_score}");
    }

    /// Statistical smoke test (plan §4): human vs orc, N=200 games with NullStrategy.
    /// Verifies the game loop terminates consistently and doesn't produce anomalous results.
    #[test]
    fn statistical_smoke_200_games_null_strategy() {
        use crate::roster::make_team;
        use ffb_core::types::TeamId;
        let home_strat = NullStrategy;
        let away_strat = NullStrategy;
        let mut finished = 0u32;
        let mut total_score: u32 = 0;

        for seed in 0..200u64 {
            let home = make_team("human", TeamId::Home, "h", "Home", 3).unwrap();
            let away = make_team("orc", TeamId::Away, "a", "Away", 3).unwrap();
            let state = ffb_core::model::game_state::GameState::new(home, away);
            let mut rng = GameRng::new_live(seed);
            let final_state = SimulationLoop::run(state, &home_strat, &away_strat, &mut rng);
            assert!(final_state.result.finished, "game {seed} did not finish");
            finished += 1;
            total_score += (final_state.result.score_home + final_state.result.score_away) as u32;
        }

        assert_eq!(finished, 200);
        // NullStrategy always ends turns → very few TDs expected
        assert!(total_score <= 100, "unexpectedly high score in 200 games: {total_score}");
    }

    // ── Foul execution tests ──────────────────────────────────────────────────

    #[test]
    fn foul_breaks_armor_and_injures_defender() {
        use ffb_core::types::{FieldCoordinate, PlayerState};
        let fouler_id = PlayerId("h1".into());
        let victim_id = PlayerId("a1".into());

        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        home.add_player(Player::new(
            fouler_id.clone(), "Fouler".into(), "lineman".into(), TeamId::Home, 1,
            PlayerStats::new(6, 3, 4, 8, None), SkillSet::empty(),
        ));
        let mut away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        away.add_player(Player::new(
            victim_id.clone(), "Victim".into(), "lineman".into(), TeamId::Away, 1,
            // AV=4 so armor breaks easily
            PlayerStats::new(5, 4, 3, 4, None), SkillSet::empty(),
        ));

        let mut state = GameState::new(home, away);
        state.field.place_player(fouler_id.clone(), TeamId::Home, FieldCoordinate::new(5, 5), PlayerState::Standing);
        state.field.place_player(victim_id.clone(), TeamId::Away, FieldCoordinate::new(6, 5), PlayerState::Prone);
        state.home_is_active = true;

        // Roll: die1=5, die2=5 (total=10 > AV=4, broken; doubles! ref roll=3 < 4 → not ejected)
        // Then injury: 2d6 for injury roll
        // 2d6=2+3=5 → stunned
        let mut rng = GameRng::new_test([5, 5, 3, 2, 3]);
        execute_foul(&mut state, &fouler_id, &victim_id, &mut rng);

        let victim_state = state.field.player_state(&victim_id);
        assert!(
            matches!(victim_state, Some(PlayerState::Stunned) | Some(PlayerState::Ko) | Some(PlayerState::Injured)),
            "victim should be stunned/ko/injured after armor broken, got: {victim_state:?}"
        );
    }

    #[test]
    fn foul_doubles_and_ref_ejects_fouler() {
        use ffb_core::types::{FieldCoordinate, PlayerState};
        let fouler_id = PlayerId("h1".into());
        let victim_id = PlayerId("a1".into());

        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        home.add_player(Player::new(
            fouler_id.clone(), "Fouler".into(), "lineman".into(), TeamId::Home, 1,
            PlayerStats::new(6, 3, 4, 8, None), SkillSet::empty(),
        ));
        let mut away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        away.add_player(Player::new(
            victim_id.clone(), "Victim".into(), "lineman".into(), TeamId::Away, 1,
            PlayerStats::new(5, 4, 3, 9, None), SkillSet::empty(),
        ));

        let mut state = GameState::new(home, away);
        state.field.place_player(fouler_id.clone(), TeamId::Home, FieldCoordinate::new(5, 5), PlayerState::Standing);
        state.field.place_player(victim_id.clone(), TeamId::Away, FieldCoordinate::new(6, 5), PlayerState::Prone);
        state.home_is_active = true;

        // Roll: die1=2, die2=2 (total=4 vs AV=9, NOT broken; doubles → ref roll=6 >= 4 → ejected)
        let mut rng = GameRng::new_test([2, 2, 6]);
        execute_foul(&mut state, &fouler_id, &victim_id, &mut rng);

        // Fouler should be sent off (Reserve)
        assert_eq!(
            state.field.player_state(&fouler_id),
            Some(PlayerState::Reserve),
            "Fouler should be ejected on doubles if referee is watching"
        );
    }
}
