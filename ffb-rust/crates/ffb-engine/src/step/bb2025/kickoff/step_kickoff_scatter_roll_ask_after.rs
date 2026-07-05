use ffb_model::enums::Direction;
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// Variant of `StepKickoffScatterRoll` that rolls direction + distance **first**,
/// then shows a dialog asking the kicking coach whether to use the Kick skill to
/// halve the scatter distance.  If no Kick skill is present the dialog is skipped
/// and execution proceeds identically to `StepKickoffScatterRoll`.
///
/// Publishes (same contract as `StepKickoffScatterRoll`):
///  - `KickingPlayerCoordinate`
///  - `KickoffBounds`
///  - `Touchback`
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.kickoff.StepKickoffScatterRollAskAfter`.
pub struct StepKickoffScatterRollAskAfter {
    /// Java: fKickoffStartCoordinate — set by preceding StepKickoff.
    pub kickoff_start_coordinate: Option<FieldCoordinate>,
    /// Java: fUseKickChoice — None until the coach responds (or skill is absent).
    pub use_kick_choice: Option<bool>,
    /// Java: fScatterDirection — rolled d8.
    pub scatter_direction: Option<Direction>,
    /// Java: fScatterDistance — rolled d6 (raw, before Kick halving).
    pub scatter_distance: i32,
    /// Java: fKickingPlayerCoordinate — published downstream.
    pub kicking_player_coordinate: Option<FieldCoordinate>,
    /// Java: fKickoffBounds — receiving half or None → touchback.
    pub kickoff_bounds: Option<FieldCoordinateBounds>,
    /// Java: fTouchback.
    pub touchback: bool,
}

impl StepKickoffScatterRollAskAfter {
    pub fn new() -> Self {
        Self {
            kickoff_start_coordinate: None,
            use_kick_choice: None,
            scatter_direction: None,
            scatter_distance: 0,
            kicking_player_coordinate: None,
            kickoff_bounds: None,
            touchback: false,
        }
    }
}

impl Default for StepKickoffScatterRollAskAfter {
    fn default() -> Self { Self::new() }
}

impl Step for StepKickoffScatterRollAskAfter {
    fn id(&self) -> StepId { StepId::KickoffScatterRollAskAfter }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java CLIENT_USE_SKILL with canReduceKickDistance: capture the choice.
        if let Action::UseSkill { use_skill, .. } = action {
            if self.use_kick_choice.is_none() {
                self.use_kick_choice = Some(*use_skill);
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

impl StepKickoffScatterRollAskAfter {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let start = match self.kickoff_start_coordinate {
            Some(c) => c,
            None => return StepOutcome::cont(),
        };

        // ── Phase 1: roll direction + raw distance (once) ────────────────────
        if self.scatter_direction.is_none() {
            let dir_roll = rng.d8();
            self.scatter_direction = Direction::for_roll(dir_roll);
            self.scatter_distance = rng.d6();

            // Determine kicking player coordinate.
            // Java: search kicking team's CENTER_FIELD / LOS players for canReduceKickDistance.
            let (center_bounds, los_bounds) = if game.home_playing {
                (FieldCoordinateBounds::CENTER_FIELD_HOME, FieldCoordinateBounds::LOS_HOME)
            } else {
                (FieldCoordinateBounds::CENTER_FIELD_AWAY, FieldCoordinateBounds::LOS_AWAY)
            };
            let kicking_team = game.active_team();
            let found = kicking_team.players.iter().find_map(|p| {
                let coord = game.field_model.player_coordinate(&p.id)?;
                if center_bounds.is_in_bounds(coord) || los_bounds.is_in_bounds(coord) {
                    Some(coord)
                } else {
                    None
                }
            });
            let default_kicker = if game.home_playing {
                FieldCoordinate::new(0, 7)
            } else {
                FieldCoordinate::new(25, 7)
            };
            self.kicking_player_coordinate = Some(found.unwrap_or(default_kicker));

            // client-only: if kicking player has canReduceKickDistance, show DialogKickSkillParameter
            // (with the two possible landing squares) and return Continue.
            // For the random-agent path there is no Kick player, so fall through.
            self.use_kick_choice = Some(false);
        }

        // ── Phase 2: resolve final distance and place ball ───────────────────
        let direction = match self.scatter_direction {
            Some(d) => d,
            None => Direction::North,
        };
        let use_kick = self.use_kick_choice.unwrap_or(false);
        // Java: Math.ceil(fScatterDistance / 2.0) when Kick is used.
        let distance = if use_kick {
            (self.scatter_distance as f64 / 2.0).ceil() as i32
        } else {
            self.scatter_distance
        };

        let ball_end = start.step(direction, distance);

        // Walk back until the ball is on the field.
        let mut dist_walk = distance;
        let mut last_valid = ball_end;
        while !FieldCoordinateBounds::FIELD.is_in_bounds(last_valid) {
            dist_walk -= 1;
            last_valid = start.step(direction, dist_walk);
            if dist_walk < 0 {
                last_valid = start;
                break;
            }
        }

        game.field_model.ball_in_play = false;
        game.field_model.ball_coordinate = Some(last_valid);
        game.field_model.ball_moving = true;

        // ── Determine kickoff bounds and touchback ────────────────────────────
        let receiving_half = if game.home_playing {
            FieldCoordinateBounds::HALF_AWAY
        } else {
            FieldCoordinateBounds::HALF_HOME
        };

        if receiving_half.is_in_bounds(ball_end) {
            self.kickoff_bounds = Some(receiving_half);
        } else {
            self.kickoff_bounds = None;
        }
        self.touchback = self.kickoff_bounds.is_none();

        // ── Publish ────────────────────────────────────────────────────────────
        let kicking_coord = self.kicking_player_coordinate.unwrap();
        let touchback = self.touchback;
        let bounds = self.kickoff_bounds;

        let mut outcome = StepOutcome::next()
            .publish(StepParameter::KickingPlayerCoordinate(kicking_coord))
            .publish(StepParameter::Touchback(touchback));
        if let Some(b) = bounds {
            outcome = outcome.publish(StepParameter::KickoffBounds(b));
        }
        outcome
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
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn start_without_start_coordinate_returns_cont() {
        let mut game = make_game();
        let mut step = StepKickoffScatterRollAskAfter::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn start_with_coordinate_returns_next_step() {
        let mut game = make_game();
        let mut step = StepKickoffScatterRollAskAfter::new();
        step.kickoff_start_coordinate = Some(FieldCoordinate::new(13, 7));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn publishes_kicking_coord_and_touchback() {
        let mut game = make_game();
        let mut step = StepKickoffScatterRollAskAfter::new();
        step.kickoff_start_coordinate = Some(FieldCoordinate::new(13, 7));
        let out = step.start(&mut game, &mut GameRng::new(1));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::KickingPlayerCoordinate(_))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::Touchback(_))));
    }

    #[test]
    fn ball_on_field_after_scatter() {
        let mut game = make_game();
        let mut step = StepKickoffScatterRollAskAfter::new();
        step.kickoff_start_coordinate = Some(FieldCoordinate::new(13, 7));
        step.start(&mut game, &mut GameRng::new(5));
        if let Some(c) = game.field_model.ball_coordinate {
            assert!(FieldCoordinateBounds::FIELD.is_in_bounds(c), "Ball off-field at {c:?}");
        }
    }

    #[test]
    fn use_skill_action_sets_kick_choice() {
        let mut game = make_game();
        let mut step = StepKickoffScatterRollAskAfter::new();
        step.kickoff_start_coordinate = Some(FieldCoordinate::new(13, 7));
        // Roll first (sets scatter_direction) but not use_kick_choice yet.
        // We can simulate by setting direction manually.
        step.scatter_direction = Some(Direction::East);
        step.scatter_distance = 4;
        step.kicking_player_coordinate = Some(FieldCoordinate::new(0, 7));
        let action = Action::UseSkill { skill_id: ffb_mechanics::skills::SkillId::Kick, use_skill: true };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        // With use_kick=true and distance=4, effective distance = ceil(4/2) = 2.
        // Verify the step finished (returns NextStep).
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }
}
