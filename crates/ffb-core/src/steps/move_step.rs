/// Move step: wraps `mechanics::execute_move_step` with dialog integration.
use crate::mechanics::movement::{execute_move_step, MoveOutcome};
use crate::model::game_state::{GameState, PendingMoveState};
use crate::rng::GameRng;
use crate::skills::SkillId;
use crate::types::{FieldCoordinate, PlayerId};

#[derive(Clone, Debug)]
pub enum MoveStepResult {
    Success,
    PickedUpBall,
    FailedPickup { at: FieldCoordinate },
    KnockedDown { at: FieldCoordinate },
    /// Player was caught by Tentacles and cannot move further (not knocked down).
    StoppedByTentacles { at: FieldCoordinate },
    Turnover { at: FieldCoordinate },
    Touchdown { at: FieldCoordinate },
    /// Dodge or GFI failed; a team reroll has been offered (SelectReroll dialog set).
    /// Simulation loop must handle UseReroll(true/false) to resume or finalize the knockdown.
    PendingTeamReroll,
}

/// Execute a move along `path` for `player_id`.
/// Handles ball pickup and touchdown detection.
pub fn begin_move(
    state: &mut GameState,
    player_id: &PlayerId,
    path: &[FieldCoordinate],
    rng: &mut GameRng,
) -> MoveStepResult {
    let team = state.field.player_team(player_id).expect("player team");

    // Prone stand-up: if player is Prone and does NOT have JumpUp (JumpUp is
    // handled in begin_activation), they must spend 3 MA to stand up.
    if state.field.player_state(player_id) == Some(crate::types::PlayerState::Prone) {
        let has_jump_up = state.team(team)
            .player_by_id(player_id)
            .map(|p| p.has_skill(SkillId::JumpUp))
            .unwrap_or(false);
        if !has_jump_up {
            if let Some(ap) = state.acting_player.as_mut() {
                ap.movement_used = ap.movement_used.saturating_add(3);
            }
            state.field.set_player_state(player_id, crate::types::PlayerState::Standing);
        }
    }

    let carries_ball = state.field.ball.coord == state.field.player_coord(player_id)
        && state.field.ball.in_play;

    let result = execute_move_step(state, player_id, path, rng);
    let current_coord = state.field.player_coord(player_id);

    match result {
        MoveOutcome::KnockedDown { at } => {
            // If player was carrying ball, scatter the ball then turnover.
            // Exception: SafePairOfHands — ball stays at square, no turnover.
            if carries_ball {
                let has_safe_pair_of_hands = state.team(team)
                    .player_by_id(player_id)
                    .map(|p| p.has_skill(SkillId::SafePairOfHands))
                    .unwrap_or(false);
                if has_safe_pair_of_hands {
                    return MoveStepResult::KnockedDown { at };
                }
                // Ball scatters one square in a random direction (d8)
                let scatter = crate::mechanics::pass::pass_scatter_coord(at, rng);
                state.field.ball.coord = Some(scatter);
                return MoveStepResult::Turnover { at };
            }
            MoveStepResult::KnockedDown { at }
        }
        MoveOutcome::CaughtByTentacles { at } => {
            // Player is held in place by Tentacles — not knocked down, move simply stops.
            // No turnover; ball is not dropped.
            MoveStepResult::StoppedByTentacles { at }
        }
        MoveOutcome::NeedsTeamReroll { current_dest, remaining_after, min_roll, is_gfi } => {
            // Skill rerolls exhausted — check if a team reroll is available.
            let team_reroll_avail = crate::steps::turn_step::active_team_reroll_available(state);
            if team_reroll_avail {
                // Store pending state, offer reroll dialog.
                if let Some(ap) = state.acting_player.as_mut() {
                    ap.pending_move = Some(PendingMoveState {
                        current_dest,
                        remaining_after,
                        min_roll,
                        is_gfi,
                    });
                }
                state.dialog = crate::model::game_state::DialogState::SelectReroll {
                    action_name: if is_gfi { "GFI".into() } else { "Dodge".into() },
                    reroll_available: true,
                    skill_reroll_available: false,
                };
                MoveStepResult::PendingTeamReroll
            } else {
                // No team reroll available — finalize knockdown immediately.
                finalize_move_knockdown(state, player_id, team, is_gfi, carries_ball, rng)
            }
        }
        MoveOutcome::Success => {
            let coord = match current_coord { Some(c) => c, None => return MoveStepResult::Success };

            // Check for ball pickup (player moved to ball square and doesn't have it yet)
            let ball_here = state.field.ball.coord == Some(coord) && state.field.ball.in_play && !carries_ball;
            if ball_here {
                let pickup_result = attempt_pickup(state, player_id, coord, rng);
                if !pickup_result {
                    // Check if team reroll can be offered before finalizing turnover
                    let team_reroll_avail = crate::steps::turn_step::active_team_reroll_available(state);
                    if team_reroll_avail {
                        if let Some(ap) = state.acting_player.as_mut() {
                            ap.pending_pickup_at = Some(coord);
                        }
                        state.dialog = crate::model::game_state::DialogState::SelectReroll {
                            action_name: "Pickup".into(),
                            reroll_available: true,
                            skill_reroll_available: false,
                        };
                        return MoveStepResult::PendingTeamReroll;
                    }
                    // Failed pickup: ball bounces to adjacent square, then turnover
                    let scatter = crate::mechanics::pass::pass_scatter_coord(coord, rng);
                    state.field.ball.coord = Some(scatter);
                    return MoveStepResult::Turnover { at: coord };
                }
                // Successfully picked up ball — check for immediate touchdown
                return check_touchdown(state, player_id, coord, team);
            }

            // If already carrying ball, check for touchdown
            if carries_ball {
                return check_touchdown(state, player_id, coord, team);
            }

            MoveStepResult::Success
        }
    }
}

/// Finalize a knockdown when no team reroll is available (or declined).
fn finalize_move_knockdown(
    state: &mut GameState,
    player_id: &PlayerId,
    team: crate::types::TeamId,
    is_gfi: bool,
    carries_ball: bool,
    rng: &mut GameRng,
) -> MoveStepResult {
    let at = state.field.player_coord(player_id).unwrap_or(FieldCoordinate::new(0, 0));
    state.field.set_player_state(player_id, crate::types::PlayerState::Prone);
    if carries_ball {
        let has_safe_pair = state.team(team)
            .player_by_id(player_id)
            .map(|p| p.has_skill(SkillId::SafePairOfHands))
            .unwrap_or(false);
        if !has_safe_pair {
            let scatter = crate::mechanics::pass::pass_scatter_coord(at, rng);
            state.field.ball.coord = Some(scatter);
            return MoveStepResult::Turnover { at };
        }
    }
    let _ = is_gfi;
    MoveStepResult::KnockedDown { at }
}

