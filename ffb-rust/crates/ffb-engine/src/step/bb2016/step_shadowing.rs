use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.StepShadowing.
///
/// Handles the Shadowing skill (and optionally Diving Tackle).
/// Entirely hook-driven: all roll logic is in executeStepHooks(this, state).
///
/// Expects: COORDINATE_FROM, DEFENDER_POSITION, USING_DIVING_TACKLE.
/// Java: ActionStatus for Shadowing step
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadowingStatus {
    None,
    Success,
    Failure,
}

pub struct StepShadowing {
    /// Java: state.status (ActionStatus — hook output)
    pub status: ShadowingStatus,
    /// Java: state.defenderPosition
    pub defender_position: Option<FieldCoordinate>,
    /// Java: state.coordinateFrom
    pub coordinate_from: Option<FieldCoordinate>,
    /// Java: state.usingDivingTackle
    pub using_diving_tackle: bool,
    /// Java: state.usingShadowing (Boolean tristate)
    pub using_shadowing: Option<bool>,
}

impl StepShadowing {
    pub fn new() -> Self {
        Self {
            status: ShadowingStatus::None,
            defender_position: None,
            coordinate_from: None,
            using_diving_tackle: false,
            using_shadowing: None,
        }
    }
}

impl Default for StepShadowing {
    fn default() -> Self { Self::new() }
}

impl Step for StepShadowing {
    fn id(&self) -> StepId { StepId::Shadowing }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_PLAYER_CHOICE (PlayerChoiceMode.SHADOWING)
        //   state.usingShadowing = StringTool.isProvided(playerId)
        //   game.setDefenderId(playerId)
        if let Action::SelectPlayer { player_id } = action {
            self.using_shadowing = Some(!player_id.is_empty());
            game.defender_id = if player_id.is_empty() { None } else { Some(player_id.clone()) };
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::CoordinateFrom(c) => {
                self.coordinate_from = Some(*c);
                true
            }
            StepParameter::DefenderPosition(c) => {
                self.defender_position = Some(*c);
                true
            }
            StepParameter::UsingDivingTackle(v) => {
                self.using_diving_tackle = *v;
                true
            }
            StepParameter::UsingShadowing(v) => {
                self.using_shadowing = *v;
                true
            }
            _ => false,
        }
    }
}

impl StepShadowing {
    fn execute_step(&mut self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: getGameState().executeStepHooks(this, state)
        // All logic (AG check, position comparison, DivingTackle modifier) is in hooks.
        // DEFERRED(hooks): executeStepHooks (ShadowingHook — AG check, position, DivingTackle) not yet ported.
        StepOutcome::next()
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
        Game::new(home, away, Rules::Bb2016)
    }

    #[test]
    fn step_id_is_shadowing() {
        let step = StepShadowing::new();
        assert_eq!(step.id(), StepId::Shadowing);
    }

    #[test]
    fn coordinate_from_parameter_accepted() {
        let mut step = StepShadowing::new();
        let coord = FieldCoordinate::new(5, 3);
        let ok = step.set_parameter(&StepParameter::CoordinateFrom(coord));
        assert!(ok);
        assert_eq!(step.coordinate_from, Some(coord));
    }

    #[test]
    fn defender_position_parameter_accepted() {
        let mut step = StepShadowing::new();
        let coord = FieldCoordinate::new(7, 4);
        let ok = step.set_parameter(&StepParameter::DefenderPosition(coord));
        assert!(ok);
        assert_eq!(step.defender_position, Some(coord));
    }

    #[test]
    fn using_diving_tackle_parameter_accepted() {
        let mut step = StepShadowing::new();
        let ok = step.set_parameter(&StepParameter::UsingDivingTackle(true));
        assert!(ok);
        assert!(step.using_diving_tackle);
    }

    #[test]
    fn using_shadowing_parameter_accepted() {
        let mut step = StepShadowing::new();
        let ok = step.set_parameter(&StepParameter::UsingShadowing(Some(true)));
        assert!(ok);
        assert_eq!(step.using_shadowing, Some(true));
    }

    #[test]
    fn start_returns_next_step() {
        let mut step = StepShadowing::new();
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn select_player_sets_using_shadowing() {
        let mut step = StepShadowing::new();
        let mut game = make_game();
        step.handle_command(
            &Action::SelectPlayer { player_id: "p1".to_string() },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(step.using_shadowing, Some(true));
        assert_eq!(game.defender_id.as_deref(), Some("p1"));
    }

    #[test]
    fn select_empty_player_declines_shadowing() {
        let mut step = StepShadowing::new();
        let mut game = make_game();
        step.handle_command(
            &Action::SelectPlayer { player_id: "".to_string() },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(step.using_shadowing, Some(false));
        assert!(game.defender_id.is_none());
    }
}
