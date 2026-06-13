use ffb_model::model::game::Game;
use ffb_model::model::player::PlayerId;
use ffb_model::types::FieldCoordinate;
use ffb_mechanics::skills::SkillId;
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

    let ball_coord = game.field_model.ball_coordinate;

    let mut actions = Vec::new();

    for player in &team.players {
        let pid = player.id.clone();
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

        // Prone players can stand up (StandUp / StandUpBlitz)
        if is_prone {
            actions.push(Action::ActivatePlayer {
                player_id: pid.clone(),
                player_action: PlayerActionChoice::StandUp,
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
                        player_action: PlayerActionChoice::StandUpBlitz,
                    });
                }
            }
            continue;
        }

        // Standing player actions
        actions.push(Action::ActivatePlayer {
            player_id: pid.clone(),
            player_action: PlayerActionChoice::Move,
        });

        // Block / Blitz: require adjacent standing opponent; Block + Blitz both use blitz slot
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
                });
                actions.push(Action::ActivatePlayer {
                    player_id: pid.clone(),
                    player_action: PlayerActionChoice::Blitz,
                });
            } else {
                actions.push(Action::ActivatePlayer {
                    player_id: pid.clone(),
                    player_action: PlayerActionChoice::Blitz,
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
                });
            }
        }

        // ThrowBomb: Bombardier skill, uses pass-action slot
        if !turn_data.pass_used && player.has_skill(SkillId::Bombardier) {
            actions.push(Action::ActivatePlayer {
                player_id: pid.clone(),
                player_action: PlayerActionChoice::ThrowBomb,
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
            });
        }
    }

    actions
}

/// Returns all squares the active player can legally move to in one step.
///
/// Simplified: returns all valid, empty squares adjacent to current position.
pub fn legal_move_targets(game: &Game, player_id: &str) -> Vec<FieldCoordinate> {
    let coord = match game.field_model.player_coordinate(player_id) {
        Some(c) => c,
        None => return vec![],
    };
    coord.neighbours()
        .into_iter()
        .filter(|c| c.is_on_pitch() && game.field_model.player_at(*c).is_none())
        .collect()
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

    opponent_team.players.iter()
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
        .collect()
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

    opponent_team.players.iter()
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
        .collect()
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
