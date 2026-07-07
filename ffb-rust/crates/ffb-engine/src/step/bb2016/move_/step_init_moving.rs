use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds};
use ffb_model::enums::{PlayerAction, SkillId};
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::report::mixed::report_fumblerooskie::ReportFumblerooskie;
use ffb_model::report::report_skill_use::ReportSkillUse;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepAction};
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::bb2016::KickTeamMate;
use crate::step::generator::bb2016::kick_team_mate::KickTeamMateParams;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.move.StepInitMoving.
///
/// Initialises the move sequence. Handles client move commands and dispatches
/// player actions.
///
/// Init params: GOTO_LABEL_ON_END (mandatory), GAZE_VICTIM_ID (optional),
///              MOVE_STACK (optional).
///
/// On start/execute:
/// - fEndTurn → publish END_TURN + GOTO_LABEL_ON_END
/// - fEndPlayerAction → publish END_PLAYER_ACTION + GOTO_LABEL_ON_END
/// - fGazeVictimId → setDefenderId + setPlayerAction(GAZE) + NEXT_STEP
/// - fKickedPlayerId → push KickTeamMate sequence + publish KICKED_PLAYER_ID + NR_OF_DICE + NEXT_STEP
/// - fMoveStack provided → publish COORDINATE_FROM / COORDINATE_TO / MOVE_STACK (shifted),
///   set dodging/goingForIt/hasMoved, update turnStarted, NEXT_STEP
///
/// no-op: UtilServerPlayerMove.isValidMove check — headless engine trusts agent-provided paths.
pub struct StepInitMoving {
    /// Java: fGotoLabelOnEnd (init param)
    pub goto_label_on_end: String,
    /// Java: fMoveStack
    pub move_stack: Vec<FieldCoordinate>,
    /// Java: fGazeVictimId
    pub gaze_victim_id: Option<String>,
    /// Java: fEndTurn
    pub end_turn: bool,
    /// Java: fEndPlayerAction
    pub end_player_action: bool,
    /// Java: fKickedPlayerId
    pub kicked_player_id: Option<String>,
    /// Java: fNumDice
    pub num_dice: i32,
}

impl StepInitMoving {
    pub fn new(goto_label_on_end: String) -> Self {
        Self {
            goto_label_on_end,
            move_stack: Vec::new(),
            gaze_victim_id: None,
            end_turn: false,
            end_player_action: false,
            kicked_player_id: None,
            num_dice: 0,
        }
    }
}

impl Default for StepInitMoving {
    fn default() -> Self { Self::new(String::new()) }
}