/// Resume movement after a team reroll decision.
/// Called from the simulation loop when `UseReroll` is received while `pending_move` is set.
/// Returns the final move result (Turnover, Success, Touchdown, etc.).
pub fn resume_move_after_reroll(
    state: &mut GameState,
    player_id: &PlayerId,
    use_reroll: bool,
    rng: &mut GameRng,
) -> MoveStepResult {
    // Check if this is a pickup reroll
    let pickup_at = state.acting_player.as_mut().and_then(|ap| ap.pending_pickup_at.take());
    if let Some(coord) = pickup_at {
        let team = state.field.player_team(player_id).expect("team");
        if use_reroll {
            let reroll_ok = crate::steps::turn_step::use_team_reroll(state, player_id, rng);
            if reroll_ok {
                let pickup_ok = attempt_pickup(state, player_id, coord, rng);
                if pickup_ok {
                    return check_touchdown(state, player_id, coord, team);
                }
                // Still failed — ball scatters, turnover
            }
            // Loner failed or pickup failed again
        }
        // Declined or still failed
        let scatter = crate::mechanics::pass::pass_scatter_coord(coord, rng);
        state.field.ball.coord = Some(scatter);
        return MoveStepResult::Turnover { at: coord };
    }

    let pending = match state.acting_player.as_mut().and_then(|ap| ap.pending_move.take()) {
        Some(p) => p,
        None => return MoveStepResult::Success, // nothing pending, shouldn't happen
    };
    let team = state.field.player_team(player_id).expect("player team");
    let carries_ball = state.field.ball.coord == state.field.player_coord(player_id)
        && state.field.ball.in_play;

    if use_reroll {
        // Consume team reroll (Loner check included)
        let reroll_ok = crate::steps::turn_step::use_team_reroll(state, player_id, rng);
        if !reroll_ok {
            // Loner failed — treat as declined
            return finalize_move_knockdown(state, player_id, team, pending.is_gfi, carries_ball, rng);
        }

        // Re-roll the die
        let roll = rng.roll_d6();
        let success = roll >= pending.min_roll;

        if success {
            // For dodge: move player to current_dest first
            if !pending.is_gfi {
                if !state.field.is_occupied(pending.current_dest) {
                    state.field.move_player(player_id, pending.current_dest);
                }
                if let Some(ap) = state.acting_player.as_mut() {
                    ap.movement_used += 1;
                }
            }
            // Continue executing the remaining path
            if !pending.remaining_after.is_empty() {
                begin_move(state, player_id, &pending.remaining_after, rng)
            } else {
                // No more squares to move — check touchdown / pickup at current location
                let coord = state.field.player_coord(player_id).unwrap_or(FieldCoordinate::new(0, 0));
                let ball_here = state.field.ball.coord == Some(coord)
                    && state.field.ball.in_play
                    && !carries_ball;
                if ball_here {
                    let pickup_ok = attempt_pickup(state, player_id, coord, rng);
                    if !pickup_ok {
                        let scatter = crate::mechanics::pass::pass_scatter_coord(coord, rng);
                        state.field.ball.coord = Some(scatter);
                        return MoveStepResult::Turnover { at: coord };
                    }
                    return check_touchdown(state, player_id, coord, team);
                }
                if carries_ball {
                    return check_touchdown(state, player_id, coord, team);
                }
                MoveStepResult::Success
            }
        } else {
            // Reroll also failed — finalize knockdown
            finalize_move_knockdown(state, player_id, team, pending.is_gfi, carries_ball, rng)
        }
    } else {
        // Declined reroll — finalize knockdown
        finalize_move_knockdown(state, player_id, team, pending.is_gfi, carries_ball, rng)
    }
}

// ── Skill-based movement capability checks ────────────────────────────────────

/// Returns true if the player may leap (has Leap or Pogo skill).
pub fn can_leap(state: &GameState, player_id: &PlayerId) -> bool {
    state.field.player_team(player_id)
        .and_then(|team| state.team(team).player_by_id(player_id))
        .map(|p| p.has_skill(SkillId::Leap) || p.has_skill(SkillId::Pogo))
        .unwrap_or(false)
}

/// Returns true if the player has Swoop (may fly over occupied squares without dodge rolls).
pub fn can_swoop(state: &GameState, player_id: &PlayerId) -> bool {
    state.field.player_team(player_id)
        .and_then(|team| state.team(team).player_by_id(player_id))
        .map(|p| p.has_skill(SkillId::Swoop))
        .unwrap_or(false)
}

/// Returns true if the player has SteadyFooting (no GFI roll when dodging out of tackle zones).
pub fn steady_footing_gfi_exempt(state: &GameState, player_id: &PlayerId) -> bool {
    state.field.player_team(player_id)
        .and_then(|team| state.team(team).player_by_id(player_id))
        .map(|p| p.has_skill(SkillId::SteadyFooting))
        .unwrap_or(false)
}

/// Returns true if the player has HitAndRun (may move up to 2 squares after a block).
pub fn hit_and_run_available(state: &GameState, player_id: &PlayerId) -> bool {
    state.field.player_team(player_id)
        .and_then(|team| state.team(team).player_by_id(player_id))
        .map(|p| p.has_skill(SkillId::HitAndRun))
        .unwrap_or(false)
}

/// Returns true if the player has Fumblerooski and is currently carrying the ball.
pub fn can_fumblerooski(state: &GameState, player_id: &PlayerId) -> bool {
    let coord = match state.field.player_coord(player_id) {
        Some(c) => c,
        None => return false,
    };
    let carries_ball = state.field.ball.coord == Some(coord) && state.field.ball.in_play;
    if !carries_ball {
        return false;
    }
    state.field.player_team(player_id)
        .and_then(|team| state.team(team).player_by_id(player_id))
        .map(|p| p.has_skill(SkillId::Fumblerooski))
        .unwrap_or(false)
}

/// Attempt to pick up the ball at `coord`.
/// Returns true on success.
fn attempt_pickup(state: &mut GameState, player_id: &PlayerId, coord: FieldCoordinate, rng: &mut GameRng) -> bool {
    // NoBall: player cannot pick up the ball
    {
        let has_no_ball = state.field.player_team(player_id)
            .and_then(|team| state.team(team).player_by_id(player_id))
            .map(|p| p.has_skill(SkillId::NoBall))
            .unwrap_or(false);
        if has_no_ball {
            return false;
        }
    }

    let team = state.field.player_team(player_id).expect("team");
    let opp = team.opponent();

    let (ag, has_sure_hands, has_extra_arms, opp_tz) = {
        let p = state.team(team).player_by_id(player_id).expect("player");
        // tackle_zones_on(coord, team) = opposing TZs threatening team's player at coord
        let tz = state.field.tackle_zones_on(coord, team);
        (p.effective_ag(), p.has_skill(SkillId::SureHands), p.has_skill(SkillId::ExtraArms), tz)
    };

    let base = (5u8).saturating_sub(ag).max(2);
    let tz_mod = if has_extra_arms && opp_tz > 0 { opp_tz - 1 } else { opp_tz };
    // PouringRain or Blizzard: +1 to ball-handling rolls
    let weather_mod = match state.field.weather {
        crate::types::Weather::PouringRain | crate::types::Weather::Blizzard => 1u8,
        _ => 0u8,
    };
    let min_roll = (base + tz_mod + weather_mod).min(6);

    let roll = rng.roll_d6();
    if roll >= min_roll {
        return true;
    }
    // SureHands: re-roll once
    if has_sure_hands {
        return rng.roll_d6() >= min_roll;
    }
    false
}

/// Check if player carrying ball has scored a touchdown.
fn check_touchdown(state: &mut GameState, _player_id: &PlayerId, coord: FieldCoordinate, team: crate::types::TeamId) -> MoveStepResult {
    use crate::types::PITCH_WIDTH;
    let is_td = match team {
        crate::types::TeamId::Home => coord.x == PITCH_WIDTH - 1, // home attacks right end zone
        crate::types::TeamId::Away => coord.x == 0, // away attacks left end zone
    };

    if is_td {
        state.team_mut(team).score_touchdown();
        // Sync to GameResult so winner detection and external observers see the score
        state.result.score_home = state.home.score;
        state.result.score_away = state.away.score;
        state.field.ball.in_play = false;
        MoveStepResult::Touchdown { at: coord }
    } else {
        MoveStepResult::PickedUpBall
    }
}

// ── T-57 / T-58 skill helpers ─────────────────────────────────────────────────

// ── Shadowing ─────────────────────────────────────────────────────────────────

