use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::report::report_skill_use::ReportSkillUse;
use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{CatchScatterThrowInMode, StepId, StepParameter};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.shared.StepPlaceBall.
/// Places the ball at a player/coordinate; handles the Safe Pair of Hands skill dialog.
/// The dialog (Phase::Select/Place) is not yet fully ported — skill use auto-declines.
pub struct StepPlaceBall {
    /// Java: playerId
    pub player_id: Option<String>,
    /// Java: catchScatterThrowInMode
    pub catch_scatter_throw_in_mode: Option<CatchScatterThrowInMode>,
    /// Java: phase (Phase enum) — stored as name until Phase enum is ported
    pub phase_name: String,
    /// Java: ballCarrierTeamTurn
    pub ball_carrier_team_turn: bool,
    /// Java: revertEndTurn
    pub revert_end_turn: bool,
    /// Java: selectedCoordinate
    pub selected_coordinate: Option<FieldCoordinate>,
    /// Java: adjacentSquares — the free squares next to the ball (FieldCoordinateBounds.FIELD),
    /// kept coordinate-sorted (Java's HashSet order never reaches the outcome: the coach names
    /// one square, and both agents sort before picking).
    pub adjacent_squares: Vec<FieldCoordinate>,
}

impl StepPlaceBall {
    pub fn new() -> Self {
        Self {
            player_id: None,
            catch_scatter_throw_in_mode: None,
            phase_name: "ASK".to_string(),
            // true = no homePlaying flip pending; setup() overwrites when it shows the dialog.
            ball_carrier_team_turn: true,
            revert_end_turn: false,
            selected_coordinate: None,
            adjacent_squares: Vec::new(),
        }
    }
}

impl Default for StepPlaceBall {
    fn default() -> Self { Self::new() }
}

impl Step for StepPlaceBall {
    fn id(&self) -> StepId { StepId::PlaceBall }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java CLIENT_USE_SKILL (canPlaceBallWhenKnockedDownOrPlacedProne): used → Phase::SELECT
        // (the coach place dialog — unreachable under the parity contract, both agents decline);
        // declined → Phase::DONE → leave(), publishing DROPPED_BALL_CARRIER = null, with a
        // ReportSkillUse(PLACE_BALL) either way.
        match action {
            Action::UseSkill { skill_id, use_skill } => {
                let has_prop = skill_id.properties().contains(&NamedProperties::CAN_PLACE_BALL_WHEN_KNOCKED_DOWN_OR_PLACED_PRONE);
                if has_prop {
                    game.report_list.add(ReportSkillUse::new(
                        self.player_id.clone(), *skill_id, *use_skill, SkillUse::PLACE_BALL,
                    ));
                    // Java: phase = used ? SELECT : DONE, then executeStep.
                    self.phase_name = if *use_skill { "SELECT" } else { "DONE" }.to_string();
                }
                self.execute_step(game, rng)
            }
            // Java CLIENT_FIELD_COORDINATE: accepted only when it names one of adjacentSquares
            // (real board coordinates here; Java un-mirrors the away coach's command).
            Action::PlaceBallSquare { coord } => {
                if self.adjacent_squares.contains(coord) {
                    self.selected_coordinate = Some(*coord);
                    self.phase_name = "PLACE".to_string();
                    return self.execute_step(game, rng);
                }
                StepOutcome::cont()
            }
            _ => self.execute_step(game, rng),
        }
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            // Java setParameter: `case DROPPED_BALL_CARRIER: playerId = (String) value` — the
            // carrier id ARRIVES in this parameter (published by UtilServerInjury.dropPlayer for
            // an eligible Safe-Pair-of-Hands carrier). The old arm consumed it and threw it away
            // while waiting for a PLAYER_ID Java never sends, so player_id stayed None and the
            // skill dialog never fired (chaos_pact bb2020 seed 8 i=46: the KO'd Renegade Thrower).
            StepParameter::DroppedBallCarrier(v) => { self.player_id = v.clone(); true }
            StepParameter::CatchScatterThrowInMode(v) => { self.catch_scatter_throw_in_mode = Some(*v); true }
            // Java bb2025 only (bb2020 has no such arm): `case REVERT_END_TURN`.
            StepParameter::RevertEndTurn(v) => { self.revert_end_turn = *v; true }
            _ => false,
        }
    }
}

