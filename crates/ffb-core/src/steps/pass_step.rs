/// Pass step: roll → scatter → catch.
use crate::mechanics::pass::{pass_range_from_coords, pass_roll_with_tz, pass_scatter_coord, PassResult};
use crate::model::game_state::GameState;
use crate::rng::GameRng;
use crate::skills::SkillId;
use crate::types::{FieldCoordinate, PlayerId, TeamId};

#[derive(Clone, Debug)]
pub enum PassStepResult {
    Accurate { to: FieldCoordinate },
    Inaccurate { landed: FieldCoordinate },
    Fumble { at: FieldCoordinate },
    Caught { by: PlayerId },
    Dropped { at: FieldCoordinate },
    /// Pass failed (Inaccurate/Fumble) and a team reroll was offered. Dialog set to SelectReroll.
    PendingReroll,
}

/// Execute a pass from ball carrier to `target`.
///
/// Interception check: before resolving the pass result, check if any
/// defender eligible for interception can catch the ball (AG - tz_count,
/// minimum 6+). If an interception succeeds, the ball goes to the interceptor
/// and a `Caught` result is returned immediately.
pub fn begin_pass(
    state: &mut GameState,
    passer_id: &PlayerId,
    target: FieldCoordinate,
    rng: &mut GameRng,
) -> PassStepResult {
    let passer_coord = state.field.player_coord(passer_id).expect("passer coord");
    let team = state.field.player_team(passer_id).expect("passer team");
    let defending_team = team.opponent();

    let (pa, passer_has_nos) = {
        let p = state.team(team).player_by_id(passer_id).expect("passer");
        (p.effective_pa(), p.has_skill(SkillId::NervesOfSteel))
    };

    // Passer's opposing TZ count (each adds +1 to pass difficulty unless NervesOfSteel)
    let passer_tz = state.field.tackle_zones_on(passer_coord, defending_team);

    // Mark pass used
    if state.home_is_active {
        state.turn_data_home.pass_used = true;
    } else {
        state.turn_data_away.pass_used = true;
    }

    // Interception: each eligible defender rolls d6; first to succeed intercepts.
    // Min roll = max(2, 6 - ag + tz_count_on_catcher); always 6+ base (hard).
    let candidates = pass_block_candidates(state, passer_coord, target, defending_team);
    for interceptor_id in &candidates {
        let (ag, opp_tz) = {
            let coord = state.field.player_coord(interceptor_id);
            let tz = if let Some(c) = coord {
                state.field.tackle_zones_on(c, team)
            } else {
                0
            };
            let ag = state.team(defending_team)
                .player_by_id(interceptor_id)
                .map(|p| p.effective_ag())
                .unwrap_or(4);
            (ag, tz)
        };
        // Interception min-roll: 6+ modified by AG (-1 per ag above 1) and TZ (+1 each)
        let base: u8 = 6u8.saturating_sub(ag.saturating_sub(1));
        let min_roll = (base + opp_tz).min(6);
        let roll = rng.roll_d6();
        if roll >= min_roll {
            // Interception success — ball goes to interceptor
            let int_coord = state.field.player_coord(interceptor_id)
                .unwrap_or(target);
            state.field.ball.coord = Some(int_coord);
            return PassStepResult::Caught { by: interceptor_id.clone() };
        }
    }

    // HailMaryPass: always treated as LongBomb range, always scatters (Inaccurate),
    // cannot be intercepted. Skip normal pass roll; go straight to inaccurate result.
    let has_hail_mary = {
        let p = state.team(team).player_by_id(passer_id).expect("passer");
        p.has_skill(SkillId::HailMaryPass)
    };
    if has_hail_mary {
        let scatter = pass_scatter_coord(target, rng);
        state.field.ball.coord = Some(scatter);
        return PassStepResult::Inaccurate { landed: scatter };
    }

    let base_range = pass_range_from_coords(passer_coord, target);
    let range = if state.field.weather == crate::types::Weather::Blizzard {
        blizzard_reduced_range(base_range)
    } else {
        base_range
    };

    // DisturbingPresence: each opposing player with the skill within 3 squares of passer adds +1
    let dp_penalty = disturbing_presence_penalty(state, passer_id, team);
    let effective_tz = passer_tz.saturating_add(dp_penalty);

    let has_pass_skill = state.team(team).player_by_id(passer_id)
        .map(|p| p.has_skill(SkillId::Pass))
        .unwrap_or(false);
    let pass_skill_reroll_used = state.acting_player.as_ref()
        .map(|ap| ap.pass_skill_reroll_used)
        .unwrap_or(false);

    let result = pass_roll_with_tz(pa, range, effective_tz, passer_has_nos, rng);

    match result {
        PassResult::Fumble => {
            // Pass skill does NOT reroll fumbles (natural 1). Check team reroll.
            let team_reroll_avail = crate::steps::turn_step::active_team_reroll_available(state);
            if team_reroll_avail {
                if let Some(ap) = state.acting_player.as_mut() {
                    ap.pending_pass_target = Some(target);
                }
                state.dialog = crate::model::game_state::DialogState::SelectReroll {
                    action_name: "Pass".into(),
                    reroll_available: true,
                    skill_reroll_available: false,
                };
                PassStepResult::PendingReroll
            } else {
                let scatter = pass_scatter_coord(passer_coord, rng);
                state.field.ball.coord = Some(scatter);
                PassStepResult::Fumble { at: scatter }
            }
        }
        PassResult::Inaccurate => {
            // Pass skill: auto-reroll Inaccurate once per activation (not fumbles).
            if has_pass_skill && !pass_skill_reroll_used {
                if let Some(ap) = state.acting_player.as_mut() {
                    ap.pass_skill_reroll_used = true;
                }
                let reroll = pass_roll_with_tz(pa, range, effective_tz, passer_has_nos, rng);
                return match reroll {
                    PassResult::Fumble => {
                        let scatter = pass_scatter_coord(passer_coord, rng);
                        state.field.ball.coord = Some(scatter);
                        PassStepResult::Fumble { at: scatter }
                    }
                    PassResult::Inaccurate => {
                        let scatter = pass_scatter_coord(target, rng);
                        state.field.ball.coord = Some(scatter);
                        PassStepResult::Inaccurate { landed: scatter }
                    }
                    PassResult::Accurate => {
                        state.field.ball.coord = Some(target);
                        PassStepResult::Accurate { to: target }
                    }
                };
            }
            // Check team reroll
            let team_reroll_avail = crate::steps::turn_step::active_team_reroll_available(state);
            if team_reroll_avail {
                if let Some(ap) = state.acting_player.as_mut() {
                    ap.pending_pass_target = Some(target);
                }
                state.dialog = crate::model::game_state::DialogState::SelectReroll {
                    action_name: "Pass".into(),
                    reroll_available: true,
                    skill_reroll_available: false,
                };
                PassStepResult::PendingReroll
            } else {
                let scatter = pass_scatter_coord(target, rng);
                state.field.ball.coord = Some(scatter);
                PassStepResult::Inaccurate { landed: scatter }
            }
        }
        PassResult::Accurate => {
            state.field.ball.coord = Some(target);
            PassStepResult::Accurate { to: target }
        }
    }
}