/// Determine whether a Shadowing attempt succeeds.
///
/// Blood Bowl 2025 Shadowing rules (simplified):
/// - After a player dodges out of an opponent's tackle zone, if that opponent has
///   the Shadowing skill, they may attempt to follow.
/// - Both players roll d6 + their MA. If the shadower's total ≥ the dodger's total,
///   shadowing succeeds and the shadower moves with the dodger.
/// - This function returns `true` if shadowing succeeds.
///
/// Note: full integration into `execute_move` (moving the shadower) is complex
/// state-management and is left to the caller. This function only resolves the
/// dice check.
pub fn shadowing_succeeds(mover_ma: u8, shadower_ma: u8, rng: &mut GameRng) -> bool {
    let mover_roll = rng.roll_d6() as u16 + mover_ma as u16;
    let shadower_roll = rng.roll_d6() as u16 + shadower_ma as u16;
    shadower_roll >= mover_roll
}

/// Full Shadowing check against game state.
/// Finds the opponent with Shadowing adjacent to `mover_id` (at their current position)
/// and runs the dice check. Returns `true` if shadowing succeeds.
/// If no adjacent shadower exists, returns `false`.
pub fn apply_shadowing_check(
    state: &GameState,
    mover_id: &PlayerId,
    shadower_id: &PlayerId,
    rng: &mut GameRng,
) -> bool {
    let mover_team = match state.field.player_team(mover_id) {
        Some(t) => t,
        None => return false,
    };
    let shadower_team = match state.field.player_team(shadower_id) {
        Some(t) => t,
        None => return false,
    };
    // Shadower must be on opposing team
    if mover_team == shadower_team {
        return false;
    }
    // Shadower must have Shadowing skill
    let has_shadowing = state.team(shadower_team)
        .player_by_id(shadower_id)
        .map(|p| p.has_skill(SkillId::Shadowing))
        .unwrap_or(false);
    if !has_shadowing {
        return false;
    }

    let mover_ma = state.team(mover_team)
        .player_by_id(mover_id)
        .map(|p| p.effective_ma())
        .unwrap_or(0);
    let shadower_ma = state.team(shadower_team)
        .player_by_id(shadower_id)
        .map(|p| p.effective_ma())
        .unwrap_or(0);

    shadowing_succeeds(mover_ma, shadower_ma, rng)
}

/// T-57 #7: Incorporeal — player may move through occupied squares.
/// Returns true if the player has the Incorporeal skill.
pub fn is_incorporeal(state: &GameState, player_id: &PlayerId) -> bool {
    state.home.player_by_id(player_id)
        .or_else(|| state.away.player_by_id(player_id))
        .map(|p| p.has_skill(SkillId::Incorporeal))
        .unwrap_or(false)
}

/// T-57 #9: WoodlandFury — free Leap once per turn.
/// Returns true if the player has WoodlandFury (leap usage tracking is a
/// responsibility of the turn state layer).
pub fn woodland_fury_leap_available(state: &GameState, player_id: &PlayerId) -> bool {
    state.home.player_by_id(player_id)
        .or_else(|| state.away.player_by_id(player_id))
        .map(|p| p.has_skill(SkillId::WoodlandFury))
        .unwrap_or(false)
}

/// T-58 #10: PumpUpTheCrowd — adjacent teammates get +1 to AG rolls.
/// Returns 1 if `player_id` has PumpUpTheCrowd and is adjacent to `ally_id`.
pub fn pump_up_ag_bonus(state: &GameState, player_id: &PlayerId, ally_id: &PlayerId) -> u8 {
    let has_skill = state.home.player_by_id(player_id)
        .or_else(|| state.away.player_by_id(player_id))
        .map(|p| p.has_skill(SkillId::PumpUpTheCrowd))
        .unwrap_or(false);
    if !has_skill {
        return 0;
    }
    let p_coord = match state.field.player_coord(player_id) {
        Some(c) => c,
        None => return 0,
    };
    let a_coord = match state.field.player_coord(ally_id) {
        Some(c) => c,
        None => return 0,
    };
    let dx = (p_coord.x as i16 - a_coord.x as i16).abs();
    let dy = (p_coord.y as i16 - a_coord.y as i16).abs();
    if dx <= 1 && dy <= 1 && (dx + dy) > 0 { 1 } else { 0 }
}

/// T-58 #11: ExcuseMeAreYouAZoat — Wild Animal rules + +2 ST when activated.
/// Returns 2 if player has ExcuseMeAreYouAZoat.
pub fn zoat_st_bonus(state: &GameState, player_id: &PlayerId) -> u8 {
    let has = state.home.player_by_id(player_id)
        .or_else(|| state.away.player_by_id(player_id))
        .map(|p| p.has_skill(SkillId::ExcuseMeAreYouAZoat))
        .unwrap_or(false);
    if has { 2 } else { 0 }
}

// ── BallAndChain movement ────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
pub enum BallAndChainResult {
    /// Moved normally.
    Moved { to: FieldCoordinate },
    /// Player ran into an opponent — defender pushed and knocked down with armor roll.
    Collision { at: FieldCoordinate, victim: PlayerId },
    /// Player went off the pitch — crowd push, take injury without armor (Stunned).
    OffPitch { from: FieldCoordinate },
    /// Player's activation complete — falls over (Prone).
    FellOver { at: FieldCoordinate },
}

/// Execute one BallAndChain movement square.
/// Rolls d8 for direction; moves player one square in that direction.
/// Returns what happened.
pub fn ball_and_chain_move_square(
    state: &mut GameState,
    player_id: &PlayerId,
    rng: &mut GameRng,
) -> BallAndChainResult {
    use crate::pathfinding::scatter_delta;
    use crate::types::{PITCH_WIDTH, PITCH_HEIGHT};
    use crate::mechanics::injury::{armor_roll, resolve_injury, ArmorOutcome};

    let from = match state.field.player_coord(player_id) {
        Some(c) => c,
        None => return BallAndChainResult::FellOver { at: FieldCoordinate::new(0, 0) },
    };

    let dir = rng.roll_scatter_direction(); // d8 = 1-8
    let (dx, dy) = scatter_delta(dir);
    let nx = from.x as i16 + dx as i16;
    let ny = from.y as i16 + dy as i16;

    // Off-pitch check
    if nx < 0 || nx >= PITCH_WIDTH as i16 || ny < 0 || ny >= PITCH_HEIGHT as i16 {
        // Crowd push — B&C player takes injury without armor roll (Stun)
        state.field.set_player_state(player_id, crate::types::PlayerState::Stunned);
        return BallAndChainResult::OffPitch { from };
    }

    let to = FieldCoordinate::new(nx as u8, ny as u8);

    // Collision: occupied square
    if let Some(victim_id) = state.field.player_at(to).cloned() {
        // Auto-knockdown defender + armor roll
        let victim_team = state.field.player_team(&victim_id);
        let av = victim_team
            .and_then(|t| state.team(t).player_by_id(&victim_id))
            .map(|p| p.effective_av())
            .unwrap_or(7);
        state.field.set_player_state(&victim_id, crate::types::PlayerState::Prone);
        if armor_roll(av, 0, 0, rng) == ArmorOutcome::Broken {
            let inj = resolve_injury(0, rng);
            state.field.set_player_state(&victim_id, inj.new_state);
        }
        // Move B&C player anyway (push the victim aside isn't modeled here;
        // in full BB2025 the victim is pushed — for now B&C player stays)
        return BallAndChainResult::Collision { at: to, victim: victim_id };
    }

    // Normal move
    state.field.move_player(player_id, to);
    BallAndChainResult::Moved { to }
}

/// Execute a full BallAndChain activation: move MA squares in random directions.
/// At the end, player falls over (Stunned via injury without armor).
/// Returns the sequence of results, one per square.
pub fn execute_ball_and_chain(
    state: &mut GameState,
    player_id: &PlayerId,
    rng: &mut GameRng,
) -> Vec<BallAndChainResult> {
    let ma = {
        let team = state.field.player_team(player_id).expect("team");
        state.team(team).player_by_id(player_id)
            .map(|p| p.effective_ma())
            .unwrap_or(4)
    };

    let mut results = Vec::new();
    for _ in 0..ma {
        let r = ball_and_chain_move_square(state, player_id, rng);
        let done = matches!(r, BallAndChainResult::OffPitch { .. });
        results.push(r);
        if done {
            return results;
        }
    }

    // After full movement, player falls over — Stunned (no armor roll in BB2025)
    let final_pos = state.field.player_coord(player_id)
        .unwrap_or(FieldCoordinate::new(0, 0));
    state.field.set_player_state(player_id, crate::types::PlayerState::Stunned);
    results.push(BallAndChainResult::FellOver { at: final_pos });
    results
}

