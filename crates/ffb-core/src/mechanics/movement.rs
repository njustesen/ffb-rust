use crate::model::game_state::GameState;
use crate::pathfinding::dodge_min_roll;
use crate::rng::GameRng;
use crate::skills::SkillId;
use crate::types::{FieldCoordinate, PlayerId, PlayerState};

/// Returns the extra TZ count contributed by adjacent opponents with PrehensileTail.
/// Each adjacent active opponent with PrehensileTail adds +1 to effective TZ count.
fn prehensile_tail_tz_bonus(state: &GameState, coord: FieldCoordinate, moving_team: crate::types::TeamId) -> u8 {
    let opponent = moving_team.opponent();
    coord.neighbors().filter(|&n| {
        state.field.player_at(n)
            .and_then(|pid| {
                if state.field.player_team(pid) == Some(opponent) {
                    if let Some(st) = state.field.player_state(pid) {
                        if st.is_active() {
                            return state.team(opponent).player_by_id(pid);
                        }
                    }
                }
                None
            })
            .map(|p| p.has_skill(SkillId::PrehensileTail))
            .unwrap_or(false)
    }).count() as u8
}

/// Returns true if any player in the opponent's tackle zones on `coord` has the Tackle skill.
/// Tackle prevents the dodging player from using the Dodge skill re-roll.
fn tackle_in_tz(state: &GameState, coord: FieldCoordinate, moving_team: crate::types::TeamId) -> bool {
    let opponent = moving_team.opponent();
    coord.neighbors().any(|n| {
        state.field.player_at(n)
            .and_then(|pid| {
                if state.field.player_team(pid) == Some(opponent) {
                    state.team(opponent).player_by_id(pid)
                } else {
                    None
                }
            })
            .map(|p| p.has_skill(SkillId::Tackle))
            .unwrap_or(false)
    })
}

// ── DivingTackle ──────────────────────────────────────────────────────────────

/// Returns the extra penalty to add to the dodge min_roll when a player leaves `coord`.
///
/// Per BB2025: each opponent adjacent to `coord` with the DivingTackle skill may
/// declare its use, adding +2 to the difficulty of the dodge roll.
/// This function counts how many such opponents exist and returns 2 × count.
pub fn diving_tackle_bonus_on_square(
    state: &GameState,
    coord: FieldCoordinate,
    moving_team: crate::types::TeamId,
) -> u8 {
    let opponent = moving_team.opponent();
    let count = coord.neighbors().filter(|&n| {
        state.field.player_at(n)
            .and_then(|pid| {
                if state.field.player_team(pid) == Some(opponent) {
                    // Only standing (active) opponents can use DivingTackle
                    if let Some(st) = state.field.player_state(pid) {
                        if st.is_active() {
                            return state.team(opponent).player_by_id(pid);
                        }
                    }
                }
                None
            })
            .map(|p| p.has_skill(SkillId::DivingTackle))
            .unwrap_or(false)
    }).count() as u8;
    count * 2
}

// ── AlwaysHungry ──────────────────────────────────────────────────────────────

/// Roll for AlwaysHungry when a player attempts to perform a ThrowTeamMate action.
/// Returns `true` if the throw is safe (roll >= 2), `false` if the carrier is eaten (roll == 1).
pub fn always_hungry_check(rng: &mut GameRng) -> bool {
    rng.roll_d6() >= 2
}

// ── Tentacles ─────────────────────────────────────────────────────────────────

/// Returns true if any adjacent opponent of `moving_team` at `coord` has the Tentacles skill.
fn tentacles_in_tz(state: &GameState, coord: FieldCoordinate, moving_team: crate::types::TeamId) -> bool {
    let opponent = moving_team.opponent();
    coord.neighbors().any(|n| {
        state.field.player_at(n)
            .and_then(|pid| {
                if state.field.player_team(pid) == Some(opponent) {
                    state.team(opponent).player_by_id(pid)
                } else {
                    None
                }
            })
            .map(|p| p.has_skill(SkillId::Tentacles))
            .unwrap_or(false)
    })
}

// ── Move outcome ──────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
pub enum MoveOutcome {
    /// Player reached destination successfully.
    Success,
    /// Player was knocked down at `at` square (dodge/GFI failure).
    KnockedDown { at: FieldCoordinate },
    /// Player was caught by Tentacles at `at` square and cannot move further.
    /// Player is not knocked down — they simply cannot complete the move.
    CaughtByTentacles { at: FieldCoordinate },
    /// A dodge or GFI roll failed after all skill rerolls were exhausted.
    /// The player has NOT been knocked down yet — caller should offer a team reroll dialog.
    /// If reroll accepted: move to `current_dest` (dodge) or player already there (GFI),
    /// then continue with `remaining_after`. If declined: knock down at player's current coord.
    NeedsTeamReroll {
        current_dest: FieldCoordinate,
        remaining_after: Vec<FieldCoordinate>,
        min_roll: u8,
        is_gfi: bool,
    },
}

// ── Execute a single move step ────────────────────────────────────────────────

