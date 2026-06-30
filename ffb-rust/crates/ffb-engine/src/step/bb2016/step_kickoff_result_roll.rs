/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.StepKickoffResultRoll`.
///
/// Rolls 2D6 and maps to a BB2016 kickoff table result, then publishes it.
/// Java: `DiceInterpreter.interpretRollKickoff(game, roll)` → `KickoffResultFactory.forRoll(2d6)`.
///
/// BB2016 kickoff table:
///  2→GetTheRef, 3→Riot, 4→PerfectDefence, 5→HighKick, 6→CheeringFans,
///  7→WeatherChange, 8→BrilliantCoaching, 9→QuickSnap, 10→Blitz, 11→ThrowARock, 12→PitchInvasion
use ffb_model::enums::KickoffResult;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepKickoffResultRoll` (bb2016).
pub struct StepKickoffResultRoll {
    /// Java: `fKickoffResult` — the last rolled result.
    kickoff_result: Option<KickoffResult>,
}

impl StepKickoffResultRoll {
    pub fn new() -> Self { Self { kickoff_result: None } }

    /// BB2016 kickoff table: 2D6 → `KickoffResult`.
    fn interpret_roll(roll: i32) -> KickoffResult {
        match roll {
            2  => KickoffResult::GetTheRef,
            3  => KickoffResult::Riot,
            4  => KickoffResult::PerfectDefence,
            5  => KickoffResult::HighKick,
            6  => KickoffResult::CheeringFans,
            7  => KickoffResult::WeatherChange,
            8  => KickoffResult::BrilliantCoaching,
            9  => KickoffResult::QuickSnap,
            10 => KickoffResult::Blitz,
            11 => KickoffResult::ThrowARock,
            _  => KickoffResult::PitchInvasion, // 12
        }
    }

    fn execute_step(&mut self, _game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let roll = rng.d6_two();
        let result = Self::interpret_roll(roll);
        self.kickoff_result = Some(result);
        StepOutcome::next()
            .publish(StepParameter::KickoffResult(result))
    }
}

impl Default for StepKickoffResultRoll {
    fn default() -> Self { Self::new() }
}

impl Step for StepKickoffResultRoll {
    fn id(&self) -> StepId { StepId::KickoffResultRoll }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    #[test]
    fn id_is_kickoff_result_roll() {
        assert_eq!(StepKickoffResultRoll::new().id(), StepId::KickoffResultRoll);
    }

    #[test]
    fn interpret_roll_covers_all_values() {
        assert_eq!(StepKickoffResultRoll::interpret_roll(2),  KickoffResult::GetTheRef);
        assert_eq!(StepKickoffResultRoll::interpret_roll(3),  KickoffResult::Riot);
        assert_eq!(StepKickoffResultRoll::interpret_roll(4),  KickoffResult::PerfectDefence);
        assert_eq!(StepKickoffResultRoll::interpret_roll(5),  KickoffResult::HighKick);
        assert_eq!(StepKickoffResultRoll::interpret_roll(6),  KickoffResult::CheeringFans);
        assert_eq!(StepKickoffResultRoll::interpret_roll(7),  KickoffResult::WeatherChange);
        assert_eq!(StepKickoffResultRoll::interpret_roll(8),  KickoffResult::BrilliantCoaching);
        assert_eq!(StepKickoffResultRoll::interpret_roll(9),  KickoffResult::QuickSnap);
        assert_eq!(StepKickoffResultRoll::interpret_roll(10), KickoffResult::Blitz);
        assert_eq!(StepKickoffResultRoll::interpret_roll(11), KickoffResult::ThrowARock);
        assert_eq!(StepKickoffResultRoll::interpret_roll(12), KickoffResult::PitchInvasion);
    }

    #[test]
    fn publishes_kickoff_result_parameter() {
        let mut step = StepKickoffResultRoll::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        let has_result = outcome.published.iter().any(|p| matches!(p, StepParameter::KickoffResult(_)));
        assert!(has_result);
    }

    #[test]
    fn returns_next_step() {
        let mut step = StepKickoffResultRoll::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, StepAction::NextStep));
    }

    #[test]
    fn stores_kickoff_result() {
        let mut step = StepKickoffResultRoll::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(step.kickoff_result.is_some());
    }
}
