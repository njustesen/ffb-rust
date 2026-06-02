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

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::team::Team;
    use ffb_model::model::player::Player;
    use ffb_model::enums::{Rules, PlayerState as PS, PlayerGender, PlayerType, PS_STANDING, PS_PRONE};
    use ffb_model::types::FieldCoordinate;
    use crate::engine::GameEngine;

    fn make_player(id: &str) -> Player {
        Player {
            id: id.into(),
            name: id.into(),
            nr: 1,
            position_id: String::new(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Neutral,
            movement: 6,
            strength: 3,
            agility: 3,
            passing: 4,
            armour: 8,
            starting_skills: vec![],
            extra_skills: vec![],
            temporary_skills: vec![],
            used_skills: std::collections::HashSet::new(),
            niggling_injuries: 0,
            stat_injuries: vec![],
            current_spps: 0,
            career_spps: 0,
            race: None,
        }
    }

    fn make_team(id: &str) -> Team {
        Team {
            id: id.into(),
            name: id.into(),
            race: String::new(),
            roster_id: String::new(),
            coach: String::new(),
            rerolls: 2,
            apothecaries: 0,
            bribes: 0,
            master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            fan_factor: 3,
            assistant_coaches: 0,
            cheerleaders: 0,
            dedicated_fans: 3,
            treasury: 0,
            team_value: 0,
            players: vec![],
            special_rules: vec![],
        }
    }

    fn make_engine_with_teams(home: Team, away: Team) -> GameEngine {
        GameEngine::new(home, away, Rules::Bb2020, 42)
    }

    #[test]
    fn legal_activate_standing_player_includes_move() {
        let mut home = make_team("home");
        home.players.push(make_player("p1"));
        let mut engine = make_engine_with_teams(home, make_team("away"));
        engine.game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 7));
        engine.game.field_model.set_player_state("p1", PS::new(PS_STANDING));

        let actions = legal_activate_player_actions(&engine.game, TeamSide::Home);
        assert!(
            actions.iter().any(|a| matches!(a, crate::action::Action::ActivatePlayer { player_id, player_action }
                if player_id == "p1" && *player_action == PlayerActionChoice::Move)),
            "standing player must have Move action available"
        );
    }

    #[test]
    fn legal_activate_player_excludes_off_pitch_players() {
        let mut home = make_team("home");
        home.players.push(make_player("reserve"));
        let engine = make_engine_with_teams(home, make_team("away"));
        // No coordinate set → off pitch → should not appear in activation list

        let actions = legal_activate_player_actions(&engine.game, TeamSide::Home);
        assert!(
            !actions.iter().any(|a| matches!(a, crate::action::Action::ActivatePlayer { player_id, .. }
                if player_id == "reserve")),
            "off-pitch players must not be activatable"
        );
    }

    #[test]
    fn legal_activate_blitz_excluded_when_already_used() {
        let mut home = make_team("home");
        home.players.push(make_player("blocker"));
        let mut engine = make_engine_with_teams(home, make_team("away"));
        engine.game.field_model.set_player_coordinate("blocker", FieldCoordinate::new(5, 7));
        engine.game.field_model.set_player_state("blocker", PS::new(PS_STANDING));
        engine.game.turn_data_home.blitz_used = true;

        let actions = legal_activate_player_actions(&engine.game, TeamSide::Home);
        assert!(
            !actions.iter().any(|a| matches!(a, crate::action::Action::ActivatePlayer { player_id, player_action }
                if player_id == "blocker" && *player_action == PlayerActionChoice::Blitz)),
            "Blitz must not be available when blitz_used is true"
        );
    }

    #[test]
    fn legal_move_targets_returns_adjacent_empty_squares() {
        let mut home = make_team("home");
        home.players.push(make_player("mover"));
        let mut engine = make_engine_with_teams(home, make_team("away"));
        engine.game.field_model.set_player_coordinate("mover", FieldCoordinate::new(12, 7));
        engine.game.field_model.set_player_state("mover", PS::new(PS_STANDING));

        let targets = legal_move_targets(&engine.game, "mover");
        assert!(!targets.is_empty(), "should have adjacent squares");
        for t in &targets {
            assert!(t.is_on_pitch(), "all targets must be on pitch");
            assert!(t.is_adjacent(FieldCoordinate::new(12, 7)), "all targets must be adjacent");
        }
    }

    #[test]
    fn legal_move_targets_excludes_occupied_squares() {
        let mut home = make_team("home");
        home.players.push(make_player("mover"));
        home.players.push(make_player("blocker"));
        let mut engine = make_engine_with_teams(home, make_team("away"));
        engine.game.field_model.set_player_coordinate("mover", FieldCoordinate::new(12, 7));
        engine.game.field_model.set_player_state("mover", PS::new(PS_STANDING));
        engine.game.field_model.set_player_coordinate("blocker", FieldCoordinate::new(13, 7));
        engine.game.field_model.set_player_state("blocker", PS::new(PS_STANDING));

        let targets = legal_move_targets(&engine.game, "mover");
        assert!(
            !targets.contains(&FieldCoordinate::new(13, 7)),
            "occupied square (13,7) must be excluded from move targets"
        );
    }

    #[test]
    fn legal_block_targets_returns_adjacent_standing_opponents() {
        let mut home = make_team("home");
        home.players.push(make_player("attacker"));
        let mut away = make_team("away");
        away.players.push(make_player("defender"));
        let mut engine = make_engine_with_teams(home, away);
        engine.game.field_model.set_player_coordinate("attacker", FieldCoordinate::new(12, 7));
        engine.game.field_model.set_player_state("attacker", PS::new(PS_STANDING));
        engine.game.field_model.set_player_coordinate("defender", FieldCoordinate::new(13, 7));
        engine.game.field_model.set_player_state("defender", PS::new(PS_STANDING));

        let targets = legal_block_targets(&engine.game, "attacker", TeamSide::Home);
        assert!(targets.contains(&"defender".to_string()), "adjacent standing opponent must be a valid block target");
    }

    #[test]
    fn legal_block_targets_excludes_prone_opponents() {
        let mut home = make_team("home");
        home.players.push(make_player("attacker"));
        let mut away = make_team("away");
        away.players.push(make_player("downed"));
        let mut engine = make_engine_with_teams(home, away);
        engine.game.field_model.set_player_coordinate("attacker", FieldCoordinate::new(12, 7));
        engine.game.field_model.set_player_state("attacker", PS::new(PS_STANDING));
        engine.game.field_model.set_player_coordinate("downed", FieldCoordinate::new(13, 7));
        engine.game.field_model.set_player_state("downed", PS::new(PS_PRONE));

        let targets = legal_block_targets(&engine.game, "attacker", TeamSide::Home);
        assert!(!targets.contains(&"downed".to_string()), "prone player cannot be blocked");
    }

    #[test]
    fn legal_foul_targets_returns_adjacent_prone_opponents() {
        let mut home = make_team("home");
        home.players.push(make_player("fouler"));
        let mut away = make_team("away");
        away.players.push(make_player("victim"));
        let mut engine = make_engine_with_teams(home, away);
        engine.game.field_model.set_player_coordinate("fouler", FieldCoordinate::new(12, 7));
        engine.game.field_model.set_player_state("fouler", PS::new(PS_STANDING));
        engine.game.field_model.set_player_coordinate("victim", FieldCoordinate::new(13, 7));
        engine.game.field_model.set_player_state("victim", PS::new(PS_PRONE));

        let targets = legal_foul_targets(&engine.game, "fouler", TeamSide::Home);
        assert!(targets.contains(&"victim".to_string()), "adjacent prone opponent is a valid foul target");
    }

    #[test]
    fn legal_foul_targets_excludes_standing_opponents() {
        let mut home = make_team("home");
        home.players.push(make_player("fouler"));
        let mut away = make_team("away");
        away.players.push(make_player("upright"));
        let mut engine = make_engine_with_teams(home, away);
        engine.game.field_model.set_player_coordinate("fouler", FieldCoordinate::new(12, 7));
        engine.game.field_model.set_player_state("fouler", PS::new(PS_STANDING));
        engine.game.field_model.set_player_coordinate("upright", FieldCoordinate::new(13, 7));
        engine.game.field_model.set_player_state("upright", PS::new(PS_STANDING));

        let targets = legal_foul_targets(&engine.game, "fouler", TeamSide::Home);
        assert!(!targets.contains(&"upright".to_string()), "standing player cannot be fouled");
    }

    #[test]
    fn legal_kickoff_targets_home_kicks_to_away_half() {
        let engine = make_engine_with_teams(make_team("home"), make_team("away"));
        let targets = legal_kickoff_targets(&engine.game, TeamSide::Home);
        assert!(!targets.is_empty(), "home team must have kickoff targets");
        for t in &targets {
            assert!(t.x >= 13, "home team kicks to x >= 13 (away half), got x={}", t.x);
        }
    }

    #[test]
    fn legal_kickoff_targets_away_kicks_to_home_half() {
        let engine = make_engine_with_teams(make_team("home"), make_team("away"));
        let targets = legal_kickoff_targets(&engine.game, TeamSide::Away);
        assert!(!targets.is_empty(), "away team must have kickoff targets");
        for t in &targets {
            assert!(t.x <= 12, "away team kicks to x <= 12 (home half), got x={}", t.x);
        }
    }

    #[test]
    fn legal_activate_pass_only_for_ball_carrier() {
        // Only the player holding the ball should have Pass as a legal action.
        let mut home = make_team("home");
        home.players.push(make_player("carrier"));
        home.players.push(make_player("decoy"));
        let mut engine = make_engine_with_teams(home, make_team("away"));

        let carrier_coord = FieldCoordinate::new(10, 7);
        let decoy_coord = FieldCoordinate::new(5, 7);
        engine.game.field_model.set_player_coordinate("carrier", carrier_coord);
        engine.game.field_model.set_player_state("carrier", PS::new(PS_STANDING));
        engine.game.field_model.set_player_coordinate("decoy", decoy_coord);
        engine.game.field_model.set_player_state("decoy", PS::new(PS_STANDING));
        engine.game.field_model.ball_coordinate = Some(carrier_coord);
        engine.game.field_model.ball_in_play = true;

        let actions = legal_activate_player_actions(&engine.game, TeamSide::Home);

        assert!(
            actions.iter().any(|a| matches!(a, crate::action::Action::ActivatePlayer { player_id, player_action }
                if player_id == "carrier" && *player_action == PlayerActionChoice::Pass)),
            "ball carrier must have Pass available"
        );
        assert!(
            !actions.iter().any(|a| matches!(a, crate::action::Action::ActivatePlayer { player_id, player_action }
                if player_id == "decoy" && *player_action == PlayerActionChoice::Pass)),
            "non-carrier must NOT have Pass available"
        );
    }

    #[test]
    fn legal_activate_hand_off_only_when_teammate_adjacent() {
        // HandOff available only when ball-carrier has an adjacent teammate.
        let mut home = make_team("home");
        home.players.push(make_player("carrier"));
        home.players.push(make_player("receiver"));
        let mut engine = make_engine_with_teams(home, make_team("away"));

        let carrier_coord = FieldCoordinate::new(10, 7);
        let receiver_coord = FieldCoordinate::new(11, 7); // adjacent
        engine.game.field_model.set_player_coordinate("carrier", carrier_coord);
        engine.game.field_model.set_player_state("carrier", PS::new(PS_STANDING));
        engine.game.field_model.set_player_coordinate("receiver", receiver_coord);
        engine.game.field_model.set_player_state("receiver", PS::new(PS_STANDING));
        engine.game.field_model.ball_coordinate = Some(carrier_coord);
        engine.game.field_model.ball_in_play = true;

        let actions = legal_activate_player_actions(&engine.game, TeamSide::Home);
        assert!(
            actions.iter().any(|a| matches!(a, crate::action::Action::ActivatePlayer { player_id, player_action }
                if player_id == "carrier" && *player_action == PlayerActionChoice::HandOff)),
            "carrier with adjacent teammate must have HandOff available"
        );
    }

    #[test]
    fn legal_activate_hand_off_not_available_when_no_teammate_adjacent() {
        let mut home = make_team("home");
        home.players.push(make_player("carrier"));
        home.players.push(make_player("receiver"));
        let mut engine = make_engine_with_teams(home, make_team("away"));

        let carrier_coord = FieldCoordinate::new(5, 7);
        let receiver_coord = FieldCoordinate::new(15, 7); // NOT adjacent
        engine.game.field_model.set_player_coordinate("carrier", carrier_coord);
        engine.game.field_model.set_player_state("carrier", PS::new(PS_STANDING));
        engine.game.field_model.set_player_coordinate("receiver", receiver_coord);
        engine.game.field_model.set_player_state("receiver", PS::new(PS_STANDING));
        engine.game.field_model.ball_coordinate = Some(carrier_coord);
        engine.game.field_model.ball_in_play = true;

        let actions = legal_activate_player_actions(&engine.game, TeamSide::Home);
        assert!(
            !actions.iter().any(|a| matches!(a, crate::action::Action::ActivatePlayer { player_id, player_action }
                if player_id == "carrier" && *player_action == PlayerActionChoice::HandOff)),
            "carrier with no adjacent teammate must NOT have HandOff available"
        );
    }

    #[test]
    fn legal_activate_prone_player_gets_standup() {
        // A prone player should get StandUp action.
        let mut home = make_team("home");
        home.players.push(make_player("fallen"));
        let mut engine = make_engine_with_teams(home, make_team("away"));
        engine.game.field_model.set_player_coordinate("fallen", FieldCoordinate::new(10, 7));
        engine.game.field_model.set_player_state("fallen", PS::new(PS_PRONE));

        let actions = legal_activate_player_actions(&engine.game, TeamSide::Home);
        assert!(
            actions.iter().any(|a| matches!(a, crate::action::Action::ActivatePlayer { player_id, player_action }
                if player_id == "fallen" && *player_action == PlayerActionChoice::StandUp)),
            "prone player must have StandUp available"
        );
    }

    #[test]
    fn legal_activate_block_only_when_adjacent_opponent() {
        // Block should only appear when an adjacent standing opponent is present.
        let mut home = make_team("home");
        home.players.push(make_player("blocker"));
        let mut away = make_team("away");
        away.players.push(make_player("target"));
        let mut engine = make_engine_with_teams(home, away);

        let blocker_coord = FieldCoordinate::new(10, 7);
        let target_coord = FieldCoordinate::new(11, 7); // adjacent
        engine.game.field_model.set_player_coordinate("blocker", blocker_coord);
        engine.game.field_model.set_player_state("blocker", PS::new(PS_STANDING));
        engine.game.field_model.set_player_coordinate("target", target_coord);
        engine.game.field_model.set_player_state("target", PS::new(PS_STANDING));

        let actions = legal_activate_player_actions(&engine.game, TeamSide::Home);
        assert!(
            actions.iter().any(|a| matches!(a, crate::action::Action::ActivatePlayer { player_id, player_action }
                if player_id == "blocker" && *player_action == PlayerActionChoice::Block)),
            "player adjacent to opponent must have Block available"
        );
    }

    #[test]
    fn legal_activate_throw_team_mate_requires_skill_and_adjacent_teammate() {
        use ffb_model::model::skill_def::SkillWithValue;
        let mut home = make_team("home");
        let mut thrower = make_player("thrower");
        thrower.starting_skills.push(SkillWithValue::new(SkillId::ThrowTeamMate));
        home.players.push(thrower);
        let mut teammate = make_player("thrown");
        home.players.push(teammate.clone());
        let engine_ref = make_engine_with_teams(home.clone(), make_team("away"));
        // Should not appear without an adjacent teammate
        let mut engine = engine_ref;
        engine.game.field_model.set_player_coordinate("thrower", FieldCoordinate::new(10, 7));
        engine.game.field_model.set_player_state("thrower", PS::new(PS_STANDING));
        engine.game.field_model.set_player_coordinate("thrown", FieldCoordinate::new(15, 7)); // far away
        engine.game.field_model.set_player_state("thrown", PS::new(PS_STANDING));

        let actions = legal_activate_player_actions(&engine.game, TeamSide::Home);
        assert!(!actions.iter().any(|a| matches!(a, crate::action::Action::ActivatePlayer { player_id, player_action }
            if player_id == "thrower" && *player_action == PlayerActionChoice::ThrowTeamMate)),
            "ThrowTeamMate must not appear without adjacent teammate");

        // Place teammate adjacent
        engine.game.field_model.set_player_coordinate("thrown", FieldCoordinate::new(11, 7));
        let actions2 = legal_activate_player_actions(&engine.game, TeamSide::Home);
        assert!(actions2.iter().any(|a| matches!(a, crate::action::Action::ActivatePlayer { player_id, player_action }
            if player_id == "thrower" && *player_action == PlayerActionChoice::ThrowTeamMate)),
            "ThrowTeamMate must appear when adjacent teammate present");

        // Mark ttm_used — should disappear
        engine.game.turn_data_home.ttm_used = true;
        let actions3 = legal_activate_player_actions(&engine.game, TeamSide::Home);
        assert!(!actions3.iter().any(|a| matches!(a, crate::action::Action::ActivatePlayer { player_id, player_action }
            if player_id == "thrower" && *player_action == PlayerActionChoice::ThrowTeamMate)),
            "ThrowTeamMate must not appear when ttm_used");
    }

    #[test]
    fn legal_activate_kick_team_mate_requires_bb2025_skill_and_adjacent() {
        use ffb_model::model::skill_def::SkillWithValue;
        let mut home = make_team("home");
        let mut kicker = make_player("kicker");
        kicker.starting_skills.push(SkillWithValue::new(SkillId::KickTeamMate));
        home.players.push(kicker);
        home.players.push(make_player("kicked"));
        let mut engine = GameEngine::new(home, make_team("away"), Rules::Bb2025, 42);

        engine.game.field_model.set_player_coordinate("kicker", FieldCoordinate::new(10, 7));
        engine.game.field_model.set_player_state("kicker", PS::new(PS_STANDING));
        engine.game.field_model.set_player_coordinate("kicked", FieldCoordinate::new(11, 7));
        engine.game.field_model.set_player_state("kicked", PS::new(PS_STANDING));

        let actions = legal_activate_player_actions(&engine.game, TeamSide::Home);
        assert!(actions.iter().any(|a| matches!(a, crate::action::Action::ActivatePlayer { player_id, player_action }
            if player_id == "kicker" && *player_action == PlayerActionChoice::KickTeamMate)),
            "KickTeamMate must appear in BB2025 with adjacent teammate");
    }

    #[test]
    fn legal_activate_kick_team_mate_not_in_bb2020() {
        use ffb_model::model::skill_def::SkillWithValue;
        let mut home = make_team("home");
        let mut kicker = make_player("kicker");
        kicker.starting_skills.push(SkillWithValue::new(SkillId::KickTeamMate));
        home.players.push(kicker);
        home.players.push(make_player("kicked"));
        let mut engine = GameEngine::new(home, make_team("away"), Rules::Bb2020, 42);

        engine.game.field_model.set_player_coordinate("kicker", FieldCoordinate::new(10, 7));
        engine.game.field_model.set_player_state("kicker", PS::new(PS_STANDING));
        engine.game.field_model.set_player_coordinate("kicked", FieldCoordinate::new(11, 7));
        engine.game.field_model.set_player_state("kicked", PS::new(PS_STANDING));

        let actions = legal_activate_player_actions(&engine.game, TeamSide::Home);
        assert!(!actions.iter().any(|a| matches!(a, crate::action::Action::ActivatePlayer { player_id, player_action }
            if player_id == "kicker" && *player_action == PlayerActionChoice::KickTeamMate)),
            "KickTeamMate must NOT appear in BB2020");
    }

    #[test]
    fn legal_activate_punt_requires_bb2025_skill_ball_carrier() {
        use ffb_model::model::skill_def::SkillWithValue;
        let mut home = make_team("home");
        let mut punter = make_player("punter");
        punter.starting_skills.push(SkillWithValue::new(SkillId::Punt));
        home.players.push(punter);
        let mut engine = GameEngine::new(home, make_team("away"), Rules::Bb2025, 42);

        let coord = FieldCoordinate::new(10, 7);
        engine.game.field_model.set_player_coordinate("punter", coord);
        engine.game.field_model.set_player_state("punter", PS::new(PS_STANDING));
        engine.game.field_model.ball_coordinate = Some(coord);
        engine.game.field_model.ball_in_play = true;

        let actions = legal_activate_player_actions(&engine.game, TeamSide::Home);
        assert!(actions.iter().any(|a| matches!(a, crate::action::Action::ActivatePlayer { player_id, player_action }
            if player_id == "punter" && *player_action == PlayerActionChoice::Punt)),
            "Punt must appear for ball-carrier with Punt skill in BB2025");

        // Without the ball, Punt should not appear
        engine.game.field_model.ball_coordinate = Some(FieldCoordinate::new(20, 7));
        let actions2 = legal_activate_player_actions(&engine.game, TeamSide::Home);
        assert!(!actions2.iter().any(|a| matches!(a, crate::action::Action::ActivatePlayer { player_id, player_action }
            if player_id == "punter" && *player_action == PlayerActionChoice::Punt)),
            "Punt must not appear when player is not the ball carrier");
    }

    #[test]
    fn legal_activate_foul_requires_prone_opponent() {
        // Foul is only available when there is an adjacent prone opponent.
        let mut home = make_team("home");
        home.players.push(make_player("fouler"));
        let mut away = make_team("away");
        away.players.push(make_player("victim"));
        let mut engine = make_engine_with_teams(home, away);

        engine.game.field_model.set_player_coordinate("fouler", FieldCoordinate::new(10, 7));
        engine.game.field_model.set_player_state("fouler", PS::new(PS_STANDING));
        engine.game.field_model.set_player_coordinate("victim", FieldCoordinate::new(11, 7));
        engine.game.field_model.set_player_state("victim", PS::new(PS_PRONE));

        let actions = legal_activate_player_actions(&engine.game, TeamSide::Home);
        assert!(
            actions.iter().any(|a| matches!(a, crate::action::Action::ActivatePlayer { player_id, player_action }
                if player_id == "fouler" && *player_action == PlayerActionChoice::Foul)),
            "player adjacent to prone opponent must have Foul available"
        );

        // Standing opponent — Foul should NOT appear
        engine.game.field_model.set_player_state("victim", PS::new(PS_STANDING));
        let actions2 = legal_activate_player_actions(&engine.game, TeamSide::Home);
        assert!(
            !actions2.iter().any(|a| matches!(a, crate::action::Action::ActivatePlayer { player_id, player_action }
                if player_id == "fouler" && *player_action == PlayerActionChoice::Foul)),
            "player with only standing opponent must NOT have Foul available"
        );
    }

    #[test]
    fn legal_move_targets_includes_all_adjacent_empty_squares_with_opponent_nearby() {
        // legal_move_targets always returns all adjacent empty squares.
        // Even with an opponent adjacent (TZ present), the squares are still legal targets.
        let mut home = make_team("home");
        home.players.push(make_player("mover"));
        let mut away = make_team("away");
        away.players.push(make_player("tackler"));
        let mut engine = make_engine_with_teams(home, away);

        engine.game.field_model.set_player_coordinate("mover", FieldCoordinate::new(10, 7));
        engine.game.field_model.set_player_state("mover", PS::new(PS_STANDING));
        // Opponent adjacent — would need a dodge, but all adjacent EMPTY squares still legal targets
        engine.game.field_model.set_player_coordinate("tackler", FieldCoordinate::new(11, 7));
        engine.game.field_model.set_player_state("tackler", PS::new(PS_STANDING));

        let targets = legal_move_targets(&engine.game, "mover");
        // (11,7) is occupied by tackler → not in targets; all other 7 adjacent squares should be
        assert!(!targets.contains(&FieldCoordinate::new(11, 7)), "occupied square must not be a legal target");
        assert!(targets.contains(&FieldCoordinate::new(10, 8)), "empty adjacent square must be legal");
        assert!(targets.contains(&FieldCoordinate::new(10, 6)), "empty adjacent square must be legal");
        assert!(targets.len() >= 5, "at least 5 adjacent empty squares must be legal from (10,7)");
    }

    #[test]
    fn legal_block_targets_includes_opponent_carrying_ball() {
        // A ball-carrying opponent is still a valid block target.
        let mut home = make_team("home");
        home.players.push(make_player("blocker"));
        let mut away = make_team("away");
        away.players.push(make_player("carrier"));
        let mut engine = make_engine_with_teams(home, away);

        engine.game.field_model.set_player_coordinate("blocker", FieldCoordinate::new(10, 7));
        engine.game.field_model.set_player_state("blocker", PS::new(PS_STANDING));
        engine.game.field_model.set_player_coordinate("carrier", FieldCoordinate::new(11, 7));
        engine.game.field_model.set_player_state("carrier", PS::new(PS_STANDING));
        // Ball at carrier's square
        engine.game.field_model.ball_coordinate = Some(FieldCoordinate::new(11, 7));
        engine.game.field_model.ball_in_play = true;

        let targets = legal_block_targets(&engine.game, "blocker", TeamSide::Home);
        assert!(targets.contains(&"carrier".to_string()), "ball carrier must be a legal block target");
    }

    #[test]
    fn legal_foul_targets_empty_when_foul_already_used() {
        // When turn_data.foul_used = true, no foul should be legal for activation.
        // (The engine honors foul_used in legal_activate_player_actions; legal_foul_targets
        // itself returns targets, but activation won't offer Foul when foul_used is set.)
        let mut home = make_team("home");
        home.players.push(make_player("fouler"));
        let mut away = make_team("away");
        away.players.push(make_player("victim"));
        let mut engine = make_engine_with_teams(home, away);

        engine.game.field_model.set_player_coordinate("fouler", FieldCoordinate::new(10, 7));
        engine.game.field_model.set_player_state("fouler", PS::new(PS_STANDING));
        engine.game.field_model.set_player_coordinate("victim", FieldCoordinate::new(11, 7));
        engine.game.field_model.set_player_state("victim", PS::new(PS_PRONE));
        engine.game.turn_data_home.foul_used = true;

        let actions = legal_activate_player_actions(&engine.game, TeamSide::Home);
        assert!(
            !actions.iter().any(|a| matches!(a, crate::action::Action::ActivatePlayer { player_action, .. }
                if *player_action == PlayerActionChoice::Foul)),
            "Foul must not be offered when foul_used=true"
        );
    }

    #[test]
    fn legal_actions_end_turn_accepted_in_regular_mode() {
        // The engine accepts EndTurn in Regular turn mode.
        let home = make_team("home");
        let away = make_team("away");
        let mut engine = make_engine_with_teams(home, away);
        engine.game.turn_data_home.turn_nr = 1;
        engine.game.turn_data_away.turn_nr = 1;
        engine.game.home_playing = true;
        engine.game.turn_mode = ffb_model::enums::TurnMode::Regular;
        engine.game.status = ffb_model::enums::GameStatus::Active;

        let result = engine.apply(TeamSide::Home, crate::action::Action::EndTurn);
        assert!(result.is_ok(), "EndTurn must be accepted in Regular mode, got {:?}", result.err());
    }

    #[test]
    fn legal_move_targets_returns_empty_for_off_pitch_player() {
        let home = make_team("home");
        let away = make_team("away");
        let engine = make_engine_with_teams(home, away);
        // "ghost" has no coordinate
        let targets = legal_move_targets(&engine.game, "ghost");
        assert!(targets.is_empty(), "off-pitch player has no legal move targets");
    }
}
