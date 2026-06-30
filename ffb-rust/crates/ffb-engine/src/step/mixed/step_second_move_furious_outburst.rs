/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.StepSecondMoveFuriousOutburst`.
///
/// Second teleport move in the Furious Outburst sequence (after the attack).
/// Needs `GOTO_LABEL_ON_END` init parameter.
///
/// Java logic (executeStep):
///   - CONTINUE by default.
///   - If `end_turn` or `end_player_action` → GOTO goto_label_on_end.
///   - If `coordinate` not set:
///     - Clear selectedStabTarget flag on previously selected player.
///     - Check `stayInEndzone` (if touching down).
///     - Compute eligible squares (up to 3 away, empty, filtered by endzone if needed).
///     - Add MoveSquares, record `with_ball`.
///   - If `coordinate` is set:
///     - Animate TRICKSTER, teleport player.
///     - If `with_ball`: move ball.
///     - → NEXT_STEP + bounceBall().
///
/// Java fields: `eligibleSquares`, `endPlayerAction`, `endTurn`, `withBall`,
///              `goToLabelOnEnd`, `coordinate`.
use std::collections::HashSet;
use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter, CatchScatterThrowInMode};

/// Java: `StepSecondMoveFuriousOutburst` (mixed, BB2020 + BB2025).
pub struct StepSecondMoveFuriousOutburst {
    /// Java: `eligibleSquares`
    pub eligible_squares: HashSet<FieldCoordinate>,
    /// Java: `endPlayerAction`
    pub end_player_action: bool,
    /// Java: `endTurn`
    pub end_turn: bool,
    /// Java: `withBall`
    pub with_ball: bool,
    /// Java: `goToLabelOnEnd`
    pub goto_label_on_end: String,
    /// Java: `coordinate`
    pub coordinate: Option<FieldCoordinate>,
}

impl StepSecondMoveFuriousOutburst {
    pub fn new(goto_label_on_end: impl Into<String>) -> Self {
        Self {
            eligible_squares: HashSet::new(),
            end_player_action: false,
            end_turn: false,
            with_ball: false,
            goto_label_on_end: goto_label_on_end.into(),
            coordinate: None,
        }
    }

    fn bounce_ball(&self, game: &Game) -> Option<StepParameter> {
        let coord = self.coordinate?;
        if !self.with_ball && game.field_model.ball_coordinate == Some(coord) && game.field_model.ball_moving {
            Some(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ScatterBall))
        } else {
            None
        }
    }

    fn execute_step(&mut self, game: &mut Game) -> StepOutcome {
        if self.end_turn || self.end_player_action {
            return StepOutcome::goto(&self.goto_label_on_end);
        }

        if self.coordinate.is_none() {
            // Java: clear selectedStabTarget, compute eligible squares (radius 3, empty)
            // TODO(TargetSelectionState port): clear selected stab target flag
            // TODO(UtilServerSteps port): stayInEndzone check
            // For now wait for coordinate selection
            return StepOutcome::cont();
        }

        let coord = self.coordinate.unwrap();
        let acting_id = game.acting_player.player_id.clone();
        if let Some(ref pid) = acting_id {
            game.field_model.set_player_coordinate(pid, coord);
            if self.with_ball {
                game.field_model.ball_coordinate = Some(coord);
            }
        }

        let mut outcome = StepOutcome::next();
        if let Some(scatter_param) = self.bounce_ball(game) {
            outcome = outcome.publish(scatter_param);
        }
        outcome
    }
}

impl Default for StepSecondMoveFuriousOutburst {
    fn default() -> Self { Self::new("") }
}

impl Step for StepSecondMoveFuriousOutburst {
    fn id(&self) -> StepId { StepId::SecondMoveFuriousOutburst }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::Move { path } => {
                if let Some(&coord) = path.first() {
                    if self.eligible_squares.contains(&coord) {
                        self.coordinate = Some(coord);
                    }
                }
            }
            Action::EndTurn => { self.end_turn = true; }
            _ => {}
        }
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::EndPlayerAction(v) => { self.end_player_action = *v; true }
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
            StepParameter::GotoLabelOnEnd(v) => { self.goto_label_on_end = v.clone(); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{test_team, StepAction};
    use ffb_model::enums::{Rules, PS_STANDING, PlayerAction};
    use ffb_model::model::game::Game;
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_player(game: &mut Game, id: &str, state: u32) {
        let pos = FieldCoordinate::new(5, 5);
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
        });
        game.field_model.set_player_coordinate(id, pos);
        game.field_model.set_player_state(id, ffb_model::enums::PlayerState::new(state));
        game.acting_player.set_player(id.into(), PlayerAction::Block);
    }

    #[test]
    fn id_is_second_move_furious_outburst() {
        assert_eq!(StepSecondMoveFuriousOutburst::new("end").id(), StepId::SecondMoveFuriousOutburst);
    }

    #[test]
    fn no_coordinate_continues() {
        let mut step = StepSecondMoveFuriousOutburst::new("end");
        let mut game = make_game();
        add_player(&mut game, "att", PS_STANDING);
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn end_turn_goes_to_label() {
        let mut step = StepSecondMoveFuriousOutburst::new("end_label");
        step.end_turn = true;
        let mut game = make_game();
        add_player(&mut game, "att", PS_STANDING);
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label, Some("end_label".into()));
    }

    #[test]
    fn coordinate_set_teleports_and_next_steps() {
        let mut step = StepSecondMoveFuriousOutburst::new("end");
        step.coordinate = Some(FieldCoordinate::new(8, 8));
        let mut game = make_game();
        add_player(&mut game, "att", PS_STANDING);
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.field_model.player_coordinate("att"), Some(FieldCoordinate::new(8, 8)));
    }

    #[test]
    fn set_parameter_accepts_expected_keys() {
        let mut step = StepSecondMoveFuriousOutburst::new("x");
        assert!(step.set_parameter(&StepParameter::EndPlayerAction(true)));
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.set_parameter(&StepParameter::GotoLabelOnEnd("y".into())));
        assert!(!step.set_parameter(&StepParameter::HomeTeam(false)));
    }
}
