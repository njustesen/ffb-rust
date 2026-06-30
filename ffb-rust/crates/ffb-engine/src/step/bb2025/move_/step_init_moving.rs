use ffb_model::enums::PlayerAction;
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepAction, StepId, StepParameter};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.move.StepInitMoving.
///
/// Initialises a move action: decodes the move command, publishes
/// COORDINATE_FROM / COORDINATE_TO / MOVE_STACK, dispatches block/foul/pass/gaze
/// sub-sequences via GOTO_LABEL_ON_END + DISPATCH_PLAYER_ACTION.
///
/// Init params: GOTO_LABEL_ON_END (mandatory), GAZE_VICTIM_ID (optional),
///              MOVE_STACK (optional), BALL_AND_CHAIN_RE_ROLL_SETTING (optional).
///
/// Command dispatch (Move/Block/Foul/Pass/HandOff/ThrowTeamMate/KickTeamMate/Gaze/EndTurn) ported.
/// CLIENT_USE_FUMBLEROOSKIE / CLIENT_USE_SKILL (canAddBlockDie) not yet ported.
/// TODO: UtilServerPlayerMove.isValidMove path validation not yet ported (agent paths trusted).
/// TODO: getMoveSquare → actingPlayer.setDodging/setGoingForIt not yet ported.
/// TODO: commitTargetSelection not yet ported.
/// setTurnStarted, concessionPossible, per-action TurnData flags are wired.
pub struct StepInitMoving {
    /// Java: fGotoLabelOnEnd
    pub goto_label_on_end: String,
    /// Java: fMoveStack
    pub move_stack: Vec<FieldCoordinate>,
    /// Java: fGazeVictimId
    pub gaze_victim_id: Option<String>,
    /// Java: ballAndChainRrSetting
    pub ball_and_chain_rr_setting: Option<String>,
    /// Java: fEndTurn
    pub end_turn: bool,
    /// Java: fEndPlayerAction
    pub end_player_action: bool,
}

impl StepInitMoving {
    pub fn new(goto_label_on_end: String) -> Self {
        Self {
            goto_label_on_end,
            move_stack: Vec::new(),
            gaze_victim_id: None,
            ball_and_chain_rr_setting: None,
            end_turn: false,
            end_player_action: false,
        }
    }
}

impl Step for StepInitMoving {
    fn id(&self) -> StepId { StepId::InitMoving }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let player_action = game.acting_player.player_action;
        let has_blocked = game.acting_player.has_blocked;
        let has_fouled = game.acting_player.has_fouled;

