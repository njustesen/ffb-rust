/// 1:1 translation of com.fumbbl.ffb.server.step.phase.kickoff.StepKickoffReturn.
///
/// Handles the KICKOFF_RETURN skill. Finds the eligible kickoff-return player on the
/// receiving team and, if one exists and there is no touchback, flips the turn to the
/// receiving team and pushes a select sequence so they can activate that player.
///
/// Expects stepParameter TOUCHBACK, END_PLAYER_ACTION, END_TURN to be set by preceding steps.
/// May push new select sequence on the stack.
use ffb_model::enums::TurnMode;
use ffb_model::model::game::Game;
use ffb_model::model::property::NamedProperties;
use ffb_model::prompts::AgentPrompt;
use ffb_model::types::FieldCoordinateBounds;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use crate::action::Action;
use crate::step::framework::{SequenceStep, Step, StepId, StepOutcome, StepParameter};

pub struct StepKickoffReturn {
    /// Java: fTouchback
    touchback: bool,
    /// Java: fEndPlayerAction
    end_player_action: bool,
    /// Java: fEndTurn
    end_turn: bool,
    /// True from the moment THIS step opens the kickoff-return window until it closes it again.
    ///
    /// Java guards its `setParameter` consumption on `game.getTurnMode() == KICKOFF_RETURN`, and
    /// `consumes_parameter` has no `&Game`. It does not need one: the window is open exactly when
    /// this step opened it, so the step can answer from its own state.
    window_open: bool,
}

impl StepKickoffReturn {
    pub fn new() -> Self {
        Self { touchback: false, end_player_action: false, end_turn: false, window_open: false }
    }

    /// Java: `generator.pushSequence(new Select.SequenceParams(getGameState(), false))` -- the
    /// edition's REAL Select generator with `updatePersistence = false`, exactly as
    /// `StepPassBlock::select_seq` already builds the pass-block window's.
    ///
    /// Until ITER29 this step pushed `sequences::select_sequence()`, a stub of `InitSelecting`
    /// + 18 `NoOp` + `EndSelecting` of which it was the only live caller. The real sequence
    /// carries the negatrait / JUMP_UP / STAND_UP / RESET_FUMBLEROOSKIE steps and, crucially, the
    /// `END_SELECTING` label on RESET_FUMBLEROOSKIE that `InitSelecting`'s `GotoLabelOnEnd` jumps
    /// to. Dispatched on `game.rules` because each edition has its own generator (and the
    /// `..Default::default()` lesson: an edition-gated params struct silently defaults to BB2025).
    fn window_select_sequence(game: &Game) -> Vec<SequenceStep> {
        use ffb_model::enums::Rules;
        match game.rules {
            Rules::Bb2016 => {
                use crate::step::generator::bb2016::select::{Select, SelectParams};
                Select::build_sequence(&SelectParams { update_persistence: false })
            }
            Rules::Bb2020 => {
                use crate::step::generator::bb2020::select::{Select, SelectParams};
                Select::build_sequence(&SelectParams {
                    update_persistence: false,
                    is_blitz_move: false,
                    block_targets: Vec::new(),
                })
            }
            // `Common` is the mechanic dispatchers' convention too (`mechanic/mod.rs`).
            Rules::Bb2025 | Rules::Common => {
                use crate::step::generator::bb2025::select::{Select, SelectParams};
                Select::build_sequence(&SelectParams {
                    update_persistence: false,
                    is_blitz_move: false,
                    block_targets: Vec::new(),
                    rules: Rules::Bb2025,
                })
            }
        }
    }
}

impl Default for StepKickoffReturn {
    fn default() -> Self { Self::new() }
}

impl Step for StepKickoffReturn {
    fn id(&self) -> StepId { StepId::KickoffReturn }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::Touchback(v) => {
                self.touchback = *v;
                true
            }
            StepParameter::EndPlayerAction(v) => {
                self.end_player_action = *v;
                true
            }
            StepParameter::EndTurn(v) => {
                self.end_turn = *v;
                true
            }
            _ => false,
        }
    }

    // Java: setParameter consume()s these keys — but ONLY when
    // game.getTurnMode() == TurnMode.KICKOFF_RETURN (TOUCHBACK is set WITHOUT consuming).
    //
    // This step is stack-resident for the whole kickoff, where the mode is NOT KickoffReturn and
    // Java does not consume; consuming unconditionally would eat END_TURN publishes meant for
    // steps below it. The old comment here said the guard was "not expressible" because
    // `consumes_parameter` has no `&Game`, and left it as never-consume on the grounds that the
    // headless port never enters KICKOFF_RETURN mode. It now does (amazon: On the Ball), and the
    // guard needs no game at all — the window is open exactly when THIS step opened it.
    //
    // Without consuming, the END_TURN that closes the window travels past this step, the exit
    // branch never fires, the mode stays KickoffReturn forever and the game plays no activations
    // at all: 486 driver iterations, 0 recorded steps, final score 0-0.
    fn consumes_parameter(&self, param: &StepParameter) -> bool {
        self.window_open
            && matches!(param, StepParameter::EndTurn(_) | StepParameter::EndPlayerAction(_))
    }
}