impl Step for StepInitMoving {
    fn id(&self) -> StepId { StepId::InitMoving }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let player_action = game.acting_player.player_action;
        match action {
            Action::Move { path } if !path.is_empty() => {
                if self.move_stack.is_empty() {
                    // Java: UtilServerPlayerMove.fetchMoveStack(moveCommand) → fMoveStack
                    self.move_stack = path.clone();
                    return self.execute_step(game, rng);
                }
            }
            Action::Block { .. } => {
                // Java: BLITZ_MOVE + !hasBlocked → dispatchPlayerAction(BLITZ)
                if player_action == Some(PlayerAction::BlitzMove)
                    && !game.acting_player.has_blocked
                    && self.move_stack.is_empty()
                {
                    return self.dispatch_player_action(PlayerAction::Blitz);
                }
            }
            Action::Foul { .. } => {
                if player_action == Some(PlayerAction::FoulMove)
                    && !game.acting_player.has_fouled
                    && self.move_stack.is_empty()
                {
                    return self.dispatch_player_action(PlayerAction::Foul);
                }
            }
            Action::HandOff { .. } => {
                if player_action == Some(PlayerAction::HandOverMove)
                    || player_action == Some(PlayerAction::HandOver)
                {
                    return self.dispatch_player_action(PlayerAction::HandOver);
                }
            }
            Action::Pass { .. } => {
                if player_action == Some(PlayerAction::PassMove)
                    || player_action == Some(PlayerAction::Pass)
                {
                    return self.dispatch_player_action(PlayerAction::Pass);
                }
                if player_action == Some(PlayerAction::HailMaryPass) {
                    return self.dispatch_player_action(PlayerAction::HailMaryPass);
                }
            }
            Action::ThrowTeamMate { .. } => {
                if player_action == Some(PlayerAction::ThrowTeamMateMove) && self.move_stack.is_empty() {
                    return self.dispatch_player_action(PlayerAction::ThrowTeamMate);
                }
            }
            Action::KickTeamMate { player_id, coord: _ } => {
                if player_action == Some(PlayerAction::KickTeamMateMove) && self.move_stack.is_empty() {
                    self.kicked_player_id = Some(player_id.clone());
                    // Java: fNumDice = kickTeamMateCommand.getNumDice()
                    return self.execute_step(game, rng);
                }
            }
            Action::HypnoticGaze { target_id } => {
                // Java: CLIENT_GAZE → fGazeVictimId = victimId, EXECUTE_STEP
                self.gaze_victim_id = Some(target_id.clone());
                return self.execute_step(game, rng);
            }
            // Java: CLIENT_USE_FUMBLEROOSKIE → ReportFumblerooskie(player.getId(), true)
            Action::UseSkill { skill_id: SkillId::Fumblerooskie, use_skill: true } => {
                let player_id = game.acting_player.player_id.clone();
                game.report_list.add(ReportFumblerooskie::new(player_id, true));
            }
            // Java: CLIENT_USE_SKILL → canAddBlockDie → ReportSkillUse(skill, true, ADD_BLOCK_DIE)
            Action::UseSkill { skill_id, use_skill: true } => {
                if skill_id.properties().contains(&NamedProperties::CAN_ADD_BLOCK_DIE) {
                    let player_id = game.acting_player.player_id.clone();
                    game.report_list.add(ReportSkillUse::new(
                        player_id,
                        *skill_id,
                        true,
                        SkillUse::ADD_BLOCK_DIE,
                    ));
                }
            }
            Action::EndTurn => {
                self.end_turn = true;
                return self.execute_step(game, rng);
            }
            _ => {}
        }
        // Still waiting
        StepOutcome::cont()
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(v) => { self.goto_label_on_end = v.clone(); true }
            StepParameter::MoveStack(v) => { self.move_stack = v.clone(); true }
            StepParameter::GazeVictimId(v) => { self.gaze_victim_id = v.clone(); true }
            _ => false,
        }
    }
}

impl StepInitMoving {
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        let label = self.goto_label_on_end.clone();

        if self.end_turn {
            return StepOutcome::goto(&label)
                .publish(StepParameter::EndTurn(true));
        }

        if self.end_player_action {
            return StepOutcome::goto(&label)
                .publish(StepParameter::EndPlayerAction(true));
        }

        if let Some(ref victim_id) = self.gaze_victim_id.clone() {
            // Java: game.setDefenderId(fGazeVictimId)
            game.defender_id = Some(victim_id.clone());
            game.acting_player.player_action = Some(PlayerAction::Gaze);
            return StepOutcome::next();
        }

        if let Some(ref kicked_id) = self.kicked_player_id.clone() {
            let seq = KickTeamMate::build_sequence(&KickTeamMateParams::default());
            return StepOutcome::next()
                .push_seq(seq)
                .publish(StepParameter::KickedPlayerId(Some(kicked_id.clone())))
                .publish(StepParameter::NrOfDice(self.num_dice));
        }

