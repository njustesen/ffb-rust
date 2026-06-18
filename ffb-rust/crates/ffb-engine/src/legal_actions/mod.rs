use std::collections::{HashSet, VecDeque};
use ffb_model::model::game::Game;
use ffb_model::model::player::PlayerId;
use ffb_model::types::FieldCoordinate;
use ffb_mechanics::skills::SkillId;
use ffb_mechanics::mechanics::STANDARD_GFI_SQUARES;
use ffb_model::enums::{Rules, PS_PRONE};
use crate::action::{Action, PlayerActionChoice};

/// The side (home or away) in the current game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeamSide {
    Home,
    Away,
}

/// Returns all legal `ActivatePlayer` actions for the given side on their turn.
///
/// This lists which players can be activated and with which action type.
pub fn legal_activate_player_actions(game: &Game, side: TeamSide) -> Vec<Action> {
    let team = match side {
        TeamSide::Home => &game.team_home,
        TeamSide::Away => &game.team_away,
    };
    let opponent_team = match side {
        TeamSide::Home => &game.team_away,
        TeamSide::Away => &game.team_home,
    };
    let turn_data = match side {
        TeamSide::Home => &game.turn_data_home,
        TeamSide::Away => &game.turn_data_away,
    };

    let acted = match side {
        TeamSide::Home => &game.turn_data_home.acted_player_ids,
        TeamSide::Away => &game.turn_data_away.acted_player_ids,
    };
    let ball_coord = game.field_model.ball_coordinate;

    let mut actions = Vec::new();

    for player in &team.players {
        let pid = player.id.clone();
        if acted.contains(&pid) { continue; } // already activated this turn

        let coord = game.field_model.player_coordinate(&pid);
        let state = game.field_model.player_state(&pid);

        let coord = match coord {
            Some(c) => c,
            None => continue, // off-pitch
        };
        let state = match state {
            Some(s) => s,
            None => continue,
        };

        let is_standing = state.is_standing();
        let is_prone = state.base() == PS_PRONE;

        if !is_standing && !is_prone {
            continue; // stunned / KO / dead
        }

        // Prone players can stand up (StandUp) or blitz without pre-block move (Blitz).
        // Java computeEligiblePlayers offers MOVE + BLITZ for prone players — not STAND_UP_BLITZ.
        if is_prone {
            actions.push(Action::ActivatePlayer {
                player_id: pid.clone(),
                player_action: PlayerActionChoice::StandUp,
                block_defender_id: None,
            });
            if !turn_data.blitz_used {
                let adj_opponent = opponent_team.players.iter().any(|op| {
                    game.field_model.player_coordinate(&op.id)
                        .map(|oc| oc.is_adjacent(coord))
                        .unwrap_or(false)
                        && game.field_model.player_state(&op.id)
                            .map(|os| os.has_tacklezones())
                            .unwrap_or(false)
                });
                if adj_opponent {
                    actions.push(Action::ActivatePlayer {
                        player_id: pid.clone(),
                        player_action: PlayerActionChoice::Blitz,
                        block_defender_id: None,
                    });
                }
            }
            continue;
        }

        // Standing player actions
        actions.push(Action::ActivatePlayer {
            player_id: pid.clone(),
            player_action: PlayerActionChoice::Move,
            block_defender_id: None,
        });

        // Block / Blitz: require adjacent standing opponent; Block + Blitz both use blitz slot.
        // Mirrors Java computeEligiblePlayers: only offered when hasAdjacentBlockTarget().
        // No standalone move-into-contact Blitz — matches Java parity contract exactly.
        if !turn_data.blitz_used {
            let adj_standing_opponent = opponent_team.players.iter().any(|op| {
                game.field_model.player_coordinate(&op.id)
                    .map(|oc| oc.is_adjacent(coord))
                    .unwrap_or(false)
                    && game.field_model.player_state(&op.id)
                        .map(|os| os.has_tacklezones())
                        .unwrap_or(false)
            });
            if adj_standing_opponent {
                actions.push(Action::ActivatePlayer {
                    player_id: pid.clone(),
                    player_action: PlayerActionChoice::Block,
                    block_defender_id: None,
                });
                actions.push(Action::ActivatePlayer {
                    player_id: pid.clone(),
                    player_action: PlayerActionChoice::Blitz,
                    block_defender_id: None,
                });
            }
        }

        // Pass: only ball-carrier can pass; NoBall prevents it
        let has_no_ball = player.has_skill(SkillId::NoBall);
        let has_my_ball = player.has_skill(SkillId::MyBall);
        if !turn_data.pass_used && !has_my_ball && !has_no_ball && ball_coord == Some(coord) {
            actions.push(Action::ActivatePlayer {
                player_id: pid.clone(),
                player_action: PlayerActionChoice::Pass,
                block_defender_id: None,
            });
        }

        // HandOff: ball-carrier can hand off to an adjacent teammate
        if !turn_data.hand_over_used && ball_coord == Some(coord) {
            let teammate_adjacent = team.players.iter().any(|tp| {
                tp.id != pid
                    && game.field_model.player_coordinate(&tp.id)
                        .map(|tc| tc.is_adjacent(coord))
                        .unwrap_or(false)
            });
            if teammate_adjacent {
                actions.push(Action::ActivatePlayer {
                    player_id: pid.clone(),
                    player_action: PlayerActionChoice::HandOff,
                    block_defender_id: None,
                });
            }
        }

        // Foul: requires adjacent prone/stunned opponent
        if !turn_data.foul_used {
            let foul_target_exists = opponent_team.players.iter().any(|op| {
                game.field_model.player_coordinate(&op.id)
                    .map(|oc| oc.is_adjacent(coord))
                    .unwrap_or(false)
                    && game.field_model.player_state(&op.id)
                        .map(|os| !os.has_tacklezones() && (os.base() == PS_PRONE || os.base() == ffb_model::enums::PS_STUNNED))
                        .unwrap_or(false)
            });
            if foul_target_exists {
                actions.push(Action::ActivatePlayer {
                    player_id: pid.clone(),
                    player_action: PlayerActionChoice::Foul,
                    block_defender_id: None,
                });
            }
        }

        // ThrowBomb: Bombardier skill, uses pass-action slot
        if !turn_data.pass_used && player.has_skill(SkillId::Bombardier) {
            actions.push(Action::ActivatePlayer {
                player_id: pid.clone(),
                player_action: PlayerActionChoice::ThrowBomb,
                block_defender_id: None,
            });
        }

        // ThrowTeamMate: player must have ThrowTeamMate skill and an adjacent teammate
        if !turn_data.ttm_used && player.has_skill(SkillId::ThrowTeamMate) {
            let adjacent_teammate = team.players.iter().any(|tp| {
                tp.id != pid
                    && game.field_model.player_coordinate(&tp.id)
                        .map(|tc| tc.is_adjacent(coord))
                        .unwrap_or(false)
            });
            if adjacent_teammate {
                actions.push(Action::ActivatePlayer {
                    player_id: pid.clone(),
                    player_action: PlayerActionChoice::ThrowTeamMate,
                    block_defender_id: None,
                });
            }
        }

        // KickTeamMate (BB2025): player must have KickTeamMate skill and an adjacent teammate
        if game.rules == Rules::Bb2025 && !turn_data.ktm_used && player.has_skill(SkillId::KickTeamMate) {
            let adjacent_teammate = team.players.iter().any(|tp| {
                tp.id != pid
                    && game.field_model.player_coordinate(&tp.id)
                        .map(|tc| tc.is_adjacent(coord))
                        .unwrap_or(false)
            });
            if adjacent_teammate {
                actions.push(Action::ActivatePlayer {
                    player_id: pid.clone(),
                    player_action: PlayerActionChoice::KickTeamMate,
                    block_defender_id: None,
                });
            }
        }

        // Punt (BB2025): ball-carrier with Punt skill; once per drive
        if game.rules == Rules::Bb2025
            && !turn_data.punt_used
            && player.has_skill(SkillId::Punt)
            && game.field_model.ball_in_play
            && ball_coord == Some(coord)
        {
            actions.push(Action::ActivatePlayer {
                player_id: pid.clone(),
                player_action: PlayerActionChoice::Punt,
                block_defender_id: None,
            });
        }

        // SecureTheBall (BB2025): player at ball square can secure with 2+ roll; once per turn.
        if game.rules == Rules::Bb2025
            && game.field_model.ball_in_play
            && game.field_model.ball_moving
            && ball_coord == Some(coord)
            && !turn_data.secure_the_ball_used
            && !player.has_skill(SkillId::Unsteady)
        {
            actions.push(Action::ActivatePlayer {
                player_id: pid.clone(),
                player_action: PlayerActionChoice::SecureTheBall,
                block_defender_id: None,
            });
        }
    }

    actions
}

