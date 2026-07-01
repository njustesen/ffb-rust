use ffb_model::enums::KickoffResult;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// Rolls 2d6 and maps the result to the BB2025 kickoff event table; publishes
/// `KickoffResult`.
///
/// BB2025 kickoff table (2d6 total):
///  2  → Get the Ref
///  3  → Time-out
///  4  → Solid Defence
///  5  → High Kick
///  6  → Cheering Fans
///  7  → Brilliant Coaching
///  8  → Weather Change
///  9  → Quick Snap
///  10 → Charge
///  11 → Dodgy Snack
///  12 → Pitch Invasion
///
/// Java also handles overtime options (GameOptionId::OVERTIME_KICK_OFF_RESULTS) and
/// a client-commanded result choice dialog.  Those paths are TODO.
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.kickoff.StepKickoffResultRoll`.
pub struct StepKickoffResultRoll {
    /// Java: fKickoffResult — None means "not yet rolled".
    pub kickoff_result: Option<KickoffResult>,
}

impl StepKickoffResultRoll {
    pub fn new() -> Self {
        Self { kickoff_result: None }
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
        // Java CLIENT_KICK_OFF_RESULT_CHOICE: set fKickoffResult from the command.
        // DEFERRED: handle overtime kickoff choice dialog (GameOptionId::OVERTIME_KICK_OFF_RESULTS).
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

impl StepKickoffResultRoll {
    fn execute_step(&mut self, _game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if self.kickoff_result.is_none() {
            // Java: rollKickoff() → two individual d6 summed; we use d6_two().
            let roll = rng.d6_two();
            self.kickoff_result = Some(kickoff_result_for_roll(roll));
        }

        let result = self.kickoff_result.unwrap();
        StepOutcome::next()
            .publish(StepParameter::KickoffResult(result))
    }
}

/// BB2025 kickoff event table mapping (2d6 → KickoffResult).
/// Mirrors Java `com.fumbbl.ffb.kickoff.bb2025.KickoffResultMapping`.
fn kickoff_result_for_roll(roll: i32) -> KickoffResult {
    match roll {
        2  => KickoffResult::GetTheRef,
        3  => KickoffResult::TimeOut,
        4  => KickoffResult::SolidDefence,
        5  => KickoffResult::HighKick,
        6  => KickoffResult::CheeringFans,
        7  => KickoffResult::BrilliantCoaching,
        8  => KickoffResult::WeatherChange,
        9  => KickoffResult::QuickSnap,
        10 => KickoffResult::Charge,
        11 => KickoffResult::DodgySnack,
        12 => KickoffResult::PitchInvasion,
        // Out-of-range (should never happen with 2d6):
        _ => KickoffResult::BrilliantCoaching,
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
    fn start_returns_next_step() {
        let mut game = make_game();
        let mut step = StepKickoffResultRoll::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn start_publishes_kickoff_result_parameter() {
        let mut game = make_game();
        let mut step = StepKickoffResultRoll::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::KickoffResult(_))));
    }

    #[test]
    fn kickoff_result_stored_after_roll() {
        let mut game = make_game();
        let mut step = StepKickoffResultRoll::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(step.kickoff_result.is_some());
    }

    #[test]
    fn pre_set_result_reused_without_re_roll() {
        let mut game = make_game();
        let mut step = StepKickoffResultRoll::new();
        step.kickoff_result = Some(KickoffResult::HighKick);
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Should reuse the pre-set result, not overwrite it.
        assert_eq!(step.kickoff_result, Some(KickoffResult::HighKick));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::KickoffResult(KickoffResult::HighKick))));
    }

    #[test]
    fn kickoff_table_all_rolls() {
        let cases = [
            (2,  KickoffResult::GetTheRef),
            (3,  KickoffResult::TimeOut),
            (4,  KickoffResult::SolidDefence),
            (5,  KickoffResult::HighKick),
            (6,  KickoffResult::CheeringFans),
            (7,  KickoffResult::BrilliantCoaching),
            (8,  KickoffResult::WeatherChange),
            (9,  KickoffResult::QuickSnap),
            (10, KickoffResult::Charge),
            (11, KickoffResult::DodgySnack),
            (12, KickoffResult::PitchInvasion),
        ];
        for (roll, expected) in cases {
            assert_eq!(kickoff_result_for_roll(roll), expected, "roll={roll}");
        }
    }
}