        if !self.move_stack.is_empty() {
            // Java: coordinateTo = fMoveStack[0]; newMoveStack = fMoveStack[1..]
            let coordinate_to = self.move_stack[0];
            let new_move_stack: Vec<FieldCoordinate> = self.move_stack[1..].to_vec();

            // Only proceed if coordinateTo is on the field
            if FieldCoordinateBounds::FIELD.is_in_bounds(coordinate_to) {
                let coordinate_from = game.acting_player.player_id.as_deref()
                    .and_then(|id| game.field_model.player_coordinate(id))
                    .unwrap_or(FieldCoordinate::new(0, 0));

                // Java: MoveSquare moveSquare = game.getFieldModel().getMoveSquare(coordinateTo);
                // Java: actingPlayer.setDodging((moveSquare != null) && moveSquare.isDodging() && !actingPlayer.isJumping());
                // Java: actingPlayer.setGoingForIt((moveSquare != null) && moveSquare.isGoingForIt());
                let move_square = game.field_model.get_move_square(coordinate_to);
                game.acting_player.dodging = move_square
                    .map(|ms| ms.is_dodging() && !game.acting_player.jumping)
                    .unwrap_or(false);
                game.acting_player.goes_for_it = move_square
                    .map(|ms| ms.is_going_for_it())
                    .unwrap_or(false);
                game.acting_player.has_moved = true;

                game.turn_data_mut().turn_started = true;

                match game.acting_player.player_action {
                    Some(PlayerAction::BlitzMove) | Some(PlayerAction::KickTeamMateMove) => {
                        game.turn_data_mut().blitz_used = true;
                    }
                    Some(PlayerAction::FoulMove) => {
                        game.turn_data_mut().foul_used = true;
                    }
                    Some(PlayerAction::HandOverMove) => {
                        game.turn_data_mut().hand_over_used = true;
                    }
                    Some(PlayerAction::PassMove) | Some(PlayerAction::ThrowTeamMateMove) => {
                        game.turn_data_mut().pass_used = true;
                    }
                    _ => {}
                }

                game.concession_possible = false;

                return StepOutcome::next()
                    .publish(StepParameter::MoveStack(new_move_stack))
                    .publish(StepParameter::CoordinateFrom(coordinate_from))
                    .publish(StepParameter::CoordinateTo(coordinate_to));
            }
        }