/// Returns all squares the player can legally move to (BFS, within MA moves, empty squares only).
/// Excludes the player's current square. Sorted by (x, y).
/// Returns the 8 adjacent squares of the player that are on-pitch and unoccupied,
/// sorted by (x, y). Mirrors Java ParityRunner.sendMoveAction: 1-step neighbours only.
/// Returns empty when the player has exhausted MA + GFI allowance (mirrors Java StepGoForIt cap).
pub fn legal_move_targets(game: &Game, player_id: &str) -> Vec<FieldCoordinate> {
    let start = match game.field_model.player_coordinate(player_id) {
        Some(c) => c,
        None => return vec![],
    };
    // Cap: no move offered once current_move has reached MA + GFI (Java: max MA + 2 rush squares).
    let ma = find_player_ma(game, player_id);
    let cur = game.acting_player.current_move;
    if cur >= ma + STANDARD_GFI_SQUARES {
        return vec![];
    }
    let mut targets: Vec<FieldCoordinate> = start.neighbours().into_iter()
        .filter(|n| n.is_on_pitch() && game.field_model.player_at(*n).is_none())
        .collect();
    targets.sort_by_key(|c| (c.x, c.y));
    targets
}

/// Returns all squares the blitzing player can legally move to for a BLITZ action:
/// empty squares within MA moves that are adjacent to `defender_id`, plus the player's
/// current square if it is already adjacent to the defender (0-move BLITZ).
/// Sorted by (x, y).
pub fn legal_blitz_move_targets(game: &Game, player_id: &str, defender_id: &str) -> Vec<FieldCoordinate> {
    let start = match game.field_model.player_coordinate(player_id) {
        Some(c) => c,
        None => return vec![],
    };
    let def_coord = match game.field_model.player_coordinate(defender_id) {
        Some(c) => c,
        None => return vec![],
    };
    let ma = find_player_ma(game, player_id);
    let reachable = bfs_reachable_empty(game, player_id, start, ma);

    let mut targets: Vec<FieldCoordinate> = reachable.into_iter()
        .filter(|c| c.is_adjacent(def_coord))
        .collect();

    // Include current position if already adjacent to defender (0-move BLITZ)
    if start.is_adjacent(def_coord) && !targets.contains(&start) {
        targets.push(start);
    }

    targets.sort_by_key(|c| (c.x, c.y));
    targets
}

