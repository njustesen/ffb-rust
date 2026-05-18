/// Turn lifecycle: activation, end turn, half-time, game end.
use crate::mechanics::injury::apply_apothecary;
use crate::model::game_state::{ActingPlayer, DialogState, GameState};
use crate::rng::GameRng;
use crate::skills::SkillId;
use crate::types::{CasualtyType, Half, PlayerId, TeamId, TurnMode};

#[derive(Clone, Debug, PartialEq)]
pub enum TurnStepResult {
    Ok,
    TurnOver,
    HalfEnd,
    GameEnd,
}

/// Begin the active team's turn: increment turn counter, reset turn flags.
pub fn begin_turn(state: &mut GameState) -> TurnStepResult {
    let td = state.active_turn_data_mut();
    td.reset_for_new_turn();

    // Check if this was the last turn of the half
    if state.active_turn_data().turn_number > state.options.max_turns_per_half {
        return handle_half_end(state);
    }

    state.turn_mode = TurnMode::Regular;
    state.dialog = DialogState::None;
    // Clear per-turn tracking
    state.players_activated_this_turn.clear();
    state.hypnotized_this_turn.clear();

    // BB2025 rule: at the start of each team's turn, all Stunned players on the
    // active team transition from Stunned → Prone (they recover from stun).
    let active_team = state.active_team_id();
    let stunned_players: Vec<crate::types::PlayerId> = state
        .field
        .team_players_on_pitch(active_team)
        .filter(|(_, _, ps)| *ps == crate::types::PlayerState::Stunned)
        .map(|(pid, _, _)| pid.clone())
        .collect();
    for pid in stunned_players {
        state.field.set_player_state(&pid, crate::types::PlayerState::Prone);
    }

    TurnStepResult::Ok
}

/// End the active team's turn; switch to opponent.
pub fn end_turn(state: &mut GameState) -> TurnStepResult {
    state.acting_player = None;
    state.dialog = DialogState::None;

    let was_home = state.home_is_active;
    state.home_is_active = !state.home_is_active;
    let _ = was_home; // used for clarity only

    // Half ends only when BOTH teams have used all their turns.
    // Checking after every end_turn prevents the old bug where the half ended
    // after away's turn 8 before home could play their turn 8.
    let both_done = state.turn_data_home.turn_number >= state.options.max_turns_per_half
        && state.turn_data_away.turn_number >= state.options.max_turns_per_half;
    if both_done {
        return handle_half_end(state);
    }

    TurnStepResult::Ok
}

#[allow(dead_code)]
fn inactive_turn_data_number(state: &GameState) -> u8 {
    if state.home_is_active { state.turn_data_away.turn_number } else { state.turn_data_home.turn_number }
}

impl GameState {
    fn inactive_turn_data(&self) -> &crate::model::team::TurnData {
        if self.home_is_active { &self.turn_data_away } else { &self.turn_data_home }
    }
}

fn handle_half_end(state: &mut GameState) -> TurnStepResult {
    // Secret Weapon ejection: remove players with SecretWeapon from pitch → Injured
    eject_secret_weapons(state);

    if state.half == Half::Second {
        state.result.finished = true;
        let h = state.result.score_home;
        let a = state.result.score_away;
        state.result.winner = if h > a { Some(TeamId::Home) }
            else if a > h { Some(TeamId::Away) }
            else { None };
        return TurnStepResult::GameEnd;
    }
    // Move to second half.
    // Reset turn_number to 0 for both teams so begin_turn() increments to 1.
    // Do NOT call reset_for_new_turn() here — that would double-increment.
    state.half = Half::Second;
    state.turn_data_home.turn_number = 0;
    state.turn_data_home.reset_flags();
    state.turn_data_away.turn_number = 0;
    state.turn_data_away.reset_flags();
    // swap offense/defense
    state.home_is_offense = !state.home_is_offense;
    state.home_is_active = !state.home_is_offense; // offense kicks off
    TurnStepResult::HalfEnd
}

/// Eject all players on pitch with SecretWeapon — move them to Injured state.
/// Called at end of each drive (touchdown, half, game end).
pub fn eject_secret_weapons(state: &mut GameState) {
    use crate::skills::SkillId;
    use crate::types::PlayerState;
    let mut to_eject: Vec<(crate::types::PlayerId, crate::types::TeamId)> = Vec::new();
    for team_id in [crate::types::TeamId::Home, crate::types::TeamId::Away] {
        for player in state.team(team_id).players() {
            if player.has_skill(SkillId::SecretWeapon) {
                if let Some(ps) = state.field.player_state(&player.id) {
                    if ps.is_on_pitch() {
                        to_eject.push((player.id.clone(), team_id));
                    }
                }
            }
        }
    }
    for (pid, _team) in to_eject {
        state.field.set_player_state(&pid, PlayerState::Injured);
    }
}