impl StepKickoffReturn {
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        if std::env::var_os("FFB_KR").is_some() {
            eprintln!(
                "KR enter mode={:?} end_turn={} end_pa={} window={} acted={}",
                game.turn_mode, self.end_turn, self.end_player_action, self.window_open,
                game.acting_player.has_acted
            );
        }
        if game.turn_mode == TurnMode::KickoffReturn {
            // Already inside the kickoff-return mini-turn.
            //
            // `acted()`, not the stored `has_acted`: Java's `ActingPlayer.hasActed()` is DERIVED
            // (`hasMoved() || hasFouled() || hasBlocked() || hasPassed() || ...`) and nothing in the
            // Java tree ever sets a stored flag. For the window mover the answer is the same either
            // way -- the harness deselects him before he moves (see `handle_move`) -- but the
            // derived form is the 1:1 one. ITER28 tried `acted()` ALONE, while the agent still
            // moved the window player, and it measured worse: with the mover having acted this
            // branch no longer fired, so the harness's window-closing EndTurn was never asked for
            // and the two step sequences parted. A correct component of a unit measured wrong on
            // its own; that is why the unit is gated together.
            if self.end_player_action && !game.acting_player.acted() && !self.end_turn {
                // Java: UtilServerSteps.changePlayerAction(this, null, null, false) -- the full
                // changeActingPlayer(null): MOVING -> STANDING (inactive if acted, PRONE if
                // standing up), end-of-turn enhancements dropped if not acted, then setPlayer(null).
                // Was a raw `player_id = None` that left every per-activation flag standing.
                crate::step::util_server_steps::change_player_action_to_none(game);
                // Java: `getGameState().pushCurrentStepOnStack()` + `Select.pushSequence(...)` --
                // the IDENTICAL idiom the window-open branch below uses: `push_self`, so the step
                // resumes BELOW the pushed sequence. (`repeat` re-ran it immediately and the Select
                // never got control -- ITER26.) `fEndPlayerAction` is NOT cleared here: Java's
                // field is assigned on every publish and this step only ever sees it on the publish
                // that carried it; the `push_self` semantics are what stop the ITER20 spin.
                let seq = Self::window_select_sequence(game);
                return StepOutcome::cont().push_self().push_seq(seq);
            } else if self.end_player_action || self.end_turn {
                // Java: UtilServerSteps.changePlayerAction(this, null, null, false)
                crate::step::util_server_steps::change_player_action_to_none(game);
                // Java: game.setHomePlaying(!game.isHomePlaying())
                game.home_playing = !game.home_playing;
                // Java: game.setTurnMode(TurnMode.KICKOFF)
                game.turn_mode = TurnMode::Kickoff;
                self.window_open = false;
                // Java: UtilPlayer.refreshPlayersForTurnStart(game)
                let mechanic = crate::mechanic::game_mechanic_for(game.rules);
                UtilPlayer::refresh_players_for_turn_start(game, &mechanic.enhancements_to_remove_at_end_of_turn(), &mechanic.enhancements_to_remove_at_end_of_turn_when_not_setting_active());
                // Java: game.getFieldModel().clearTrackNumbers()
                game.field_model.clear_track_numbers();
            }
        } else {
            // Java: determine receiving/kicking teams
            // home_playing = the kicking team is playing (kicking team sends the ball)
            let kickoff_return_team_ids: Vec<String> = if game.home_playing {
                game.team_away.players.iter().map(|p| p.id.clone()).collect()
            } else {
                game.team_home.players.iter().map(|p| p.id.clone()).collect()
            };

            let mut kickoff_return_player: Option<String> = None;
            let mut passive_players: Vec<String> = Vec::new();

            for pid in &kickoff_return_team_ids {
                let coord = match game.field_model.player_coordinate(pid) {
                    Some(c) if !c.is_box_coordinate() => c,
                    _ => continue,
                };

                // Java: player.hasSkillProperty(NamedProperties.canMoveDuringKickOffScatter)
                let has_property = game.team_home.players.iter()
                    .chain(game.team_away.players.iter())
                    .find(|p| p.id == *pid)
                    .map(|p| p.has_skill_property(NamedProperties::CAN_MOVE_DURING_KICK_OFF_SCATTER))
                    .unwrap_or(false);

                if has_property {
                    let los_bounds = if game.home_playing {
                        FieldCoordinateBounds::LOS_AWAY
                    } else {
                        FieldCoordinateBounds::LOS_HOME
                    };

                    if los_bounds.is_in_bounds(coord) {
                        passive_players.push(pid.clone());
                    } else {
                        let other_team = if game.home_playing { &game.team_home } else { &game.team_away };
                        let adj_opponents = UtilPlayer::find_adjacent_players_with_tacklezones(
                            game, other_team, coord, false,
                        );
                        if !adj_opponents.is_empty() {
                            passive_players.push(pid.clone());
                        } else {
                            kickoff_return_player = Some(pid.clone());
                        }
                    }
                } else {
                    passive_players.push(pid.clone());
                }
            }

            if kickoff_return_player.is_some() && !self.touchback {
                // Java: setPlayerState inactive for passive players
                for pid in &passive_players {
                    if let Some(ps) = game.field_model.player_state(pid) {
                        game.field_model.set_player_state(pid, ps.change_active(false));
                    }
                }
                // Java: game.setHomePlaying(!game.isHomePlaying())
                game.home_playing = !game.home_playing;
                // Java: game.setTurnMode(TurnMode.KICKOFF_RETURN)
                game.turn_mode = TurnMode::KickoffReturn;
                self.window_open = true;
                // Java: UtilServerDialog.showDialog(…, new DialogKickoffReturnParameter(), false)
                let eligible: Vec<String> = kickoff_return_player.into_iter().collect();
                // Java: `pushCurrentStepOnStack()` + `Select.pushSequence(...)`. That is
                // `push_self`, NOT `repeat`: the step must resume BELOW the pushed sequence, once
                // it finishes. `repeat` re-runs the step immediately instead, so the Select
                // sequence never gets control and the window never closes.
                let seq = Self::window_select_sequence(game);
                return StepOutcome::cont()
                    .with_prompt(AgentPrompt::KickoffReturn { eligible_players: eligible })
                    .push_self()
                    .push_seq(seq);
            }
        }