// ── ThrowTeamMate action ─────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
pub enum ThrowTeamMateResult {
    /// The thrown player landed at `at` (accurate scatter).
    Landed { at: FieldCoordinate },
    /// AlwaysHungry triggered — the ogre ate the halfling. Target is KO'd.
    Eaten { victim: PlayerId },
    /// The thrown player went off-pitch or the landing failed with injury.
    Injured { victim: PlayerId, at: FieldCoordinate },
}

/// Execute a ThrowTeamMate action.
/// `thrower_id`: the thrower (must have ThrowTeamMate skill).
/// `target_id`: the player being thrown (must have RightStuff or be Stunty-eligible).
/// `landing`: the target landing square.
pub fn execute_throw_team_mate(
    state: &mut GameState,
    thrower_id: &PlayerId,
    target_id: &PlayerId,
    landing: FieldCoordinate,
    rng: &mut GameRng,
) -> ThrowTeamMateResult {
    use crate::mechanics::movement::always_hungry_check;
    use crate::mechanics::injury::{armor_roll, resolve_injury, ArmorOutcome};
    use crate::types::{PITCH_WIDTH, PITCH_HEIGHT};

    // AlwaysHungry check: thrower may eat target instead of throwing
    let has_always_hungry = state.home.player_by_id(thrower_id)
        .or_else(|| state.away.player_by_id(thrower_id))
        .map(|p| p.has_skill(SkillId::AlwaysHungry))
        .unwrap_or(false);
    if has_always_hungry && always_hungry_check(rng) {
        state.field.set_player_state(target_id, crate::types::PlayerState::Ko);
        return ThrowTeamMateResult::Eaten { victim: target_id.clone() };
    }

    // Scatter the landing: d8 direction + d6 distance deviation
    let dir = rng.roll_scatter_direction();
    let dist_roll = rng.roll_d6();
    let (dx, dy) = crate::pathfinding::scatter_delta(dir);
    // Scatter: 1-3 → no scatter (accurate), 4-6 → scatter 1 square in direction
    let actual_landing = if dist_roll <= 3 {
        landing
    } else {
        let nx = landing.x as i16 + dx as i16;
        let ny = landing.y as i16 + dy as i16;
        if nx < 0 || nx >= PITCH_WIDTH as i16 || ny < 0 || ny >= PITCH_HEIGHT as i16 {
            // Off pitch — crowd push on target
            let av = state.home.player_by_id(target_id)
                .or_else(|| state.away.player_by_id(target_id))
                .map(|p| p.effective_av())
                .unwrap_or(7);
            if armor_roll(av, 0, 1, rng) == ArmorOutcome::Broken {
                let inj = resolve_injury(0, rng);
                state.field.set_player_state(target_id, inj.new_state);
            } else {
                state.field.set_player_state(target_id, crate::types::PlayerState::Ko);
            }
            return ThrowTeamMateResult::Injured { victim: target_id.clone(), at: landing };
        }
        FieldCoordinate::new(nx as u8, ny as u8)
    };

    // Move target to landing square
    state.field.move_player(target_id, actual_landing);

    // Landing roll: target must pass an AG roll or be injured
    let ag = state.home.player_by_id(target_id)
        .or_else(|| state.away.player_by_id(target_id))
        .map(|p| p.effective_ag())
        .unwrap_or(3);
    let tz = state.field.tackle_zones_on(actual_landing, {
        let team = state.field.player_team(target_id).expect("team");
        team
    });
    let min_roll = (6u8.saturating_sub(ag.saturating_sub(1)) + tz).min(6);
    let roll = rng.roll_d6();
    if roll >= min_roll {
        ThrowTeamMateResult::Landed { at: actual_landing }
    } else {
        // Failed landing — knocked down + armor roll
        state.field.set_player_state(target_id, crate::types::PlayerState::Prone);
        let av = state.home.player_by_id(target_id)
            .or_else(|| state.away.player_by_id(target_id))
            .map(|p| p.effective_av())
            .unwrap_or(7);
        if armor_roll(av, 0, 0, rng) == ArmorOutcome::Broken {
            let inj = resolve_injury(0, rng);
            state.field.set_player_state(target_id, inj.new_state);
        }
        ThrowTeamMateResult::Injured { victim: target_id.clone(), at: actual_landing }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::game_state::{ActingPlayer, GameState};
    use crate::model::player::{Player, PlayerStats};
    use crate::model::team::Team;
    use crate::skills::SkillSet;
    use crate::types::{FieldCoordinate, PlayerId, PlayerState, TeamId};

    fn setup(pid: &str, start: FieldCoordinate, ma: u8) -> (GameState, PlayerId) {
        let id = PlayerId(pid.into());
        // Use 0 rerolls so tests exercise direct knockdown/turnover behavior without reroll dialogs.
        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 0, true);
        home.add_player(Player::new(
            id.clone(), pid.into(), "lineman".into(), TeamId::Home, 1,
            PlayerStats::new(ma, 3, 4, 8, None), SkillSet::empty(),
        ));
        let away = Team::new("a".into(), "Away".into(), "Orc".into(), 0, false);
        let mut state = GameState::new(home, away);
        state.field.place_player(id.clone(), TeamId::Home, start, PlayerState::Standing);
        state.acting_player = Some(ActingPlayer::new(id.clone(), TeamId::Home));
        state.home_is_active = true;
        (state, id)
    }

    // ── Prone stand-up costs 3 MA ─────────────────────────────────────────────

    #[test]
    fn prone_stand_up_costs_3_ma() {
        // Player with MA=6, Prone state, no JumpUp.
        // begin_move should charge 3 MA and set state to Standing.
        use crate::rng::GameRng;
        use crate::skills::SkillSet;

        let pid = PlayerId("prone_p".into());
        let ma = 6u8;
        let mut home = crate::model::team::Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        home.add_player(crate::model::player::Player::new(
            pid.clone(), "Prone Player".into(), "lineman".into(), TeamId::Home, 1,
            crate::model::player::PlayerStats::new(ma, 3, 4, 8, None), SkillSet::empty(),
        ));
        let away = crate::model::team::Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        let mut state = crate::model::game_state::GameState::new(home, away);
        let start = FieldCoordinate::new(5, 5);
        state.field.place_player(pid.clone(), TeamId::Home, start, crate::types::PlayerState::Prone);
        state.acting_player = Some(crate::model::game_state::ActingPlayer::new(pid.clone(), TeamId::Home));
        state.home_is_active = true;

        // Move to an adjacent square (no dodge needed if no TZ, and for this test we
        // use an empty path so execute_move_step handles it cleanly)
        let dest = FieldCoordinate::new(6, 5);
        // Roll needed: GFI not needed (MA=6, after 3 MA spent still has 3 left, moving 1 square).
        // No dodge roll needed (no opposing TZ). But execute_move_step still needs no dice.
        // Actually execute_move_step rolls for dodge/GFI only when needed.
        // Movement: 1 square, not a GFI. No TZ → no dodge. So no dice consumed.
        let mut rng = GameRng::new_test([]);
        let _result = begin_move(&mut state, &pid, &[dest], &mut rng);

        let ap = state.acting_player.as_ref().expect("acting player");
        // 3 MA for stand-up + 1 for the move step = 4 total
        assert!(ap.movement_used >= 3, "movement_used should be at least 3 after stand-up");
        assert_eq!(
            state.field.player_state(&pid).unwrap_or(crate::types::PlayerState::Prone),
            crate::types::PlayerState::Standing,
            "player should be Standing after stand-up (state set in begin_move before move)"
        );
    }

    #[test]
    fn prone_stand_up_no_cost_with_jump_up() {
        // Player with JumpUp should already be Standing from begin_activation,
        // so begin_move should NOT charge the 3 MA stand-up cost.
        use crate::rng::GameRng;
        use crate::skills::{SkillId, SkillSet};

        let pid = PlayerId("jump_up_p".into());
        let skills: SkillSet = [SkillId::JumpUp].into_iter().collect();
        let ma = 6u8;
        let mut home = crate::model::team::Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        home.add_player(crate::model::player::Player::new(
            pid.clone(), "JumpUp Player".into(), "blitzer".into(), TeamId::Home, 1,
            crate::model::player::PlayerStats::new(ma, 3, 4, 8, None), skills,
        ));
        let away = crate::model::team::Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        let mut state = crate::model::game_state::GameState::new(home, away);
        let start = FieldCoordinate::new(5, 5);
        // JumpUp player is already Standing (begin_activation would have stood them up)
        state.field.place_player(pid.clone(), TeamId::Home, start, crate::types::PlayerState::Standing);
        state.acting_player = Some(crate::model::game_state::ActingPlayer::new(pid.clone(), TeamId::Home));
        state.home_is_active = true;

        let dest = FieldCoordinate::new(6, 5);
        let mut rng = GameRng::new_test([]);
        let _result = begin_move(&mut state, &pid, &[dest], &mut rng);

        let ap = state.acting_player.as_ref().expect("acting player");
        assert!(ap.movement_used < 3, "JumpUp player should not pay 3 MA stand-up cost");
    }

    #[test]
    fn pickup_success_on_good_roll() {
        let start = FieldCoordinate::new(5, 5);
        let ball_coord = FieldCoordinate::new(6, 5);
        let (mut state, pid) = setup("p1", start, 6);
        state.field.ball.coord = Some(ball_coord);
        state.field.ball.in_play = true;
        let path = vec![ball_coord];
        // AG4, no TZ → min_roll=2; roll=3 → success
        let mut rng = GameRng::new_test([3]);
        let result = begin_move(&mut state, &pid, &path, &mut rng);
        assert!(matches!(result, MoveStepResult::PickedUpBall | MoveStepResult::Success));
    }

    #[test]
    fn pickup_fail_causes_turnover() {
        let start = FieldCoordinate::new(5, 5);
        let ball_coord = FieldCoordinate::new(6, 5);
        let (mut state, pid) = setup("p1", start, 6);
        state.field.ball.coord = Some(ball_coord);
        state.field.ball.in_play = true;
        let path = vec![ball_coord];
        // Roll=1 → fail; no SureHands → ball scatters then turnover
        // [1] = pickup fail, [3] = scatter direction (d8), [2] = scatter distance (d6)
        let mut rng = GameRng::new_test([1, 3, 2]);
        let result = begin_move(&mut state, &pid, &path, &mut rng);
        assert!(matches!(result, MoveStepResult::Turnover { .. }));
        // Ball should have scattered from the pickup square
        assert_ne!(state.field.ball.coord, Some(ball_coord), "Ball should bounce on pickup failure");
    }

    fn setup_with_skills(pid: &str, start: FieldCoordinate, skills: SkillSet) -> (GameState, PlayerId) {
        let id = PlayerId(pid.into());
        // Use 0 rerolls so tests exercise direct behavior without reroll dialogs.
        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 0, true);
        home.add_player(Player::new(
            id.clone(), pid.into(), "lineman".into(), TeamId::Home, 1,
            PlayerStats::new(6, 3, 4, 8, None), skills,
        ));
        let away = Team::new("a".into(), "Away".into(), "Orc".into(), 0, false);
        let mut state = GameState::new(home, away);
        state.field.place_player(id.clone(), TeamId::Home, start, PlayerState::Standing);
        state.acting_player = Some(ActingPlayer::new(id.clone(), TeamId::Home));
        state.home_is_active = true;
        (state, id)
    }

    #[test]
    fn can_leap_with_pogo() {
        use crate::skills::SkillId;
        let start = FieldCoordinate::new(5, 5);
        let skills: SkillSet = [SkillId::Pogo].into_iter().collect();
        let (state, pid) = setup_with_skills("p_pogo", start, skills);
        assert!(super::can_leap(&state, &pid));
    }

    #[test]
    fn can_leap_with_leap_skill() {
        use crate::skills::SkillId;
        let start = FieldCoordinate::new(5, 5);
        let skills: SkillSet = [SkillId::Leap].into_iter().collect();
        let (state, pid) = setup_with_skills("p_leap", start, skills);
        assert!(super::can_leap(&state, &pid));
    }

    #[test]
    fn can_leap_without_skill_is_false() {
        let start = FieldCoordinate::new(5, 5);
        let (state, pid) = setup("p_noleap", start, 6);
        assert!(!super::can_leap(&state, &pid));
    }

    #[test]
    fn can_swoop_with_swoop_skill() {
        use crate::skills::SkillId;
        let start = FieldCoordinate::new(5, 5);
        let skills: SkillSet = [SkillId::Swoop].into_iter().collect();
        let (state, pid) = setup_with_skills("p_swoop", start, skills);
        assert!(super::can_swoop(&state, &pid));
    }

    #[test]
    fn can_swoop_without_skill_is_false() {
        let start = FieldCoordinate::new(5, 5);
        let (state, pid) = setup("p_noswoop", start, 6);
        assert!(!super::can_swoop(&state, &pid));
    }

    #[test]
    fn steady_footing_with_skill() {
        use crate::skills::SkillId;
        let start = FieldCoordinate::new(5, 5);
        let skills: SkillSet = [SkillId::SteadyFooting].into_iter().collect();
        let (state, pid) = setup_with_skills("p_sf", start, skills);
        assert!(super::steady_footing_gfi_exempt(&state, &pid));
    }

    #[test]
    fn steady_footing_without_skill_is_false() {
        let start = FieldCoordinate::new(5, 5);
        let (state, pid) = setup("p_nosf", start, 6);
        assert!(!super::steady_footing_gfi_exempt(&state, &pid));
    }

    #[test]
    fn hit_and_run_with_skill() {
        use crate::skills::SkillId;
        let start = FieldCoordinate::new(5, 5);
        let skills: SkillSet = [SkillId::HitAndRun].into_iter().collect();
        let (state, pid) = setup_with_skills("p_har", start, skills);
        assert!(super::hit_and_run_available(&state, &pid));
    }

    #[test]
    fn hit_and_run_without_skill_is_false() {
        let start = FieldCoordinate::new(5, 5);
        let (state, pid) = setup("p_nohar", start, 6);
        assert!(!super::hit_and_run_available(&state, &pid));
    }

    #[test]
    fn fumblerooski_player_with_ball() {
        use crate::skills::SkillId;
        let start = FieldCoordinate::new(5, 5);
        let skills: SkillSet = [SkillId::Fumblerooski].into_iter().collect();
        let (mut state, pid) = setup_with_skills("p_fumb", start, skills);
        state.field.ball.coord = Some(start);
        state.field.ball.in_play = true;
        assert!(super::can_fumblerooski(&state, &pid));
    }

    #[test]
    fn fumblerooski_player_without_ball_is_false() {
        use crate::skills::SkillId;
        let start = FieldCoordinate::new(5, 5);
        let skills: SkillSet = [SkillId::Fumblerooski].into_iter().collect();
        let (mut state, pid) = setup_with_skills("p_fumb2", start, skills);
        // Ball is elsewhere
        state.field.ball.coord = Some(FieldCoordinate::new(10, 10));
        state.field.ball.in_play = true;
        assert!(!super::can_fumblerooski(&state, &pid));
    }

    // ── Shadowing tests ───────────────────────────────────────────────────────

    #[test]
    fn shadowing_succeeds_when_shadower_rolls_higher() {
        // mover MA=6, shadower MA=6
        // mover rolls d6=1 → total=7; shadower rolls d6=3 → total=9 ≥ 7 → success
        let mut rng = GameRng::new_test([1, 3]); // mover_roll then shadower_roll
        assert!(super::shadowing_succeeds(6, 6, &mut rng));
    }

    #[test]
    fn shadowing_fails_when_mover_rolls_higher() {
        // mover MA=6, shadower MA=3
        // mover rolls d6=6 → total=12; shadower rolls d6=2 → total=5 < 12 → fail
        let mut rng = GameRng::new_test([6, 2]);
        assert!(!super::shadowing_succeeds(6, 3, &mut rng));
    }

    #[test]
    fn shadowing_succeeds_on_tie() {
        // Shadowing succeeds on tie (≥)
        // mover MA=4 + d6=3 = 7; shadower MA=4 + d6=3 = 7 → tie → success
        let mut rng = GameRng::new_test([3, 3]);
        assert!(super::shadowing_succeeds(4, 4, &mut rng));
    }

    fn make_shadow_state() -> (GameState, PlayerId, PlayerId) {
        use crate::model::player::{Player, PlayerStats};
        use crate::model::team::Team;
        use crate::skills::{SkillId, SkillSet};
        use crate::types::{FieldCoordinate, PlayerState, TeamId};

        let mover_id = PlayerId("mover".into());
        let shadower_id = PlayerId("shadower".into());
        let shadow_skills: SkillSet = [SkillId::Shadowing].into_iter().collect();

        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        home.add_player(Player::new(
            mover_id.clone(), "Mover".into(), "blitzer".into(), TeamId::Home, 1,
            PlayerStats::new(7, 3, 4, 8, None), SkillSet::empty(),
        ));
        let mut away = Team::new("a".into(), "Away".into(), "Skaven".into(), 3, false);
        away.add_player(Player::new(
            shadower_id.clone(), "Shadower".into(), "gutter_runner".into(), TeamId::Away, 1,
            PlayerStats::new(9, 2, 4, 7, None), shadow_skills,
        ));

        let mut state = GameState::new(home, away);
        state.field.place_player(mover_id.clone(), TeamId::Home, FieldCoordinate::new(8, 5), PlayerState::Standing);
        state.field.place_player(shadower_id.clone(), TeamId::Away, FieldCoordinate::new(7, 5), PlayerState::Standing);
        state.home_is_active = true;
        (state, mover_id, shadower_id)
    }

    #[test]
    fn apply_shadowing_check_succeeds_when_shadower_wins() {
        let (state, mover_id, shadower_id) = make_shadow_state();
        // mover MA=7, shadower MA=9
        // mover d6=1 → total=8; shadower d6=5 → total=14 ≥ 8 → success
        let mut rng = GameRng::new_test([1, 5]);
        assert!(super::apply_shadowing_check(&state, &mover_id, &shadower_id, &mut rng));
    }

    #[test]
    fn apply_shadowing_check_fails_when_mover_wins() {
        let (state, mover_id, shadower_id) = make_shadow_state();
        // mover d6=6 → total=13; shadower d6=1 → total=10 < 13 → fail
        let mut rng = GameRng::new_test([6, 1]);
        assert!(!super::apply_shadowing_check(&state, &mover_id, &shadower_id, &mut rng));
    }

    #[test]
    fn apply_shadowing_check_fails_without_skill() {
        let (mut state, mover_id, shadower_id) = make_shadow_state();
        // Remove Shadowing skill from shadower
        state.away.player_by_id_mut(&shadower_id).unwrap().skills = SkillSet::empty();
        let mut rng = GameRng::new_test([]);
        assert!(!super::apply_shadowing_check(&state, &mover_id, &shadower_id, &mut rng));
    }

    #[test]
    fn noball_player_always_fails_pickup() {
        use crate::skills::SkillId;
        let start = FieldCoordinate::new(5, 5);
        let ball_coord = FieldCoordinate::new(6, 5);
        let skills: SkillSet = [SkillId::NoBall].into_iter().collect();
        let (mut state, pid) = setup_with_skills("p_noball", start, skills);
        state.field.ball.coord = Some(ball_coord);
        state.field.ball.in_play = true;
        let path = vec![ball_coord];
        // Roll would succeed (high value) but NoBall prevents it
        let mut rng = GameRng::new_test([6, 6, 6]);
        let result = begin_move(&mut state, &pid, &path, &mut rng);
        assert!(matches!(result, MoveStepResult::Turnover { .. }));
    }

    // ── T-57 Incorporeal tests ─────────────────────────────────────────────

    #[test]
    fn incorporeal_with_skill() {
        use crate::skills::SkillId;
        let start = FieldCoordinate::new(5, 5);
        let skills: SkillSet = [SkillId::Incorporeal].into_iter().collect();
        let (state, pid) = setup_with_skills("p_inc", start, skills);
        assert!(super::is_incorporeal(&state, &pid));
    }

    #[test]
    fn incorporeal_without_skill() {
        let start = FieldCoordinate::new(5, 5);
        let (state, pid) = setup("p_noinc", start, 6);
        assert!(!super::is_incorporeal(&state, &pid));
    }

    // ── T-57 WoodlandFury tests ────────────────────────────────────────────

    #[test]
    fn woodland_fury_with_skill() {
        use crate::skills::SkillId;
        let start = FieldCoordinate::new(5, 5);
        let skills: SkillSet = [SkillId::WoodlandFury].into_iter().collect();
        let (state, pid) = setup_with_skills("p_wf", start, skills);
        assert!(super::woodland_fury_leap_available(&state, &pid));
    }

    #[test]
    fn woodland_fury_without_skill() {
        let start = FieldCoordinate::new(5, 5);
        let (state, pid) = setup("p_nowf", start, 6);
        assert!(!super::woodland_fury_leap_available(&state, &pid));
    }

    // ── T-58 PumpUpTheCrowd tests ──────────────────────────────────────────

    #[test]
    fn pump_up_adjacent_ally_gets_bonus() {
        use crate::skills::SkillId;
        let start = FieldCoordinate::new(5, 5);
        let ally_start = FieldCoordinate::new(6, 5); // adjacent
        let skills: SkillSet = [SkillId::PumpUpTheCrowd].into_iter().collect();
        let (mut state, pid) = setup_with_skills("p_pump", start, skills);
        // Add an ally player
        let ally_id = PlayerId("ally".into());
        let ally = Player::new(
            ally_id.clone(), "ally".into(), "lineman".into(), TeamId::Home, 2,
            PlayerStats::new(6, 3, 4, 8, None), SkillSet::empty(),
        );
        state.home.add_player(ally);
        state.field.place_player(ally_id.clone(), TeamId::Home, ally_start, PlayerState::Standing);
        assert_eq!(super::pump_up_ag_bonus(&state, &pid, &ally_id), 1);
    }

    #[test]
    fn pump_up_non_adjacent_ally_no_bonus() {
        use crate::skills::SkillId;
        let start = FieldCoordinate::new(5, 5);
        let ally_start = FieldCoordinate::new(10, 10); // not adjacent
        let skills: SkillSet = [SkillId::PumpUpTheCrowd].into_iter().collect();
        let (mut state, pid) = setup_with_skills("p_pump2", start, skills);
        let ally_id = PlayerId("ally2".into());
        let ally = Player::new(
            ally_id.clone(), "ally2".into(), "lineman".into(), TeamId::Home, 2,
            PlayerStats::new(6, 3, 4, 8, None), SkillSet::empty(),
        );
        state.home.add_player(ally);
        state.field.place_player(ally_id.clone(), TeamId::Home, ally_start, PlayerState::Standing);
        assert_eq!(super::pump_up_ag_bonus(&state, &pid, &ally_id), 0);
    }

    // ── T-58 ZoatStBonus tests ─────────────────────────────────────────────

    #[test]
    fn zoat_st_bonus_with_skill() {
        use crate::skills::SkillId;
        let start = FieldCoordinate::new(5, 5);
        let skills: SkillSet = [SkillId::ExcuseMeAreYouAZoat].into_iter().collect();
        let (state, pid) = setup_with_skills("p_zoat", start, skills);
        assert_eq!(super::zoat_st_bonus(&state, &pid), 2);
    }

    #[test]
    fn zoat_st_bonus_without_skill() {
        let start = FieldCoordinate::new(5, 5);
        let (state, pid) = setup("p_nozoat", start, 6);
        assert_eq!(super::zoat_st_bonus(&state, &pid), 0);
    }

    // ── SafePairOfHands tests ──────────────────────────────────────────────

    #[test]
    fn safe_pair_of_hands_no_turnover_on_knockdown() {
        use crate::rng::GameRng;
        use crate::skills::SkillId;

        // Player with SafePairOfHands at (5,5), carrying ball.
        // Opponent at (6,5) creates a TZ. Player tries to move to (4,5).
        // Dodge fails → knocked down. Ball stays, no turnover.
        let pid = PlayerId("sph_p".into());
        let away_pid = PlayerId("opp_sph".into());
        let skills: SkillSet = [SkillId::SafePairOfHands].into_iter().collect();

        // Use 0 rerolls so no team reroll dialog is offered (tests direct knockdown behavior)
        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 0, true);
        home.add_player(Player::new(
            pid.clone(), "SPH Player".into(), "catcher".into(), TeamId::Home, 1,
            PlayerStats::new(6, 3, 4, 8, None), skills,
        ));
        let mut away = Team::new("a".into(), "Away".into(), "Orc".into(), 0, false);
        away.add_player(Player::new(
            away_pid.clone(), "Blocker".into(), "lineman".into(), TeamId::Away, 1,
            PlayerStats::new(5, 3, 3, 9, None), SkillSet::empty(),
        ));
        let start = FieldCoordinate::new(5, 5);
        let mut state = GameState::new(home, away);
        state.field.place_player(pid.clone(), TeamId::Home, start, PlayerState::Standing);
        state.field.place_player(away_pid.clone(), TeamId::Away, FieldCoordinate::new(6, 5), PlayerState::Standing);
        // Ball is at player's square
        state.field.ball.coord = Some(start);
        state.field.ball.in_play = true;
        state.acting_player = Some(ActingPlayer::new(pid.clone(), TeamId::Home));
        state.home_is_active = true;

        // dodge_min_roll(ag=4, tz=1, no_dodge) = 3; roll=1 → fail, no reroll
        let path = vec![FieldCoordinate::new(4, 5)];
        let mut rng = GameRng::new_test([1]);
        let result = begin_move(&mut state, &pid, &path, &mut rng);

        // Should be KnockedDown, NOT Turnover
        assert!(
            matches!(result, MoveStepResult::KnockedDown { .. }),
            "SafePairOfHands: knocked down while carrying ball should return KnockedDown not Turnover, got {:?}", result
        );
    }

    #[test]
    fn no_safe_pair_of_hands_causes_turnover_on_knockdown() {
        use crate::rng::GameRng;
        use crate::skills::SkillId;

        // Player WITHOUT SafePairOfHands, carrying ball, knocked down → Turnover.
        let pid = PlayerId("nosph_p".into());
        let away_pid = PlayerId("opp_nosph".into());

        // Use 0 rerolls so no team reroll dialog is offered (tests direct knockdown behavior)
        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 0, true);
        home.add_player(Player::new(
            pid.clone(), "Normal Player".into(), "lineman".into(), TeamId::Home, 1,
            PlayerStats::new(6, 3, 4, 8, None), SkillSet::empty(),
        ));
        let mut away = Team::new("a".into(), "Away".into(), "Orc".into(), 0, false);
        away.add_player(Player::new(
            away_pid.clone(), "Blocker".into(), "lineman".into(), TeamId::Away, 1,
            PlayerStats::new(5, 3, 3, 9, None), SkillSet::empty(),
        ));
        let start = FieldCoordinate::new(5, 5);
        let mut state = GameState::new(home, away);
        state.field.place_player(pid.clone(), TeamId::Home, start, PlayerState::Standing);
        state.field.place_player(away_pid.clone(), TeamId::Away, FieldCoordinate::new(6, 5), PlayerState::Standing);
        state.field.ball.coord = Some(start);
        state.field.ball.in_play = true;
        state.acting_player = Some(ActingPlayer::new(pid.clone(), TeamId::Home));
        state.home_is_active = true;

        let path = vec![FieldCoordinate::new(4, 5)];
        // [1] = dodge fail (knocked down), [3] = scatter direction (d8), [1] = scatter distance (d6)
        let mut rng = GameRng::new_test([1, 3, 1]);
        let result = begin_move(&mut state, &pid, &path, &mut rng);

        assert!(
            matches!(result, MoveStepResult::Turnover { .. }),
            "Without SafePairOfHands: knocked down while carrying ball should be Turnover, got {:?}", result
        );
        // Ball should have scattered away from start square
        assert_ne!(state.field.ball.coord, Some(start), "Ball should have scattered from knockdown square");
    }

    // ── ExtraArms pickup tests ─────────────────────────────────────────────────

    #[test]
    fn extra_arms_reduces_pickup_min_roll() {
        // Player with AG=2, ExtraArms, with 1 opposing TZ.
        // Normal: base = max(2, 5-2) = 3. tz_mod = max(0, 1-1) = 0 (ExtraArms already
        // reduces opp_tz by 1). min_roll = 3+0 = 3.
        // Without ExtraArms: base=3, tz_mod=1, min_roll=4.
        // Roll=3: without ExtraArms → fail. With ExtraArms → success.
        use crate::rng::GameRng;
        use crate::skills::SkillId;

        let pid = PlayerId("ea_player".into());
        let opp_id = PlayerId("ea_opp".into());
        let skills: SkillSet = [SkillId::ExtraArms].into_iter().collect();

        let start = FieldCoordinate::new(5, 5);
        let ball_coord = FieldCoordinate::new(6, 5);

        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        home.add_player(Player::new(
            pid.clone(), "EA Player".into(), "lineman".into(), TeamId::Home, 1,
            PlayerStats::new(6, 3, 2, 8, None), // AG=2 → base=3
            skills,
        ));
        let mut away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        away.add_player(Player::new(
            opp_id.clone(), "Blocker".into(), "lineman".into(), TeamId::Away, 1,
            PlayerStats::new(5, 3, 3, 9, None), SkillSet::empty(),
        ));
        let mut state = GameState::new(home, away);
        state.field.place_player(pid.clone(), TeamId::Home, start, PlayerState::Standing);
        // Opponent at ball square neighbor (7,5) exerts TZ on ball_coord (6,5)
        state.field.place_player(opp_id.clone(), TeamId::Away, FieldCoordinate::new(7, 5), PlayerState::Standing);
        state.field.ball.coord = Some(ball_coord);
        state.field.ball.in_play = true;
        state.acting_player = Some(ActingPlayer::new(pid.clone(), TeamId::Home));
        state.home_is_active = true;

        // ExtraArms: opp_tz=1, tz_mod = 1-1 = 0. min_roll = 3. Roll=3 → success.
        let path = vec![ball_coord];
        let mut rng = GameRng::new_test([3]);
        let result = begin_move(&mut state, &pid, &path, &mut rng);
        assert!(
            matches!(result, MoveStepResult::PickedUpBall | MoveStepResult::Success),
            "ExtraArms should reduce effective TZ so roll=3 succeeds pickup, got {:?}", result
        );
    }

    #[test]
    fn without_extra_arms_pickup_fails_on_low_roll() {
        // Same scenario without ExtraArms: opp_tz=1, min_roll=4. Roll=3 → fail.
        use crate::rng::GameRng;

        let pid = PlayerId("no_ea_player".into());
        let opp_id = PlayerId("no_ea_opp".into());

        let start = FieldCoordinate::new(5, 5);
        let ball_coord = FieldCoordinate::new(6, 5);

        // Use 0 rerolls so no team reroll dialog is offered
        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 0, true);
        home.add_player(Player::new(
            pid.clone(), "Normal Player".into(), "lineman".into(), TeamId::Home, 1,
            PlayerStats::new(6, 3, 2, 8, None), // AG=2 → base=3
            SkillSet::empty(),
        ));
        let mut away = Team::new("a".into(), "Away".into(), "Orc".into(), 0, false);
        away.add_player(Player::new(
            opp_id.clone(), "Blocker".into(), "lineman".into(), TeamId::Away, 1,
            PlayerStats::new(5, 3, 3, 9, None), SkillSet::empty(),
        ));
        let mut state = GameState::new(home, away);
        state.field.place_player(pid.clone(), TeamId::Home, start, PlayerState::Standing);
        state.field.place_player(opp_id.clone(), TeamId::Away, FieldCoordinate::new(7, 5), PlayerState::Standing);
        state.field.ball.coord = Some(ball_coord);
        state.field.ball.in_play = true;
        state.acting_player = Some(ActingPlayer::new(pid.clone(), TeamId::Home));
        state.home_is_active = true;

        // Without ExtraArms: opp_tz=1, min_roll=4. Roll=3 < 4 → fail → ball scatters → Turnover.
        let path = vec![ball_coord];
        // [3] = pickup fail, [5] = scatter direction (d8), [1] = scatter distance (d6)
        let mut rng = GameRng::new_test([3, 5, 1]);
        let result = begin_move(&mut state, &pid, &path, &mut rng);
        assert!(
            matches!(result, MoveStepResult::Turnover { .. }),
            "Without ExtraArms, roll=3 should fail pickup (min_roll=4 with 1 TZ), got {:?}", result
        );
    }

    // ── Team reroll dialog for movement ───────────────────────────────────────

    #[test]
    fn team_reroll_offered_when_dodge_fails_and_rerolls_available() {
        use crate::model::game_state::DialogState;
        // Player with 3 team rerolls, dodge fails → PendingTeamReroll dialog shown.
        let pid = PlayerId("dodger".into());
        let opp_pid = PlayerId("opp".into());

        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        home.add_player(Player::new(
            pid.clone(), "Dodger".into(), "lineman".into(), TeamId::Home, 1,
            PlayerStats::new(6, 3, 4, 8, None), SkillSet::empty(),
        ));
        let mut away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        away.add_player(Player::new(
            opp_pid.clone(), "Blocker".into(), "lineman".into(), TeamId::Away, 1,
            PlayerStats::new(5, 3, 3, 9, None), SkillSet::empty(),
        ));
        let start = FieldCoordinate::new(5, 5);
        let mut state = GameState::new(home, away);
        state.field.place_player(pid.clone(), TeamId::Home, start, PlayerState::Standing);
        state.field.place_player(opp_pid.clone(), TeamId::Away, FieldCoordinate::new(6, 5), PlayerState::Standing);
        state.acting_player = Some(ActingPlayer::new(pid.clone(), TeamId::Home));
        state.home_is_active = true;

        let path = vec![FieldCoordinate::new(4, 5)];
        let mut rng = GameRng::new_test([1]); // dodge fail
        let result = begin_move(&mut state, &pid, &path, &mut rng);

        assert!(
            matches!(result, MoveStepResult::PendingTeamReroll),
            "With rerolls available, dodge fail should offer team reroll, got {:?}", result
        );
        assert!(
            matches!(state.dialog, DialogState::SelectReroll { .. }),
            "Dialog should be SelectReroll when team reroll is offered"
        );
        assert!(
            state.acting_player.as_ref().unwrap().pending_move.is_some(),
            "pending_move should be set when team reroll is offered"
        );
    }

    #[test]
    fn team_reroll_accepted_resumes_move_success() {
        // Player with team rerolls: dodge fails, team reroll accepted, second roll succeeds.
        let pid = PlayerId("dodger2".into());
        let opp_pid = PlayerId("opp2".into());

        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        home.add_player(Player::new(
            pid.clone(), "Dodger".into(), "lineman".into(), TeamId::Home, 1,
            PlayerStats::new(6, 3, 4, 8, None), SkillSet::empty(),
        ));
        let mut away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        away.add_player(Player::new(
            opp_pid.clone(), "Blocker".into(), "lineman".into(), TeamId::Away, 1,
            PlayerStats::new(5, 3, 3, 9, None), SkillSet::empty(),
        ));
        let start = FieldCoordinate::new(5, 5);
        let mut state = GameState::new(home, away);
        state.field.place_player(pid.clone(), TeamId::Home, start, PlayerState::Standing);
        state.field.place_player(opp_pid.clone(), TeamId::Away, FieldCoordinate::new(6, 5), PlayerState::Standing);
        state.acting_player = Some(ActingPlayer::new(pid.clone(), TeamId::Home));
        state.home_is_active = true;

        let dest = FieldCoordinate::new(4, 5);
        let path = vec![dest];
        let mut rng = GameRng::new_test([1]); // dodge fail
        begin_move(&mut state, &pid, &path, &mut rng);

        // Now accept the reroll with a successful second roll
        let mut rng2 = GameRng::new_test([5]); // success on reroll (min_roll=3, 5>=3)
        let resume = resume_move_after_reroll(&mut state, &pid, true, &mut rng2);

        assert!(
            matches!(resume, MoveStepResult::Success),
            "After accepted reroll with success, move should complete, got {:?}", resume
        );
        assert_eq!(state.field.player_coord(&pid), Some(dest), "Player should be at destination");
        assert_eq!(state.home.rerolls_remaining, 2, "One team reroll should be consumed");
    }

    #[test]
    fn team_reroll_declined_causes_knockdown() {
        // Player with team rerolls: dodge fails, team reroll declined → knockdown.
        let pid = PlayerId("dodger3".into());
        let opp_pid = PlayerId("opp3".into());

        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        home.add_player(Player::new(
            pid.clone(), "Dodger".into(), "lineman".into(), TeamId::Home, 1,
            PlayerStats::new(6, 3, 4, 8, None), SkillSet::empty(),
        ));
        let mut away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        away.add_player(Player::new(
            opp_pid.clone(), "Blocker".into(), "lineman".into(), TeamId::Away, 1,
            PlayerStats::new(5, 3, 3, 9, None), SkillSet::empty(),
        ));
        let start = FieldCoordinate::new(5, 5);
        let mut state = GameState::new(home, away);
        state.field.place_player(pid.clone(), TeamId::Home, start, PlayerState::Standing);
        state.field.place_player(opp_pid.clone(), TeamId::Away, FieldCoordinate::new(6, 5), PlayerState::Standing);
        state.acting_player = Some(ActingPlayer::new(pid.clone(), TeamId::Home));
        state.home_is_active = true;

        let path = vec![FieldCoordinate::new(4, 5)];
        let mut rng = GameRng::new_test([1]); // dodge fail
        begin_move(&mut state, &pid, &path, &mut rng);

        let mut rng2 = GameRng::new_test([]);
        let resume = resume_move_after_reroll(&mut state, &pid, false, &mut rng2);

        assert!(
            matches!(resume, MoveStepResult::KnockedDown { .. }),
            "Declining reroll should cause knockdown, got {:?}", resume
        );
        assert_eq!(state.field.player_state(&pid), Some(PlayerState::Prone));
        assert_eq!(state.home.rerolls_remaining, 3, "Reroll count unchanged when declined");
    }
}