        // Still waiting for a command
        StepOutcome::cont()
    }

    fn dispatch_player_action(&self, action: PlayerAction) -> StepOutcome {
        // Java: publishParameter(DISPATCH_PLAYER_ACTION, pPlayerAction)
        //       setNextAction(GOTO_LABEL_AND_REPEAT, fGotoLabelOnEnd)
        let label = self.goto_label_on_end.clone();
        StepOutcome {
            action: StepAction::GotoLabelAndRepeat,
            goto_label: Some(label),
            published: vec![StepParameter::DispatchPlayerAction(Some(action))],
            pushes: vec![],
            events: vec![],
            prompt: None,
            clear_stack: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2016)
    }

    #[test]
    fn start_with_no_move_stack_waits_for_command() {
        let mut game = make_game();
        let mut step = StepInitMoving::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn end_turn_publishes_and_gotos_label() {
        let mut game = make_game();
        let mut step = StepInitMoving::new("end".into());
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn end_player_action_publishes_and_gotos_label() {
        let mut game = make_game();
        let mut step = StepInitMoving::new("end".into());
        step.end_player_action = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))));
    }

    #[test]
    fn gaze_victim_sets_defender_and_returns_next_step() {
        let mut game = make_game();
        let mut step = StepInitMoving::new("end".into());
        step.gaze_victim_id = Some("victim".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.defender_id.as_deref(), Some("victim"));
    }

    #[test]
    fn move_stack_with_one_coord_publishes_from_to() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        let mut step = StepInitMoving::new("end".into());
        step.move_stack = vec![FieldCoordinate::new(6, 5)];
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::CoordinateTo(_))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::CoordinateFrom(_))));
    }

    #[test]
    fn move_stack_is_shifted_when_published() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        let mut step = StepInitMoving::new("end".into());
        let remaining = FieldCoordinate::new(7, 5);
        step.move_stack = vec![FieldCoordinate::new(6, 5), remaining];
        let out = step.start(&mut game, &mut GameRng::new(0));
        let stack_param = out.published.iter().find(|p| matches!(p, StepParameter::MoveStack(_)));
        if let Some(StepParameter::MoveStack(s)) = stack_param {
            assert_eq!(s.len(), 1);
            assert_eq!(s[0], remaining);
        } else {
            panic!("MoveStack not published");
        }
    }

    #[test]
    fn set_parameter_move_stack_accepted() {
        let mut step = StepInitMoving::new("end".into());
        let stack = vec![FieldCoordinate::new(5, 5)];
        assert!(step.set_parameter(&StepParameter::MoveStack(stack.clone())));
        assert_eq!(step.move_stack, stack);
    }

    #[test]
    fn set_parameter_goto_label_on_end_accepted() {
        let mut step = StepInitMoving::new("old".into());
        assert!(step.set_parameter(&StepParameter::GotoLabelOnEnd("new".into())));
        assert_eq!(step.goto_label_on_end, "new");
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepInitMoving::new("end".into());
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn end_turn_command_publishes_and_gotos() {
        let mut game = make_game();
        let mut step = StepInitMoving::new("end".into());
        let out = step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    fn make_game_with_player_at(coord: FieldCoordinate, action: PlayerAction) -> Game {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.field_model.set_player_coordinate("p1", coord);
        game.acting_player.player_action = Some(action);
        game
    }

    fn move_step_to(dest: FieldCoordinate, action: PlayerAction) -> (Game, StepOutcome) {
        let from = FieldCoordinate::new(5, 5);
        let mut game = make_game_with_player_at(from, action);
        let mut step = StepInitMoving::new("end".into());
        step.move_stack = vec![dest];
        let out = step.start(&mut game, &mut GameRng::new(0));
        (game, out)
    }

    #[test]
    fn move_sets_turn_started_and_concession_possible_false() {
        let dest = FieldCoordinate::new(6, 5);
        let (game, out) = move_step_to(dest, PlayerAction::Move);
        assert_eq!(out.action, StepAction::NextStep);
        assert!(game.turn_data().turn_started);
        assert!(!game.concession_possible);
    }

    #[test]
    fn blitz_move_sets_blitz_used() {
        let dest = FieldCoordinate::new(6, 5);
        let (game, out) = move_step_to(dest, PlayerAction::BlitzMove);
        assert_eq!(out.action, StepAction::NextStep);
        assert!(game.turn_data().blitz_used);
        assert!(!game.turn_data().foul_used);
        assert!(!game.turn_data().pass_used);
        assert!(!game.turn_data().hand_over_used);
    }

    #[test]
    fn kick_team_mate_move_sets_blitz_used() {
        let dest = FieldCoordinate::new(6, 5);
        let (game, out) = move_step_to(dest, PlayerAction::KickTeamMateMove);
        assert_eq!(out.action, StepAction::NextStep);
        assert!(game.turn_data().blitz_used);
    }

    #[test]
    fn foul_move_sets_foul_used() {
        let dest = FieldCoordinate::new(6, 5);
        let (game, out) = move_step_to(dest, PlayerAction::FoulMove);
        assert_eq!(out.action, StepAction::NextStep);
        assert!(game.turn_data().foul_used);
        assert!(!game.turn_data().blitz_used);
        assert!(!game.turn_data().pass_used);
        assert!(!game.turn_data().hand_over_used);
    }

    #[test]
    fn pass_move_sets_pass_used() {
        let dest = FieldCoordinate::new(6, 5);
        let (game, out) = move_step_to(dest, PlayerAction::PassMove);
        assert_eq!(out.action, StepAction::NextStep);
        assert!(game.turn_data().pass_used);
        assert!(!game.turn_data().blitz_used);
        assert!(!game.turn_data().foul_used);
        assert!(!game.turn_data().hand_over_used);
    }

    #[test]
    fn throw_team_mate_move_sets_pass_used() {
        let dest = FieldCoordinate::new(6, 5);
        let (game, out) = move_step_to(dest, PlayerAction::ThrowTeamMateMove);
        assert_eq!(out.action, StepAction::NextStep);
        assert!(game.turn_data().pass_used);
    }

    #[test]
    fn hand_over_move_sets_hand_over_used() {
        let dest = FieldCoordinate::new(6, 5);
        let (game, out) = move_step_to(dest, PlayerAction::HandOverMove);
        assert_eq!(out.action, StepAction::NextStep);
        assert!(game.turn_data().hand_over_used);
        assert!(!game.turn_data().blitz_used);
        assert!(!game.turn_data().foul_used);
        assert!(!game.turn_data().pass_used);
    }

    #[test]
    fn move_to_dodge_square_sets_dodging_flag() {
        use ffb_model::types::MoveSquare;
        let dodge_coord = FieldCoordinate::new(6, 5);
        let from = FieldCoordinate::new(5, 5);
        let mut game = make_game_with_player_at(from, PlayerAction::Move);
        game.field_model.add_move_square(MoveSquare::new(dodge_coord, 3, 0));
        let mut step = StepInitMoving::new("end".into());
        step.move_stack = vec![dodge_coord];
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.acting_player.dodging, "setDodging should be true for dodge square");
        assert!(!game.acting_player.goes_for_it);
    }

    #[test]
    fn move_to_gfi_square_sets_goes_for_it_flag() {
        use ffb_model::types::MoveSquare;
        let gfi_coord = FieldCoordinate::new(6, 5);
        let from = FieldCoordinate::new(5, 5);
        let mut game = make_game_with_player_at(from, PlayerAction::Move);
        game.field_model.add_move_square(MoveSquare::new(gfi_coord, 0, 2));
        let mut step = StepInitMoving::new("end".into());
        step.move_stack = vec![gfi_coord];
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.acting_player.dodging);
        assert!(game.acting_player.goes_for_it, "setGoingForIt should be true for GFI square");
    }

    // ── report_list: Fumblerooskie and ADD_BLOCK_DIE ─────────────────────────

    #[test]
    fn use_fumblerooskie_skill_adds_fumblerooskie_report() {
        use ffb_model::report::report_id::ReportId;
        use ffb_model::enums::SkillId;
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        let mut step = StepInitMoving::new("end".into());
        step.handle_command(
            &Action::UseSkill { skill_id: SkillId::Fumblerooskie, use_skill: true },
            &mut game,
            &mut GameRng::new(0),
        );
        assert!(game.report_list.has_report(ReportId::FUMBLEROOSKIE),
            "expected FUMBLEROOSKIE report when Fumblerooskie skill is used");
    }

    #[test]
    fn use_skill_without_can_add_block_die_does_not_add_skill_use_report() {
        use ffb_model::report::report_id::ReportId;
        use ffb_model::enums::SkillId;
        // Dodge does NOT have CAN_ADD_BLOCK_DIE — no SKILL_USE report should be added
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        let mut step = StepInitMoving::new("end".into());
        step.handle_command(
            &Action::UseSkill { skill_id: SkillId::Dodge, use_skill: true },
            &mut game,
            &mut GameRng::new(0),
        );
        assert!(!game.report_list.has_report(ReportId::SKILL_USE),
            "Dodge does not have CAN_ADD_BLOCK_DIE — no SKILL_USE report should be added");
    }

    #[test]
    fn dodge_suppressed_when_jumping() {
        use ffb_model::types::MoveSquare;
        let dodge_coord = FieldCoordinate::new(6, 5);
        let from = FieldCoordinate::new(5, 5);
        let mut game = make_game_with_player_at(from, PlayerAction::Move);
        game.field_model.add_move_square(MoveSquare::new(dodge_coord, 3, 0));
        game.acting_player.jumping = true;
        let mut step = StepInitMoving::new("end".into());
        step.move_stack = vec![dodge_coord];
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.acting_player.dodging, "dodging suppressed while jumping");
    }
}