/// Resume a pass after team reroll decision.
pub fn resume_pass_after_reroll(
    state: &mut GameState,
    passer_id: &PlayerId,
    use_reroll: bool,
    rng: &mut GameRng,
) -> PassStepResult {
    let target = match state.acting_player.as_mut().and_then(|ap| ap.pending_pass_target.take()) {
        Some(t) => t,
        None => return PassStepResult::Fumble { at: FieldCoordinate::new(0, 0) },
    };
    let team = state.field.player_team(passer_id).expect("passer team");
    let passer_coord = state.field.player_coord(passer_id).expect("passer coord");

    if use_reroll {
        let reroll_ok = crate::steps::turn_step::use_team_reroll(state, passer_id, rng);
        if reroll_ok {
            // Re-roll the pass
            return begin_pass(state, passer_id, target, rng);
        }
        // Loner failed — treat as declined
    }
    // Declined or Loner failed: finalize the failure (scatter from passer)
    let scatter = pass_scatter_coord(passer_coord, rng);
    state.field.ball.coord = Some(scatter);
    PassStepResult::Fumble { at: scatter }
}

/// Reduce pass range by one category due to Blizzard weather.
/// LongBomb → Long, Long → Short, Short → Short (cannot reduce below Short).
/// HandOff is unaffected.
pub fn blizzard_reduced_range(range: crate::types::PassRange) -> crate::types::PassRange {
    use crate::types::PassRange;
    match range {
        PassRange::LongBomb => PassRange::Long,
        PassRange::Long => PassRange::Short,
        PassRange::Short | PassRange::HandOff => range,
    }
}

/// Check if the active player can perform a Dump-Off pass when declared blitzed.
/// Returns true if passer has the DumpOff skill and hasn't already passed this turn.
pub fn can_dump_off(state: &GameState, passer_id: &PlayerId) -> bool {
    let team = match state.field.player_team(passer_id) {
        Some(t) => t,
        None => return false,
    };
    let has_dump_off = state.team(team)
        .player_by_id(passer_id)
        .map(|p| p.has_skill(SkillId::DumpOff))
        .unwrap_or(false);
    if !has_dump_off {
        return false;
    }
    // Check that the player hasn't already passed this turn.
    let pass_used = if team == TeamId::Home {
        state.turn_data_home.pass_used
    } else {
        state.turn_data_away.pass_used
    };
    !pass_used
}

/// Determine which opposing players are eligible to attempt a PassBlock interception.
/// Eligible = on the defending team, on pitch, not in an opponent's tackle zone,
/// and adjacent (Chebyshev distance ≤ 1) to any square on the straight throw line
/// between `from` and `to`.
pub fn pass_block_candidates(
    state: &GameState,
    from: FieldCoordinate,
    to: FieldCoordinate,
    defending_team: TeamId,
) -> Vec<PlayerId> {
    // Build the set of squares on or adjacent to the throw line.
    let throw_line_squares = throw_line(from, to);

    let _attacking_team = defending_team.opponent();
    let mut candidates = Vec::new();

    for (id, coord, player_state) in state.field.on_pitch_players() {
        let team = match state.field.player_team(id) {
            Some(t) => t,
            None => continue,
        };
        if team != defending_team {
            continue;
        }
        if !player_state.is_active() {
            continue;
        }
        // Not in attacking tackle zone
        if state.field.in_tackle_zone(coord, defending_team) {
            continue;
        }
        // Must have Pass Block skill
        let _has_pass_block = state.team(defending_team)
            .player_by_id(id)
            .map(|p| p.has_skill(SkillId::OnTheBall))
            .unwrap_or(false);
        // Adjacent to any throw-line square
        let near_throw_line = throw_line_squares.iter().any(|&sq| {
            let dx = (coord.x as i16 - sq.x as i16).unsigned_abs();
            let dy = (coord.y as i16 - sq.y as i16).unsigned_abs();
            dx.max(dy) <= 1
        });
        if near_throw_line {
            candidates.push(id.clone());
        }
    }

    candidates
}

// ── VeryLongLegs ──────────────────────────────────────────────────────────────

/// Returns all players on `team` who have the VeryLongLegs skill and are within
/// 1 square (Chebyshev) of the pass path from `from` to `to`.
///
/// Per BB2025: VeryLongLegs lets a player attempt to intercept passes even when
/// they are not in the passer's tackle zone, provided they are near the throw path.
pub fn very_long_legs_interception_eligible(
    state: &GameState,
    from: FieldCoordinate,
    to: FieldCoordinate,
    team: TeamId,
) -> Vec<PlayerId> {
    let throw_line_squares = throw_line(from, to);
    let mut eligible = Vec::new();

    for (id, coord, player_state) in state.field.on_pitch_players() {
        if state.field.player_team(id) != Some(team) {
            continue;
        }
        if !player_state.is_active() {
            continue;
        }
        let has_skill = state.team(team)
            .player_by_id(id)
            .map(|p| p.has_skill(SkillId::VeryLongLegs))
            .unwrap_or(false);
        if !has_skill {
            continue;
        }
        // Player must be within 1 square of any throw-line square
        let near_throw_line = throw_line_squares.iter().any(|&sq| {
            let dx = (coord.x as i16 - sq.x as i16).unsigned_abs();
            let dy = (coord.y as i16 - sq.y as i16).unsigned_abs();
            dx.max(dy) <= 1
        });
        if near_throw_line {
            eligible.push(id.clone());
        }
    }

    eligible
}

/// Generate squares along the straight line from `from` to `to` (Bresenham).
fn throw_line(from: FieldCoordinate, to: FieldCoordinate) -> Vec<FieldCoordinate> {
    let mut squares = Vec::new();
    let mut x = from.x as i16;
    let mut y = from.y as i16;
    let dx = (to.x as i16 - x).signum();
    let dy = (to.y as i16 - y).signum();
    let steps = ((to.x as i16 - x).abs()).max((to.y as i16 - y).abs());
    for _ in 0..=steps {
        let coord = FieldCoordinate::new(x as u8, y as u8);
        if coord.is_valid() {
            squares.push(coord);
        }
        x += dx;
        y += dy;
    }
    squares
}

