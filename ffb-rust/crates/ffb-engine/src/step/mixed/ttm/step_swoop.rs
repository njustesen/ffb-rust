/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.ttm.StepSwoop`.
///
/// Step in the TTM sequence to handle the Swoop skill (BB2016 + BB2020).
///
/// Structure mirrors the BB2025 version exactly except this edition is tagged
/// `@RulesCollection(BB2016)` and `@RulesCollection(BB2020)`.  The Java logic
/// is identical to BB2025 — the inner `StepState` fields and `executeStep` /
/// `executeSwoop` methods are the same.
///
/// See `crates/ffb-engine/src/step/bb2025/ttm/step_swoop.rs` for the full
/// commentary.
use ffb_model::enums::{Direction, PlayerState};
use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepSwoop` (mixed/ttm, BB2016 + BB2020).
///
/// Inner class `StepState` translated as struct fields.
pub struct StepSwoop {
    /// Java: state.status
    pub status: Option<String>,
    /// Java: state.thrownPlayerId (mandatory init param)
    pub thrown_player_id: Option<String>,
    /// Java: state.thrownPlayerState (mandatory init param)
    pub thrown_player_state: Option<PlayerState>,
    /// Java: state.thrownPlayerHasBall
    pub thrown_player_has_ball: bool,
    /// Java: state.thrownPlayerCoordinate (mandatory init param)
    pub thrown_player_coordinate: Option<FieldCoordinate>,
    /// Java: state.throwScatter
    pub throw_scatter: bool,
    /// Java: state.coordinateFrom
    pub coordinate_from: Option<FieldCoordinate>,
    /// Java: state.coordinateTo (set by CLIENT_SWOOP)
    pub coordinate_to: Option<FieldCoordinate>,
    /// Java: state.goToLabelOnFallDown (mandatory init param)
    pub goto_label_on_fall_down: String,
    /// Java: (not present in mixed; present in bb2025 variant for usingSwoop dialog)
    pub using_swoop: Option<bool>,
    /// Java: (not present in mixed version's StepState)
    pub swoop_direction: Option<Direction>,
}

impl StepSwoop {
    pub fn new(goto_label_on_fall_down: String) -> Self {
        Self {
            status: None,
            thrown_player_id: None,
            thrown_player_state: None,
            thrown_player_has_ball: false,
            thrown_player_coordinate: None,
            throw_scatter: false,
            coordinate_from: None,
            coordinate_to: None,
            goto_label_on_fall_down,
            using_swoop: None,
            swoop_direction: None,
        }
    }
}

impl Default for StepSwoop {
    fn default() -> Self { Self::new(String::new()) }
}

impl Step for StepSwoop {
    fn id(&self) -> StepId { StepId::Swoop }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    /// Java: handleCommand dispatches:
    ///   CLIENT_SWOOP  → coordinateTo = transformed coord → executeSwoop()
    ///   EXECUTE_STEP  → executeStep()
    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::Pass { coord } => {
                // CLIENT_SWOOP — target square selected
                // TODO: transform coordinate for away team
                self.coordinate_to = Some(*coord);
                // TODO: executeSwoop() hooks
                return StepOutcome::next();
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::CoordinateFrom(v) => { self.coordinate_from = Some(*v); true }
            StepParameter::CoordinateTo(v)   => { self.coordinate_to = Some(*v); true }
            _ => false,
        }
    }
}

impl StepSwoop {
    fn execute_step(&self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Guard: both player and coordinate must be set
        if self.thrown_player_id.is_none() || self.thrown_player_coordinate.is_none() {
            return StepOutcome::next();
        }

        // Java: if throwScatter → animate + move player (TODO: field model + animation)
        // Java: if coordinateTo == null → updateSwoopSquares (TODO) → wait
        if self.coordinate_to.is_none() {
            // TODO: UtilServerPlayerSwoop.updateSwoopSquares
            return StepOutcome::cont();
        }

        // coordinateTo is known; executeSwoop hook runs (TODO: step hooks)
        StepOutcome::next()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn id_is_swoop() {
        assert_eq!(StepSwoop::new("fall".into()).id(), StepId::Swoop);
    }

    #[test]
    fn no_player_returns_next_step() {
        let mut game = make_game();
        let mut step = StepSwoop::new("fall".into());
        // thrown_player_id is None → NEXT_STEP
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn player_with_no_coord_to_waits() {
        let mut game = make_game();
        let mut step = StepSwoop::new("fall".into());
        step.thrown_player_id = Some("p1".into());
        step.thrown_player_coordinate = Some(FieldCoordinate { x: 3, y: 3 });
        // coordinate_to is None → Continue (wait for CLIENT_SWOOP)
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn player_with_coord_to_returns_next() {
        let mut game = make_game();
        let mut step = StepSwoop::new("fall".into());
        step.thrown_player_id = Some("p1".into());
        step.thrown_player_coordinate = Some(FieldCoordinate { x: 3, y: 3 });
        step.coordinate_to = Some(FieldCoordinate { x: 5, y: 5 });
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_coordinate_from_accepted() {
        let mut step = StepSwoop::default();
        let c = FieldCoordinate { x: 1, y: 2 };
        assert!(step.set_parameter(&StepParameter::CoordinateFrom(c)));
        assert_eq!(step.coordinate_from, Some(c));
    }

    #[test]
    fn set_coordinate_to_accepted() {
        let mut step = StepSwoop::default();
        let c = FieldCoordinate { x: 7, y: 4 };
        assert!(step.set_parameter(&StepParameter::CoordinateTo(c)));
        assert_eq!(step.coordinate_to, Some(c));
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepSwoop::default();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }
}