/// Begin a player's activation: validate eligibility, roll negastats.
pub fn begin_activation(
    state: &mut GameState,
    player_id: &PlayerId,
    rng: &mut GameRng,
) -> TurnStepResult {
    let team_id = state.active_team_id();
    let (has_bonehead, has_really_stupid, has_wild_animal) = {
        let p = state.active_team().player_by_id(player_id).expect("player");
        (
            p.has_skill(SkillId::BoneHead),
            p.has_skill(SkillId::ReallyStupid),
            p.has_skill(SkillId::WildAnimal),
        )
    };

    // BoneHead: roll d6; needs 2+ (BB2025 — fail only on 1)
    if has_bonehead {
        let roll = rng.roll_d6();
        if roll < 2 {
            return TurnStepResult::TurnOver;
        }
    }

    // Really Stupid: needs 2+ with adjacent teammate, 4+ without
    if has_really_stupid {
        let coord = state.field.player_coord(player_id).expect("coord");
        let has_teammate_adjacent = coord.neighbors().any(|n| {
            state.field.player_at(n)
                .and_then(|pid| state.field.player_team(pid))
                .map(|t| t == team_id)
                .unwrap_or(false)
        });
        let min_roll = if has_teammate_adjacent { 2u8 } else { 4u8 };
        let roll = rng.roll_d6();
        if roll < min_roll {
            return TurnStepResult::TurnOver;
        }
    }

    // Wild Animal: needs 2+ for Block/Blitz action, 4+ otherwise
    // Restriction applies only if fails: on Block/Blitz 2+ is fine (they can act freely on success)
    // We don't know the specific action yet, so assume Block/Blitz intent → use 4+ for non-combat actions
    // (The action type is validated after activation; use 4+ threshold since Move/Pass are restricted)
    let wild_animal_restricted = if has_wild_animal {
        let roll = rng.roll_d6();
        // roll < 4 means can only Block/Blitz
        roll < 4
    } else {
        false
    };

    // Bloodlust (Vampire mechanic): roll d6; on 2+ feeds successfully, on 1 must bite a teammate.
    let has_bloodlust = state.active_team()
        .player_by_id(player_id)
        .map(|p| p.has_skill(SkillId::Bloodlust))
        .unwrap_or(false);
    if has_bloodlust {
        let roll = rng.roll_d6();
        if roll == 1 {
            // Vampire must bite an adjacent teammate
            let vamp_coord = state.field.player_coord(player_id).expect("vampire coord");
            let adjacent_teammate: Option<crate::types::PlayerId> = vamp_coord
                .neighbors()
                .filter_map(|n| state.field.player_at(n).cloned())
                .find(|pid| state.field.player_team(pid) == Some(team_id));
            if let Some(victim_id) = adjacent_teammate {
                state.field.set_player_state(&victim_id, crate::types::PlayerState::Injured);
            }
            return TurnStepResult::TurnOver;
        }
    }

    // TakeRoot: roll d6; on 1 player is rooted — set movement_used to full MA so no movement is left.
    let has_take_root = state.active_team()
        .player_by_id(player_id)
        .map(|p| p.has_skill(SkillId::TakeRoot))
        .unwrap_or(false);
    let take_root_activated = if has_take_root {
        let roll = rng.roll_d6();
        roll == 1
    } else {
        false
    };

    // JumpUp: prone player with JumpUp stands up for free
    if state.field.player_state(player_id) == Some(crate::types::PlayerState::Prone) {
        let has_jump_up = state.active_team()
            .player_by_id(player_id)
            .map(|p| p.has_skill(SkillId::JumpUp))
            .unwrap_or(false);
        if has_jump_up {
            state.field.set_player_state(player_id, crate::types::PlayerState::Standing);
        }
    }

    let mut ap = ActingPlayer::new(player_id.clone(), team_id);
    ap.wild_animal_restricted = wild_animal_restricted;
    // TakeRoot: if rooted, consume all movement by setting movement_used to full MA.
    if take_root_activated {
        let full_ma = state.active_team()
            .player_by_id(player_id)
            .map(|p| p.effective_ma())
            .unwrap_or(0);
        ap.movement_used = full_ma;
    }
    state.acting_player = Some(ap);
    TurnStepResult::Ok
}

/// End a player's activation: mark player as having acted this turn.
pub fn end_activation(state: &mut GameState) {
    if let Some(ap) = state.acting_player.take() {
        state.players_activated_this_turn.insert(ap.player_id);
    }
    state.dialog = DialogState::None;
}

// ── Re-roll helpers ───────────────────────────────────────────────────────────

/// Trigger the apothecary dialog if one is available for the player's team.
/// Rolls the reroll and sets up the SelectApothecary dialog with both options.
/// Returns true if apothecary was triggered (dialog set), false if not available.
pub fn maybe_trigger_apothecary(
    state: &mut GameState,
    player_id: &PlayerId,
    original_casualty: CasualtyType,
    rng: &mut GameRng,
) -> bool {
    // Determine which team the player belongs to
    let team = match state.field.player_team(player_id) {
        Some(t) => t,
        None => {
            // Player not on pitch — check both teams by ID
            if state.home.player_by_id(player_id).is_some() {
                TeamId::Home
            } else if state.away.player_by_id(player_id).is_some() {
                TeamId::Away
            } else {
                return false;
            }
        }
    };

    let apothecary_available = state.team(team).apothecary_available && !state.team(team).apothecary_used;
    if !apothecary_available {
        return false;
    }

    let (reroll_result, _was_improved) = apply_apothecary(original_casualty, rng);
    state.dialog = DialogState::SelectApothecary {
        original: original_casualty,
        reroll: reroll_result,
    };
    true
}