/// BFS from `start` over empty squares (excluding `player_id`'s own square if it would
/// block a revisit). Returns up to `ma` moves deep, sorted by (x, y).
fn bfs_reachable_empty(game: &Game, player_id: &str, start: FieldCoordinate, ma: i32) -> Vec<FieldCoordinate> {
    let mut visited: HashSet<FieldCoordinate> = HashSet::new();
    let mut queue: VecDeque<(FieldCoordinate, i32)> = VecDeque::new();
    let mut result: Vec<FieldCoordinate> = Vec::new();

    visited.insert(start);
    queue.push_back((start, 0));

    while let Some((coord, dist)) = queue.pop_front() {
        for n in coord.neighbours() {
            if !n.is_on_pitch() { continue; }
            if visited.contains(&n) { continue; }
            // Occupied by any player (including the mover's own square is already visited)
            if let Some(occ) = game.field_model.player_at(n) {
                if occ != player_id {
                    visited.insert(n); // blocked, don't revisit
                    continue;
                }
            }
            visited.insert(n);
            result.push(n);
            if dist + 1 < ma {
                queue.push_back((n, dist + 1));
            }
        }
    }

    result.sort_by_key(|c| (c.x, c.y));
    result
}

/// Compute the shortest path from `src` to `dest` for `player_id` using BFS.
/// Returns each square visited, excluding `src`, including `dest`.
/// Returns an empty vec if `dest` is unreachable.
pub fn bfs_path(game: &Game, player_id: &str, src: FieldCoordinate, dest: FieldCoordinate) -> Vec<FieldCoordinate> {
    if src == dest { return vec![]; }
    let mut visited: HashSet<FieldCoordinate> = HashSet::new();
    let mut queue: VecDeque<(FieldCoordinate, Vec<FieldCoordinate>)> = VecDeque::new();
    visited.insert(src);
    queue.push_back((src, vec![]));
    while let Some((coord, path)) = queue.pop_front() {
        // Sort neighbours by (x, y) ascending so BFS path ties break the same way as Java's
        // smallest-coordinate-first traversal — without this, neighbour enum order gives a
        // different path than Java (SW before W), shifting dodge-roll positions.
        let mut ns: Vec<FieldCoordinate> = coord.neighbours().into_iter()
            .filter(|n| n.is_on_pitch() && !visited.contains(n))
            .collect();
        ns.sort_by_key(|n| (n.x, n.y));
        for n in ns {
            let occupied = game.field_model.player_at(n).map(|occ| occ != player_id).unwrap_or(false);
            if occupied && n != dest { visited.insert(n); continue; }
            visited.insert(n);
            let mut new_path = path.clone();
            new_path.push(n);
            if n == dest { return new_path; }
            queue.push_back((n, new_path));
        }
    }
    vec![]
}

