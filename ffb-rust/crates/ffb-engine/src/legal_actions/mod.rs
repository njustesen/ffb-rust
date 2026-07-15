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

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerState, PlayerType, PlayerGender, Rules, PS_PRONE, PS_STANDING, PS_STUNNED, PS_KNOCKED_OUT};

    fn make_game(rules: Rules) -> Game {
        Game::new(crate::step::framework::test_team("home", 0), crate::step::framework::test_team("away", 0), rules)
    }

    fn add_player(game: &mut Game, home: bool, id: &str, coord: FieldCoordinate, state_base: u32, skills: Vec<SkillId>) {
        let p = Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: skills.into_iter().map(ffb_model::model::SkillWithValue::new).collect(),
            extra_skills: vec![], temporary_skills: vec![], used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        if home { game.team_home.players.push(p); } else { game.team_away.players.push(p); }
        game.field_model.set_player_coordinate(id, coord);
        game.field_model.set_player_state(id, PlayerState::new(state_base));
    }

    fn c(x: i32, y: i32) -> FieldCoordinate { FieldCoordinate::new(x, y) }

    fn has_action(actions: &[Action], player_id: &str, choice: PlayerActionChoice) -> bool {
        actions.iter().any(|a| matches!(a, Action::ActivatePlayer { player_id: pid, player_action, .. }
            if pid == player_id && *player_action == choice))
    }

    // ── legal_activate_player_actions ─────────────────────────────────────────

    #[test]
    fn already_acted_player_excluded() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        game.turn_data_home.acted_player_ids.push("p1".into());
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(actions.is_empty());
    }

    #[test]
    fn off_pitch_player_excluded() {
        let mut game = make_game(Rules::Bb2025);
        game.team_home.players.push(Player {
            id: "p1".into(), name: "p1".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None, is_big_guy: false,
            ..Default::default()
        });
        // Never placed on the field: no coordinate/state set.
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(actions.is_empty());
    }

    #[test]
    fn stunned_player_excluded() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STUNNED, vec![]);
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(actions.is_empty());
    }

    #[test]
    fn knocked_out_player_excluded() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_KNOCKED_OUT, vec![]);
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(actions.is_empty());
    }

    #[test]
    fn standing_player_offered_move() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(has_action(&actions, "p1", PlayerActionChoice::Move));
    }

    #[test]
    fn prone_player_offered_only_stand_up_without_adjacent_opponent() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_PRONE, vec![]);
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(has_action(&actions, "p1", PlayerActionChoice::StandUp));
        assert!(!has_action(&actions, "p1", PlayerActionChoice::Blitz));
        assert!(!has_action(&actions, "p1", PlayerActionChoice::Move));
    }

    #[test]
    fn prone_player_offered_blitz_with_adjacent_standing_opponent() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_PRONE, vec![]);
        add_player(&mut game, false, "op1", c(6, 5), PS_STANDING, vec![]);
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(has_action(&actions, "p1", PlayerActionChoice::Blitz));
    }

    #[test]
    fn prone_player_blitz_omitted_when_blitz_used() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_PRONE, vec![]);
        add_player(&mut game, false, "op1", c(6, 5), PS_STANDING, vec![]);
        game.turn_data_home.blitz_used = true;
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(!has_action(&actions, "p1", PlayerActionChoice::Blitz));
    }

    #[test]
    fn block_and_blitz_offered_with_adjacent_standing_opponent() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        add_player(&mut game, false, "op1", c(6, 5), PS_STANDING, vec![]);
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(has_action(&actions, "p1", PlayerActionChoice::Block));
        assert!(has_action(&actions, "p1", PlayerActionChoice::Blitz));
    }

    #[test]
    fn block_and_blitz_omitted_without_adjacent_opponent() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        add_player(&mut game, false, "op1", c(20, 5), PS_STANDING, vec![]);
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(!has_action(&actions, "p1", PlayerActionChoice::Block));
        assert!(!has_action(&actions, "p1", PlayerActionChoice::Blitz));
    }

    #[test]
    fn block_omitted_against_prone_adjacent_opponent() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        add_player(&mut game, false, "op1", c(6, 5), PS_PRONE, vec![]);
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(!has_action(&actions, "p1", PlayerActionChoice::Block));
    }

    #[test]
    fn pass_offered_only_to_ball_carrier() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        game.field_model.ball_coordinate = Some(c(5, 5));
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(has_action(&actions, "p1", PlayerActionChoice::Pass));
    }

    #[test]
    fn pass_omitted_without_ball() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(!has_action(&actions, "p1", PlayerActionChoice::Pass));
    }

    #[test]
    fn pass_omitted_with_no_ball_skill() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![SkillId::NoBall]);
        game.field_model.ball_coordinate = Some(c(5, 5));
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(!has_action(&actions, "p1", PlayerActionChoice::Pass));
    }

    #[test]
    fn hand_off_offered_with_ball_and_adjacent_teammate() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        add_player(&mut game, true, "p2", c(6, 5), PS_STANDING, vec![]);
        game.field_model.ball_coordinate = Some(c(5, 5));
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(has_action(&actions, "p1", PlayerActionChoice::HandOff));
    }

    #[test]
    fn hand_off_omitted_without_adjacent_teammate() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        game.field_model.ball_coordinate = Some(c(5, 5));
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(!has_action(&actions, "p1", PlayerActionChoice::HandOff));
    }

    #[test]
    fn foul_offered_against_adjacent_prone_opponent() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        add_player(&mut game, false, "op1", c(6, 5), PS_PRONE, vec![]);
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(has_action(&actions, "p1", PlayerActionChoice::Foul));
    }

    #[test]
    fn foul_omitted_against_standing_opponent() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        add_player(&mut game, false, "op1", c(6, 5), PS_STANDING, vec![]);
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(!has_action(&actions, "p1", PlayerActionChoice::Foul));
    }

    #[test]
    fn throw_bomb_offered_for_bombardier_skill() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![SkillId::Bombardier]);
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(has_action(&actions, "p1", PlayerActionChoice::ThrowBomb));
    }

    #[test]
    fn throw_team_mate_offered_with_skill_and_adjacent_teammate() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![SkillId::ThrowTeamMate]);
        add_player(&mut game, true, "p2", c(6, 5), PS_STANDING, vec![]);
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(has_action(&actions, "p1", PlayerActionChoice::ThrowTeamMate));
    }

    #[test]
    fn kick_team_mate_only_offered_in_bb2025() {
        let mut game16 = make_game(Rules::Bb2016);
        add_player(&mut game16, true, "p1", c(5, 5), PS_STANDING, vec![SkillId::KickTeamMate]);
        add_player(&mut game16, true, "p2", c(6, 5), PS_STANDING, vec![]);
        let actions16 = legal_activate_player_actions(&game16, TeamSide::Home);
        assert!(!has_action(&actions16, "p1", PlayerActionChoice::KickTeamMate));

        let mut game25 = make_game(Rules::Bb2025);
        add_player(&mut game25, true, "p1", c(5, 5), PS_STANDING, vec![SkillId::KickTeamMate]);
        add_player(&mut game25, true, "p2", c(6, 5), PS_STANDING, vec![]);
        let actions25 = legal_activate_player_actions(&game25, TeamSide::Home);
        assert!(has_action(&actions25, "p1", PlayerActionChoice::KickTeamMate));
    }

    #[test]
    fn punt_only_offered_in_bb2025_with_ball_in_play() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![SkillId::Punt]);
        game.field_model.ball_coordinate = Some(c(5, 5));
        game.field_model.ball_in_play = true;
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(has_action(&actions, "p1", PlayerActionChoice::Punt));
    }

    #[test]
    fn secure_the_ball_offered_when_ball_moving_at_coord() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        game.field_model.ball_coordinate = Some(c(5, 5));
        game.field_model.ball_in_play = true;
        game.field_model.ball_moving = true;
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(has_action(&actions, "p1", PlayerActionChoice::SecureTheBall));
    }

    #[test]
    fn secure_the_ball_omitted_for_unsteady_player() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![SkillId::Unsteady]);
        game.field_model.ball_coordinate = Some(c(5, 5));
        game.field_model.ball_in_play = true;
        game.field_model.ball_moving = true;
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(!has_action(&actions, "p1", PlayerActionChoice::SecureTheBall));
    }

    // ── legal_move_targets / legal_blitz_move_targets ─────────────────────────

    #[test]
    fn legal_move_targets_returns_adjacent_empty_squares() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        let targets = legal_move_targets(&game, "p1");
        assert!(targets.contains(&c(6, 5)));
        assert!(!targets.is_empty());
    }

    #[test]
    fn legal_move_targets_excludes_occupied_squares() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        add_player(&mut game, false, "op1", c(6, 5), PS_STANDING, vec![]);
        let targets = legal_move_targets(&game, "p1");
        assert!(!targets.contains(&c(6, 5)));
    }

    #[test]
    fn legal_move_targets_empty_when_ma_and_gfi_exhausted() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        game.acting_player.current_move = 8; // movement 6 + STANDARD_GFI_SQUARES 2
        let targets = legal_move_targets(&game, "p1");
        assert!(targets.is_empty());
    }

    #[test]
    fn legal_move_targets_unknown_player_returns_empty() {
        let game = make_game(Rules::Bb2025);
        let targets = legal_move_targets(&game, "ghost");
        assert!(targets.is_empty());
    }

    #[test]
    fn legal_blitz_move_targets_includes_zero_move_when_already_adjacent() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        add_player(&mut game, false, "op1", c(6, 5), PS_STANDING, vec![]);
        let targets = legal_blitz_move_targets(&game, "p1", "op1");
        assert!(targets.contains(&c(5, 5)));
    }

    #[test]
    fn legal_blitz_move_targets_reachable_and_adjacent_to_defender() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        add_player(&mut game, false, "op1", c(8, 5), PS_STANDING, vec![]);
        let targets = legal_blitz_move_targets(&game, "p1", "op1");
        assert!(targets.iter().all(|t| t.is_adjacent(c(8, 5))));
        assert!(!targets.is_empty());
    }

    // ── bfs_path ───────────────────────────────────────────────────────────────

    #[test]
    fn bfs_path_same_square_returns_empty() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        assert!(bfs_path(&game, "p1", c(5, 5), c(5, 5)).is_empty());
    }

    #[test]
    fn bfs_path_unreachable_returns_empty() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        // Fully surround p1's destination-adjacent square isn't necessary; use an
        // out-of-pitch destination to force unreachability.
        let dest = c(-1, -1);
        assert!(bfs_path(&game, "p1", c(5, 5), dest).is_empty());
    }

    #[test]
    fn bfs_path_finds_short_path() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        let path = bfs_path(&game, "p1", c(5, 5), c(7, 5));
        assert_eq!(path.last(), Some(&c(7, 5)));
        assert!(!path.is_empty());
    }

    // ── legal_block_targets / legal_foul_targets / legal_handoff_receivers / legal_pass_receivers ──

    #[test]
    fn legal_block_targets_returns_adjacent_blockable_opponents_sorted() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        add_player(&mut game, false, "op_far", c(6, 4), PS_STANDING, vec![]);
        add_player(&mut game, false, "op_near", c(4, 5), PS_STANDING, vec![]);
        let targets = legal_block_targets(&game, "p1", TeamSide::Home);
        assert_eq!(targets, vec!["op_near".to_string(), "op_far".to_string()]);
    }

    #[test]
    fn legal_block_targets_excludes_prone_opponent() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        add_player(&mut game, false, "op1", c(6, 5), PS_PRONE, vec![]);
        let targets = legal_block_targets(&game, "p1", TeamSide::Home);
        assert!(targets.is_empty());
    }

    #[test]
    fn legal_foul_targets_returns_adjacent_prone_or_stunned_opponents() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        add_player(&mut game, false, "op1", c(6, 5), PS_STUNNED, vec![]);
        let targets = legal_foul_targets(&game, "p1", TeamSide::Home);
        assert_eq!(targets, vec!["op1".to_string()]);
    }

    #[test]
    fn legal_handoff_receivers_returns_adjacent_teammates_sorted() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        add_player(&mut game, true, "p2", c(6, 5), PS_STANDING, vec![]);
        add_player(&mut game, true, "p3", c(4, 5), PS_STANDING, vec![]);
        let targets = legal_handoff_receivers(&game, "p1", TeamSide::Home);
        assert_eq!(targets, vec!["p3".to_string(), "p2".to_string()]);
    }

    #[test]
    fn legal_pass_receivers_excludes_self_includes_all_onfield_teammates() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        add_player(&mut game, true, "p2", c(20, 10), PS_STANDING, vec![]);
        let targets = legal_pass_receivers(&game, "p1", TeamSide::Home);
        assert_eq!(targets, vec!["p2".to_string()]);
    }

    // ── legal_kickoff_targets ──────────────────────────────────────────────────

    #[test]
    fn legal_kickoff_targets_home_kicks_to_away_half() {
        let game = make_game(Rules::Bb2025);
        let targets = legal_kickoff_targets(&game, TeamSide::Home);
        assert!(targets.iter().all(|t| t.x >= 13 && t.x <= 25));
        assert!(!targets.is_empty());
    }

    #[test]
    fn legal_kickoff_targets_away_kicks_to_home_half() {
        let game = make_game(Rules::Bb2025);
        let targets = legal_kickoff_targets(&game, TeamSide::Away);
        assert!(targets.iter().all(|t| t.x >= 0 && t.x <= 12));
        assert!(!targets.is_empty());
    }
}

// Tests: legal_actions takes `&Game`, so its tests are rebuilt as `&Game` fixtures
// (no engine dependency) when the first selection/action step consumes it in Phase D.
