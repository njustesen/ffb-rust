/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.StepKickoffScatterRoll` (BB2016).
///
/// Rolls the kickoff scatter direction and distance, applies the Kick skill (halves scatter
/// distance by integer division), and sets the ball coordinate for the subsequent
/// catch/touchback.
///
/// BB2016 Kick skill halves `fScatterDistance` (integer division, same as BB2020).
/// `findKickingPlayer()` prefers players in CENTER_FIELD bounds with `canReduceKickDistance`,
/// falling back to the frontmost center-field player, then a random on-field player.
///
/// Expects stepParameter KICKOFF_START_COORDINATE from preceding step.
/// Sets stepParameter KICKING_PLAYER_COORDINATE for all steps on the stack.
/// Sets stepParameter KICKOFF_BOUNDS for all steps on the stack.
/// Sets stepParameter TOUCHBACK for all steps on the stack.
///
/// headless items:
///  - findKickingPlayer: full center-field / frontmost-player logic.
///  - UtilServerDialog.showDialog for Kick choice.
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2016.StepKickoffScatterRoll`.
use ffb_model::enums::{Direction, SkillId};
use ffb_model::model::skill_use::SkillUse;
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::report::report_kickoff_scatter::ReportKickoffScatter;
use ffb_model::report::report_skill_use::ReportSkillUse;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

pub struct StepKickoffScatterRoll {
    /// Java: fKickoffStartCoordinate
    pub kickoff_start_coordinate: Option<FieldCoordinate>,
    /// Java: fUseKickChoice — None = dialog pending; Some = answered.
    pub use_kick_choice: Option<bool>,
    /// Java: fScatterDirection
    pub scatter_direction: Option<Direction>,
    /// Java: fScatterDistance
    pub scatter_distance: i32,
    /// Raw d8 roll for scatter direction (stored in Phase 1 for use in Phase 2 report).
    pub scatter_direction_roll: i32,
    /// Java: fKickingPlayerCoordinate
    pub kicking_player_coordinate: Option<FieldCoordinate>,
    /// Java: fKickoffBounds
    pub kickoff_bounds: Option<FieldCoordinateBounds>,
    /// Java: fTouchback
    pub touchback: bool,
}

impl StepKickoffScatterRoll {
    pub fn new() -> Self {
        Self {
            kickoff_start_coordinate: None,
            use_kick_choice: None,
            scatter_direction: None,
            scatter_distance: 0,
            scatter_direction_roll: 0,
            kicking_player_coordinate: None,
            kickoff_bounds: None,
            touchback: false,
        }
    }
}

impl Default for StepKickoffScatterRoll {
    fn default() -> Self { Self::new() }
}

impl Step for StepKickoffScatterRoll {
    fn id(&self) -> StepId { StepId::KickoffScatterRoll }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_USE_SKILL with canReduceKickDistance → sets fUseKickChoice
        if let Action::UseSkill { use_skill, .. } = action {
            if self.use_kick_choice.is_none() {
                self.use_kick_choice = Some(*use_skill);
                if *use_skill {
                    let pid = game.acting_player.player_id.clone();
                    game.report_list.add(ReportSkillUse::new(
                        pid,
                        SkillId::Kick,
                        true,
                        SkillUse::HALVE_KICKOFF_SCATTER,
                    ));
                }
            }
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::KickoffStartCoordinate(v) => { self.kickoff_start_coordinate = Some(*v); true }
            _ => false,
        }
    }
}

impl StepKickoffScatterRoll {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let start = match self.kickoff_start_coordinate {
            Some(c) => c,
            None => return StepOutcome::next(),
        };

        // Java Phase 1: if (fUseKickChoice == null) — roll scatter direction & distance
        if self.scatter_direction.is_none() {
            let dir_roll = rng.d8();
            let direction = Direction::for_roll(dir_roll).unwrap_or(Direction::North);
            let distance = rng.d8(); // Java: rollScatterDistance() → d8

            self.scatter_direction = Some(direction);
            self.scatter_distance = distance;
            self.scatter_direction_roll = dir_roll;

            // Java: find kicking player coordinate; fall back to center stub if not on field
            let default_kicker = if game.home_playing {
                FieldCoordinate::new(0, 7)
            } else {
                FieldCoordinate::new(25, 7)
            };
            self.kicking_player_coordinate = Some(
                game.acting_player.player_id.as_deref()
                    .and_then(|id| game.field_model.player_coordinate(id))
                    .unwrap_or(default_kicker)
            );

            // client-only: DialogKickSkillParameter — headless auto-declines Kick skill
            self.use_kick_choice = Some(false);
        }