fn find_player_ma(game: &Game, player_id: &str) -> i32 {
    game.team_home.players.iter()
        .chain(game.team_away.players.iter())
        .find(|p| p.id == player_id)
        .map(|p| p.movement_with_modifiers())
        .unwrap_or(6)
}

/// Returns all opponent players adjacent to the given player (valid block targets).
pub fn legal_block_targets(game: &Game, player_id: &str, side: TeamSide) -> Vec<PlayerId> {
    let coord = match game.field_model.player_coordinate(player_id) {
        Some(c) => c,
        None => return vec![],
    };

    let opponent_team = match side {
        TeamSide::Home => &game.team_away,
        TeamSide::Away => &game.team_home,
    };

    let mut targets: Vec<PlayerId> = opponent_team.players.iter()
        .filter(|p| {
            game.field_model.player_coordinate(&p.id)
                .map(|c| c.is_adjacent(coord))
                .unwrap_or(false)
        })
        .filter(|p| {
            game.field_model.player_state(&p.id)
                .map(|s| s.can_be_blocked())
                .unwrap_or(false)
        })
        .map(|p| p.id.clone())
        .collect();
    // Java pickBlockTarget sorts by (x, y) before picking — match that order.
    targets.sort_by_key(|id| {
        game.field_model.player_coordinate(id)
            .map(|c| (c.x, c.y))
            .unwrap_or((i32::MAX, i32::MAX))
    });
    targets
}