        match action {
            // Java: CLIENT_MOVE / CLIENT_BLITZ_MOVE — agent provides the path to move through
            // UtilServerPlayerMove.isValidMove + fetchMoveStack not ported; trust agent path
            Action::Move { path } if !path.is_empty() => {
                if self.move_stack.is_empty() {
                    self.move_stack = path.clone();
                }
                return self.execute_step(game, rng);
            }

            // Java: CLIENT_BLOCK → dispatchPlayerAction(BLITZ/KICK_EM_BLITZ)
            // Guard: (BLITZ_MOVE || KICK_EM_BLITZ) && !hasBlocked || PUTRID_REGURGITATION_BLITZ
            Action::Block { .. } => {
                let is_blitz_dispatch = matches!(player_action,
                    Some(PlayerAction::BlitzMove) | Some(PlayerAction::KickEmBlitz)
                    | Some(PlayerAction::PutridRegurgitationBlitz))
                    && (!has_blocked || matches!(player_action, Some(PlayerAction::PutridRegurgitationBlitz)));
                if is_blitz_dispatch {
                    let dispatch = if player_action == Some(PlayerAction::KickEmBlitz) {
                        PlayerAction::KickEmBlitz
                    } else {
                        PlayerAction::Blitz
                    };
                    return self.dispatch_player_action(dispatch)
                        .publish(StepParameter::UsingChainsaw(false));
                }
            }

            // Java: CLIENT_FOUL → dispatchPlayerAction(FOUL)
            // Guard: FOUL_MOVE && !hasFouled
            Action::Foul { .. } => {
                if player_action == Some(PlayerAction::FoulMove) && !has_fouled {
                    return self.dispatch_player_action(PlayerAction::Foul);
                }
            }

            // Java: CLIENT_HAND_OVER → dispatchPlayerAction(HAND_OVER)
            // Guard: HAND_OVER_MOVE || HAND_OVER
            Action::HandOff { .. } => {
                if matches!(player_action, Some(PlayerAction::HandOverMove) | Some(PlayerAction::HandOver)) {
                    return self.dispatch_player_action(PlayerAction::HandOver);
                }
            }

            // Java: CLIENT_PASS → dispatchPlayerAction(PASS or HAIL_MARY_PASS)
            // Guard: PASS_MOVE || PASS → PASS; HAIL_MARY_PASS → HAIL_MARY_PASS
            Action::Pass { .. } => {
                match player_action {
                    Some(PlayerAction::PassMove) | Some(PlayerAction::Pass) => {
                        return self.dispatch_player_action(PlayerAction::Pass);
                    }
                    Some(PlayerAction::HailMaryPass) => {
                        return self.dispatch_player_action(PlayerAction::HailMaryPass);
                    }
                    _ => {}
                }
            }

            // Java: CLIENT_THROW_TEAM_MATE → dispatchPlayerAction(THROW_TEAM_MATE or KICK_TEAM_MATE)
            // Guard: THROW_TEAM_MATE_MOVE || KICK_TEAM_MATE_MOVE
            Action::ThrowTeamMate { player_id, .. } => {
                if matches!(player_action, Some(PlayerAction::ThrowTeamMateMove)) {
                    let pid = player_id.clone();
                    return self.dispatch_player_action(PlayerAction::ThrowTeamMate)
                        .publish(StepParameter::ThrownPlayerId(Some(pid)));
                }
            }
            Action::KickTeamMate { player_id, .. } => {
                if matches!(player_action, Some(PlayerAction::KickTeamMateMove)) {
                    let pid = player_id.clone();
                    return self.dispatch_player_action(PlayerAction::KickTeamMate)
                        .publish(StepParameter::ThrownPlayerId(Some(pid)));
                }
            }

            // Java: CLIENT_GAZE → fGazeVictimId = victimId, EXECUTE_STEP
            Action::HypnoticGaze { target_id } => {
                self.gaze_victim_id = Some(target_id.clone());
                return self.execute_step(game, rng);
            }

            // Java: CLIENT_END_TURN → fEndTurn = true, EXECUTE_STEP
            Action::EndTurn => {
                self.end_turn = true;
                return self.execute_step(game, rng);
            }

            // Java: CLIENT_ACTING_PLAYER with no player_id → fEndPlayerAction = true, EXECUTE_STEP
            // In Rust this could arrive as some kind of "end action" signal — leave as fallthrough
            _ => {}
        }

        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(v) => { self.goto_label_on_end = v.clone(); true }
            StepParameter::MoveStack(v) => { self.move_stack = v.clone(); true }
            StepParameter::GazeVictimId(v) => { self.gaze_victim_id = v.clone(); true }
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
            StepParameter::EndPlayerAction(v) => { self.end_player_action = *v; true }
            StepParameter::BallAndChainRrSetting(v) => { self.ball_and_chain_rr_setting = v.clone(); true }
            _ => false,
        }
    }
}