        // Java: getResult().setNextAction(StepAction.NEXT_STEP)
        StepOutcome::next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn start_with_no_kickoff_return_players_returns_next_step() {
        // No players on the field with CAN_MOVE_DURING_KICK_OFF_SCATTER -> no return player -> next step
        let mut game = make_game();
        let mut step = StepKickoffReturn::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_touchback_accepted() {
        let mut step = StepKickoffReturn::new();
        assert!(step.set_parameter(&StepParameter::Touchback(true)));
        assert!(step.touchback);
    }

    #[test]
    fn set_parameter_end_player_action_accepted() {
        let mut step = StepKickoffReturn::new();
        assert!(step.set_parameter(&StepParameter::EndPlayerAction(true)));
        assert!(step.end_player_action);
    }

    #[test]
    fn set_parameter_end_turn_accepted() {
        let mut step = StepKickoffReturn::new();
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.end_turn);
    }

    #[test]
    fn set_parameter_unrecognized_returns_false() {
        let mut step = StepKickoffReturn::new();
        assert!(!step.set_parameter(&StepParameter::NrOfDice(2)));
    }

    #[test]
    fn kickoff_return_mode_end_turn_flips_home_playing_and_sets_kickoff_mode() {
        let mut game = make_game();
        game.turn_mode = TurnMode::KickoffReturn;
        game.home_playing = true;
        let mut step = StepKickoffReturn::new();
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!game.home_playing);
        assert_eq!(game.turn_mode, TurnMode::Kickoff);
    }

    #[test]
    fn kickoff_return_mode_end_player_action_and_end_turn_flips_and_sets_kickoff() {
        let mut game = make_game();
        game.turn_mode = TurnMode::KickoffReturn;
        game.home_playing = false;
        let mut step = StepKickoffReturn::new();
        step.end_player_action = true;
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(game.home_playing);
        assert_eq!(game.turn_mode, TurnMode::Kickoff);
    }

    #[test]
    fn touchback_flag_prevents_kickoff_return_select() {
        let mut game = make_game();
        let mut step = StepKickoffReturn::new();
        step.touchback = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    // ── Written from the Java (StepKickoffReturn + UtilActingPlayer.changeActingPlayer), not
    //    from reading this file. ITER21's test asserted `Repeat` here and pinned a bug.

    fn game_with_mover(rules: Rules) -> Game {
        use ffb_model::enums::{PlayerState, PS_MOVING};
        use ffb_model::types::FieldCoordinate;
        let mut home = test_team("home", 0);
        home.players.push(ffb_model::model::player::Player { id: "h1".into(), nr: 1, ..Default::default() });
        let mut g = Game::new(home, test_team("away", 0), rules);
        g.turn_mode = TurnMode::KickoffReturn;
        g.home_playing = true;
        g.field_model.set_player_coordinate("h1", FieldCoordinate::new(5, 5));
        g.field_model.set_player_state("h1", PlayerState::new(PS_MOVING));
        g.acting_player.set_player("h1".into(), ffb_model::enums::PlayerAction::Move);
        g
    }

    /// Java: `if (fEndPlayerAction && !actingPlayer.hasActed() && !fEndTurn)` re-opens Select;
    /// `hasActed()` is DERIVED from hasMoved/hasFouled/hasBlocked/hasPassed/..., so a mover who
    /// moved does NOT re-open, whatever a stored flag says.
    #[test]
    fn reopen_only_when_the_mover_has_not_acted_derived() {
        let mut g = game_with_mover(Rules::Bb2025);
        let mut step = StepKickoffReturn::new();
        step.window_open = true;
        step.end_player_action = true;
        let out = step.start(&mut g, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue, "not acted: the window re-opens Select");
        assert!(out.push_self, "Java: pushCurrentStepOnStack()");
        assert_eq!(out.pushes.len(), 1);

        let mut g = game_with_mover(Rules::Bb2025);
        g.acting_player.has_moved = true;          // hasActed() == true; stored has_acted stays false
        assert!(!g.acting_player.has_acted);
        let mut step = StepKickoffReturn::new();
        step.window_open = true;
        step.end_player_action = true;
        let out = step.start(&mut g, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep, "acted: the window CLOSES");
        assert!(!out.push_self && out.pushes.is_empty());
        assert_eq!(g.turn_mode, TurnMode::Kickoff);
        assert!(!g.home_playing, "setHomePlaying(!isHomePlaying())");
    }

    /// Java closes the window through `changePlayerAction(null)`: the mover leaves MOVING and the
    /// acting player is fully reset -- not a bare id clear that leaves `hasMoved` standing.
    #[test]
    fn closing_the_window_runs_change_acting_player_null() {
        use ffb_model::enums::PS_MOVING;
        let mut g = game_with_mover(Rules::Bb2025);
        g.acting_player.has_moved = true;
        let mut step = StepKickoffReturn::new();
        step.window_open = true;
        step.end_player_action = true;
        step.start(&mut g, &mut GameRng::new(0));
        assert!(g.acting_player.player_id.is_none());
        assert!(!g.acting_player.has_moved, "setPlayer(null) resets the per-activation flags");
        assert_ne!(g.field_model.player_state("h1").unwrap().base(), PS_MOVING,
                   "changeActingPlayer(null) takes the old player out of MOVING");
    }

    /// Java pushes the edition's REAL Select generator (`Select.pushSequence(params, false)`),
    /// not a stub: bb2025 carries JUMP_UP/STAND_UP, bb2020 and bb2016 carry BONE_HEAD.
    #[test]
    fn window_select_is_the_editions_real_sequence() {
        for (rules, must_have) in [
            (Rules::Bb2025, StepId::StandUp),
            (Rules::Bb2020, StepId::BoneHead),
            (Rules::Bb2016, StepId::BoneHead),
        ] {
            let g = game_with_mover(rules);
            let seq = StepKickoffReturn::window_select_sequence(&g);
            assert_eq!(seq.first().map(|s| s.step_id), Some(StepId::InitSelecting), "{rules:?}");
            assert_eq!(seq.last().map(|s| s.step_id), Some(StepId::EndSelecting), "{rules:?}");
            assert!(seq.iter().any(|s| s.step_id == must_have), "{rules:?} lacks {must_have:?}");
            assert!(seq.iter().all(|s| s.step_id != StepId::NoOp), "{rules:?} still has the stub's NoOps");
            assert!(seq.iter().any(|s| s.label.as_deref() == Some("END_SELECTING")),
                    "{rules:?}: InitSelecting's GotoLabelOnEnd needs the END_SELECTING label");
        }
    }

    /// Java: `fEndPlayerAction = (Boolean) parameter.getValue()` on EVERY publish -- a published
    /// `false` clears it. No manual clearing anywhere in the step.
    #[test]
    fn end_player_action_follows_every_publish() {
        let mut step = StepKickoffReturn::new();
        assert!(step.set_parameter(&StepParameter::EndPlayerAction(true)));
        assert!(step.end_player_action);
        assert!(step.set_parameter(&StepParameter::EndPlayerAction(false)));
        assert!(!step.end_player_action);
    }
}