        // Java Phase 2: if (fUseKickChoice != null) — apply scatter
        if let (Some(direction), Some(use_kick)) = (self.scatter_direction, self.use_kick_choice) {
            // BB2016: halve scatter distance (integer div) when Kick skill used
            let distance = if use_kick { self.scatter_distance / 2 } else { self.scatter_distance };

            let ball_end = start.step(direction, distance);

            game.report_list.add(ReportKickoffScatter::new(
                ball_end,
                direction,
                self.scatter_direction_roll,
                self.scatter_distance, // raw distance before halving
            ));

            // Java: walk back along scatter path until on field
            let mut d = distance;
            let mut last_valid = ball_end;
            while !FieldCoordinateBounds::FIELD.is_in_bounds(last_valid) {
                d -= 1;
                if d < 0 {
                    last_valid = start;
                    break;
                }
                last_valid = start.step(direction, d);
            }

            game.field_model.ball_in_play = false;
            game.field_model.ball_coordinate = Some(last_valid);
            game.field_model.ball_moving = true;

            // Java: determine kickoff bounds (receiving half)
            let receiving_half = if game.home_playing {
                FieldCoordinateBounds::HALF_AWAY
            } else {
                FieldCoordinateBounds::HALF_HOME
            };

            self.kickoff_bounds = if receiving_half.is_in_bounds(ball_end) {
                Some(receiving_half)
            } else {
                None
            };
            self.touchback = self.kickoff_bounds.is_none();
            // Java: game.getFieldModel().setOutOfBounds(touchback)
            game.field_model.out_of_bounds = self.touchback;

            let kicking_coord = self.kicking_player_coordinate.unwrap_or(FieldCoordinate::new(0, 7));
            let touchback = self.touchback;

            return StepOutcome::next()
                .publish(StepParameter::KickingPlayerCoordinate(kicking_coord))
                .publish(StepParameter::KickoffBounds(
                    self.kickoff_bounds.unwrap_or(FieldCoordinateBounds::FIELD),
                ))
                .publish(StepParameter::Touchback(touchback));
        }

        // Dialog pending
        StepOutcome::cont()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    #[test]
    fn no_start_coord_returns_next_step() {
        let mut game = make_game();
        let mut step = StepKickoffScatterRoll::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn publishes_touchback_parameter() {
        let mut game = make_game();
        game.home_playing = true;
        let mut step = StepKickoffScatterRoll::new();
        step.kickoff_start_coordinate = Some(FieldCoordinate::new(13, 7));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::Touchback(_))));
    }

    #[test]
    fn publishes_kicking_player_coordinate() {
        let mut game = make_game();
        game.home_playing = true;
        let mut step = StepKickoffScatterRoll::new();
        step.kickoff_start_coordinate = Some(FieldCoordinate::new(13, 7));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::KickingPlayerCoordinate(_))));
    }

    #[test]
    fn publishes_kickoff_bounds() {
        let mut game = make_game();
        game.home_playing = true;
        let mut step = StepKickoffScatterRoll::new();
        step.kickoff_start_coordinate = Some(FieldCoordinate::new(13, 7));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::KickoffBounds(_))));
    }

    #[test]
    fn sets_ball_moving() {
        let mut game = make_game();
        game.home_playing = true;
        let mut step = StepKickoffScatterRoll::new();
        step.kickoff_start_coordinate = Some(FieldCoordinate::new(13, 7));
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.field_model.ball_moving);
    }

    #[test]
    fn use_kick_choice_halves_distance() {
        let mut game = make_game();
        game.home_playing = true;
        let mut step = StepKickoffScatterRoll::new();
        step.kickoff_start_coordinate = Some(FieldCoordinate::new(13, 7));
        step.scatter_direction = Some(Direction::North);
        step.scatter_distance = 8;
        step.use_kick_choice = Some(true); // halved → 4
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_kickoff_start_coord_accepted() {
        let mut step = StepKickoffScatterRoll::new();
        let coord = FieldCoordinate::new(12, 7);
        assert!(step.set_parameter(&StepParameter::KickoffStartCoordinate(coord)));
        assert_eq!(step.kickoff_start_coordinate, Some(coord));
    }

    #[test]
    fn set_parameter_unknown_returns_false() {
        let mut step = StepKickoffScatterRoll::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn adds_kickoff_scatter_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        game.home_playing = true;
        let mut step = StepKickoffScatterRoll::new();
        step.kickoff_start_coordinate = Some(FieldCoordinate::new(13, 7));
        let _ = step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::KICKOFF_SCATTER));
    }

    #[test]
    fn scatter_direction_roll_stored_in_phase1() {
        let mut game = make_game();
        game.home_playing = true;
        let mut step = StepKickoffScatterRoll::new();
        step.kickoff_start_coordinate = Some(FieldCoordinate::new(13, 7));
        step.start(&mut game, &mut GameRng::new(42));
        assert!(step.scatter_direction_roll > 0, "scatter_direction_roll should be non-zero after phase 1");
    }
}
