/// BbAction enum and legal action enumeration.
use crate::model::game_state::{DialogState, GameState};
use crate::skills::SkillId;
use crate::types::{BlockResult, FieldCoordinate, PlayerId, PlayerAction, PlayerState, TeamId};
use serde::{Deserialize, Serialize};

// ── BbAction ──────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BbAction {
    // --- Turn-level actions ---
    EndTurn,

    // --- Player activation ---
    Activate { player_id: PlayerId, action: PlayerAction },

    // --- Movement ---
    MoveTo(FieldCoordinate),

    // --- Block ---
    BlockTarget(PlayerId),
    ChooseBlockDie(BlockResult),

    // --- Push ---
    ChoosePush(FieldCoordinate),

    // --- Pass / catch ---
    PassTo(FieldCoordinate),

    // --- Re-roll ---
    UseReroll(bool), // true = use reroll, false = decline

    // --- Kickoff ---
    PlaceBall(FieldCoordinate),

    // --- Follow-up after push ---
    ChooseFollowup(bool), // true = follow up into vacated square, false = stay
}

// ── Legal action enumeration ──────────────────────────────────────────────────

/// Enumerate all legal actions for `team` given the current game state.
pub fn enumerate_actions(state: &GameState, team: TeamId) -> Vec<BbAction> {
    match &state.dialog {
        DialogState::SelectBlockDice { dice, defender_chooses: _ } => {
            // Return unique dice choices
            let mut choices: Vec<BbAction> = dice
                .iter()
                .cloned()
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .map(BbAction::ChooseBlockDie)
                .collect();
            choices.sort_by_key(|a| format!("{a:?}"));
            return choices;
        }

        DialogState::SelectPush { options } => {
            return options.iter().cloned().map(BbAction::ChoosePush).collect();
        }

        DialogState::SelectReroll { reroll_available: _, skill_reroll_available: _, .. } => {
            return vec![BbAction::UseReroll(true), BbAction::UseReroll(false)];
        }

        DialogState::SelectBlockReroll { dice: _, defender_chooses: _, reroll_available: _ } => {
            return vec![BbAction::UseReroll(true), BbAction::UseReroll(false)];
        }

        DialogState::SelectFollowup { .. } => {
            return vec![BbAction::ChooseFollowup(true), BbAction::ChooseFollowup(false)];
        }

        DialogState::SelectMoveTarget { targets } => {
            let mut actions: Vec<BbAction> = targets
                .iter()
                .map(|(coord, _prob)| BbAction::MoveTo(*coord))
                .collect();
            actions.push(BbAction::EndTurn);
            return actions;
        }

        DialogState::SelectBlockTarget { targets } => {
            return targets.iter().cloned().map(BbAction::BlockTarget).collect();
        }

        DialogState::SelectPlayer { candidates } => {
            return candidates
                .iter()
                .cloned()
                .flat_map(|pid| {
                    vec![
                        BbAction::Activate { player_id: pid.clone(), action: PlayerAction::Move },
                        BbAction::Activate { player_id: pid, action: PlayerAction::Block },
                    ]
                })
                .collect();
        }

        DialogState::SelectKickTarget => {
            // For simplicity, enumerate valid kick target squares in opponent half
            let (x_start, x_end) = if state.home_is_active {
                // home kicks to away half (x > 13)
                (14u8, 25u8)
            } else {
                (1u8, 12u8)
            };
            let mut targets = Vec::new();
            for x in x_start..=x_end {
                for y in 4u8..=12u8 {
                    targets.push(BbAction::PlaceBall(FieldCoordinate::new(x, y)));
                }
            }
            return targets;
        }

        DialogState::None => {
            // No active dialog — enumerate player activations + EndTurn
        }

        // For other dialogs (injury, apothecary, etc.) just allow EndTurn as a fallback
        _ => {
            return vec![BbAction::EndTurn];
        }
    }

    // DialogState::None — enumerate available player activations
    let mut actions: Vec<BbAction> = Vec::new();

    // Check if there is an already activated player with no pending dialog
    if let Some(acting) = &state.acting_player {
        if acting.team == team {
            // If the acting player has chosen Pass, enumerate pass targets
            if acting.current_action == Some(PlayerAction::Pass) {
                let passer_id = acting.player_id.clone();
                if let Some(passer_coord) = state.field.player_coord(&passer_id) {
                    // Enumerate friendly players on pitch as pass targets
                    for player in state.team(team).players() {
                        if player.id == passer_id { continue; }
                        if let Some(target_coord) = state.field.player_coord(&player.id) {
                            if state.field.player_state(&player.id).map(|s| s.is_active()).unwrap_or(false) {
                                actions.push(BbAction::PassTo(target_coord));
                            }
                        }
                    }
                    // Also allow passes to empty squares (inaccurate landing)
                    let _ = passer_coord; // passer_coord used for potential range filtering
                }
                if actions.is_empty() { actions.push(BbAction::EndTurn); }
                return actions;
            }
            // If the acting player has chosen HandOff, enumerate adjacent friendly players
            if acting.current_action == Some(PlayerAction::HandOff) {
                let carrier_id = acting.player_id.clone();
                if let Some(carrier_coord) = state.field.player_coord(&carrier_id) {
                    for neighbor in carrier_coord.neighbors() {
                        if let Some(pid) = state.field.player_at(neighbor) {
                            if state.field.player_team(pid) == Some(team) {
                                actions.push(BbAction::PassTo(neighbor));
                            }
                        }
                    }
                }
                if actions.is_empty() { actions.push(BbAction::EndTurn); }
                return actions;
            }
            // If the acting player has chosen Foul, enumerate adjacent prone opponents
            if acting.current_action == Some(PlayerAction::Foul) {
                let fouler_id = acting.player_id.clone();
                if let Some(fouler_coord) = state.field.player_coord(&fouler_id) {
                    let targets: Vec<PlayerId> = fouler_coord
                        .neighbors()
                        .filter_map(|n| {
                            let opp_id = state.field.player_at(n)?;
                            let opp_team = state.field.player_team(opp_id)?;
                            if opp_team != team {
                                let opp_state = state.field.player_state(opp_id)?;
                                if opp_state == PlayerState::Prone {
                                    return Some(opp_id.clone());
                                }
                            }
                            None
                        })
                        .collect();
                    if !targets.is_empty() {
                        return targets.into_iter().map(BbAction::BlockTarget).collect();
                    }
                }
                actions.push(BbAction::EndTurn);
                return actions;
            }
            // Wild Animal restricted: only Block/Blitz allowed — no move/pass/handoff
            if acting.wild_animal_restricted {
                let wa_id = acting.player_id.clone();
                if let Some(coord) = state.field.player_coord(&wa_id) {
                    let has_adj_opp = coord.neighbors().any(|n| {
                        state.field.player_at(n)
                            .and_then(|pid| state.field.player_team(pid))
                            .map(|t| t != team)
                            .unwrap_or(false)
                    });
                    if has_adj_opp {
                        let targets: Vec<PlayerId> = coord.neighbors()
                            .filter_map(|n| {
                                let pid = state.field.player_at(n)?;
                                if state.field.player_team(pid) != Some(team) { Some(pid.clone()) } else { None }
                            })
                            .collect();
                        return targets.into_iter().map(BbAction::BlockTarget).collect();
                    }
                }
                // No adjacent opponent — forced EndTurn
                actions.push(BbAction::EndTurn);
                return actions;
            }

            // The acting player can end their activation (EndTurn at action level = EndTurn turn)
            actions.push(BbAction::EndTurn);
            return actions;
        }
    }

    // Gather activatable players for the active team
    let active_team = state.team(team);
    let _turn_data = if team == TeamId::Home {
        &state.turn_data_home
    } else {
        &state.turn_data_away
    };

    let is_active_team = state.active_team_id() == team;

    if is_active_team {
        for player in active_team.players() {
            let pid = &player.id;
            let on_pitch = state
                .field
                .player_state(pid)
                .map(|s| {
                    s.is_active()
                        || (s == PlayerState::Prone && player.has_skill(SkillId::JumpUp))
                })
                .unwrap_or(false);

            if !on_pitch {
                continue;
            }

            // Skip players who already took an action this turn
            if state.players_activated_this_turn.contains(pid) {
                continue;
            }

            // Move action
            actions.push(BbAction::Activate {
                player_id: pid.clone(),
                action: PlayerAction::Move,
            });

            // Block action — only if there are adjacent opponents
            let coord = state.field.player_coord(pid);
            let has_adjacent_opponent = coord
                .map(|coord| {
                    coord.neighbors().any(|n| {
                        state
                            .field
                            .player_at(n)
                            .and_then(|opp_id| state.field.player_team(opp_id))
                            .map(|t| t != team)
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false);

            if has_adjacent_opponent {
                actions.push(BbAction::Activate {
                    player_id: pid.clone(),
                    action: PlayerAction::Block,
                });
            }

            // Ball carrier check — needed for Pass and HandOff
            let has_ball = state.field.ball.in_play
                && state.field.ball.coord == state.field.player_coord(pid);

            // Pass action — only if player is the ball carrier and has a PA value
            let has_pa = player.effective_pa().is_some();
            let has_pass_targets = state.team(team).players().iter().any(|other| {
                other.id != *pid
                    && state.field.player_state(&other.id)
                        .map(|s| s.is_active())
                        .unwrap_or(false)
            });
            if has_ball && has_pa && has_pass_targets {
                actions.push(BbAction::Activate {
                    player_id: pid.clone(),
                    action: PlayerAction::Pass,
                });
            }

            // Blitz action — once per turn; player moves then blocks; requires adjacent opp OR reachable opp
            let blitz_used = if team == TeamId::Home {
                state.turn_data_home.blitz_used
            } else {
                state.turn_data_away.blitz_used
            };
            if !blitz_used {
                // Blitz is available if there are any opponents on pitch (reachable via movement)
                let has_any_opp_on_pitch = state.team(team.opponent()).players().iter().any(|opp| {
                    state.field.player_state(&opp.id)
                        .map(|s| s.is_active() || s == PlayerState::Prone)
                        .unwrap_or(false)
                });
                if has_any_opp_on_pitch {
                    actions.push(BbAction::Activate {
                        player_id: pid.clone(),
                        action: PlayerAction::Blitz,
                    });
                }
            }

            // HandOff action — if player is on the ball and has an adjacent teammate
            let has_adjacent_friendly = coord
                .map(|c| {
                    c.neighbors().any(|n| {
                        state.field.player_at(n)
                            .and_then(|oid| state.field.player_team(oid))
                            .map(|t| t == team)
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false);
            if has_ball && has_adjacent_friendly {
                actions.push(BbAction::Activate {
                    player_id: pid.clone(),
                    action: PlayerAction::HandOff,
                });
            }

            // Foul action — only one foul per turn, and only if there is an adjacent prone opponent
            let foul_used = if team == TeamId::Home {
                state.turn_data_home.foul_used
            } else {
                state.turn_data_away.foul_used
            };
            if !foul_used {
                let has_adjacent_prone_opponent = coord
                    .map(|c| {
                        c.neighbors().any(|n| {
                            state.field.player_at(n)
                                .and_then(|opp_id| {
                                    let opp_team = state.field.player_team(opp_id)?;
                                    if opp_team != team {
                                        state.field.player_state(opp_id)
                                    } else {
                                        None
                                    }
                                })
                                .map(|s| s == PlayerState::Prone)
                                .unwrap_or(false)
                        })
                    })
                    .unwrap_or(false);

                if has_adjacent_prone_opponent {
                    actions.push(BbAction::Activate {
                        player_id: pid.clone(),
                        action: PlayerAction::Foul,
                    });
                }
            }
        }
    }

    // EndTurn is always available when there's no mandatory dialog
    actions.push(BbAction::EndTurn);

    actions
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::player::{Player, PlayerStats};
    use crate::model::team::Team;
    use crate::skills::SkillSet;
    use crate::types::{FieldCoordinate, PlayerId, PlayerState, TeamId};
    use crate::model::game_state::GameState;

    fn make_state_with_players() -> GameState {
        let pid_h = PlayerId("h1".into());
        let pid_a = PlayerId("a1".into());

        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        home.add_player(Player::new(
            pid_h.clone(),
            "Griff".into(),
            "blitzer".into(),
            TeamId::Home,
            1,
            PlayerStats::new(6, 3, 4, 8, None),
            SkillSet::empty(),
        ));

        let mut away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        away.add_player(Player::new(
            pid_a.clone(),
            "Grimgor".into(),
            "blitzer".into(),
            TeamId::Away,
            1,
            PlayerStats::new(5, 4, 3, 9, None),
            SkillSet::empty(),
        ));

        let mut state = GameState::new(home, away);
        state.field.place_player(pid_h, TeamId::Home, FieldCoordinate::new(5, 5), PlayerState::Standing);
        state.field.place_player(pid_a, TeamId::Away, FieldCoordinate::new(15, 5), PlayerState::Standing);
        state.home_is_active = true;
        state
    }

    #[test]
    fn enumerate_actions_has_end_turn() {
        let state = make_state_with_players();
        let actions = enumerate_actions(&state, TeamId::Home);
        assert!(actions.contains(&BbAction::EndTurn), "EndTurn must always be present");
    }

    #[test]
    fn enumerate_actions_includes_activations() {
        let state = make_state_with_players();
        let actions = enumerate_actions(&state, TeamId::Home);
        let has_activate = actions.iter().any(|a| matches!(a, BbAction::Activate { .. }));
        assert!(has_activate, "should include player activations");
    }

    #[test]
    fn enumerate_actions_select_block_dice() {
        let mut state = make_state_with_players();
        state.dialog = DialogState::SelectBlockDice {
            dice: vec![BlockResult::Skull, BlockResult::Pow, BlockResult::Skull],
            defender_chooses: false,
        };
        let actions = enumerate_actions(&state, TeamId::Home);
        // Unique dice only: Skull and Pow
        assert_eq!(actions.len(), 2);
        assert!(actions.iter().any(|a| *a == BbAction::ChooseBlockDie(BlockResult::Skull)));
        assert!(actions.iter().any(|a| *a == BbAction::ChooseBlockDie(BlockResult::Pow)));
    }

    #[test]
    fn enumerate_actions_select_push() {
        let mut state = make_state_with_players();
        let opt = FieldCoordinate::new(7, 5);
        state.dialog = DialogState::SelectPush { options: vec![opt] };
        let actions = enumerate_actions(&state, TeamId::Home);
        assert_eq!(actions, vec![BbAction::ChoosePush(opt)]);
    }

    #[test]
    fn enumerate_actions_reroll() {
        let mut state = make_state_with_players();
        state.dialog = DialogState::SelectReroll {
            action_name: "dodge".into(),
            reroll_available: true,
            skill_reroll_available: false,
        };
        let actions = enumerate_actions(&state, TeamId::Home);
        assert!(actions.contains(&BbAction::UseReroll(true)));
        assert!(actions.contains(&BbAction::UseReroll(false)));
    }

    // ── Foul action tests ─────────────────────────────────────────────────────

    #[test]
    fn foul_action_appears_when_adjacent_prone_opponent() {
        let pid_h = PlayerId("h1".into());
        let pid_a = PlayerId("a1".into());

        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        home.add_player(Player::new(
            pid_h.clone(), "Fouler".into(), "lineman".into(), TeamId::Home, 1,
            PlayerStats::new(6, 3, 4, 8, None), SkillSet::empty(),
        ));
        let mut away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        away.add_player(Player::new(
            pid_a.clone(), "Victim".into(), "lineman".into(), TeamId::Away, 1,
            PlayerStats::new(5, 4, 3, 9, None), SkillSet::empty(),
        ));

        let mut state = GameState::new(home, away);
        state.field.place_player(pid_h.clone(), TeamId::Home, FieldCoordinate::new(5, 5), PlayerState::Standing);
        state.field.place_player(pid_a.clone(), TeamId::Away, FieldCoordinate::new(6, 5), PlayerState::Prone);
        state.home_is_active = true;

        let actions = enumerate_actions(&state, TeamId::Home);
        let has_foul = actions.iter().any(|a| {
            matches!(a, BbAction::Activate { player_id, action: PlayerAction::Foul } if player_id == &pid_h)
        });
        assert!(has_foul, "Foul action should be available when adjacent prone opponent exists");
    }

    #[test]
    fn foul_action_absent_when_no_prone_opponent() {
        let pid_h = PlayerId("h1".into());
        let pid_a = PlayerId("a1".into());

        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        home.add_player(Player::new(
            pid_h.clone(), "Fouler".into(), "lineman".into(), TeamId::Home, 1,
            PlayerStats::new(6, 3, 4, 8, None), SkillSet::empty(),
        ));
        let mut away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        away.add_player(Player::new(
            pid_a.clone(), "Victim".into(), "lineman".into(), TeamId::Away, 1,
            PlayerStats::new(5, 4, 3, 9, None), SkillSet::empty(),
        ));

        let mut state = GameState::new(home, away);
        state.field.place_player(pid_h.clone(), TeamId::Home, FieldCoordinate::new(5, 5), PlayerState::Standing);
        state.field.place_player(pid_a.clone(), TeamId::Away, FieldCoordinate::new(6, 5), PlayerState::Standing);
        state.home_is_active = true;

        let actions = enumerate_actions(&state, TeamId::Home);
        let has_foul = actions.iter().any(|a| {
            matches!(a, BbAction::Activate { action: PlayerAction::Foul, .. })
        });
        assert!(!has_foul, "Foul action should not appear when no prone opponents are adjacent");
    }

    #[test]
    fn foul_action_absent_when_foul_already_used() {
        let pid_h = PlayerId("h1".into());
        let pid_a = PlayerId("a1".into());

        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        home.add_player(Player::new(
            pid_h.clone(), "Fouler".into(), "lineman".into(), TeamId::Home, 1,
            PlayerStats::new(6, 3, 4, 8, None), SkillSet::empty(),
        ));
        let mut away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        away.add_player(Player::new(
            pid_a.clone(), "Victim".into(), "lineman".into(), TeamId::Away, 1,
            PlayerStats::new(5, 4, 3, 9, None), SkillSet::empty(),
        ));

        let mut state = GameState::new(home, away);
        state.field.place_player(pid_h.clone(), TeamId::Home, FieldCoordinate::new(5, 5), PlayerState::Standing);
        state.field.place_player(pid_a.clone(), TeamId::Away, FieldCoordinate::new(6, 5), PlayerState::Prone);
        state.home_is_active = true;
        state.turn_data_home.foul_used = true;

        let actions = enumerate_actions(&state, TeamId::Home);
        let has_foul = actions.iter().any(|a| {
            matches!(a, BbAction::Activate { action: PlayerAction::Foul, .. })
        });
        assert!(!has_foul, "Foul action should not appear when foul_used is true");
    }

    // ── JumpUp action tests ───────────────────────────────────────────────────

    #[test]
    fn jump_up_allows_prone_player_to_activate() {
        use crate::skills::SkillId;
        let pid_h = PlayerId("h1".into());

        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        home.add_player(Player::new(
            pid_h.clone(), "Jumper".into(), "blitzer".into(), TeamId::Home, 1,
            PlayerStats::new(6, 3, 4, 8, None),
            [SkillId::JumpUp].into_iter().collect(),
        ));
        let away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);

        let mut state = GameState::new(home, away);
        state.field.place_player(pid_h.clone(), TeamId::Home, FieldCoordinate::new(5, 5), PlayerState::Prone);
        state.home_is_active = true;

        let actions = enumerate_actions(&state, TeamId::Home);
        let has_activate = actions.iter().any(|a| {
            matches!(a, BbAction::Activate { player_id, .. } if player_id == &pid_h)
        });
        assert!(has_activate, "Prone player with JumpUp should be activatable");
    }

    #[test]
    fn prone_without_jump_up_cannot_activate() {
        let pid_h = PlayerId("h1".into());

        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        home.add_player(Player::new(
            pid_h.clone(), "Grounded".into(), "lineman".into(), TeamId::Home, 1,
            PlayerStats::new(6, 3, 4, 8, None), SkillSet::empty(),
        ));
        let away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);

        let mut state = GameState::new(home, away);
        state.field.place_player(pid_h.clone(), TeamId::Home, FieldCoordinate::new(5, 5), PlayerState::Prone);
        state.home_is_active = true;

        let actions = enumerate_actions(&state, TeamId::Home);
        let has_activate = actions.iter().any(|a| {
            matches!(a, BbAction::Activate { player_id, .. } if player_id == &pid_h)
        });
        assert!(!has_activate, "Prone player without JumpUp should NOT be activatable");
    }
}
