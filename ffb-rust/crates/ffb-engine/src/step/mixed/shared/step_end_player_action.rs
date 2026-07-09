/// Ends the current player action and transitions to the next game phase.
///
/// This step is the Rust equivalent of the Java `EndPlayerAction` sequence generator.
/// In Java the generator pushes an `InitFeeding … EndFeeding` sequence; in the Rust
/// rewrite this step:
///   1. Records the acting player in `acted_player_ids`.
///   2. Clears `acting_player`.
///   3. Pushes the edition-appropriate `EndPlayerAction` sequence (bb2025 generator).
///
/// When `end_turn` is true (e.g. after a turnover) the step pushes the EndTurn
/// sequence instead, bypassing feeding entirely — mirroring the short-circuit path
/// in `com.fumbbl.ffb.server.step.generator.bb2025.EndPlayerAction.pushSequence` and
/// `StepEndFeeding.executeStep()`.
///
/// Parameters accepted:
///   - `EndPlayerAction(bool)` — forwarded to `InitFeeding`.
///   - `EndTurn(bool)`         — if true, push EndTurn sequence instead of EndFeeding.
///   - `FeedingAllowed(bool)`  — forwarded to `InitFeeding`.
///   - `CheckForgo(bool)`      — forwarded to `EndFeeding`.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter, SequenceStep};
use crate::step::generator::bb2025::end_player_action::{EndPlayerAction, EndPlayerActionParams};
use crate::step::sequences::end_turn_sequence;

/// Java: the `EndPlayerAction` sequence generator (bb2025).
///
/// Fields mirror `EndPlayerAction.SequenceParams`:
///   `feedingAllowed`, `endPlayerAction`, `endTurn`, `checkForgo`.
pub struct StepEndPlayerAction {
    /// Java: params.feedingAllowed — forwarded to InitFeeding.
    pub feeding_allowed: bool,
    /// Java: params.endPlayerAction — forwarded to InitFeeding.
    pub end_player_action: bool,
    /// Java: params.endTurn — if true the EndTurn sequence is pushed instead.
    pub end_turn: bool,
    /// Java: params.checkForgo — forwarded to EndFeeding.
    pub check_forgo: bool,
}

impl StepEndPlayerAction {
    pub fn new() -> Self {
        Self {
            feeding_allowed: false,
            end_player_action: false,
            end_turn: false,
            check_forgo: false,
        }
    }

    fn execute_step(&mut self, game: &mut Game) -> StepOutcome {
        // Java generator preamble: record acting player, clear activation context.
        if let Some(pid) = game.acting_player.player_id.take() {
            let td = if game.home_playing {
                &mut game.turn_data_home
            } else {
                &mut game.turn_data_away
            };
            td.acted_player_ids.push(pid);
        }
        game.acting_player.clear();

        // Short-circuit: if end_turn push the EndTurn sequence (no feeding needed).
        if self.end_turn {
            return StepOutcome::next().push_seq(end_turn_sequence(self.check_forgo));
        }

        // Default: push the EndPlayerAction (feeding) sequence.
        let params = EndPlayerActionParams {
            feeding_allowed: self.feeding_allowed,
            end_player_action: self.end_player_action,
            end_turn: false,
            check_forgo: self.check_forgo,
        };
        StepOutcome::next().push_seq(EndPlayerAction::build_sequence(&params))
    }
}

impl Default for StepEndPlayerAction {
    fn default() -> Self { Self::new() }
}

impl Step for StepEndPlayerAction {
    fn id(&self) -> StepId { StepId::EndPlayerAction }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::EndPlayerAction(v) => { self.end_player_action = *v; true }
            StepParameter::EndTurn(v)         => { self.end_turn = *v; true }
            StepParameter::FeedingAllowed(v)  => { self.feeding_allowed = *v; true }
            StepParameter::CheckForgo(v)      => { self.check_forgo = *v; true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{test_team, StepAction};
    use ffb_model::enums::Rules;
    use ffb_model::model::game::Game;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn id_is_end_player_action() {
        assert_eq!(StepEndPlayerAction::new().id(), StepId::EndPlayerAction);
    }

    #[test]
    fn start_returns_next_step() {
        let mut step = StepEndPlayerAction::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn start_records_acting_player_in_acted_ids() {
        let mut step = StepEndPlayerAction::new();
        let mut game = make_game();
        game.home_playing = true;
        game.acting_player.player_id = Some("p1".to_string());
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(game.turn_data_home.acted_player_ids.contains(&"p1".to_string()));
    }

    #[test]
    fn start_clears_acting_player() {
        let mut step = StepEndPlayerAction::new();
        let mut game = make_game();
        game.acting_player.player_id = Some("p2".to_string());
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(game.acting_player.player_id.is_none());
    }

    #[test]
    fn end_turn_true_pushes_end_turn_sequence() {
        let mut step = StepEndPlayerAction::new();
        step.end_turn = true;
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        // end_turn_sequence ends with StepId::EndTurn
        let last = out.pushes.last().and_then(|s| s.last());
        assert!(last.is_some());
        assert_eq!(last.unwrap().step_id, StepId::EndTurn);
    }

    #[test]
    fn normal_path_pushes_feeding_sequence() {
        let mut step = StepEndPlayerAction::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        // The EndPlayerAction sequence starts with RemoveTargetSelectionState.
        let first = out.pushes.last().and_then(|s| s.first());
        assert!(first.is_some());
        assert_eq!(first.unwrap().step_id, StepId::RemoveTargetSelectionState);
    }

    #[test]
    fn set_parameter_end_player_action_accepted() {
        let mut step = StepEndPlayerAction::new();
        assert!(step.set_parameter(&StepParameter::EndPlayerAction(true)));
        assert!(step.end_player_action);
    }

    #[test]
    fn set_parameter_end_turn_accepted() {
        let mut step = StepEndPlayerAction::new();
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.end_turn);
    }

    #[test]
    fn set_parameter_feeding_allowed_accepted() {
        let mut step = StepEndPlayerAction::new();
        assert!(step.set_parameter(&StepParameter::FeedingAllowed(true)));
        assert!(step.feeding_allowed);
    }

    #[test]
    fn set_parameter_check_forgo_accepted() {
        let mut step = StepEndPlayerAction::new();
        assert!(step.set_parameter(&StepParameter::CheckForgo(true)));
        assert!(step.check_forgo);
    }

    #[test]
    fn set_parameter_rejects_unknown() {
        let mut step = StepEndPlayerAction::new();
        assert!(!step.set_parameter(&StepParameter::AdminMode(false)));
    }

    #[test]
    fn no_acting_player_does_not_panic() {
        let mut step = StepEndPlayerAction::new();
        let mut game = make_game();
        // acting_player.player_id is None by default — should not panic
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn away_team_acting_player_recorded_in_away_turn_data() {
        let mut step = StepEndPlayerAction::new();
        let mut game = make_game();
        game.home_playing = false;
        game.acting_player.player_id = Some("px".to_string());
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(game.turn_data_away.acted_player_ids.contains(&"px".to_string()));
        assert!(!game.turn_data_home.acted_player_ids.contains(&"px".to_string()));
    }
}