impl StepPlaceBall {
    /// Java: leave() — (bb2025: clear the move squares,) publish DROPPED_BALL_CARRIER = null
    /// and, when the dialog ran on the carrier's (non-active) side, flip homePlaying back.
    fn leave(&self, game: &mut Game) -> StepOutcome {
        self.leave_with(game, StepOutcome::next())
    }

    fn leave_with(&self, game: &mut Game, out: StepOutcome) -> StepOutcome {
        if game.rules != ffb_model::enums::Rules::Bb2020 {
            game.field_model.clear_move_squares();
        }
        if !self.ball_carrier_team_turn {
            game.home_playing = !game.home_playing;
        }
        out.publish(StepParameter::DroppedBallCarrier(None))
    }

    /// Java: executeStep().
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: if (playerId == null || catchScatterThrowInMode != SCATTER_BALL) → NEXT_STEP
        let player_id = match self.player_id.as_deref() {
            Some(id) if self.catch_scatter_throw_in_mode == Some(CatchScatterThrowInMode::ScatterBall) => id,
            _ => return StepOutcome::next().publish(StepParameter::DroppedBallCarrier(None)),
        };
        let player_id = player_id.to_string();
        match self.phase_name.as_str() {
            // Java Phase.SELECT: publish adjacentSquares as MoveSquares, TurnMode
            // SAFE_PAIR_OF_HANDS, and wait for the coach's square.
            "SELECT" => {
                game.field_model.clear_move_squares();
                for c in &self.adjacent_squares {
                    game.field_model.add_move_square(ffb_model::types::MoveSquare {
                        coordinate: *c, minimum_roll_dodge: 0, minimum_roll_gfi: 0 });
                }
                game.turn_mode = ffb_model::enums::TurnMode::SafePairOfHands;
                return StepOutcome::cont().with_prompt(ffb_model::prompts::AgentPrompt::PlaceBallSquare {
                    player_id,
                    squares: self.adjacent_squares.clone(),
                });
            }
            // Java Phase.PLACE.
            "PLACE" => {
                if !self.ball_carrier_team_turn {
                    game.waiting_for_opponent = true;
                }
                let selected = self.selected_coordinate.expect("PLACE requires a selected square");
                let direction = game.field_model.ball_coordinate.and_then(|b| b.direction_to(selected));
                game.report_list.add(
                    ffb_model::report::mixed::report_place_ball_direction::ReportPlaceBallDirection::new(
                        Some(player_id.clone()), direction));
                game.field_model.ball_coordinate = Some(selected);
                game.field_model.ball_moving = true;
                if let Some(last) = game.last_turn_mode {
                    game.turn_mode = last;
                }
                let mut out = StepOutcome::next().publish(StepParameter::CatchScatterThrowInModeCleared);
                if self.revert_end_turn && game.rules != ffb_model::enums::Rules::Bb2020 {
                    out = out.publish(StepParameter::EndTurn(false));
                }
                return self.leave_with(game, out);
            }
            "DONE" => return self.leave(game),
            _ => {}
        }
        let player_id = player_id.as_str();

        // Java Phase::ASK → setup(): check canPlaceBallWhenKnockedDownOrPlacedProne skill.
        let has_skill = game.player(player_id)
            .map(|p| p.has_skill_property(NamedProperties::CAN_PLACE_BALL_WHEN_KNOCKED_DOWN_OR_PLACED_PRONE))
            .unwrap_or(false);
        let can_use = if has_skill {
            game.field_model.player_state(player_id)
                .map(|s| !s.is_hypnotized() && !s.is_confused())
                .unwrap_or(false)
        } else {
            false
        };

        if !can_use {
            // No skill or state prevents use → skip directly.
            return StepOutcome::next().publish(StepParameter::DroppedBallCarrier(None));
        }

