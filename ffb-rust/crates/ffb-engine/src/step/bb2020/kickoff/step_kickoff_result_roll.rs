/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.kickoff.StepKickoffResultRoll`.
///
/// Step in kickoff sequence to roll kickoff result (BB2020).
///
/// BB2020 kickoff table (2d6):
///  2  → Get the Ref
///  3  → Time-out
///  4  → Solid Defence
///  5  → High Kick
///  6  → Cheering Fans
///  7  → Brilliant Coaching
///  8  → Weather Change
///  9  → Quick Snap
///  10 → Blitz
///  11 → Officious Ref
///  12 → Pitch Invasion
///
/// Java overtime handling:
///  - half < 3 (or OVERTIME_KICK_OFF_ALL): normal 2d6 roll.
///  - OVERTIME_KICK_OFF_RANDOM_BLITZ_OR_SOLID_DEFENCE: roll d6, pick from 6 valid combos.
///  - OVERTIME_KICK_OFF_BLITZ: forced Blitz.
///  - OVERTIME_KICK_OFF_SOLID_DEFENCE: forced SolidDefence.
///  - Otherwise: show dialog and wait (StepAction::Continue).
///
/// Sets stepParameter KICKOFF_RESULT for all steps on the stack.
///
/// TODO(KickoffResultRoll-overtime): GameOptionId::OVERTIME_KICK_OFF_RESULTS handling deferred.
/// TODO(KickoffResultRoll-dialog): CLIENT_KICK_OFF_RESULT_CHOICE dialog path deferred.
use ffb_model::enums::KickoffResult;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepKickoffResultRoll` (bb2020/kickoff).
pub struct StepKickoffResultRoll {
    /// Java: `fKickoffResult` — None means "not yet rolled/chosen".
    pub kickoff_result: Option<KickoffResult>,
}

impl StepKickoffResultRoll {
    pub fn new() -> Self {
        Self { kickoff_result: None }
    }

    fn execute_step(&mut self, _game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: UtilServerDialog.hideDialog(getGameState())

        if self.kickoff_result.is_none() {
            // TODO(KickoffResultRoll-overtime): check game.half < 3 and
            // GameOptionId::OVERTIME_KICK_OFF_RESULTS to select the overtime path:
            //   - OVERTIME_KICK_OFF_RANDOM_BLITZ_OR_SOLID_DEFENCE: roll d6, use validRolls[index].
            //   - OVERTIME_KICK_OFF_BLITZ: fKickoffResult = KickoffResult::BLITZ; goto NextStep.
            //   - OVERTIME_KICK_OFF_SOLID_DEFENCE: fKickoffResult = KickoffResult::SOLID_DEFENCE; goto NextStep.
            //   - Otherwise: showDialog + return Continue.
            //
            // For now: always roll normally (covers half < 3 and the default path).

            // Java: rollKickoff = getGameState().getDiceRoller().rollKickoff() → two d6
            // Java: fKickoffResult = DiceInterpreter.interpretRollKickoff(game, rollKickoff)
            let roll = rng.d6_two();
            self.kickoff_result = Some(kickoff_result_for_roll(roll));
        }

        let result = self.kickoff_result.unwrap();
        // Java: getResult().addReport(new ReportKickoffResult(fKickoffResult, rollKickoff))
        // Java: publishParameter(new StepParameter(StepParameterKey.KICKOFF_RESULT, fKickoffResult))
        // Java: getResult().setNextAction(StepAction.NEXT_STEP)
        StepOutcome::next().publish(StepParameter::KickoffResult(result))
    }
}

impl Default for StepKickoffResultRoll {
    fn default() -> Self { Self::new() }
}

impl Step for StepKickoffResultRoll {
    fn id(&self) -> StepId { StepId::KickoffResultRoll }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: super.start(); executeStep()
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: commandStatus = super.handleCommand(pReceivedCommand)
        // Java: if NetCommandId.CLIENT_KICK_OFF_RESULT_CHOICE:
        //   command = (ClientCommandKickOffResultChoice) pReceivedCommand.getCommand()
        //   fKickoffResult = command.getKickoffResult()
        //   commandStatus = EXECUTE_STEP
        // Java: if commandStatus == EXECUTE_STEP: executeStep()
        //
        // TODO(KickoffResultRoll-dialog): When Action::KickoffResultChoice is added,
        // match it here to set self.kickoff_result before calling execute_step.
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

/// BB2020 kickoff event table mapping (2d6 → KickoffResult).
/// Mirrors Java `com.fumbbl.ffb.kickoff.bb2020.KickoffResultMapping`.
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
        10 => KickoffResult::Blitz,
        11 => KickoffResult::OficiousRef,
        12 => KickoffResult::PitchInvasion,
        // Out-of-range (should never happen with 2d6):
        _  => KickoffResult::BrilliantCoaching,
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    #[test]
    fn id_is_kickoff_result_roll() {
        assert_eq!(StepKickoffResultRoll::new().id(), StepId::KickoffResultRoll);
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
            (10, KickoffResult::Blitz),
            (11, KickoffResult::OficiousRef),
            (12, KickoffResult::PitchInvasion),
        ];
        for (roll, expected) in cases {
            assert_eq!(kickoff_result_for_roll(roll), expected, "roll={roll}");
        }
    }

    #[test]
    fn bb2020_table_differs_from_bb2025_at_10_and_11() {
        // BB2020: roll 10 = Blitz, roll 11 = OficiousRef
        // BB2025: roll 10 = Charge, roll 11 = DodgySnack
        assert_eq!(kickoff_result_for_roll(10), KickoffResult::Blitz);
        assert_eq!(kickoff_result_for_roll(11), KickoffResult::OficiousRef);
    }

    #[test]
    fn handle_command_returns_next_step() {
        let mut game = make_game();
        let mut step = StepKickoffResultRoll::new();
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }
}