// ── JumpUp ────────────────────────────────────────────────────────────────────

/// Returns true if the player has JumpUp and is currently Prone, allowing them
/// to stand up without making a roll.
pub fn can_jump_up(state: &GameState, player_id: &PlayerId) -> bool {
    let team = match state.field.player_team(player_id) {
        Some(t) => t,
        None => return false,
    };
    let has_skill = state.team(team)
        .player_by_id(player_id)
        .map(|p| p.has_skill(SkillId::JumpUp))
        .unwrap_or(false);
    if !has_skill {
        return false;
    }
    state.field.player_state(player_id) == Some(crate::types::PlayerState::Prone)
}

// ── QuickFoul ─────────────────────────────────────────────────────────────────

/// Returns true if the player has QuickFoul, which means their foul does not
/// consume the team's foul action for the turn.
pub fn has_quick_foul(state: &GameState, player_id: &PlayerId) -> bool {
    let team = match state.field.player_team(player_id) {
        Some(t) => t,
        None => return false,
    };
    state.team(team)
        .player_by_id(player_id)
        .map(|p| p.has_skill(SkillId::QuickFoul))
        .unwrap_or(false)
}

// ── Leader skill ──────────────────────────────────────────────────────────────

/// Returns true if the team has a Leader player on pitch and has not yet used
/// the leader re-roll this turn.
pub fn leader_reroll_available(state: &GameState, team: TeamId) -> bool {
    let td = match team {
        TeamId::Home => &state.turn_data_home,
        TeamId::Away => &state.turn_data_away,
    };
    if td.leader_reroll_used {
        return false;
    }
    state.field.team_players_on_pitch(team)
        .any(|(pid, _, _)| {
            state.team(team)
                .player_by_id(pid)
                .map(|p| p.has_skill(SkillId::Leader))
                .unwrap_or(false)
        })
}

// ── GiveAndGo skill ───────────────────────────────────────────────────────────

/// Returns true if the player has GiveAndGo and the active team has used a pass
/// this turn (allowing a 2-square follow-up move).
pub fn give_and_go_available(state: &GameState, player_id: &PlayerId) -> bool {
    let team = match state.field.player_team(player_id) {
        Some(t) => t,
        None => return false,
    };
    let pass_used = match team {
        TeamId::Home => state.turn_data_home.pass_used,
        TeamId::Away => state.turn_data_away.pass_used,
    };
    if !pass_used {
        return false;
    }
    state.team(team)
        .player_by_id(player_id)
        .map(|p| p.has_skill(SkillId::GiveAndGo))
        .unwrap_or(false)
}

// ── HypnoticGaze ──────────────────────────────────────────────────────────────

/// Apply a HypnoticGaze action from `gazer_id` against `target_id`.
///
/// Rules summary (BB2025):
/// - Gazer must have the HypnoticGaze skill.
/// - Target must be within 3 squares (Chebyshev distance) of the gazer.
/// - Target rolls d6; needs ≥ (4 − target_AG + opposing_TZ_count) to resist,
///   clamped to [2, 6].
/// - If the target **fails** the resistance roll, they are marked as hypnotized
///   for the rest of this turn: their tackle zone does not count.
///   Consumers check `state.hypnotized_this_turn` when computing effective TZs.
///
/// Returns `true` if the gaze succeeded (target is now hypnotized).
pub fn apply_hypnotic_gaze(
    state: &mut GameState,
    gazer_id: &PlayerId,
    target_id: &PlayerId,
    rng: &mut GameRng,
) -> bool {
    // 1. Gazer must have HypnoticGaze skill.
    let gazer_team = match state.field.player_team(gazer_id) {
        Some(t) => t,
        None => return false,
    };
    let has_skill = state.team(gazer_team)
        .player_by_id(gazer_id)
        .map(|p| p.has_skill(SkillId::HypnoticGaze))
        .unwrap_or(false);
    if !has_skill {
        return false;
    }

    // 2. Target must be on pitch and within 3 squares (Chebyshev).
    let gazer_coord = match state.field.player_coord(gazer_id) {
        Some(c) => c,
        None => return false,
    };
    let target_coord = match state.field.player_coord(target_id) {
        Some(c) => c,
        None => return false,
    };
    let dx = (gazer_coord.x as i16 - target_coord.x as i16).unsigned_abs();
    let dy = (gazer_coord.y as i16 - target_coord.y as i16).unsigned_abs();
    let chebyshev = dx.max(dy);
    if chebyshev > 3 {
        return false;
    }

    // 3. Determine resistance roll threshold.
    //    Base: 4 − target_AG; each opposing TZ on the target's square adds +1.
    let target_team = match state.field.player_team(target_id) {
        Some(t) => t,
        None => return false,
    };
    let (target_ag, opposing_tz) = {
        let p = state.team(target_team).player_by_id(target_id).expect("target player");
        let ag = p.effective_ag();
        let opp_team = target_team.opponent();
        let tz = state.field.tackle_zones_on(target_coord, target_team);
        // Also exclude hypnotized players from their TZ count (they don't contribute)
        // For simplicity, use the current TZ map directly.
        let _ = opp_team; // opposing team TZs are already on target_team's coord
        (ag, tz)
    };
    let threshold = (4i16 - target_ag as i16 + opposing_tz as i16).clamp(2, 6) as u8;

    // 4. Target rolls d6 to resist. Target must meet or exceed threshold to resist.
    let roll = rng.roll_d6();
    let resisted = roll >= threshold;

    if !resisted {
        // Target failed to resist — they are hypnotized.
        state.hypnotized_this_turn.insert(target_id.clone());
        return true;
    }

    false
}