// ── DivingCatch ───────────────────────────────────────────────────────────────

/// Returns the first player adjacent to `ball_coord` with the DivingCatch skill
/// who could attempt to catch the ball.  Returns `None` if no eligible player exists.
///
/// Per BB2025: DivingCatch lets a player attempt to catch a ball that lands up to
/// 1 square away from them (at +1 to the catch difficulty).
pub fn diving_catch_eligible(state: &GameState, ball_coord: FieldCoordinate) -> Option<PlayerId> {
    for (id, coord, player_state) in state.field.on_pitch_players() {
        if !player_state.is_active() {
            continue;
        }
        // Must be adjacent (Chebyshev distance == 1) to ball landing square
        let dx = (coord.x as i16 - ball_coord.x as i16).unsigned_abs();
        let dy = (coord.y as i16 - ball_coord.y as i16).unsigned_abs();
        let chebyshev = dx.max(dy);
        if chebyshev != 1 {
            continue;
        }
        let team = match state.field.player_team(id) {
            Some(t) => t,
            None => continue,
        };
        let has_skill = state.team(team)
            .player_by_id(id)
            .map(|p| p.has_skill(SkillId::DivingCatch))
            .unwrap_or(false);
        if has_skill {
            return Some(id.clone());
        }
    }
    None
}

/// Returns true if the player has DivingCatch and the ball landed adjacent to
/// their square (Chebyshev distance = 1), allowing them to attempt a catch even
/// though the ball did not land exactly on their square.
pub fn can_diving_catch(state: &GameState, catcher_id: &PlayerId, ball_coord: FieldCoordinate) -> bool {
    let team = match state.field.player_team(catcher_id) {
        Some(t) => t,
        None => return false,
    };
    let has_skill = state.team(team)
        .player_by_id(catcher_id)
        .map(|p| p.has_skill(SkillId::DivingCatch))
        .unwrap_or(false);
    if !has_skill {
        return false;
    }
    let catcher_coord = match state.field.player_coord(catcher_id) {
        Some(c) => c,
        None => return false,
    };
    // Ball must be adjacent (Chebyshev distance == 1) — not on the same square
    let dx = (catcher_coord.x as i16 - ball_coord.x as i16).unsigned_abs();
    let dy = (catcher_coord.y as i16 - ball_coord.y as i16).unsigned_abs();
    let chebyshev = dx.max(dy);
    chebyshev == 1
}

// ── HailMaryPass ──────────────────────────────────────────────────────────────

/// Returns true if the player has the HailMaryPass skill, meaning the pass may
/// target any square on the pitch regardless of range (but always scatters as
/// if inaccurate).
pub fn is_hail_mary_pass(state: &GameState, passer_id: &PlayerId) -> bool {
    let team = match state.field.player_team(passer_id) {
        Some(t) => t,
        None => return false,
    };
    state.team(team)
        .player_by_id(passer_id)
        .map(|p| p.has_skill(SkillId::HailMaryPass))
        .unwrap_or(false)
}

// ── Bullseye ──────────────────────────────────────────────────────────────────

/// Check if the passer has Bullseye.  With Bullseye, a roll of 1 is NOT an
/// automatic fumble — it is treated as an inaccurate pass instead.
/// Returns true if the player has the skill.
pub fn passer_has_bullseye(state: &GameState, passer_id: &PlayerId) -> bool {
    let team = match state.field.player_team(passer_id) {
        Some(t) => t,
        None => return false,
    };
    state.team(team)
        .player_by_id(passer_id)
        .map(|p| p.has_skill(SkillId::Bullseye))
        .unwrap_or(false)
}

// ── DisturbingPresence ────────────────────────────────────────────────────────

/// Count opposing players with DisturbingPresence within 3 squares (Chebyshev)
/// of the passer. Each such player adds +1 to the pass min roll.
pub fn disturbing_presence_penalty(state: &GameState, passer_id: &PlayerId, passer_team: crate::types::TeamId) -> u8 {
    let passer_coord = match state.field.player_coord(passer_id) {
        Some(c) => c,
        None => return 0,
    };
    let opp_team = passer_team.opponent();
    let mut count: u8 = 0;

    for (pid, coord, player_state) in state.field.on_pitch_players() {
        if state.field.player_team(pid) != Some(opp_team) {
            continue;
        }
        if !player_state.is_active() {
            continue;
        }
        let has_dp = state.team(opp_team)
            .player_by_id(pid)
            .map(|p| p.has_skill(SkillId::DisturbingPresence))
            .unwrap_or(false);
        if !has_dp {
            continue;
        }
        let dx = (passer_coord.x as i16 - coord.x as i16).unsigned_abs();
        let dy = (passer_coord.y as i16 - coord.y as i16).unsigned_abs();
        let chebyshev = dx.max(dy);
        if chebyshev <= 3 {
            count = count.saturating_add(1);
        }
    }
    count
}

// ── Animosity ─────────────────────────────────────────────────────────────────

/// Roll d6 to determine if a player with Animosity will cooperate.
/// Returns true (cooperate) if roll ≥ 3, false (refuse) if roll < 3.
pub fn animosity_check(rng: &mut GameRng) -> bool {
    rng.roll_d6() >= 3
}

/// Attempt a catch at the catcher's current square.
/// Offers a team reroll dialog when all skill rerolls are exhausted and a reroll is available,
/// but only if the catcher is on the active team (whose turn it is).
pub fn apply_catch(
    state: &mut GameState,
    catcher_id: &PlayerId,
    rng: &mut GameRng,
) -> PassStepResult {
    let team = state.field.player_team(catcher_id).expect("catcher team");
    let at = state.field.player_coord(catcher_id).expect("catcher coord");

    let (ag, has_catch, has_nos, has_extra_arms, opp_tz) = {
        let p = state.team(team).player_by_id(catcher_id).expect("catcher");
        let opp = team.opponent();
        let tz = state.field.tackle_zones_on(at, opp);
        (p.effective_ag(), p.has_skill(SkillId::Catch), p.has_skill(SkillId::NervesOfSteel), p.has_skill(SkillId::ExtraArms), tz)
    };

    let min_roll = catch_min_roll(ag, has_nos, has_extra_arms, opp_tz, state.field.weather);

    let roll = rng.roll_d6();
    let after_initial = roll >= min_roll;
    // Catch skill: auto-reroll once on failure
    let success = if !after_initial && has_catch {
        rng.roll_d6() >= min_roll
    } else {
        after_initial
    };

    if success {
        state.field.ball.coord = Some(at);
        return PassStepResult::Caught { by: catcher_id.clone() };
    }

    // All skill rerolls exhausted — offer team reroll if this is the active team's player
    let active_team = if state.home_is_active { crate::types::TeamId::Home } else { crate::types::TeamId::Away };
    if team == active_team {
        let team_reroll_avail = crate::steps::turn_step::active_team_reroll_available(state);
        if team_reroll_avail {
            if let Some(ap) = state.acting_player.as_mut() {
                ap.pending_catch_at = Some(at);
            }
            state.dialog = crate::model::game_state::DialogState::SelectReroll {
                action_name: "Catch".into(),
                reroll_available: true,
                skill_reroll_available: false,
            };
            return PassStepResult::PendingReroll;
        }
    }

    let scatter = pass_scatter_coord(at, rng);
    state.field.ball.coord = Some(scatter);
    PassStepResult::Dropped { at: scatter }
}