/// Move `player_id` along `path`, rolling dice per square.
/// Returns the outcome (success or knockdown location).
/// Mutates `state` to reflect the player's new position and any knockdown.
pub fn execute_move_step(
    state: &mut GameState,
    player_id: &PlayerId,
    path: &[FieldCoordinate],
    rng: &mut GameRng,
) -> MoveOutcome {
    let team = state.field.player_team(player_id).expect("player not on pitch");
    let acting_player = state.acting_player.as_mut().expect("no acting player");
    let ma_remaining = {
        let p = state.home.player_by_id(player_id)
            .or_else(|| state.away.player_by_id(player_id))
            .expect("player not found");
        p.effective_ma().saturating_sub(acting_player.movement_used)
    };

    let (has_dodge, has_sure_feet, has_pro, ag, has_break_tackle) = {
        let p = state.home.player_by_id(player_id)
            .or_else(|| state.away.player_by_id(player_id))
            .expect("player not found");
        let break_tackle = p.has_skill(SkillId::BreakTackle);
        // BreakTackle: use max(AG, ST) as effective agility for dodge rolls
        let effective_ag = if break_tackle {
            p.effective_ag().max(p.effective_st())
        } else {
            p.effective_ag()
        };
        (p.has_skill(SkillId::Dodge), p.has_skill(SkillId::SureFeet), p.has_skill(SkillId::Pro), effective_ag, break_tackle)
    };
    let _ = has_break_tackle; // consumed via ag computation above
    // Read Pro reroll state from ActingPlayer so it persists across multiple calls
    // (e.g., when movement is resumed after a team reroll)
    let mut pro_reroll_used = state.acting_player.as_ref()
        .map(|ap| ap.pro_reroll_used)
        .unwrap_or(false);

    for (steps_taken, &dest) in (0_u8..).zip(path.iter()) {
        let current = state.field.player_coord(player_id).expect("player coord");
        let base_tz = state.effective_tackle_zones_on(current, team);

        // PrehensileTail: each adjacent opponent with PrehensileTail adds +1 to effective TZ.
        let pt_bonus = prehensile_tail_tz_bonus(state, current, team);
        // DivingTackle: each adjacent opponent with DivingTackle adds +2 to effective TZ.
        let dt_bonus = diving_tackle_bonus_on_square(state, current, team);
        let leaving_tz = base_tz.saturating_add(pt_bonus).saturating_add(dt_bonus);

        // Tentacles check: if leaving a TZ from an opponent with Tentacles, roll d6.
        // On a 1, player is caught and cannot move further (not knocked down).
        // On 2+, player escapes and must still make the normal dodge roll.
        if leaving_tz > 0 && tentacles_in_tz(state, current, team) {
            let tentacles_roll = rng.roll_d6();
            if tentacles_roll == 1 {
                // Caught by Tentacles — player stays at current square.
                return MoveOutcome::CaughtByTentacles { at: current };
            }
            // On 2+, player escapes tentacles and continues (dodge still required below).
        }

        // Dodge check
        if leaving_tz > 0 {
            let min_roll = dodge_min_roll(ag, leaving_tz, has_dodge);
            let roll = rng.roll_d6();
            let success = roll >= min_roll;
            if !success {
                // Tackle skill on any TZ source prevents Dodge skill re-roll
                let tackled = tackle_in_tz(state, current, team);
                let dodge_reroll = has_dodge && !tackled;
                let final_success = if dodge_reroll {
                    rng.roll_d6() >= min_roll
                } else if has_pro && !pro_reroll_used {
                    // Pro skill: roll d6 on 4+ to get a free re-roll (once per activation)
                    pro_reroll_used = true;
                    if let Some(ap) = state.acting_player.as_mut() {
                        ap.pro_reroll_used = true;
                    }
                    if rng.roll_d6() >= 4 {
                        rng.roll_d6() >= min_roll
                    } else {
                        false
                    }
                } else {
                    false
                };
                if !final_success {
                    // Signal caller to offer team reroll; don't knock down yet.
                    // Player stays at `current`; if reroll accepted and succeeds, move to `dest`.
                    let remaining_after = path[(steps_taken as usize + 1)..].to_vec();
                    return MoveOutcome::NeedsTeamReroll {
                        current_dest: dest,
                        remaining_after,
                        min_roll,
                        is_gfi: false,
                    };
                }
            }
        }

        // GFI check
        let is_gfi = steps_taken >= ma_remaining;
        if is_gfi {
            let roll = rng.roll_d6();
            let min = 5u8; // 5+ needed for GFI (2+ on d6, but we use the GFI min=2 when rolling)
            let success = roll >= min;
            if !success {
                let reroll = has_sure_feet;
                let final_success = if reroll {
                    rng.roll_d6() >= min
                } else {
                    false
                };
                if !final_success {
                    // Move player to dest first (they attempted the GFI and landed there)
                    if !state.field.is_occupied(dest) {
                        state.field.move_player(player_id, dest);
                    }
                    // Signal caller to offer team reroll; don't knock down yet.
                    let remaining_after = path[(steps_taken as usize + 1)..].to_vec();
                    return MoveOutcome::NeedsTeamReroll {
                        current_dest: dest,
                        remaining_after,
                        min_roll: min,
                        is_gfi: true,
                    };
                }
            }
        }

        // Move
        if !state.field.is_occupied(dest) {
            state.field.move_player(player_id, dest);
        }

        // Update acting player movement count
        if let Some(ap) = state.acting_player.as_mut() {
            ap.movement_used += 1;
        }

        // Shadowing: opponents adjacent to the player's old square (current) with Shadowing
        // attempt to follow. On success (shadower rolls ≥ mover), shadower moves to `current`.
        if leaving_tz > 0 {
            apply_shadowing_after_move(state, player_id, current, team, rng);
        }
    }

    MoveOutcome::Success
}