// ── Pro reroll ────────────────────────────────────────────────────────────────

/// Attempt to use the Pro skill re-roll for `player_id`.
///
/// The Pro skill allows a re-roll of any failed skill test once per activation,
/// but only if a d6 roll of 3+ succeeds.
///
/// Returns `true` if the Pro re-roll is granted (player has Pro, hasn't used it yet, rolls 3+).
/// Marks `ap.pro_reroll_used = true` on success (or on attempt — always consumed after first try).
pub fn try_pro_reroll(state: &mut GameState, player_id: &PlayerId, rng: &mut GameRng) -> bool {
    // Player must have Pro skill
    let team = match state.field.player_team(player_id) {
        Some(t) => t,
        None => return false,
    };
    let has_pro = state.team(team)
        .player_by_id(player_id)
        .map(|p| p.has_skill(SkillId::Pro))
        .unwrap_or(false);
    if !has_pro {
        return false;
    }

    // Check Pro reroll hasn't been used this activation
    let already_used = state.acting_player.as_ref()
        .map(|ap| ap.pro_reroll_used)
        .unwrap_or(true);
    if already_used {
        return false;
    }

    // Mark as used (regardless of roll outcome — attempt is consumed)
    if let Some(ap) = state.acting_player.as_mut() {
        ap.pro_reroll_used = true;
    }

    // Roll d6: 3+ succeeds
    rng.roll_d6() >= 3
}

/// Returns true if the active team has any re-roll available this turn
/// (either from their pool or from the Leader skill).
pub fn active_team_reroll_available(state: &GameState) -> bool {
    let team_id = state.active_team_id();
    let td = state.active_turn_data();
    if td.reroll_used {
        return false;
    }
    if state.active_team().rerolls_remaining > 0 {
        return true;
    }
    leader_reroll_available(state, team_id)
}