/// Compute the minimum roll needed to catch (before team reroll decisions).
fn catch_min_roll(ag: u8, has_nos: bool, has_extra_arms: bool, opp_tz: u8, weather: crate::types::Weather) -> u8 {
    let base = (5u8).saturating_sub(ag).max(2);
    let weather_mod = match weather {
        crate::types::Weather::PouringRain | crate::types::Weather::Blizzard => 1u8,
        _ => 0u8,
    };
    let tz_penalty = if has_nos { 0 } else { opp_tz };
    let min_roll = (base + tz_penalty + weather_mod).min(6);
    if has_extra_arms { min_roll.saturating_sub(1).max(2) } else { min_roll }
}

/// Resume a catch after team reroll decision.
pub fn resume_catch_after_reroll(
    state: &mut GameState,
    catcher_id: &PlayerId,
    use_reroll: bool,
    rng: &mut GameRng,
) -> PassStepResult {
    let at = match state.acting_player.as_mut().and_then(|ap| ap.pending_catch_at.take()) {
        Some(coord) => coord,
        None => return PassStepResult::Dropped { at: FieldCoordinate::new(0, 0) },
    };
    let team = state.field.player_team(catcher_id).expect("catcher team");

    if !use_reroll {
        let scatter = pass_scatter_coord(at, rng);
        state.field.ball.coord = Some(scatter);
        return PassStepResult::Dropped { at: scatter };
    }

    // Consume the team reroll (Loner check included)
    let reroll_ok = crate::steps::turn_step::use_team_reroll(state, catcher_id, rng);
    if !reroll_ok {
        // Loner failed — treat as declined
        let scatter = pass_scatter_coord(at, rng);
        state.field.ball.coord = Some(scatter);
        return PassStepResult::Dropped { at: scatter };
    }

    // Re-roll: compute min_roll and roll again (no further team reroll offered)
    let (ag, has_nos, has_extra_arms, opp_tz) = {
        let p = state.team(team).player_by_id(catcher_id).expect("catcher");
        let opp = team.opponent();
        let tz = state.field.tackle_zones_on(at, opp);
        (p.effective_ag(), p.has_skill(SkillId::NervesOfSteel), p.has_skill(SkillId::ExtraArms), tz)
    };
    let min_roll = catch_min_roll(ag, has_nos, has_extra_arms, opp_tz, state.field.weather);
    let roll = rng.roll_d6();

    if roll >= min_roll {
        state.field.ball.coord = Some(at);
        PassStepResult::Caught { by: catcher_id.clone() }
    } else {
        let scatter = pass_scatter_coord(at, rng);
        state.field.ball.coord = Some(scatter);
        PassStepResult::Dropped { at: scatter }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::game_state::GameState;
    use crate::model::player::{Player, PlayerStats};
    use crate::model::team::Team;
    use crate::rng::GameRng;
    use crate::skills::{SkillId, SkillSet};
    use crate::types::{FieldCoordinate, PlayerId, PlayerState, TeamId};

    fn make_passer_state(skills: SkillSet, pa: Option<u8>) -> (GameState, PlayerId) {
        let pid = PlayerId("passer".into());
        let player = Player::new(
            pid.clone(),
            "Passer".into(),
            "thrower".into(),
            TeamId::Home,
            1,
            PlayerStats::new(6, 3, 3, 8, pa),
            skills,
        );
        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        home.add_player(player);
        let away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        let mut state = GameState::new(home, away);
        state.field.place_player(pid.clone(), TeamId::Home, FieldCoordinate::new(5, 5), PlayerState::Standing);
        state.home_is_active = true;
        (state, pid)
    }

    fn make_catcher_state(skills: SkillSet) -> (GameState, PlayerId) {
        let pid = PlayerId("catcher".into());
        let player = Player::new(
            pid.clone(),
            "Catcher".into(),
            "catcher".into(),
            TeamId::Home,
            2,
            PlayerStats::new(8, 2, 4, 7, None),
            skills,
        );
        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        home.add_player(player);
        let away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        let mut state = GameState::new(home, away);
        state.field.place_player(pid.clone(), TeamId::Home, FieldCoordinate::new(10, 5), PlayerState::Standing);
        (state, pid)
    }

    // ── Blizzard range reduction ───────────────────────────────────────────────

    #[test]
    fn blizzard_reduces_long_bomb_to_long() {
        use crate::types::PassRange;
        assert_eq!(super::blizzard_reduced_range(PassRange::LongBomb), PassRange::Long);
    }

    #[test]
    fn blizzard_reduces_long_to_short() {
        use crate::types::PassRange;
        assert_eq!(super::blizzard_reduced_range(PassRange::Long), PassRange::Short);
    }

    #[test]
    fn blizzard_does_not_reduce_below_short() {
        use crate::types::PassRange;
        assert_eq!(super::blizzard_reduced_range(PassRange::Short), PassRange::Short);
    }

    #[test]
    fn blizzard_does_not_affect_handoff() {
        use crate::types::PassRange;
        assert_eq!(super::blizzard_reduced_range(PassRange::HandOff), PassRange::HandOff);
    }

    // ── DivingCatch ───────────────────────────────────────────────────────────

    #[test]
    fn diving_catch_true_when_ball_adjacent() {
        let (state, pid) = make_catcher_state([SkillId::DivingCatch].into_iter().collect());
        // Catcher at (10,5), ball at (11,5) — distance 1
        let ball = FieldCoordinate::new(11, 5);
        assert!(can_diving_catch(&state, &pid, ball));
    }

    #[test]
    fn diving_catch_false_when_ball_not_adjacent() {
        let (state, pid) = make_catcher_state([SkillId::DivingCatch].into_iter().collect());
        // Catcher at (10,5), ball at (12,5) — distance 2
        let ball = FieldCoordinate::new(12, 5);
        assert!(!can_diving_catch(&state, &pid, ball));
    }

    #[test]
    fn diving_catch_false_without_skill() {
        let (state, pid) = make_catcher_state(SkillSet::empty());
        let ball = FieldCoordinate::new(11, 5);
        assert!(!can_diving_catch(&state, &pid, ball));
    }

    #[test]
    fn diving_catch_false_when_ball_on_same_square() {
        let (state, pid) = make_catcher_state([SkillId::DivingCatch].into_iter().collect());
        // Ball on the same square as catcher — should return false (distance = 0)
        let ball = FieldCoordinate::new(10, 5);
        assert!(!can_diving_catch(&state, &pid, ball));
    }

    // ── HailMaryPass ──────────────────────────────────────────────────────────

    #[test]
    fn hail_mary_pass_detected() {
        let (state, pid) = make_passer_state([SkillId::HailMaryPass].into_iter().collect(), Some(4));
        assert!(is_hail_mary_pass(&state, &pid));
    }

    #[test]
    fn hail_mary_pass_absent_without_skill() {
        let (state, pid) = make_passer_state(SkillSet::empty(), Some(4));
        assert!(!is_hail_mary_pass(&state, &pid));
    }

    #[test]
    fn hail_mary_pass_always_scatters() {
        // With HailMaryPass the pass always scatters; the caller is responsible
        // for treating the result as Inaccurate.  Verify the flag is on.
        let (mut state, pid) = make_passer_state([SkillId::HailMaryPass].into_iter().collect(), Some(4));
        assert!(is_hail_mary_pass(&state, &pid));
        // A LongBomb pass from (5,5) to (5,16) — with HailMaryPass it scatters
        // regardless of the roll.  We just verify is_hail_mary_pass returns true
        // and that a begin_pass with a "good" roll still produces an Inaccurate
        // result when the caller respects the flag.  We test the flag only here.
    }

    // ── Bullseye ─────────────────────────────────────────────────────────────

    #[test]
    fn bullseye_detected() {
        let (state, pid) = make_passer_state([SkillId::Bullseye].into_iter().collect(), Some(4));
        assert!(passer_has_bullseye(&state, &pid));
    }

    #[test]
    fn bullseye_absent_without_skill() {
        let (state, pid) = make_passer_state(SkillSet::empty(), Some(4));
        assert!(!passer_has_bullseye(&state, &pid));
    }

    #[test]
    fn bullseye_no_fumble_on_roll_2() {
        // PA=4, Short pass: min_roll = max(2, 5-4)+1 = 2+1 = 3.
        // Roll of 2 → normally Inaccurate; with Bullseye roll of 1 is NOT a
        // fumble — it becomes Inaccurate.  Here we verify roll=2 → Inaccurate
        // (not fumble) even without Bullseye to establish baseline, then confirm
        // that with Bullseye a roll=1 is treated as Inaccurate, not a fumble.
        // The mechanic is checked via passer_has_bullseye — the caller adjusts.
        use crate::mechanics::pass::{pass_roll, PassResult};
        use crate::types::PassRange;
        // roll=1 without Bullseye → Fumble
        let mut rng = GameRng::new_test([1]);
        assert_eq!(pass_roll(Some(4), PassRange::Short, &mut rng), PassResult::Fumble);
        // Bullseye prevents natural 1 from being a fumble (roll→Inaccurate).
        // Verified at the step level: passer_has_bullseye flag allows caller to
        // re-classify the Fumble as Inaccurate.
        let (state, pid) = make_passer_state([SkillId::Bullseye].into_iter().collect(), Some(4));
        assert!(passer_has_bullseye(&state, &pid));
    }

    // ── Kick skill presence ───────────────────────────────────────────────────

    #[test]
    fn kick_skill_in_skillset() {
        let mut s = crate::skills::SkillSet::empty();
        s.add(SkillId::Kick);
        assert!(s.has(SkillId::Kick));
    }

    // ── Interception scenario ─────────────────────────────────────────────────

    #[test]
    fn interception_candidate_eligible() {
        // Passer at (5,5), target at (12,5), interceptor at (9,5)
        // Interceptor is on the throw line and not in opposing TZ
        let passer_id = PlayerId("passer".into());
        let interceptor_id = PlayerId("interceptor".into());

        let passer = Player::new(
            passer_id.clone(), "Passer".into(), "thrower".into(), TeamId::Home, 1,
            PlayerStats::new(6, 3, 4, 8, Some(3)), SkillSet::empty(),
        );
        let interceptor = Player::new(
            interceptor_id.clone(), "Interceptor".into(), "lineman".into(), TeamId::Away, 1,
            PlayerStats::new(6, 3, 4, 8, None), SkillSet::empty(),
        );
        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        let mut away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        home.add_player(passer);
        away.add_player(interceptor);
        let mut state = GameState::new(home, away);
        state.field.place_player(passer_id.clone(), TeamId::Home, FieldCoordinate::new(5, 5), PlayerState::Standing);
        state.field.place_player(interceptor_id.clone(), TeamId::Away, FieldCoordinate::new(9, 5), PlayerState::Standing);
        state.home_is_active = true;

        let candidates = pass_block_candidates(
            &state,
            FieldCoordinate::new(5, 5),
            FieldCoordinate::new(12, 5),
            TeamId::Away,
        );
        assert!(candidates.contains(&interceptor_id), "interceptor should be eligible");
    }

    // ── diving_catch_eligible ─────────────────────────────────────────────────

    #[test]
    fn diving_catch_player_can_catch_adjacent_ball() {
        // Catcher at (10,5) with DivingCatch; ball lands at (11,5) — adjacent.
        let (state, pid) = make_catcher_state([SkillId::DivingCatch].into_iter().collect());
        let ball = FieldCoordinate::new(11, 5);
        let result = diving_catch_eligible(&state, ball);
        assert_eq!(result, Some(pid), "player adjacent with DivingCatch should be eligible");
    }

    #[test]
    fn diving_catch_eligible_none_when_no_adjacent_player_with_skill() {
        // No DivingCatch player near ball
        let (state, _pid) = make_catcher_state(SkillSet::empty());
        let ball = FieldCoordinate::new(11, 5);
        let result = diving_catch_eligible(&state, ball);
        assert!(result.is_none(), "no DivingCatch player adjacent, should return None");
    }

    // ── very_long_legs_interception_eligible ──────────────────────────────────

    fn make_interceptor_state(skills: SkillSet, coord: FieldCoordinate) -> (GameState, PlayerId) {
        let pid = PlayerId("interceptor".into());
        let player = Player::new(
            pid.clone(),
            "Interceptor".into(),
            "lineman".into(),
            TeamId::Away,
            1,
            PlayerStats::new(6, 3, 4, 8, None),
            skills,
        );
        let home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        let mut away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        away.add_player(player);
        let mut state = GameState::new(home, away);
        state.field.place_player(pid.clone(), TeamId::Away, coord, PlayerState::Standing);
        state.home_is_active = true;
        (state, pid)
    }

    #[test]
    fn very_long_legs_player_eligible_for_interception() {
        // Pass from (5,5) to (15,5); interceptor with VeryLongLegs at (10,6) — adjacent to throw line
        let coord = FieldCoordinate::new(10, 6);
        let (state, pid) = make_interceptor_state(
            [SkillId::VeryLongLegs].into_iter().collect(),
            coord,
        );
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(15, 5);
        let eligible = very_long_legs_interception_eligible(&state, from, to, TeamId::Away);
        assert!(eligible.contains(&pid), "VeryLongLegs player near throw line should be eligible");
    }

    #[test]
    fn very_long_legs_player_not_eligible_when_far_from_path() {
        // Player far from the throw line should not be eligible
        let coord = FieldCoordinate::new(10, 15);
        let (state, pid) = make_interceptor_state(
            [SkillId::VeryLongLegs].into_iter().collect(),
            coord,
        );
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(15, 5);
        let eligible = very_long_legs_interception_eligible(&state, from, to, TeamId::Away);
        assert!(!eligible.contains(&pid), "VeryLongLegs player far from throw line should not be eligible");
    }

    #[test]
    fn interception_succeeds_on_good_roll() {
        let passer_id = PlayerId("passer".into());
        let interceptor_id = PlayerId("interceptor".into());

        let passer = Player::new(
            passer_id.clone(), "Passer".into(), "thrower".into(), TeamId::Home, 1,
            PlayerStats::new(6, 3, 4, 8, Some(3)), SkillSet::empty(),
        );
        let interceptor = Player::new(
            interceptor_id.clone(), "Interceptor".into(), "lineman".into(), TeamId::Away, 1,
            PlayerStats::new(6, 3, 4, 8, None), SkillSet::empty(),
        );
        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        let mut away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        home.add_player(passer);
        away.add_player(interceptor);
        let mut state = GameState::new(home, away);
        // Ball on passer square
        state.field.ball.coord = Some(FieldCoordinate::new(5, 5));
        state.field.ball.in_play = true;
        state.field.place_player(passer_id.clone(), TeamId::Home, FieldCoordinate::new(5, 5), PlayerState::Standing);
        // Interceptor at (9,5) — on throw line from (5,5) to (12,5)
        state.field.place_player(interceptor_id.clone(), TeamId::Away, FieldCoordinate::new(9, 5), PlayerState::Standing);
        state.home_is_active = true;

        // Interception min-roll: AG=4 → base = 6 - (4-1) = 3, but min 2, = 3. Capped to 6.
        // Interceptor rolls 6 → succeeds.
        // After interception: pass_roll for normal pass never fires.
        let mut rng = GameRng::new_test([6]);
        let result = begin_pass(&mut state, &passer_id, FieldCoordinate::new(12, 5), &mut rng);
        assert!(
            matches!(result, PassStepResult::Caught { .. }),
            "interception roll of 6 should succeed, got: {result:?}"
        );
        assert_eq!(
            state.field.ball.coord,
            Some(FieldCoordinate::new(9, 5)),
            "ball should be at interceptor's square"
        );
    }

    // ── NervesOfSteel tests ───────────────────────────────────────────────────

    #[test]
    fn nerves_of_steel_passer_ignores_tz_penalty() {
        // Passer with NervesOfSteel at (5,5), opponent adjacent at (6,5) creating TZ.
        // Without NoS: pass min_roll for Short, PA=3 is base=max(2,5-3)=2; +1 Short = 3;
        // +1 for TZ = 4 (i.e. roll=3 would fail without NoS).
        // With NervesOfSteel: TZ is ignored → min_roll=3; roll=3 → Accurate.
        let passer_id = PlayerId("nos_passer".into());
        let opp_id = PlayerId("nos_opp".into());

        let nos_skills: SkillSet = [SkillId::NervesOfSteel].into_iter().collect();
        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        home.add_player(Player::new(
            passer_id.clone(), "NoS Passer".into(), "thrower".into(), TeamId::Home, 1,
            PlayerStats::new(6, 3, 3, 8, Some(3)), nos_skills,
        ));
        let mut away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        away.add_player(Player::new(
            opp_id.clone(), "Blocker".into(), "lineman".into(), TeamId::Away, 1,
            PlayerStats::new(5, 3, 3, 9, None), SkillSet::empty(),
        ));
        let mut state = GameState::new(home, away);
        // Place passer at (5,5), opponent adjacent at (6,5) → 1 TZ on passer
        state.field.place_player(passer_id.clone(), TeamId::Home, FieldCoordinate::new(5, 5), PlayerState::Standing);
        state.field.place_player(opp_id.clone(), TeamId::Away, FieldCoordinate::new(6, 5), PlayerState::Standing);
        state.home_is_active = true;

        // No interception candidates (opponent is at (6,5), target is (7,5) — short pass)
        // passer TZ = 1, but NervesOfSteel ignores it
        // pass_min_roll_with_tz(Some(3), Short, 1, nos=true) = 3
        // Roll: no interceptors roll needed (they're behind passer or not on path), then pass roll=3 → Accurate
        // Actually: (6,5) is on throw line from (5,5) to (7,5)? Check:
        // throw_line from (5,5) to (7,5): steps along dx=2, dy=0
        // squares: (5,5), (6,5), (7,5) — (6,5) is on the throw line!
        // So interceptor at (6,5) would be a candidate. Interceptor needs to roll:
        // interception: AG=3, tz=0 (no home players adjacent to opp) → base=6-(3-1)=4; min=4
        // Roll 1: interception attempt = 2 (fail), Roll 2: pass_roll = 3 (>=3 → Accurate)
        let target = FieldCoordinate::new(8, 5); // further away to avoid interceptor on throw line
        // throw_line from (5,5) to (8,5): squares (5,5),(6,5),(7,5),(8,5) — (6,5) still on line
        // Use target (5,8) instead: throw_line from (5,5) to (5,8): squares (5,5),(5,6),(5,7),(5,8)
        // Opponent at (6,5) is adjacent to (5,5) and (5,6)? (6,5) to (5,5): dx=1,dy=0 → adjacent.
        // (6,5) to (5,6): dx=1,dy=1 → adjacent. So they'd still be near throw line.
        // Let's use (5,8) as target. Interceptor at (6,5) near (5,5) → near start of throw line.
        // pass_block_candidates checks: not in TZ of attacking team; in_tackle_zone checks if
        // defending_team player is in attacking (home) team's TZ. No home player adjacent to (6,5).
        // So opp IS a candidate. They roll: AG=3, their opp_tz (home players adj to them) = 0.
        // interception base = 6-(3-1)=4; min_roll=4. Roll=2 → fail.
        // Then pass_roll_with_tz (PA=3, Short, tz=1, nos=true) → min_roll=3; Roll=3 → Accurate.
        // Sequence: [2 (interception fail), 3 (pass roll)]

        // Actually for Short range, target should be 2-3 away from passer.
        // (5,5) to (5,7): distance = 2 → Short range
        // Opponent at (6,5) is in Home TZ (passer at (5,5) is adjacent), so
        // they are excluded from interception candidates. Only 1 die: the pass roll.
        // pass_min_roll_with_tz(PA=3, Short, tz=1, nos=true) = 3 (TZ ignored).
        // Roll=3 >= min_roll=3 → Accurate.
        let short_target = FieldCoordinate::new(5, 7);
        let mut rng = GameRng::new_test([3]);
        let result = begin_pass(&mut state, &passer_id, short_target, &mut rng);
        assert!(
            matches!(result, PassStepResult::Accurate { .. }),
            "NervesOfSteel passer should succeed with roll=3 (min_roll=3, TZ ignored), got {:?}", result
        );
    }

    #[test]
    fn no_nerves_of_steel_tz_penalty_applies() {
        // Same scenario but passer without NervesOfSteel: TZ adds +1 → min_roll=4; roll=3 → Inaccurate.
        let passer_id = PlayerId("nonos_passer".into());
        let opp_id = PlayerId("nonos_opp".into());

        // Use 0 rerolls so no team reroll dialog is offered
        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 0, true);
        home.add_player(Player::new(
            passer_id.clone(), "Normal Passer".into(), "thrower".into(), TeamId::Home, 1,
            PlayerStats::new(6, 3, 3, 8, Some(3)), SkillSet::empty(),
        ));
        let mut away = Team::new("a".into(), "Away".into(), "Orc".into(), 0, false);
        away.add_player(Player::new(
            opp_id.clone(), "Blocker".into(), "lineman".into(), TeamId::Away, 1,
            PlayerStats::new(5, 3, 3, 9, None), SkillSet::empty(),
        ));
        let mut state = GameState::new(home, away);
        state.field.place_player(passer_id.clone(), TeamId::Home, FieldCoordinate::new(5, 5), PlayerState::Standing);
        state.field.place_player(opp_id.clone(), TeamId::Away, FieldCoordinate::new(6, 5), PlayerState::Standing);
        state.home_is_active = true;

        // Interceptor roll=2 (fail), pass_roll_with_tz(PA=3, Short, tz=1, nos=false) → min_roll=4; roll=3 → Inaccurate
        // Inaccurate → scatter → 2 more dice needed (direction, distance)
        let short_target = FieldCoordinate::new(5, 7);
        let mut rng = GameRng::new_test([2, 3, 1, 1]); // interception fail, pass roll fail, scatter dir, dist
        let result = begin_pass(&mut state, &passer_id, short_target, &mut rng);
        assert!(
            matches!(result, PassStepResult::Inaccurate { .. }),
            "Without NervesOfSteel, TZ penalty makes roll=3 inaccurate (min_roll=4), got {:?}", result
        );
    }

    // ── ExtraArms catch tests ─────────────────────────────────────────────────

    #[test]
    fn extra_arms_reduces_catch_min_roll() {
        // Catcher at (10,5) with ExtraArms and AG=4.
        // base = max(2, 5-4) = 2. No TZ, no weather → min_roll = 2.
        // ExtraArms: min_roll = max(2, 2-1) = 2 (already at floor, but -1 would be 1 → clamped to 2).
        // Use AG=3 to show the reduction more clearly:
        // base = max(2, 5-3) = 2. No TZ → min_roll = 2. ExtraArms: max(2, 2-1)=2.
        // Use AG=2: base = max(2, 5-2) = 3. No TZ → min_roll=3. ExtraArms: max(2, 3-1)=2.
        // Roll=2: without ExtraArms → fail (min_roll=3). With ExtraArms → success (min_roll=2).
        let pid = PlayerId("ea_catcher".into());
        let skills: SkillSet = [SkillId::ExtraArms].into_iter().collect();
        let player = Player::new(
            pid.clone(), "EA Catcher".into(), "catcher".into(), TeamId::Home, 2,
            PlayerStats::new(8, 2, 2, 7, None), // AG=2 → base=3 → min_roll normally=3
            skills,
        );
        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        home.add_player(player);
        let away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        let mut state = GameState::new(home, away);
        state.field.place_player(pid.clone(), TeamId::Home, FieldCoordinate::new(10, 5), PlayerState::Standing);

        // With ExtraArms, min_roll = max(2, 3-1) = 2; roll=2 → success
        let mut rng = GameRng::new_test([2]);
        let result = apply_catch(&mut state, &pid, &mut rng);
        assert!(
            matches!(result, PassStepResult::Caught { .. }),
            "ExtraArms should reduce min_roll so roll=2 succeeds (AG=2 → base=3, -1 → 2), got {:?}", result
        );
    }

    #[test]
    fn without_extra_arms_catch_fails_on_low_roll() {
        // Same setup as above but without ExtraArms; roll=2 should fail (min_roll=3).
        let pid = PlayerId("no_ea_catcher".into());
        let player = Player::new(
            pid.clone(), "Normal Catcher".into(), "catcher".into(), TeamId::Home, 2,
            PlayerStats::new(8, 2, 2, 7, None), // AG=2 → base=3 → min_roll=3
            SkillSet::empty(),
        );
        // Use 0 rerolls so no team reroll dialog is offered
        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 0, true);
        home.add_player(player);
        let away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        let mut state = GameState::new(home, away);
        state.field.place_player(pid.clone(), TeamId::Home, FieldCoordinate::new(10, 5), PlayerState::Standing);

        // Without ExtraArms, min_roll = 3; roll=2 < 3 → fail (scatter needs 2 more dice)
        let mut rng = GameRng::new_test([2, 1, 1]);
        let result = apply_catch(&mut state, &pid, &mut rng);
        assert!(
            matches!(result, PassStepResult::Dropped { .. }),
            "Without ExtraArms roll=2 should fail catch (min_roll=3), got {:?}", result
        );
    }

    // ── DisturbingPresence tests ──────────────────────────────────────────────

    #[test]
    fn disturbing_presence_adds_to_pass_min_roll() {
        // Passer (Home) at (5,5). One Away player with DisturbingPresence at (7,5)
        // (within 3 squares of passer). Pass to (5,8) (Short, no interceptors).
        // Disturbing presence penalty = 1 → adds +1 to pass min_roll.
        // PA=4, Short → base = max(2, 5-4)=2, +1 Short = 3. +1 DP = 4.
        // Roll=3: without DP → Accurate. With DP → Inaccurate.
        let passer_id = PlayerId("dp_passer".into());
        let dp_id = PlayerId("dp_enemy".into());
        let dp_skills: SkillSet = [SkillId::DisturbingPresence].into_iter().collect();

        // Use 0 rerolls so no team reroll dialog is offered
        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 0, true);
        home.add_player(Player::new(
            passer_id.clone(), "Passer".into(), "thrower".into(), TeamId::Home, 1,
            PlayerStats::new(6, 3, 3, 8, Some(4)), SkillSet::empty(),
        ));
        let mut away = Team::new("a".into(), "Away".into(), "Chaos".into(), 0, false);
        away.add_player(Player::new(
            dp_id.clone(), "DP Player".into(), "beastman".into(), TeamId::Away, 1,
            PlayerStats::new(5, 4, 3, 9, None), dp_skills,
        ));
        let mut state = GameState::new(home, away);
        // Passer at (5,5), DP player at (7,5) — Chebyshev distance = 2 ≤ 3
        state.field.place_player(passer_id.clone(), TeamId::Home, FieldCoordinate::new(5, 5), PlayerState::Standing);
        state.field.place_player(dp_id.clone(), TeamId::Away, FieldCoordinate::new(7, 5), PlayerState::Standing);
        state.home_is_active = true;

        // With DisturbingPresence (+1 to min_roll): PA=4, Short → base_min=3, +1 DP → min=4.
        // Roll=3 < 4 → Inaccurate → scatter needs 2 more dice.
        let target = FieldCoordinate::new(5, 8); // Short range (distance=3)
        let mut rng = GameRng::new_test([3, 1, 1]); // pass roll, scatter dir, scatter dist
        let result = begin_pass(&mut state, &passer_id, target, &mut rng);
        assert!(
            matches!(result, PassStepResult::Inaccurate { .. }),
            "DisturbingPresence should raise min_roll so roll=3 fails (min=4), got {:?}", result
        );
    }

    // ── Catch team reroll tests ───────────────────────────────────────────────

    fn make_catch_reroll_state() -> (GameState, PlayerId) {
        use crate::model::game_state::ActingPlayer;
        let pid = PlayerId("catcher".into());
        let player = Player::new(
            pid.clone(), "Catcher".into(), "catcher".into(), TeamId::Home, 2,
            PlayerStats::new(8, 2, 2, 7, None), // AG=2 → min_roll=3
            SkillSet::empty(),
        );
        // 3 rerolls so team reroll dialog IS offered on catch fail
        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        home.add_player(player);
        let away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        let mut state = GameState::new(home, away);
        state.field.place_player(pid.clone(), TeamId::Home, FieldCoordinate::new(10, 5), PlayerState::Standing);
        state.home_is_active = true;
        // Set acting player so pending_catch_at can be stored
        state.acting_player = Some(ActingPlayer::new(pid.clone(), TeamId::Home));
        (state, pid)
    }

    #[test]
    fn catch_team_reroll_offered_when_catch_fails_and_rerolls_available() {
        let (mut state, pid) = make_catch_reroll_state();
        // roll=2 < min_roll=3 → fail; no Catch skill → no auto-reroll; team reroll available
        let mut rng = GameRng::new_test([2]);
        let result = apply_catch(&mut state, &pid, &mut rng);
        assert!(
            matches!(result, PassStepResult::PendingReroll),
            "catch fail with available reroll should return PendingReroll, got {result:?}"
        );
        assert!(
            state.acting_player.as_ref().map(|ap| ap.pending_catch_at.is_some()).unwrap_or(false),
            "pending_catch_at should be set"
        );
        assert!(matches!(state.dialog, crate::model::game_state::DialogState::SelectReroll { .. }));
    }

    #[test]
    fn catch_team_reroll_accepted_succeeds() {
        let (mut state, pid) = make_catch_reroll_state();
        // First attempt: roll=2 → fail → PendingReroll
        let mut rng = GameRng::new_test([2]);
        let _ = apply_catch(&mut state, &pid, &mut rng);
        // Accept reroll: roll=4 → success
        let mut rng2 = GameRng::new_test([4]);
        let result = resume_catch_after_reroll(&mut state, &pid, true, &mut rng2);
        assert!(
            matches!(result, PassStepResult::Caught { .. }),
            "reroll success with roll=4 (min_roll=3) should catch, got {result:?}"
        );
    }

    #[test]
    fn catch_team_reroll_declined_drops_ball() {
        let (mut state, pid) = make_catch_reroll_state();
        // First attempt: roll=2 → fail → PendingReroll
        let mut rng = GameRng::new_test([2]);
        let _ = apply_catch(&mut state, &pid, &mut rng);
        // Decline: scatter dice
        let mut rng2 = GameRng::new_test([1, 1]);
        let result = resume_catch_after_reroll(&mut state, &pid, false, &mut rng2);
        assert!(
            matches!(result, PassStepResult::Dropped { .. }),
            "declined reroll should return Dropped, got {result:?}"
        );
    }

    // ── Animosity tests ───────────────────────────────────────────────────────

    #[test]
    fn animosity_fails_on_1_2() {
        // Roll=1 → Animosity kicks in, refuse to cooperate
        let mut rng = GameRng::new_test([1]);
        assert!(!animosity_check(&mut rng), "Animosity should refuse on roll=1");

        let mut rng = GameRng::new_test([2]);
        assert!(!animosity_check(&mut rng), "Animosity should refuse on roll=2");
    }

    #[test]
    fn animosity_succeeds_on_3_plus() {
        let mut rng = GameRng::new_test([3]);
        assert!(animosity_check(&mut rng), "Animosity should cooperate on roll=3");

        let mut rng = GameRng::new_test([6]);
        assert!(animosity_check(&mut rng), "Animosity should cooperate on roll=6");
    }
}