/// Returns all adjacent friendly players (valid hand-off receivers), sorted by (x, y).
/// Mirrors Java ParityRunner.sendHandOverAction which sorts players by coordinate.
pub fn legal_handoff_receivers(game: &Game, player_id: &str, side: TeamSide) -> Vec<PlayerId> {
    let coord = match game.field_model.player_coordinate(player_id) {
        Some(c) => c,
        None => return vec![],
    };
    let team = match side {
        TeamSide::Home => &game.team_home,
        TeamSide::Away => &game.team_away,
    };
    let mut targets: Vec<PlayerId> = team.players.iter()
        .filter(|p| p.id != player_id)
        .filter(|p| {
            game.field_model.player_coordinate(&p.id)
                .map(|c| c.is_adjacent(coord))
                .unwrap_or(false)
        })
        .map(|p| p.id.clone())
        .collect();
    targets.sort_by_key(|id| {
        game.field_model.player_coordinate(id)
            .map(|c| (c.x, c.y))
            .unwrap_or((i32::MAX, i32::MAX))
    });
    targets
}

/// Returns all on-pitch teammates for the pass-target list (Java ParityRunner.sendPassAction).
/// Matches Java: all teammates with a valid on-field coordinate, sorted by (x, y), 1 actionRng pick.
pub fn legal_pass_receivers(game: &Game, player_id: &str, side: TeamSide) -> Vec<PlayerId> {
    let team = match side {
        TeamSide::Home => &game.team_home,
        TeamSide::Away => &game.team_away,
    };
    let mut targets: Vec<PlayerId> = team.players.iter()
        .filter(|p| p.id != player_id)
        .filter(|p| {
            game.field_model.player_coordinate(&p.id)
                .map(|c| c.x >= 0 && c.x <= 25 && c.y >= 0 && c.y <= 14)
                .unwrap_or(false)
        })
        .map(|p| p.id.clone())
        .collect();
    targets.sort_by_key(|id| {
        game.field_model.player_coordinate(id)
            .map(|c| (c.x, c.y))
            .unwrap_or((i32::MAX, i32::MAX))
    });
    targets
}

/// Returns all prone/stunned opponent players adjacent to the given player (valid foul targets).
pub fn legal_foul_targets(game: &Game, player_id: &str, side: TeamSide) -> Vec<PlayerId> {
    let coord = match game.field_model.player_coordinate(player_id) {
        Some(c) => c,
        None => return vec![],
    };

    let opponent_team = match side {
        TeamSide::Home => &game.team_away,
        TeamSide::Away => &game.team_home,
    };

    let mut targets: Vec<PlayerId> = opponent_team.players.iter()
        .filter(|p| {
            game.field_model.player_coordinate(&p.id)
                .map(|c| c.is_adjacent(coord))
                .unwrap_or(false)
        })
        .filter(|p| {
            game.field_model.player_state(&p.id)
                .map(|s| s.can_be_fouled())
                .unwrap_or(false)
        })
        .map(|p| p.id.clone())
        .collect();
    // Java sortPlayersByCoordinate: sort by (x, y)
    targets.sort_by_key(|id| {
        game.field_model.player_coordinate(id)
            .map(|c| (c.x, c.y))
            .unwrap_or((i32::MAX, i32::MAX))
    });
    targets
}

/// Returns legal kickoff target coordinates (the opponent's half of the field).
///
/// The kicking team can kick to any square in the opponent's half.
pub fn legal_kickoff_targets(_game: &Game, side: TeamSide) -> Vec<FieldCoordinate> {
    // The opponent half: if home is kicking (home_playing = false), the away half is x in 0..=12
    // Field is 26 columns wide, home half is columns 0-12, away half 13-25
    let mut targets = Vec::new();
    let (x_start, x_end) = match side {
        TeamSide::Home => (13i32, 25i32), // home kicks to away half
        TeamSide::Away => (0i32, 12i32),  // away kicks to home half
    };
    for x in x_start..=x_end {
        for y in 1i32..=14i32 {
            let coord = FieldCoordinate::new(x, y);
            if coord.is_on_pitch() {
                targets.push(coord);
            }
        }
    }
    targets
}

// Tests: legal_actions takes `&Game`, so its tests are rebuilt as `&Game` fixtures
// (no engine dependency) when the first selection/action step consumes it in Phase D.