impl StepInitMoving {
    /// Java: dispatchPlayerAction(pPlayerAction) — publish DISPATCH_PLAYER_ACTION + GOTO_LABEL_ON_END.
    fn dispatch_player_action(&self, action: PlayerAction) -> StepOutcome {
        StepOutcome::goto(&self.goto_label_on_end)
            .publish(StepParameter::DispatchPlayerAction(Some(action)))
    }

    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: if (fEndTurn) → publish END_TURN + CHECK_FORGO, GOTO fGotoLabelOnEnd
        if self.end_turn {
            let label = self.goto_label_on_end.clone();
            return StepOutcome::goto(&label)
                .publish(StepParameter::EndTurn(true))
                .publish(StepParameter::CheckForgo(true));
        }
        // Java: else if (fEndPlayerAction) → publish END_PLAYER_ACTION, GOTO fGotoLabelOnEnd
        if self.end_player_action {
            let label = self.goto_label_on_end.clone();
            return StepOutcome::goto(&label)
                .publish(StepParameter::EndPlayerAction(true));
        }
        // Java: else if (StringTool.isProvided(fGazeVictimId)) → setDefenderId, setPlayerAction(GAZE), NEXT_STEP
        if self.gaze_victim_id.is_some() {
            game.acting_player.player_action = Some(ffb_model::enums::PlayerAction::Gaze);
            game.defender_id = self.gaze_victim_id.clone();
            return StepOutcome::next();
        }
        // Java: publish BALL_AND_CHAIN_RE_ROLL_SETTING
        // Java: if (ArrayTool.isProvided(fMoveStack)) → pop first coord, publish COORDINATE_FROM/TO/MOVE_STACK
        if !self.move_stack.is_empty() {
            let coordinate_to = self.move_stack[0];
            let new_stack: Vec<FieldCoordinate> = self.move_stack[1..].to_vec();
            self.move_stack = new_stack.clone();

            let coordinate_from = game.acting_player.player_id.as_deref()
                .and_then(|id| game.field_model.player_coordinate(id))
                .unwrap_or(FieldCoordinate::new(0, 0));

            if !FieldCoordinateBounds::FIELD.is_in_bounds(coordinate_to) {
                return StepOutcome::cont();
            }
            // TODO: getMoveSquare → actingPlayer.setDodging / setGoingForIt / setHasMoved
            // TODO: commitTargetSelection
            game.acting_player.has_moved = true;
            game.turn_data_mut().turn_started = true;
            // Java: per-PlayerAction TurnData flags
            let player_action = game.acting_player.player_action;
            use ffb_model::enums::PlayerAction;
            use ffb_model::model::property::named_properties::NamedProperties;
            match player_action {
                Some(PlayerAction::BlitzMove) | Some(PlayerAction::KickEmBlitz) => {
                    game.turn_data_mut().blitz_used = true;
                }
                Some(PlayerAction::FoulMove) => {
                    let allows_extra = game.acting_player.player_id.as_deref()
                        .and_then(|id| game.player(id))
                        .map(|p| p.has_skill_property(NamedProperties::ALLOWS_ADDITIONAL_FOUL))
                        .unwrap_or(false);
                    if !allows_extra {
                        game.turn_data_mut().foul_used = true;
                    }
                }
                Some(PlayerAction::HandOverMove) => {
                    game.turn_data_mut().hand_over_used = true;
                }
                Some(PlayerAction::PassMove) => {
                    game.turn_data_mut().pass_used = true;
                }
                Some(PlayerAction::ThrowTeamMateMove) => {
                    game.turn_data_mut().ttm_used = true;
                }
                Some(PlayerAction::KickTeamMateMove) => {
                    game.turn_data_mut().ktm_used = true;
                }
                Some(PlayerAction::SecureTheBall) => {
                    game.turn_data_mut().secure_the_ball_used = true;
                }
                Some(PlayerAction::PuntMove) | Some(PlayerAction::Punt) => {
                    game.turn_data_mut().punt_used = true;
                }
                _ => {}
            }
            game.concession_possible = false;

            return StepOutcome {
                action: StepAction::NextStep,
                goto_label: None,
                events: Vec::new(),
                pushes: Vec::new(),
                prompt: None,
                published: vec![
                    StepParameter::BallAndChainRrSetting(self.ball_and_chain_rr_setting.clone()),
                    StepParameter::MoveStack(new_stack),
                    StepParameter::CoordinateFrom(coordinate_from),
                    StepParameter::CoordinateTo(coordinate_to),
                ],
            };
        }
        // Empty move stack — wait for client command
        StepOutcome::cont()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepParameter;
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn end_turn_goes_to_label_with_end_turn_and_check_forgo() {
        let mut game = make_game();
        let mut step = StepInitMoving::new("end".into());
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end"));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::CheckForgo(true))));
    }

    #[test]
    fn end_player_action_goes_to_label_with_end_player_action() {
        let mut game = make_game();
        let mut step = StepInitMoving::new("end".into());
        step.end_player_action = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end"));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))));
    }