/// After a player moves from `from` to their current position, check if any adjacent
/// opponent with Shadowing can follow. If the shadower wins the MA roll-off, they move
/// to the `from` square.
fn apply_shadowing_after_move(
    state: &mut GameState,
    mover_id: &PlayerId,
    from: FieldCoordinate,
    mover_team: crate::types::TeamId,
    rng: &mut GameRng,
) {
    let mover_ma = state.home.player_by_id(mover_id)
        .or_else(|| state.away.player_by_id(mover_id))
        .map(|p| p.effective_ma())
        .unwrap_or(6);

    let opp_team = mover_team.opponent();
    // Find adjacent opponents (at their current position) with Shadowing skill
    let shadower_candidates: Vec<_> = from.neighbors()
        .filter_map(|n| state.field.player_at(n).cloned())
        .filter(|pid| state.field.player_team(pid) == Some(opp_team))
        .filter(|pid| {
            state.team(opp_team)
                .player_by_id(pid)
                .map(|p| p.has_skill(crate::skills::SkillId::Shadowing))
                .unwrap_or(false)
        })
        .filter(|pid| {
            state.field.player_state(pid)
                .map(|s| s.is_active())
                .unwrap_or(false)
        })
        .collect();

    for shadower_id in shadower_candidates {
        let shadower_ma = state.team(opp_team)
            .player_by_id(&shadower_id)
            .map(|p| p.effective_ma())
            .unwrap_or(6);

        // Roll-off: shadower rolls d6 + MA; mover rolls d6 + MA. Shadower wins on tie (≥).
        let mover_roll = rng.roll_d6() as u16 + mover_ma as u16;
        let shadower_roll = rng.roll_d6() as u16 + shadower_ma as u16;

        if shadower_roll >= mover_roll && !state.field.is_occupied(from) {
            state.field.move_player(&shadower_id, from);
        }
    }
}

