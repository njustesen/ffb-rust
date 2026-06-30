use ffb_model::model::property::named_properties::NamedProperties;
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
}

impl StepPlaceBall {
    pub fn new() -> Self {
        Self {
            player_id: None,
            catch_scatter_throw_in_mode: None,
            phase_name: "ASK".to_string(),
            ball_carrier_team_turn: false,
            revert_end_turn: false,
            selected_coordinate: None,
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

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::PlayerId(v) => { self.player_id = Some(v.clone()); true }
            StepParameter::CatchScatterThrowInMode(v) => { self.catch_scatter_throw_in_mode = Some(*v); true }
            StepParameter::DroppedBallCarrier(_) => true, // consumed
            _ => false,
        }
    }
}

impl StepPlaceBall {
    /// Java: executeStep() with Phase::ASK fast-path.
    fn execute_step(&self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: if (playerId == null || catchScatterThrowInMode != SCATTER_BALL) → NEXT_STEP
        let player_id = match self.player_id.as_deref() {
            Some(id) if self.catch_scatter_throw_in_mode == Some(CatchScatterThrowInMode::ScatterBall) => id,
            _ => return StepOutcome::next().publish(StepParameter::DroppedBallCarrier(None)),
        };

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

        // Skill available but dialog infra not yet ported: auto-decline (conservative).
        // Java would show DialogSkillUseParameter and wait for CLIENT_USE_SKILL response.
        StepOutcome::next().publish(StepParameter::DroppedBallCarrier(None))
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

    #[test]
    fn set_parameter_player_id_accepted() {
        let mut step = StepPlaceBall::new();
        assert!(step.set_parameter(&StepParameter::PlayerId("p1".into())));
        assert_eq!(step.player_id.as_deref(), Some("p1"));
    }

    #[test]
    fn set_parameter_catch_scatter_mode_accepted() {
        let mut step = StepPlaceBall::new();
        assert!(step.set_parameter(&StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ScatterBall)));
        assert_eq!(step.catch_scatter_throw_in_mode, Some(CatchScatterThrowInMode::ScatterBall));
    }
}