    #[test]
    fn empty_move_stack_returns_continue() {
        let mut game = make_game();
        let mut step = StepInitMoving::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn move_stack_pops_first_coord_and_publishes_coordinate_to() {
        let mut game = make_game();
        let mut step = StepInitMoving::new("end".into());
        let sq1 = FieldCoordinate::new(5, 3);
        let sq2 = FieldCoordinate::new(6, 3);
        step.move_stack = vec![sq1, sq2];
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        let has_to = out.published.iter().any(|p| matches!(p, StepParameter::CoordinateTo(c) if *c == sq1));
        assert!(has_to, "CoordinateTo(sq1) must be published");
        let remaining = out.published.iter().find_map(|p| {
            if let StepParameter::MoveStack(v) = p { Some(v.clone()) } else { None }
        }).unwrap();
        assert_eq!(remaining, vec![sq2]);
    }

    #[test]
    fn set_parameter_end_turn_accepted() {
        let mut step = StepInitMoving::new("end".into());
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.end_turn);
    }

    #[test]
    fn gaze_victim_causes_next_step_and_sets_gaze_action() {
        let mut game = make_game();
        let mut step = StepInitMoving::new("end".into());
        step.gaze_victim_id = Some("p1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.acting_player.player_action, Some(ffb_model::enums::PlayerAction::Gaze));
    }

    // ── handle_command dispatch tests ────────────────────────────────────────

    #[test]
    fn handle_command_move_sets_move_stack_and_executes() {
        let mut game = make_game();
        let mut step = StepInitMoving::new("end".into());
        let path = vec![FieldCoordinate::new(5, 5), FieldCoordinate::new(6, 5)];
        let action = crate::action::Action::Move { path };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        // execute_step processes move_stack → NextStep
        assert_eq!(out.action, StepAction::NextStep);
        let has_coord_to = out.published.iter().any(|p| matches!(p, StepParameter::CoordinateTo(_)));
        assert!(has_coord_to, "CoordinateTo should be published after Move command");
    }

    #[test]
    fn handle_command_end_turn_goes_to_label() {
        let mut game = make_game();
        let mut step = StepInitMoving::new("end".into());
        let out = step.handle_command(&crate::action::Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end"));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn handle_command_block_on_blitz_move_dispatches_blitz() {
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::BlitzMove);
        game.acting_player.has_blocked = false;
        let mut step = StepInitMoving::new("end".into());
        let action = crate::action::Action::Block { defender_id: "def1".into() };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end"));
        let has_dispatch = out.published.iter().any(|p| matches!(p, StepParameter::DispatchPlayerAction(Some(PlayerAction::Blitz))));
        assert!(has_dispatch, "should publish DispatchPlayerAction(Blitz)");
    }

    #[test]
    fn handle_command_block_not_dispatched_when_already_blocked() {
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::BlitzMove);
        game.acting_player.has_blocked = true;
        let mut step = StepInitMoving::new("end".into());
        let action = crate::action::Action::Block { defender_id: "def1".into() };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        // Falls through to execute_step → cont (empty move_stack)
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn handle_command_foul_on_foul_move_dispatches_foul() {
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::FoulMove);
        game.acting_player.has_fouled = false;
        let mut step = StepInitMoving::new("end".into());
        let action = crate::action::Action::Foul { target_id: "def1".into() };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        let has_dispatch = out.published.iter().any(|p| matches!(p, StepParameter::DispatchPlayerAction(Some(PlayerAction::Foul))));
        assert!(has_dispatch, "should publish DispatchPlayerAction(Foul)");
    }

    #[test]
    fn handle_command_pass_on_pass_move_dispatches_pass() {
        use ffb_model::types::FieldCoordinate;
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::PassMove);
        let mut step = StepInitMoving::new("end".into());
        let action = crate::action::Action::Pass { coord: FieldCoordinate::new(10, 10) };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        let has_dispatch = out.published.iter().any(|p| matches!(p, StepParameter::DispatchPlayerAction(Some(PlayerAction::Pass))));
        assert!(has_dispatch, "should publish DispatchPlayerAction(Pass)");
    }

    #[test]
    fn handle_command_gaze_sets_victim_and_executes() {
        let mut game = make_game();
        let mut step = StepInitMoving::new("end".into());
        let action = crate::action::Action::HypnoticGaze { target_id: "victim1".into() };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        // execute_step with gaze_victim_id set → NextStep
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(step.gaze_victim_id.as_deref(), Some("victim1"));
    }
}