fn knock_down_player(state: &mut GameState, player_id: &PlayerId) {
    state.field.set_player_state(player_id, PlayerState::Prone);
    // If this player is carrying the ball, the ball stays at their position (turnover handled by caller)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::game_state::ActingPlayer;
    use crate::model::player::{Player, PlayerStats};
    use crate::model::team::Team;
    use crate::rng::GameRng;
    use crate::skills::SkillSet;
    use crate::types::{FieldCoordinate, PlayerId, PlayerState, TeamId};

    fn setup_state_with_player(pid_str: &str, start: FieldCoordinate, ma: u8) -> (GameState, PlayerId) {
        let pid = PlayerId(pid_str.into());
        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        let player = Player::new(
            pid.clone(),
            pid_str.into(),
            "lineman".into(),
            TeamId::Home,
            1,
            PlayerStats::new(ma, 3, 4, 8, None),
            SkillSet::empty(),
        );
        home.add_player(player);
        let away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        let mut state = GameState::new(home, away);
        state.field.place_player(pid.clone(), TeamId::Home, start, PlayerState::Standing);
        state.acting_player = Some(ActingPlayer::new(pid.clone(), TeamId::Home));
        state.home_is_active = true;
        (state, pid)
    }

    #[test]
    fn move_succeeds_no_tz() {
        let start = FieldCoordinate::new(5, 5);
        let dest = FieldCoordinate::new(7, 5);
        let (mut state, pid) = setup_state_with_player("p1", start, 6);
        let path = vec![FieldCoordinate::new(6, 5), dest];
        // No dice needed (no TZs, no GFI)
        let mut rng = GameRng::new_test([]);
        let outcome = execute_move_step(&mut state, &pid, &path, &mut rng);
        assert_eq!(outcome, MoveOutcome::Success);
        assert_eq!(state.field.player_coord(&pid), Some(dest));
    }

    #[test]
    fn move_gfi_success() {
        let start = FieldCoordinate::new(5, 5);
        // MA=2, path of 3 squares → last square is GFI
        let (mut state, pid) = setup_state_with_player("p1", start, 2);
        let path = vec![
            FieldCoordinate::new(6, 5),
            FieldCoordinate::new(7, 5),
            FieldCoordinate::new(8, 5), // GFI
        ];
        // First 2 squares: no dice needed. GFI roll: 5+ → pass with roll=5
        let mut rng = GameRng::new_test([5]);
        let outcome = execute_move_step(&mut state, &pid, &path, &mut rng);
        assert_eq!(outcome, MoveOutcome::Success);
        assert_eq!(state.field.player_coord(&pid), Some(FieldCoordinate::new(8, 5)));
    }

    #[test]
    fn move_gfi_fail_knocks_down() {
        let start = FieldCoordinate::new(5, 5);
        let (mut state, pid) = setup_state_with_player("p1", start, 2);
        let path = vec![
            FieldCoordinate::new(6, 5),
            FieldCoordinate::new(7, 5),
            FieldCoordinate::new(8, 5), // GFI → fail
        ];
        // GFI roll = 1 (fail); no Sure Feet
        let mut rng = GameRng::new_test([1]);
        let outcome = execute_move_step(&mut state, &pid, &path, &mut rng);
        // Either KnockedDown (no reroll available) or NeedsTeamReroll (reroll offered to caller)
        assert!(
            matches!(outcome, MoveOutcome::KnockedDown { .. } | MoveOutcome::NeedsTeamReroll { .. }),
            "GFI fail must produce a failure outcome, got {:?}", outcome
        );
    }

    // ── Pro skill re-roll tests ───────────────────────────────────────────────

    fn setup_state_with_skills(
        pid_str: &str,
        start: FieldCoordinate,
        ma: u8,
        skills: SkillSet,
    ) -> (GameState, PlayerId) {
        let pid = PlayerId(pid_str.into());
        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        let player = Player::new(
            pid.clone(), pid_str.into(), "lineman".into(), TeamId::Home, 1,
            PlayerStats::new(ma, 3, 4, 8, None), skills,
        );
        home.add_player(player);
        let away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        let mut state = GameState::new(home, away);
        state.field.place_player(pid.clone(), TeamId::Home, start, PlayerState::Standing);
        state.acting_player = Some(ActingPlayer::new(pid.clone(), TeamId::Home));
        state.home_is_active = true;
        (state, pid)
    }

    #[test]
    fn pro_reroll_saves_failed_dodge() {
        use crate::skills::SkillId;
        // Setup: home player with Pro at (5,5), away player at (6,5) creating a tackle zone.
        // Player moves from (5,5) to (7,5) — first step leaves TZ of away player.
        // Dodge min roll (ag=4, 1 TZ, no Dodge skill) = 3.
        // Roll sequence: dodge roll=1 (fail), Pro check=5 (>=4, pro succeeds), re-roll=3 (>=3, success).
        let start = FieldCoordinate::new(5, 5);
        let pid = PlayerId("p1".into());
        let away_pid = PlayerId("a1".into());
        let skills: SkillSet = [SkillId::Pro].into_iter().collect();
        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        let mut away_team = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        home.add_player(Player::new(
            pid.clone(), "Pro".into(), "lineman".into(), TeamId::Home, 1,
            PlayerStats::new(6, 3, 4, 8, None), skills,
        ));
        away_team.add_player(Player::new(
            away_pid.clone(), "Tackler".into(), "lineman".into(), TeamId::Away, 2,
            PlayerStats::new(5, 4, 3, 9, None), SkillSet::empty(),
        ));
        let mut state = GameState::new(home, away_team);
        state.field.place_player(pid.clone(), TeamId::Home, start, PlayerState::Standing);
        state.field.place_player(away_pid.clone(), TeamId::Away, FieldCoordinate::new(6, 5), PlayerState::Standing);
        state.acting_player = Some(ActingPlayer::new(pid.clone(), TeamId::Home));
        state.home_is_active = true;

        let path = vec![FieldCoordinate::new(7, 5)];
        // dodge_min_roll(ag=4, tz=1, no_dodge) = 3
        // Roll 1: dodge=1 (fail), Roll 2: Pro check=5 (pass), Roll 3: re-roll=3 (>=3 success)
        let mut rng = GameRng::new_test([1, 5, 3]);
        let outcome = execute_move_step(&mut state, &pid, &path, &mut rng);
        assert_eq!(outcome, MoveOutcome::Success, "Pro re-roll should save the failed dodge");
    }

    // ── DivingTackle tests ────────────────────────────────────────────────────

    #[test]
    fn diving_tackle_bonus_zero_without_skill() {
        // No opponents with DivingTackle — bonus should be 0.
        let start = FieldCoordinate::new(5, 5);
        let (mut state, pid) = setup_state_with_player("p1", start, 6);
        // Place an away player adjacent (6,5) but without DivingTackle
        let away_pid = PlayerId("a1".into());
        let mut away = crate::model::team::Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        away.add_player(crate::model::player::Player::new(
            away_pid.clone(), "lineman".into(), "lineman".into(), TeamId::Away, 2,
            crate::model::player::PlayerStats::new(5, 3, 3, 9, None), SkillSet::empty(),
        ));
        state.away = away;
        state.field.place_player(away_pid.clone(), TeamId::Away, FieldCoordinate::new(6, 5), PlayerState::Standing);
        let bonus = diving_tackle_bonus_on_square(&state, start, TeamId::Home);
        assert_eq!(bonus, 0);
    }

    #[test]
    fn diving_tackle_bonus_two_per_adjacent_opponent_with_skill() {
        // One adjacent opponent with DivingTackle — bonus should be 2.
        let start = FieldCoordinate::new(5, 5);
        let (mut state, _pid) = setup_state_with_player("p1", start, 6);
        let away_pid = PlayerId("a1".into());
        let mut skills = SkillSet::empty();
        skills.add(SkillId::DivingTackle);
        let mut away = crate::model::team::Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        away.add_player(crate::model::player::Player::new(
            away_pid.clone(), "dt".into(), "blitzer".into(), TeamId::Away, 2,
            crate::model::player::PlayerStats::new(6, 3, 3, 9, None), skills,
        ));
        state.away = away;
        state.field.place_player(away_pid.clone(), TeamId::Away, FieldCoordinate::new(6, 5), PlayerState::Standing);
        let bonus = diving_tackle_bonus_on_square(&state, start, TeamId::Home);
        assert_eq!(bonus, 2);
    }

    // ── AlwaysHungry tests ────────────────────────────────────────────────────

    #[test]
    fn always_hungry_fails_on_one() {
        let mut rng = GameRng::new_test([1]);
        assert!(!always_hungry_check(&mut rng), "roll of 1 should cause eating");
    }

    #[test]
    fn always_hungry_succeeds_on_two_plus() {
        for roll in 2u8..=6 {
            let mut rng = GameRng::new_test([roll]);
            assert!(always_hungry_check(&mut rng), "roll of {roll} should be safe");
        }
    }

    // ── Tentacles tests ──────────────────────────────────────────────────────

    fn setup_with_away_tentacles(
        home_pid: &str, home_start: FieldCoordinate,
        away_pid: &str, away_coord: FieldCoordinate,
    ) -> (GameState, PlayerId) {
        let hpid = PlayerId(home_pid.into());
        let apid = PlayerId(away_pid.into());
        let tentacles_skills: SkillSet = [SkillId::Tentacles].into_iter().collect();

        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        home.add_player(Player::new(
            hpid.clone(), home_pid.into(), "lineman".into(), TeamId::Home, 1,
            PlayerStats::new(6, 3, 4, 8, None), SkillSet::empty(),
        ));
        let mut away = Team::new("a".into(), "Away".into(), "Lizard".into(), 3, false);
        away.add_player(Player::new(
            apid.clone(), away_pid.into(), "kroxigor".into(), TeamId::Away, 1,
            PlayerStats::new(6, 5, 1, 9, None), tentacles_skills,
        ));
        let mut state = GameState::new(home, away);
        state.field.place_player(hpid.clone(), TeamId::Home, home_start, PlayerState::Standing);
        state.field.place_player(apid, TeamId::Away, away_coord, PlayerState::Standing);
        state.acting_player = Some(ActingPlayer::new(hpid.clone(), TeamId::Home));
        state.home_is_active = true;
        (state, hpid)
    }

    #[test]
    fn tentacles_stops_player_on_1() {
        use crate::skills::SkillId;
        // Home player at (5,5) adjacent to away Tentacles player at (6,5).
        // Player tries to move to (4,5) — leaving TZ of Tentacles player.
        // Tentacles roll=1 → caught, move stops.
        let start = FieldCoordinate::new(5, 5);
        let (mut state, pid) = setup_with_away_tentacles(
            "mover", start, "tentacler", FieldCoordinate::new(6, 5),
        );
        let dest = FieldCoordinate::new(4, 5);
        // Roll sequence: tentacles=1 (caught)
        let mut rng = GameRng::new_test([1]);
        let outcome = execute_move_step(&mut state, &pid, &[dest], &mut rng);
        assert!(
            matches!(outcome, MoveOutcome::CaughtByTentacles { at } if at == start),
            "player should be caught by tentacles at their starting square"
        );
        // Player should still be at their original square
        assert_eq!(state.field.player_coord(&pid), Some(start));
    }

    #[test]
    fn tentacles_allows_move_on_2_plus() {
        use crate::skills::SkillId;
        // Home player at (5,5) adjacent to away Tentacles player at (6,5).
        // Tentacles roll=2 → escapes. No dodge needed (dodge roll would need TZ,
        // but player escapes and is no longer in TZ after moving).
        // Actually, the dodge roll IS still needed when leaving a TZ.
        // dodge_min_roll(ag=4, tz=1, no_dodge) = 3; we'll roll high enough.
        let start = FieldCoordinate::new(5, 5);
        let (mut state, pid) = setup_with_away_tentacles(
            "mover2", start, "tentacler2", FieldCoordinate::new(6, 5),
        );
        let dest = FieldCoordinate::new(4, 5);
        // Roll sequence: tentacles=2 (escapes), dodge=3 (success, >=3)
        let mut rng = GameRng::new_test([2, 3]);
        let outcome = execute_move_step(&mut state, &pid, &[dest], &mut rng);
        assert_eq!(outcome, MoveOutcome::Success, "player should escape tentacles on 2+");
        assert_eq!(state.field.player_coord(&pid), Some(dest));
    }

    #[test]
    fn tentacles_not_triggered_without_skill() {
        // Same setup but opponent has no Tentacles — only dodge roll needed.
        let start = FieldCoordinate::new(5, 5);
        let pid = PlayerId("mover3".into());
        let apid = PlayerId("opp3".into());
        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        home.add_player(Player::new(
            pid.clone(), "mover3".into(), "lineman".into(), TeamId::Home, 1,
            PlayerStats::new(6, 3, 4, 8, None), SkillSet::empty(),
        ));
        let mut away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        away.add_player(Player::new(
            apid.clone(), "opp3".into(), "lineman".into(), TeamId::Away, 1,
            PlayerStats::new(5, 3, 3, 9, None), SkillSet::empty(), // no Tentacles
        ));
        let mut state = GameState::new(home, away);
        state.field.place_player(pid.clone(), TeamId::Home, start, PlayerState::Standing);
        state.field.place_player(apid, TeamId::Away, FieldCoordinate::new(6, 5), PlayerState::Standing);
        state.acting_player = Some(ActingPlayer::new(pid.clone(), TeamId::Home));
        state.home_is_active = true;

        let dest = FieldCoordinate::new(4, 5);
        // Only dodge roll needed: dodge_min_roll(ag=4, tz=1, no_dodge) = 3; roll=3
        let mut rng = GameRng::new_test([3]);
        let outcome = execute_move_step(&mut state, &pid, &[dest], &mut rng);
        assert_eq!(outcome, MoveOutcome::Success, "no tentacles check without Tentacles skill");
    }

    #[test]
    fn pro_reroll_fails_and_player_goes_down() {
        use crate::skills::SkillId;
        let pid = PlayerId("p1".into());
        let away_pid = PlayerId("a1".into());
        let skills: SkillSet = [SkillId::Pro].into_iter().collect();
        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        let mut away_team = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        home.add_player(Player::new(
            pid.clone(), "Pro".into(), "lineman".into(), TeamId::Home, 1,
            PlayerStats::new(6, 3, 4, 8, None), skills,
        ));
        away_team.add_player(Player::new(
            away_pid.clone(), "Tackler".into(), "lineman".into(), TeamId::Away, 2,
            PlayerStats::new(5, 4, 3, 9, None), SkillSet::empty(),
        ));
        let start = FieldCoordinate::new(5, 5);
        let mut state = GameState::new(home, away_team);
        state.field.place_player(pid.clone(), TeamId::Home, start, PlayerState::Standing);
        state.field.place_player(away_pid.clone(), TeamId::Away, FieldCoordinate::new(6, 5), PlayerState::Standing);
        state.acting_player = Some(ActingPlayer::new(pid.clone(), TeamId::Home));
        state.home_is_active = true;

        let path = vec![FieldCoordinate::new(7, 5)];
        // dodge_min_roll(ag=4, tz=1, no dodge) = 3
        // Roll 1: dodge=1 (fail), Roll 2: Pro check=3 (fail, < 4)
        let mut rng = GameRng::new_test([1, 3]);
        let outcome = execute_move_step(&mut state, &pid, &path, &mut rng);
        assert!(
            matches!(outcome, MoveOutcome::KnockedDown { .. } | MoveOutcome::NeedsTeamReroll { .. }),
            "Pro check failure should cause knockdown or offer team reroll"
        );
    }

    // ── Tackle prevents Dodge skill reroll tests ──────────────────────────────

    #[test]
    fn tackle_prevents_dodge_reroll() {
        use crate::skills::SkillId;
        // Home player at (5,5) with Dodge skill.
        // Away opponent at (6,5) with Tackle skill.
        // Player tries to move to (4,5) — leaving TZ of Tackle opponent.
        // Dodge roll=1 fails. Normally Dodge skill would let them reroll.
        // But Tackle negates Dodge skill reroll → player is knocked down.
        let pid = PlayerId("dodger".into());
        let tackle_pid = PlayerId("tackler".into());

        let dodge_skills: SkillSet = [SkillId::Dodge].into_iter().collect();
        let tackle_skills: SkillSet = [SkillId::Tackle].into_iter().collect();

        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        home.add_player(Player::new(
            pid.clone(), "Dodger".into(), "blitzer".into(), TeamId::Home, 1,
            PlayerStats::new(6, 3, 4, 8, None), dodge_skills,
        ));
        let mut away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        away.add_player(Player::new(
            tackle_pid.clone(), "Tackler".into(), "blitzer".into(), TeamId::Away, 1,
            PlayerStats::new(5, 3, 3, 9, None), tackle_skills,
        ));

        let start = FieldCoordinate::new(5, 5);
        let mut state = GameState::new(home, away);
        state.field.place_player(pid.clone(), TeamId::Home, start, PlayerState::Standing);
        state.field.place_player(tackle_pid.clone(), TeamId::Away, FieldCoordinate::new(6, 5), PlayerState::Standing);
        state.acting_player = Some(ActingPlayer::new(pid.clone(), TeamId::Home));
        state.home_is_active = true;

        let dest = FieldCoordinate::new(4, 5);
        // dodge_min_roll(ag=4, tz=1, has_dodge=true) — with Dodge the min_roll is adjusted.
        // Since opponent has Tackle: dodge skill reroll is negated.
        // No Pro skill either. Roll=1 → knocked down (no reroll possible).
        let mut rng = GameRng::new_test([1]);
        let outcome = execute_move_step(&mut state, &pid, &[dest], &mut rng);
        assert!(
            matches!(outcome, MoveOutcome::KnockedDown { .. } | MoveOutcome::NeedsTeamReroll { .. }),
            "Tackle should prevent Dodge skill reroll, causing knockdown or team reroll offer on roll=1"
        );
    }

    #[test]
    fn dodge_skill_reroll_works_without_tackle() {
        use crate::skills::SkillId;
        // Same as above but opponent does NOT have Tackle.
        // Dodge roll=1 fails, but Dodge skill reroll=5 saves the player.
        let pid = PlayerId("dodger2".into());
        let opp_pid = PlayerId("normal_opp".into());

        let dodge_skills: SkillSet = [SkillId::Dodge].into_iter().collect();

        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        home.add_player(Player::new(
            pid.clone(), "Dodger".into(), "blitzer".into(), TeamId::Home, 1,
            PlayerStats::new(6, 3, 4, 8, None), dodge_skills,
        ));
        let mut away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        away.add_player(Player::new(
            opp_pid.clone(), "Normal Opp".into(), "lineman".into(), TeamId::Away, 1,
            PlayerStats::new(5, 3, 3, 9, None), SkillSet::empty(), // no Tackle
        ));

        let start = FieldCoordinate::new(5, 5);
        let mut state = GameState::new(home, away);
        state.field.place_player(pid.clone(), TeamId::Home, start, PlayerState::Standing);
        state.field.place_player(opp_pid.clone(), TeamId::Away, FieldCoordinate::new(6, 5), PlayerState::Standing);
        state.acting_player = Some(ActingPlayer::new(pid.clone(), TeamId::Home));
        state.home_is_active = true;

        let dest = FieldCoordinate::new(4, 5);
        // dodge_min_roll with Dodge skill and ag=4, tz=1
        // No Tackle → dodge reroll is allowed. Roll=1 (fail), reroll=5 (>=min_roll) → success.
        let mut rng = GameRng::new_test([1, 5]);
        let outcome = execute_move_step(&mut state, &pid, &[dest], &mut rng);
        assert_eq!(
            outcome, MoveOutcome::Success,
            "Without Tackle, Dodge skill reroll should save the player"
        );
    }

    // ── BreakTackle tests ─────────────────────────────────────────────────────

    #[test]
    fn break_tackle_uses_st_instead_of_ag_for_dodge() {
        use crate::skills::SkillId;
        // Player with AG=1, ST=5, BreakTackle.
        // Normal dodge with AG=1 and 1 TZ: base = max(2, 5-1) = 4; min_roll = 4+1 = 5.
        // With BreakTackle: use max(AG=1, ST=5) = 5 → base = max(2, 5-5) = 2; min_roll = 2+1 = 3.
        // Roll=3: without BreakTackle → fail (min=5). With BreakTackle → success (min=3).
        let pid = PlayerId("bt_p".into());
        let opp_id = PlayerId("bt_opp".into());

        let skills: SkillSet = [SkillId::BreakTackle].into_iter().collect();
        let start = FieldCoordinate::new(5, 5);

        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        home.add_player(Player::new(
            pid.clone(), "BT Player".into(), "ogre".into(), TeamId::Home, 1,
            PlayerStats::new(5, 5, 1, 10, None), // AG=1, ST=5
            skills,
        ));
        let mut away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        away.add_player(Player::new(
            opp_id.clone(), "Blocker".into(), "lineman".into(), TeamId::Away, 1,
            PlayerStats::new(5, 3, 3, 9, None), SkillSet::empty(),
        ));
        let mut state = GameState::new(home, away);
        state.field.place_player(pid.clone(), TeamId::Home, start, PlayerState::Standing);
        state.field.place_player(opp_id.clone(), TeamId::Away, FieldCoordinate::new(6, 5), PlayerState::Standing);
        state.acting_player = Some(ActingPlayer::new(pid.clone(), TeamId::Home));
        state.home_is_active = true;

        let dest = FieldCoordinate::new(4, 5);
        // With BreakTackle: effective AG = max(1, 5) = 5; base = max(2, 5-5) = 2; min_roll = 2+1 = 3.
        // Roll=3 → success.
        let mut rng = GameRng::new_test([3]);
        let outcome = execute_move_step(&mut state, &pid, &[dest], &mut rng);
        assert_eq!(
            outcome, MoveOutcome::Success,
            "BreakTackle should use ST=5 instead of AG=1, making roll=3 succeed (min_roll=3)"
        );
    }

    #[test]
    fn without_break_tackle_high_st_player_still_uses_ag() {
        // Same player without BreakTackle: AG=1, 1 TZ → min_roll=5; roll=3 → fail.
        let pid = PlayerId("no_bt_p".into());
        let opp_id = PlayerId("no_bt_opp".into());

        let start = FieldCoordinate::new(5, 5);
        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        home.add_player(Player::new(
            pid.clone(), "No BT Player".into(), "ogre".into(), TeamId::Home, 1,
            PlayerStats::new(5, 5, 1, 10, None), // AG=1, ST=5, NO BreakTackle
            SkillSet::empty(),
        ));
        let mut away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        away.add_player(Player::new(
            opp_id.clone(), "Blocker".into(), "lineman".into(), TeamId::Away, 1,
            PlayerStats::new(5, 3, 3, 9, None), SkillSet::empty(),
        ));
        let mut state = GameState::new(home, away);
        state.field.place_player(pid.clone(), TeamId::Home, start, PlayerState::Standing);
        state.field.place_player(opp_id.clone(), TeamId::Away, FieldCoordinate::new(6, 5), PlayerState::Standing);
        state.acting_player = Some(ActingPlayer::new(pid.clone(), TeamId::Home));
        state.home_is_active = true;

        let dest = FieldCoordinate::new(4, 5);
        // Without BreakTackle: AG=1, 1 TZ → min_roll=5; roll=3 → fail → knocked down.
        let mut rng = GameRng::new_test([3]);
        let outcome = execute_move_step(&mut state, &pid, &[dest], &mut rng);
        assert!(
            matches!(outcome, MoveOutcome::KnockedDown { .. } | MoveOutcome::NeedsTeamReroll { .. }),
            "Without BreakTackle, AG=1 player with roll=3 should fail dodge (min_roll=5)"
        );
    }

    // ── PrehensileTail tests ──────────────────────────────────────────────────

    #[test]
    fn prehensile_tail_adds_extra_tz() {
        use crate::skills::SkillId;
        // Home player at (5,5) with AG=4. Away player at (6,5) with PrehensileTail.
        // Normal: base = max(2, 5-4) = 2; TZ=1 → min_roll = 3.
        // With PrehensileTail: +1 effective TZ → TZ=2 → min_roll = 4.
        // Roll=3: without PrehensileTail → success (min=3). With PrehensileTail → fail (min=4).
        let pid = PlayerId("pt_mover".into());
        let pt_pid = PlayerId("pt_tail".into());

        let pt_skills: SkillSet = [SkillId::PrehensileTail].into_iter().collect();
        let start = FieldCoordinate::new(5, 5);

        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        home.add_player(Player::new(
            pid.clone(), "Mover".into(), "lineman".into(), TeamId::Home, 1,
            PlayerStats::new(6, 3, 4, 8, None), SkillSet::empty(), // AG=4
        ));
        let mut away = Team::new("a".into(), "Away".into(), "Lizard".into(), 3, false);
        away.add_player(Player::new(
            pt_pid.clone(), "Tail".into(), "kroxigor".into(), TeamId::Away, 1,
            PlayerStats::new(6, 5, 1, 9, None), pt_skills,
        ));
        let mut state = GameState::new(home, away);
        state.field.place_player(pid.clone(), TeamId::Home, start, PlayerState::Standing);
        state.field.place_player(pt_pid.clone(), TeamId::Away, FieldCoordinate::new(6, 5), PlayerState::Standing);
        state.acting_player = Some(ActingPlayer::new(pid.clone(), TeamId::Home));
        state.home_is_active = true;

        let dest = FieldCoordinate::new(4, 5);
        // With PrehensileTail: effective TZ = 1 (normal) + 1 (PT) = 2 → min_roll = max(2,5-4)+2 = 1+2=3? Wait:
        // base = max(2, 5-4) = 2, TZ=2 → min_roll = (2+2).min(6) = 4.
        // Roll=3 < 4 → fail → knocked down.
        let mut rng = GameRng::new_test([3]);
        let outcome = execute_move_step(&mut state, &pid, &[dest], &mut rng);
        assert!(
            matches!(outcome, MoveOutcome::KnockedDown { .. } | MoveOutcome::NeedsTeamReroll { .. }),
            "PrehensileTail should add +1 effective TZ, making roll=3 fail (min_roll=4)"
        );
    }

    #[test]
    fn no_prehensile_tail_normal_tz() {
        // Same setup but opponent has no PrehensileTail: min_roll=3; roll=3 → success.
        let pid = PlayerId("nopt_mover".into());
        let opp_pid = PlayerId("nopt_opp".into());

        let start = FieldCoordinate::new(5, 5);
        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        home.add_player(Player::new(
            pid.clone(), "Mover".into(), "lineman".into(), TeamId::Home, 1,
            PlayerStats::new(6, 3, 4, 8, None), SkillSet::empty(),
        ));
        let mut away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        away.add_player(Player::new(
            opp_pid.clone(), "Normal Opp".into(), "lineman".into(), TeamId::Away, 1,
            PlayerStats::new(5, 3, 3, 9, None), SkillSet::empty(), // no PT
        ));
        let mut state = GameState::new(home, away);
        state.field.place_player(pid.clone(), TeamId::Home, start, PlayerState::Standing);
        state.field.place_player(opp_pid.clone(), TeamId::Away, FieldCoordinate::new(6, 5), PlayerState::Standing);
        state.acting_player = Some(ActingPlayer::new(pid.clone(), TeamId::Home));
        state.home_is_active = true;

        let dest = FieldCoordinate::new(4, 5);
        // Normal TZ=1: base=2, min_roll = 2+1 = 3. Roll=3 → success.
        let mut rng = GameRng::new_test([3]);
        let outcome = execute_move_step(&mut state, &pid, &[dest], &mut rng);
        assert_eq!(
            outcome, MoveOutcome::Success,
            "Without PrehensileTail, roll=3 should succeed dodge (min_roll=3)"
        );
    }
}