/// Try to consume a team re-roll for the active team.
/// Returns `true` if the re-roll was available and consumed.
/// Falls back to the Leader skill free re-roll if the regular pool is empty.
pub fn use_team_reroll(state: &mut GameState, player_id: &PlayerId, rng: &mut GameRng) -> bool {
    // Check if player has Loner skill (5/6 chance of failing to use team reroll)
    let has_loner = state.active_team()
        .player_by_id(player_id)
        .map(|p| p.has_skill(SkillId::Loner))
        .unwrap_or(false);

    if has_loner {
        let roll = rng.roll_d6();
        if roll < 4 {
            // Loner failed — can't use team reroll but it's not consumed
            return false;
        }
    }

    let td = state.active_turn_data_mut();
    if td.reroll_used {
        return false;
    }

    // Try regular reroll pool first
    let team = state.active_team_mut();
    if team.use_reroll() {
        state.active_turn_data_mut().reroll_used = true;
        return true;
    }

    // Fall back to Leader skill free reroll
    let team_id = state.active_team_id();
    if leader_reroll_available(state, team_id) {
        let td = state.active_turn_data_mut();
        td.leader_reroll_used = true;
        td.reroll_used = true;
        return true;
    }

    false
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
    use crate::skills::{SkillId, SkillSet};
    use crate::types::{FieldCoordinate, PlayerId, PlayerState, TeamId};

    fn make_player_state(skills: SkillSet, pstate: PlayerState) -> (GameState, PlayerId) {
        let pid = PlayerId("p1".into());
        let player = Player::new(
            pid.clone(),
            "Player".into(),
            "lineman".into(),
            TeamId::Home,
            1,
            PlayerStats::new(6, 3, 3, 8, None),
            skills,
        );
        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        home.add_player(player);
        let away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        let mut state = GameState::new(home, away);
        state.field.place_player(pid.clone(), TeamId::Home, FieldCoordinate::new(5, 5), pstate);
        state.home_is_active = true;
        (state, pid)
    }

    // ── JumpUp ────────────────────────────────────────────────────────────────

    #[test]
    fn jump_up_true_when_prone_with_skill() {
        let (state, pid) = make_player_state(
            [SkillId::JumpUp].into_iter().collect(),
            PlayerState::Prone,
        );
        assert!(can_jump_up(&state, &pid));
    }

    #[test]
    fn jump_up_false_when_standing_with_skill() {
        let (state, pid) = make_player_state(
            [SkillId::JumpUp].into_iter().collect(),
            PlayerState::Standing,
        );
        assert!(!can_jump_up(&state, &pid));
    }

    #[test]
    fn jump_up_false_without_skill_even_if_prone() {
        let (state, pid) = make_player_state(SkillSet::empty(), PlayerState::Prone);
        assert!(!can_jump_up(&state, &pid));
    }

    // ── QuickFoul ─────────────────────────────────────────────────────────────

    #[test]
    fn quick_foul_detected_when_has_skill() {
        let (state, pid) = make_player_state(
            [SkillId::QuickFoul].into_iter().collect(),
            PlayerState::Standing,
        );
        assert!(has_quick_foul(&state, &pid));
    }

    #[test]
    fn quick_foul_absent_without_skill() {
        let (state, pid) = make_player_state(SkillSet::empty(), PlayerState::Standing);
        assert!(!has_quick_foul(&state, &pid));
    }

    #[test]
    fn quick_foul_does_not_set_foul_used_flag() {
        // When a player with QuickFoul fouls, has_quick_foul returns true
        // and the caller should NOT set turn_data.foul_used.
        let (mut state, pid) = make_player_state(
            [SkillId::QuickFoul].into_iter().collect(),
            PlayerState::Standing,
        );
        assert!(has_quick_foul(&state, &pid));
        // A normal foul sets the flag; QuickFoul prevents that.
        // Verify foul_used starts false, and that QuickFoul is detected.
        assert!(!state.turn_data_home.foul_used);
        // If the caller checks has_quick_foul before setting foul_used, it stays false.
        if !has_quick_foul(&state, &pid) {
            state.turn_data_home.foul_used = true;
        }
        assert!(!state.turn_data_home.foul_used);
    }

    // ── Leader reroll ─────────────────────────────────────────────────────────

    fn make_leader_state() -> (GameState, PlayerId) {
        let pid = PlayerId("leader_p".into());
        let skills: SkillSet = [SkillId::Leader].into_iter().collect();
        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        home.add_player(Player::new(
            pid.clone(), "Leader".into(), "thrower".into(), TeamId::Home, 1,
            PlayerStats::new(6, 3, 3, 8, None), skills,
        ));
        let away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        let mut state = GameState::new(home, away);
        state.field.place_player(pid.clone(), TeamId::Home, FieldCoordinate::new(5, 5), PlayerState::Standing);
        state.home_is_active = true;
        (state, pid)
    }

    #[test]
    fn leader_reroll_available_when_leader_on_pitch() {
        let (state, _) = make_leader_state();
        assert!(leader_reroll_available(&state, TeamId::Home));
    }

    #[test]
    fn leader_reroll_not_available_after_used() {
        let (mut state, _) = make_leader_state();
        state.turn_data_home.leader_reroll_used = true;
        assert!(!leader_reroll_available(&state, TeamId::Home));
    }

    #[test]
    fn leader_reroll_not_available_without_leader_player() {
        let home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        let away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        let state = GameState::new(home, away);
        assert!(!leader_reroll_available(&state, TeamId::Home));
    }

    // ── Stunned → Prone at turn start ─────────────────────────────────────────

    #[test]
    fn stunned_transitions_to_prone_on_turn_start() {
        // Place a stunned player on the active team (Home), call begin_turn,
        // verify the player's state becomes Prone.
        let pid = PlayerId("stunned_p".into());
        let player = crate::model::player::Player::new(
            pid.clone(), "Stunned".into(), "lineman".into(), TeamId::Home, 1,
            crate::model::player::PlayerStats::new(6, 3, 3, 8, None), SkillSet::empty(),
        );
        let mut home = crate::model::team::Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        home.add_player(player);
        let away = crate::model::team::Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        let mut state = GameState::new(home, away);
        state.field.place_player(pid.clone(), TeamId::Home, FieldCoordinate::new(5, 5), PlayerState::Stunned);
        state.home_is_active = true;

        begin_turn(&mut state);

        assert_eq!(
            state.field.player_state(&pid),
            Some(PlayerState::Prone),
            "stunned player should become prone at turn start"
        );
    }

    #[test]
    fn stunned_on_inactive_team_does_not_change() {
        // A stunned player on the AWAY team should NOT change when HOME's turn starts.
        let pid = PlayerId("stunned_away".into());
        let player = crate::model::player::Player::new(
            pid.clone(), "Stunned Away".into(), "lineman".into(), TeamId::Away, 1,
            crate::model::player::PlayerStats::new(6, 3, 3, 8, None), SkillSet::empty(),
        );
        let home = crate::model::team::Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        let mut away = crate::model::team::Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        away.add_player(player);
        let mut state = GameState::new(home, away);
        state.field.place_player(pid.clone(), TeamId::Away, FieldCoordinate::new(8, 8), PlayerState::Stunned);
        state.home_is_active = true; // Home is active; Away is inactive

        begin_turn(&mut state);

        assert_eq!(
            state.field.player_state(&pid),
            Some(PlayerState::Stunned),
            "stunned player on inactive team should remain stunned"
        );
    }

    // ── Bloodlust ─────────────────────────────────────────────────────────────

    #[test]
    fn bloodlust_turnover_on_bite() {
        // Vampire with Bloodlust, adjacent teammate exists, roll=1 → bites teammate → TurnOver.
        use crate::rng::GameRng;

        let vamp_id = PlayerId("vamp".into());
        let victim_id = PlayerId("victim".into());

        let vamp_skills: SkillSet = [SkillId::Bloodlust].into_iter().collect();

        let mut home = crate::model::team::Team::new("h".into(), "Home".into(), "Vampire".into(), 3, true);
        home.add_player(crate::model::player::Player::new(
            vamp_id.clone(), "Count Dracula".into(), "vampire".into(), TeamId::Home, 1,
            crate::model::player::PlayerStats::new(6, 4, 4, 8, None), vamp_skills,
        ));
        home.add_player(crate::model::player::Player::new(
            victim_id.clone(), "Thrall".into(), "lineman".into(), TeamId::Home, 2,
            crate::model::player::PlayerStats::new(6, 3, 3, 8, None), SkillSet::empty(),
        ));
        let away = crate::model::team::Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        let mut state = GameState::new(home, away);
        // Place vampire at (5,5) and victim adjacent at (6,5)
        state.field.place_player(vamp_id.clone(), TeamId::Home, FieldCoordinate::new(5, 5), PlayerState::Standing);
        state.field.place_player(victim_id.clone(), TeamId::Home, FieldCoordinate::new(6, 5), PlayerState::Standing);
        state.home_is_active = true;

        // Roll=1 → bloodlust activates (bite)
        let mut rng = GameRng::new_test([1]);
        let result = begin_activation(&mut state, &vamp_id, &mut rng);
        assert_eq!(result, TurnStepResult::TurnOver, "bloodlust bite should cause turnover");
        // Victim should be Injured
        assert_eq!(
            state.field.player_state(&victim_id),
            Some(PlayerState::Injured),
            "bitten teammate should become Injured"
        );
    }

    #[test]
    fn bloodlust_no_turnover_on_roll_2_plus() {
        // Roll=2 → vampire feeds successfully, no turnover.
        use crate::rng::GameRng;

        let vamp_id = PlayerId("vamp2".into());
        let vamp_skills: SkillSet = [SkillId::Bloodlust].into_iter().collect();
        let mut home = crate::model::team::Team::new("h".into(), "Home".into(), "Vampire".into(), 3, true);
        home.add_player(crate::model::player::Player::new(
            vamp_id.clone(), "Vampire 2".into(), "vampire".into(), TeamId::Home, 1,
            crate::model::player::PlayerStats::new(6, 4, 4, 8, None), vamp_skills,
        ));
        let away = crate::model::team::Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        let mut state = GameState::new(home, away);
        state.field.place_player(vamp_id.clone(), TeamId::Home, FieldCoordinate::new(5, 5), PlayerState::Standing);
        state.home_is_active = true;

        // Roll=2 → success (2+)
        let mut rng = GameRng::new_test([2]);
        let result = begin_activation(&mut state, &vamp_id, &mut rng);
        assert_eq!(result, TurnStepResult::Ok, "bloodlust on 2+ should not cause turnover");
    }

    // ── TakeRoot ──────────────────────────────────────────────────────────────

    #[test]
    fn take_root_activates_on_roll_1() {
        // Player with TakeRoot, roll=1 → movement_used set to full MA.
        use crate::rng::GameRng;

        let pid = PlayerId("rooted_p".into());
        let skills: SkillSet = [SkillId::TakeRoot].into_iter().collect();
        let ma = 6u8;
        let mut home = crate::model::team::Team::new("h".into(), "Home".into(), "Tree".into(), 3, true);
        home.add_player(crate::model::player::Player::new(
            pid.clone(), "Treeman".into(), "treeman".into(), TeamId::Home, 1,
            crate::model::player::PlayerStats::new(ma, 6, 1, 10, None), skills,
        ));
        let away = crate::model::team::Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        let mut state = GameState::new(home, away);
        state.field.place_player(pid.clone(), TeamId::Home, FieldCoordinate::new(5, 5), PlayerState::Standing);
        state.home_is_active = true;

        // Roll=1 → TakeRoot activates
        let mut rng = GameRng::new_test([1]);
        let result = begin_activation(&mut state, &pid, &mut rng);
        assert_eq!(result, TurnStepResult::Ok, "TakeRoot should not cause TurnOver");
        let ap = state.acting_player.as_ref().expect("acting player");
        assert_eq!(ap.movement_used, ma, "TakeRoot should set movement_used to full MA");
    }

    #[test]
    fn take_root_no_effect_on_roll_2_plus() {
        // Roll=2 → TakeRoot does NOT activate.
        use crate::rng::GameRng;

        let pid = PlayerId("free_treeman".into());
        let skills: SkillSet = [SkillId::TakeRoot].into_iter().collect();
        let ma = 6u8;
        let mut home = crate::model::team::Team::new("h".into(), "Home".into(), "Tree".into(), 3, true);
        home.add_player(crate::model::player::Player::new(
            pid.clone(), "Treeman 2".into(), "treeman".into(), TeamId::Home, 1,
            crate::model::player::PlayerStats::new(ma, 6, 1, 10, None), skills,
        ));
        let away = crate::model::team::Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        let mut state = GameState::new(home, away);
        state.field.place_player(pid.clone(), TeamId::Home, FieldCoordinate::new(5, 5), PlayerState::Standing);
        state.home_is_active = true;

        let mut rng = GameRng::new_test([2]);
        begin_activation(&mut state, &pid, &mut rng);
        let ap = state.acting_player.as_ref().expect("acting player");
        assert_eq!(ap.movement_used, 0, "TakeRoot on 2+ should not consume movement");
    }

    // ── HypnoticGaze ──────────────────────────────────────────────────────────

    fn make_hypnotic_state() -> (GameState, PlayerId, PlayerId) {
        use crate::rng::GameRng;
        let _ = GameRng::new_test([]); // type check

        let gazer_id = PlayerId("gazer".into());
        let target_id = PlayerId("target".into());

        let gazer_skills: SkillSet = [SkillId::HypnoticGaze].into_iter().collect();

        let mut home = Team::new("h".into(), "Home".into(), "Vampire".into(), 3, true);
        home.add_player(crate::model::player::Player::new(
            gazer_id.clone(), "Vampire Gazer".into(), "vampire".into(), TeamId::Home, 1,
            crate::model::player::PlayerStats::new(6, 4, 4, 8, None), gazer_skills,
        ));
        let mut away = Team::new("a".into(), "Away".into(), "Human".into(), 3, false);
        away.add_player(crate::model::player::Player::new(
            target_id.clone(), "Target".into(), "lineman".into(), TeamId::Away, 1,
            crate::model::player::PlayerStats::new(6, 3, 3, 8, None), SkillSet::empty(),
        ));

        let mut state = GameState::new(home, away);
        // Place gazer at (5,5) and target adjacent at (6,5)
        state.field.place_player(gazer_id.clone(), TeamId::Home, FieldCoordinate::new(5, 5), PlayerState::Standing);
        state.field.place_player(target_id.clone(), TeamId::Away, FieldCoordinate::new(6, 5), PlayerState::Standing);
        state.home_is_active = true;

        (state, gazer_id, target_id)
    }

    #[test]
    fn hypnotic_gaze_success_marks_target() {
        use crate::rng::GameRng;
        // target AG=3, no opposing TZ → threshold = max(2, 4 - 3 + 0) = 1 → clamped to 2
        // Roll=1 → target fails to resist (1 < 2) → hypnotized
        let (mut state, gazer_id, target_id) = make_hypnotic_state();
        let mut rng = GameRng::new_test([1]);
        let result = apply_hypnotic_gaze(&mut state, &gazer_id, &target_id, &mut rng);
        assert!(result, "gaze should succeed when target fails resistance");
        assert!(
            state.hypnotized_this_turn.contains(&target_id),
            "target should be in hypnotized set"
        );
    }

    #[test]
    fn hypnotic_gaze_fails_on_high_roll() {
        use crate::rng::GameRng;
        // target AG=3, no opposing TZ → threshold=2; roll=6 → target resists
        let (mut state, gazer_id, target_id) = make_hypnotic_state();
        let mut rng = GameRng::new_test([6]);
        let result = apply_hypnotic_gaze(&mut state, &gazer_id, &target_id, &mut rng);
        assert!(!result, "gaze should fail when target resists");
        assert!(
            !state.hypnotized_this_turn.contains(&target_id),
            "target should not be hypnotized"
        );
    }

    #[test]
    fn hypnotic_gaze_fails_without_skill() {
        use crate::rng::GameRng;
        // Gazer without HypnoticGaze skill
        let (mut state, gazer_id, target_id) = make_hypnotic_state();
        // Replace gazer with a player without the skill
        state.home.player_by_id_mut(&gazer_id).unwrap().skills = SkillSet::empty();
        let mut rng = GameRng::new_test([]);
        let result = apply_hypnotic_gaze(&mut state, &gazer_id, &target_id, &mut rng);
        assert!(!result, "gaze should fail if gazer lacks HypnoticGaze skill");
    }

    #[test]
    fn hypnotic_gaze_fails_out_of_range() {
        use crate::rng::GameRng;
        // Target is more than 3 squares away
        let (mut state, gazer_id, target_id) = make_hypnotic_state();
        // Move target to (10, 5) — 5 squares away
        state.field.remove_player(&target_id);
        state.field.place_player(target_id.clone(), TeamId::Away, FieldCoordinate::new(10, 5), PlayerState::Standing);
        let mut rng = GameRng::new_test([]);
        let result = apply_hypnotic_gaze(&mut state, &gazer_id, &target_id, &mut rng);
        assert!(!result, "gaze should fail when target is out of range");
    }

    #[test]
    fn hypnotic_gaze_cleared_at_turn_start() {
        use crate::rng::GameRng;
        let (mut state, gazer_id, target_id) = make_hypnotic_state();
        let mut rng = GameRng::new_test([1]);
        apply_hypnotic_gaze(&mut state, &gazer_id, &target_id, &mut rng);
        assert!(state.hypnotized_this_turn.contains(&target_id));
        // begin_turn should clear the set
        begin_turn(&mut state);
        assert!(state.hypnotized_this_turn.is_empty(), "hypnotized set should be cleared at turn start");
    }

    // ── GiveAndGo ─────────────────────────────────────────────────────────────

    #[test]
    fn give_and_go_available_after_pass() {
        let (mut state, pid) = make_player_state(
            [SkillId::GiveAndGo].into_iter().collect(),
            PlayerState::Standing,
        );
        // Before pass: not available
        assert!(!give_and_go_available(&state, &pid));
        // After pass used:
        state.turn_data_home.pass_used = true;
        assert!(give_and_go_available(&state, &pid));
    }

    #[test]
    fn give_and_go_not_available_without_skill() {
        let (mut state, pid) = make_player_state(SkillSet::empty(), PlayerState::Standing);
        state.turn_data_home.pass_used = true;
        assert!(!give_and_go_available(&state, &pid));
    }

    // ── Pro reroll tests ──────────────────────────────────────────────────────

    fn make_pro_state() -> (GameState, PlayerId) {
        use crate::rng::GameRng;
        let _ = GameRng::new_test([]); // ensure import
        make_player_state([SkillId::Pro].into_iter().collect(), PlayerState::Standing)
    }

    #[test]
    fn pro_reroll_succeeds_on_3_plus() {
        use crate::rng::GameRng;
        let (mut state, pid) = make_pro_state();
        // Set up acting player
        state.acting_player = Some(crate::model::game_state::ActingPlayer::new(pid.clone(), TeamId::Home));

        // Roll 3 → Pro succeeds (3+)
        let mut rng = GameRng::new_test([3]);
        let result = try_pro_reroll(&mut state, &pid, &mut rng);
        assert!(result, "Pro reroll should succeed on roll of 3");
        // Pro reroll marked as used
        assert!(
            state.acting_player.as_ref().map(|ap| ap.pro_reroll_used).unwrap_or(false),
            "pro_reroll_used should be set after attempt"
        );
    }

    #[test]
    fn pro_reroll_succeeds_on_6() {
        use crate::rng::GameRng;
        let (mut state, pid) = make_pro_state();
        state.acting_player = Some(crate::model::game_state::ActingPlayer::new(pid.clone(), TeamId::Home));

        let mut rng = GameRng::new_test([6]);
        assert!(try_pro_reroll(&mut state, &pid, &mut rng), "Pro should succeed on roll of 6");
    }

    #[test]
    fn pro_reroll_fails_on_1_2() {
        use crate::rng::GameRng;
        for roll in [1u8, 2u8] {
            let (mut state, pid) = make_pro_state();
            state.acting_player = Some(crate::model::game_state::ActingPlayer::new(pid.clone(), TeamId::Home));

            let mut rng = GameRng::new_test([roll]);
            let result = try_pro_reroll(&mut state, &pid, &mut rng);
            assert!(!result, "Pro reroll should fail on roll of {roll}");
            // Still marked as used (attempt consumed)
            assert!(
                state.acting_player.as_ref().map(|ap| ap.pro_reroll_used).unwrap_or(false),
                "pro_reroll_used should be set even on failed roll"
            );
        }
    }

    #[test]
    fn pro_reroll_cannot_be_used_twice() {
        use crate::rng::GameRng;
        let (mut state, pid) = make_pro_state();
        state.acting_player = Some(crate::model::game_state::ActingPlayer::new(pid.clone(), TeamId::Home));

        // First use: succeeds
        let mut rng = GameRng::new_test([5]);
        assert!(try_pro_reroll(&mut state, &pid, &mut rng));

        // Second use: should fail even with a good roll (queue would be exhausted if attempted)
        let mut rng2 = GameRng::new_test([]);
        let result = try_pro_reroll(&mut state, &pid, &mut rng2);
        assert!(!result, "Pro reroll should not be usable a second time");
    }

    #[test]
    fn pro_reroll_not_available_without_skill() {
        use crate::rng::GameRng;
        let (mut state, pid) = make_player_state(SkillSet::empty(), PlayerState::Standing);
        state.acting_player = Some(crate::model::game_state::ActingPlayer::new(pid.clone(), TeamId::Home));

        // Roll sequence empty — should not roll anything
        let mut rng = GameRng::new_test([]);
        let result = try_pro_reroll(&mut state, &pid, &mut rng);
        assert!(!result, "Pro reroll should not be available without Pro skill");
    }
}
