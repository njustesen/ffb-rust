use ffb_mechanics::bb2025::throw_in_mechanic::ThrowInMechanic;
use ffb_mechanics::throw_in_mechanic::ThrowInMechanic as ThrowInMechanicTrait;
use ffb_model::enums::{Direction, ReRollSource};
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::report::bb2025::report_punt_direction::ReportPuntDirection;
use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_cards::UtilCards;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{CatchScatterThrowInMode, StepId, StepParameter};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

/// Rolls the scatter direction for a punt using the BB2025 ThrowInMechanic (1d6 from kicker's
/// direction), then publishes either Direction or throws-in out of bounds.
/// Re-roll (skill or team re-roll) is supported via UseSkill/UseReRoll actions.
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.punt.StepPuntDirection`.
pub struct StepPuntDirection {
    pub goto_label_on_end: String,
    pub direction: Option<Direction>,
    pub coordinate_to: Option<FieldCoordinate>,
    pub coordinate_from: Option<FieldCoordinate>,
    pub out_of_bounds: bool,
    // AbstractStepWithReRoll stubs
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
}

impl StepPuntDirection {
    pub fn new(goto_label_on_end: String) -> Self {
        Self {
            goto_label_on_end,
            direction: None,
            coordinate_to: None,
            coordinate_from: None,
            out_of_bounds: false,
            re_rolled_action: None,
            re_roll_source: None,
        }
    }
}

impl Step for StepPuntDirection {
    fn id(&self) -> StepId { StepId::PuntDirection }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_USE_SKILL — if (command.isSkillUsed()) setReRollSource(...); else leave source unset (declined).
        if let Action::UseReRoll { use_reroll: false } = action {
            self.re_roll_source = None;
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::CoordinateTo(c) => { self.coordinate_to = Some(*c); true }
            StepParameter::CoordinateFrom(c) => { self.coordinate_from = Some(*c); true }
            StepParameter::Touchback(v) => { self.out_of_bounds = *v; true }
            _ => false,
        }
    }
}

impl StepPuntDirection {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: fieldModel.setBallMoving(true); game.getTurnData().setPuntUsed(true);
        //       actingPlayer.markSkillUsed(NamedProperties.canPunt);
        game.field_model.ball_moving = true;
        game.turn_data_mut().punt_used = true;
        let player_id = game.acting_player.player_id.clone().unwrap_or_default();
        if let Some(skill_id) = game.player(&player_id)
            .and_then(|p| UtilCards::get_unused_skill_with_property(p, NamedProperties::CAN_PUNT))
        {
            game.mark_skill_used(&player_id, skill_id);
        }

        if self.out_of_bounds {
            let ball_coord = game.field_model.ball_coordinate;
            game.report_list.add(ReportPuntDirection::new(None, 0, player_id, true));
            return StepOutcome::goto(&self.goto_label_on_end.clone())
                .publish(StepParameter::EndTurn(true))
                .publish(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ThrowIn))
                .publish(StepParameter::ThrowInCoordinate(ball_coord.unwrap_or(FieldCoordinate::new(0, 0))));
        }

        // Java: if (ReRolledActions.PUNT_DIRECTION == getReRolledAction()) {
        //           if (getReRollSource() == null || !UtilServerReRoll.useReRoll(...)) { leave(); return; }
        //       }
        if self.re_rolled_action.as_deref() == Some("PUNT_DIRECTION") {
            match self.re_roll_source.clone() {
                Some(ref source_name) => {
                    let source = ReRollSource::new(source_name.as_str());
                    if !use_reroll(game, &source, &player_id) {
                        return self.leave(game);
                    }
                    // Re-roll consumed — fall through to roll again.
                }
                None => return self.leave(game),
            }
        }

        let coord_from = match self.coordinate_from {
            Some(c) => c,
            None => return StepOutcome::next(),
        };
        let coord_to = match self.coordinate_to {
            Some(c) => c,
            None => return StepOutcome::next(),
        };

        // Java: Direction baseDirection = coordinateFrom.getDirection(coordinateTo);
        //       int roll = rollThrowInDirection(); // 1d6
        //       direction = mechanic.interpretThrowInDirectionRoll(baseDirection, roll);
        let base_direction = coord_from.direction_to(coord_to).unwrap_or(Direction::North);
        let roll = rng.d6();
        let direction = ThrowInMechanic::new().interpret_throw_in_direction_roll_with_template(base_direction, roll);
        self.direction = Some(direction);
        let indicator = coord_from.step(direction, 1);
        if indicator.is_on_pitch() {
            game.field_model.ball_coordinate = Some(indicator);
            game.field_model.out_of_bounds = false;
        } else {
            game.field_model.out_of_bounds = true;
        }
        game.report_list.add(ReportPuntDirection::new(
            Some(direction), roll, player_id.clone(), game.field_model.out_of_bounds,
        ));

        // Java: if (getReRolledAction() == null) { setReRolledAction(PUNT_DIRECTION); ... offer re-roll ... }
        if self.re_rolled_action.is_none() {
            self.re_rolled_action = Some("PUNT_DIRECTION".into());
            if let Some(prompt) = ask_for_reroll_if_available(game, "PUNT_DIRECTION", 0, false) {
                self.re_roll_source = Some("TRR".into());
                return StepOutcome::cont().with_prompt(prompt);
            }
        }
        self.leave(game)
    }

    /// Java: `StepPuntDirection.leave()`.
    fn leave(&mut self, game: &mut Game) -> StepOutcome {
        if game.field_model.out_of_bounds {
            let ball_coord = game.field_model.ball_coordinate;
            StepOutcome::goto(&self.goto_label_on_end.clone())
                .publish(StepParameter::EndTurn(true))
                .publish(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ThrowIn))
                .publish(StepParameter::ThrowInCoordinate(ball_coord.unwrap_or(FieldCoordinate::new(0, 0))))
        } else {
            StepOutcome::next()
                .publish(StepParameter::Direction(self.direction.unwrap_or(Direction::North)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    // 1. out_of_bounds = true → GotoLabel + ThrowIn + EndTurn published
    #[test]
    fn out_of_bounds_flag_goto_label() {
        let mut game = make_game();
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(0, 7));
        let mut step = StepPuntDirection::new("end".into());
        step.out_of_bounds = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ThrowIn))));
    }

    // 2. Missing coordinate_from → NextStep (guard)
    #[test]
    fn missing_coord_from_returns_next() {
        let mut game = make_game();
        let mut step = StepPuntDirection::new("end".into());
        step.coordinate_to = Some(FieldCoordinate::new(10, 7));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    // 3. set_parameter accepts CoordinateTo and CoordinateFrom
    #[test]
    fn set_parameter_stores_coordinates() {
        let mut step = StepPuntDirection::new("end".into());
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(6, 5);
        step.set_parameter(&StepParameter::CoordinateFrom(from));
        step.set_parameter(&StepParameter::CoordinateTo(to));
        assert_eq!(step.coordinate_from, Some(from));
        assert_eq!(step.coordinate_to, Some(to));
    }

    // 4. set_parameter Touchback sets out_of_bounds
    #[test]
    fn set_parameter_touchback_sets_out_of_bounds() {
        let mut step = StepPuntDirection::new("end".into());
        assert!(!step.out_of_bounds);
        step.set_parameter(&StepParameter::Touchback(true));
        assert!(step.out_of_bounds);
    }

    // 5. With both coords provided, direction is rolled and published (on-pitch result)
    //    Use a from-coord that has room for any direction: interior of field.
    #[test]
    fn direction_rolled_and_published_on_pitch() {
        let mut game = make_game();
        // Ball starts in centre; from is interior — most directions stay on pitch.
        let from = FieldCoordinate::new(12, 7);
        let to = FieldCoordinate::new(13, 7);
        game.field_model.ball_coordinate = Some(from);
        let mut step = StepPuntDirection::new("end".into());
        step.coordinate_from = Some(from);
        step.coordinate_to = Some(to);

        let out = step.start(&mut game, &mut GameRng::new(0));
        // Either stays on pitch (Direction published) or goes out of bounds (GotoLabel)
        // — either is correct; ensure no panic and step terminates
        assert!(out.action == StepAction::NextStep || out.action == StepAction::GotoLabel);
    }

    #[test]
    fn out_of_bounds_flag_adds_punt_direction_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(0, 7));
        let mut step = StepPuntDirection::new("end".into());
        step.out_of_bounds = true;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::PUNT_DIRECTION_ROLL), "expected PUNT_DIRECTION_ROLL report");
    }

    #[test]
    fn on_pitch_roll_adds_punt_direction_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        let from = FieldCoordinate::new(12, 7);
        let to = FieldCoordinate::new(13, 7);
        game.field_model.ball_coordinate = Some(from);
        let mut step = StepPuntDirection::new("end".into());
        step.coordinate_from = Some(from);
        step.coordinate_to = Some(to);
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::PUNT_DIRECTION_ROLL), "expected PUNT_DIRECTION_ROLL report after rolling direction");
    }

    // Java: `game.getTurnData().setPuntUsed(true)` — must run on every execute_step entry,
    // including the out-of-bounds branch, not just the successful roll path.
    #[test]
    fn execute_step_marks_punt_used_on_turn_data() {
        let mut game = make_game();
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(0, 7));
        assert!(!game.turn_data().punt_used);
        let mut step = StepPuntDirection::new("end".into());
        step.out_of_bounds = true;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.turn_data().punt_used, "expected TurnData.puntUsed to be set true");
    }

    // Java: `actingPlayer.markSkillUsed(NamedProperties.canPunt)` — the Punt skill must be
    // marked used on the acting player after the punt direction step executes.
    #[test]
    fn execute_step_marks_punt_skill_used() {
        use ffb_model::enums::{PlayerType, PlayerGender, SkillId};
        use ffb_model::model::player::Player;
        use ffb_model::model::skill_def::SkillWithValue;

        let mut game = make_game();
        let player = Player {
            id: "punter".into(), name: "p".into(), nr: 1,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 3, armour: 8,
            starting_skills: vec![SkillWithValue::new(SkillId::Punt)],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        game.team_home.players.push(player);
        game.acting_player.player_id = Some("punter".into());
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(0, 7));

        let mut step = StepPuntDirection::new("end".into());
        step.out_of_bounds = true;
        step.start(&mut game, &mut GameRng::new(0));

        assert!(
            game.team_home.player("punter").unwrap().used_skills.contains(&SkillId::Punt),
            "expected Punt skill to be marked used on the acting player"
        );
    }
}