        // Java setup(): the dialog also requires a FREE ball-adjacent square to place into —
        // with none, NEXT_STEP and no dialog.
        // Java: adjacentSquares.addAll(findAdjacentCoordinates(ball, FIELD, 1, false) with no
        // player on them) — kept for the SELECT/PLACE phases.
        let mut free: Vec<FieldCoordinate> = game.field_model.ball_coordinate.map(|bc| {
            game.field_model.adjacent_on_pitch(bc)
                .into_iter()
                .filter(|c| game.field_model.player_at(*c).is_none())
                .collect()
        }).unwrap_or_default();
        free.sort_by_key(|c| (c.x, c.y));
        self.adjacent_squares = free;
        if self.adjacent_squares.is_empty() {
            return StepOutcome::next();
        }
        // Java: UtilServerDialog.showDialog(DialogSkillUseParameter(playerId, skill, 0)) — the
        // Safe Pair of Hands offer. Auto-declining here spent ZERO agent draws while Java's
        // heuristic driver answers the dialog through its useSkill sampler (TWO draws), splitting
        // the two agents' streams for the rest of the game (chaos_pact bb2020 seed 8 i=47). The
        // offer is pinned to DECLINE by both agents' contracts (wUse 0.0 — using it enters a
        // PLACE_BALL coach dialog neither harness can drive), so only the decline path is
        // reachable in parity games.
        // Java setup(): `ballCarrierTeamTurn = isHomePlaying() == teamHome.hasPlayer(carrier)`;
        // a carrier on the NON-active team answers the dialog on their own turn side, so Java
        // flips homePlaying for the dialog's lifetime and leave() flips it back.
        let carrier_is_home = game.team_home.players.iter().any(|p| p.id == player_id);
        self.ball_carrier_team_turn = game.home_playing == carrier_is_home;
        if !self.ball_carrier_team_turn {
            game.home_playing = !game.home_playing;
        }
        if std::env::var_os("FFB_TRACE").is_some() {
            eprintln!("RPLACEBALL prompt pid={player_id:?} carrier_team_turn={}", self.ball_carrier_team_turn);
        }
        StepOutcome::cont().with_prompt(ffb_model::prompts::AgentPrompt::SkillUse {
            player_id: player_id.to_string(),
            skill_id: ffb_model::enums::SkillId::SafePairOfHands as u16,
            skill_name: ffb_model::enums::SkillId::SafePairOfHands.class_name().to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, CatchScatterThrowInMode};
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn start_returns_next() {
        let mut game = make_game();
        let mut step = StepPlaceBall::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn publishes_dropped_ball_carrier() {
        let mut game = make_game();
        let mut step = StepPlaceBall::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::DroppedBallCarrier(None))));
    }

    /// Java setParameter: DROPPED_BALL_CARRIER carries the carrier id (there is NO PLAYER_ID arm).
    #[test]
    fn set_parameter_dropped_ball_carrier_sets_player_id() {
        let mut step = StepPlaceBall::new();
        assert!(step.set_parameter(&StepParameter::DroppedBallCarrier(Some("p1".into()))));
        assert_eq!(step.player_id.as_deref(), Some("p1"));
        assert!(!step.set_parameter(&StepParameter::PlayerId("p2".into())), "Java has no PLAYER_ID arm");
        assert_eq!(step.player_id.as_deref(), Some("p1"));
    }

    /// Java setup(): an opponent (non-active-team) carrier's dialog flips homePlaying; the
    /// declined answer routes through leave(), which flips it back and publishes
    /// DROPPED_BALL_CARRIER = null (chaos_pact bb2020 seed 8 i=46).
    #[test]
    fn opponent_carrier_dialog_flips_home_playing_and_decline_restores() {
        use ffb_model::model::skill_def::SkillWithValue;
        let mut game = make_game();
        game.home_playing = false; // away acting; the carrier below is HOME
        game.team_home.players.push(ffb_model::model::player::Player {
            id: "carrier".into(), name: "carrier".into(), nr: 1, position_id: "p".into(),
            movement: 6, strength: 3, agility: 3, passing: 3, armour: 8,
            starting_skills: vec![SkillWithValue::new(ffb_model::enums::SkillId::SafePairOfHands)],
            ..Default::default()
        });
        game.field_model.set_player_coordinate("carrier", ffb_model::types::FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("carrier", ffb_model::enums::PlayerState::new(ffb_model::enums::PS_PRONE));
        game.field_model.ball_coordinate = Some(ffb_model::types::FieldCoordinate::new(5, 5));
        let mut step = StepPlaceBall::new();
        assert!(step.set_parameter(&StepParameter::DroppedBallCarrier(Some("carrier".into()))));
        assert!(step.set_parameter(&StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ScatterBall)));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue, "the offer must PROMPT, not auto-resolve");
        assert!(matches!(out.prompt, Some(ffb_model::prompts::AgentPrompt::SkillUse { .. })));
        assert!(game.home_playing, "dialog runs on the carrier's side (Java setHomePlaying flip)");
        let out2 = step.handle_command(
            &crate::action::Action::UseSkill {
                skill_id: ffb_model::enums::SkillId::SafePairOfHands, use_skill: false },
            &mut game, &mut GameRng::new(0));
        assert!(!game.home_playing, "leave() flips homePlaying back");
        assert!(out2.published.iter().any(|p| matches!(p, StepParameter::DroppedBallCarrier(None))));
    }

    /// Java Phase.SELECT → PLACE: an accepted Safe Pair of Hands publishes the free adjacent
    /// squares, switches to SAFE_PAIR_OF_HANDS, and the chosen square receives the (still moving)
    /// ball, restores lastTurnMode, writes ReportPlaceBallDirection and clears the catch mode.
    #[test]
    fn accepted_safe_pair_of_hands_places_the_ball_on_the_chosen_square() {
        use ffb_model::model::skill_def::SkillWithValue;
        use ffb_model::types::FieldCoordinate as FC;
        let mut game = make_game();
        game.home_playing = true;
        game.last_turn_mode = Some(ffb_model::enums::TurnMode::Regular);
        game.team_home.players.push(ffb_model::model::player::Player {
            id: "carrier".into(), name: "carrier".into(), nr: 1, position_id: "p".into(),
            movement: 6, strength: 3, agility: 3, passing: 3, armour: 8,
            starting_skills: vec![SkillWithValue::new(ffb_model::enums::SkillId::SafePairOfHands)],
            ..Default::default()
        });
        game.field_model.set_player_coordinate("carrier", FC::new(5, 5));
        game.field_model.set_player_state("carrier", ffb_model::enums::PlayerState::new(ffb_model::enums::PS_PRONE));
        game.field_model.ball_coordinate = Some(FC::new(5, 5));
        let mut step = StepPlaceBall::new();
        step.set_parameter(&StepParameter::DroppedBallCarrier(Some("carrier".into())));
        step.set_parameter(&StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ScatterBall));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.prompt, Some(ffb_model::prompts::AgentPrompt::SkillUse { .. })));
        let out = step.handle_command(
            &crate::action::Action::UseSkill { skill_id: ffb_model::enums::SkillId::SafePairOfHands, use_skill: true },
            &mut game, &mut GameRng::new(0));
        let squares = match out.prompt {
            Some(ffb_model::prompts::AgentPrompt::PlaceBallSquare { squares, .. }) => squares,
            other => panic!("expected PlaceBallSquare, got {other:?}"),
        };
        assert_eq!(squares.len(), 8, "every free neighbour of (5,5); the carrier sits on the ball");
        assert_eq!(game.turn_mode, ffb_model::enums::TurnMode::SafePairOfHands);
        let out = step.handle_command(
            &crate::action::Action::PlaceBallSquare { coord: FC::new(6, 5) }, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.field_model.ball_coordinate, Some(FC::new(6, 5)));
        assert!(game.field_model.ball_moving);
        assert_eq!(game.turn_mode, ffb_model::enums::TurnMode::Regular);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::CatchScatterThrowInModeCleared)));
        assert!(game.report_list.has_report(ffb_model::report::report_id::ReportId::PLACE_BALL_DIRECTION));
    }

    #[test]
    fn set_parameter_catch_scatter_mode_accepted() {
        let mut step = StepPlaceBall::new();
        assert!(step.set_parameter(&StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ScatterBall)));
        assert_eq!(step.catch_scatter_throw_in_mode, Some(CatchScatterThrowInMode::ScatterBall));
    }

    #[test]
    fn handle_command_use_skill_adds_skill_use_report() {
        use ffb_model::enums::SkillId;
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        let mut step = StepPlaceBall::new();
        step.player_id = Some("p1".into());
        // SafePairOfHands has CAN_PLACE_BALL_WHEN_KNOCKED_DOWN_OR_PLACED_PRONE
        let action = crate::action::Action::UseSkill { skill_id: SkillId::SafePairOfHands, use_skill: true };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::SKILL_USE), "expected SKILL_USE report on use-skill command");
    }

    #[test]
    fn handle_command_no_skill_no_report() {
        use ffb_model::enums::SkillId;
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        let mut step = StepPlaceBall::new();
        step.player_id = Some("p1".into());
        let action = crate::action::Action::UseSkill { skill_id: SkillId::Block, use_skill: true };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(!game.report_list.has_report(ReportId::SKILL_USE), "no SKILL_USE report for irrelevant skill");
    }
}
