/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.StepEndThenIStartedBlastin`.
///
/// Ends the "Then I Started Blastin" sequence.
///
/// Java logic (executeStep):
///   - If `end_turn`, `end_player_action`, player is prone/stunned/ko/casualty:
///     restore the previous TurnMode (or Regular) and push an EndPlayerAction sequence.
///   - Otherwise: update move squares + dice decorations (engine-side view updates only).
///
/// The Rust translation publishes the appropriate EndPlayerAction/EndTurn parameters
/// and moves to the next step; the driver and generator handle the sequence push.
///
/// Java fields: `endPlayerAction`, `endTurn`.
use ffb_model::enums::{PS_KNOCKED_OUT, TurnMode};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepEndThenIStartedBlastin` (mixed, BB2020 + BB2025).
pub struct StepEndThenIStartedBlastin {
    /// Java: `endPlayerAction`
    end_player_action: bool,
    /// Java: `endTurn`
    end_turn: bool,
}

impl StepEndThenIStartedBlastin {
    pub fn new() -> Self {
        Self { end_player_action: false, end_turn: false }
    }

    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        let acting_id = game.acting_player.player_id.clone();
        let player_state = acting_id.as_deref()
            .and_then(|id| game.field_model.player_state(id));

        let prone_or_worse = player_state.map(|s| {
            s.is_prone_or_stunned() || s.is_casualty() || s.base() == PS_KNOCKED_OUT
        }).unwrap_or(false);

        if self.end_turn || self.end_player_action || prone_or_worse {
            // Java: restore last_turn_mode if it's a basic mode, otherwise set REGULAR
            if game.turn_mode != TurnMode::Regular {
                match game.last_turn_mode {
                    Some(ltm) => game.turn_mode = ltm,
                    None => game.turn_mode = TurnMode::Regular,
                }
            }
            // Java: pushes EndPlayerAction sequence onto stack
            // We publish the parameters for the EndPlayerAction generator
            StepOutcome::next()
                .publish(StepParameter::EndPlayerAction(self.end_player_action))
                .publish(StepParameter::EndTurn(self.end_turn))
        } else {
            // Java: updateMoveSquares + updateDiceDecorations (view-only; no-op in headless)
            StepOutcome::next()
        }
    }
}

impl Default for StepEndThenIStartedBlastin {
    fn default() -> Self { Self::new() }
}

impl Step for StepEndThenIStartedBlastin {
    fn id(&self) -> StepId { StepId::EndThenIStartedBlastin }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::EndPlayerAction(v) => { self.end_player_action = *v; true }
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{test_team, StepAction};
    use ffb_model::enums::{Rules, PS_STANDING, PS_PRONE, PS_KNOCKED_OUT, PlayerAction};
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
    fn id_is_end_then_i_started_blastin() {
        assert_eq!(StepEndThenIStartedBlastin::new().id(), StepId::EndThenIStartedBlastin);
    }

    #[test]
    fn standing_player_no_flags_returns_next_without_epa() {
        let mut step = StepEndThenIStartedBlastin::new();
        let mut game = make_game();
        add_player(&mut game, "att", PS_STANDING);
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
        // No end_player_action published (no flags set, standing player)
        let has_epa_true = out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true)));
        assert!(!has_epa_true);
    }

    #[test]
    fn end_turn_flag_publishes_epa_and_end_turn() {
        let mut step = StepEndThenIStartedBlastin::new();
        step.end_turn = true;
        let mut game = make_game();
        add_player(&mut game, "att", PS_STANDING);
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
        let has_end_turn_true = out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true)));
        assert!(has_end_turn_true, "should publish EndTurn(true)");
    }

    #[test]
    fn prone_player_triggers_end_sequence() {
        let mut step = StepEndThenIStartedBlastin::new();
        let mut game = make_game();
        add_player(&mut game, "att", PS_PRONE);
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
        let has_epa = out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(_)));
        assert!(has_epa, "prone player should trigger end sequence parameters");
    }

    #[test]
    fn set_parameter_accepts_end_player_action_and_end_turn() {
        let mut step = StepEndThenIStartedBlastin::new();
        assert!(step.set_parameter(&StepParameter::EndPlayerAction(true)));
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(!step.set_parameter(&StepParameter::HomeTeam(true)));
    }
}
