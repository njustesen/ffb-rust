use ffb_model::enums::{PassResult, PlayerState};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// Rolls the throw for the throw-team-mate sequence.
///
/// Java executeStep delegates entirely to getGameState().executeStepHooks(this, state).
/// The hooks system (StepHook / StepThrowTeamMateHook) resolves:
///   1. Throw roll via pass mechanics (similar to StepPass but for TTM).
///   2. Bullseye modifier (using_bullseye flag).
///   3. PassResult routing: ACCURATE -> scatter at pass coord; INACCURATE -> scatter from pass
///      coord; FUMBLE -> scatter from thrower square.
///
/// handleCommand additionally handles CLIENT_USE_SKILL -> handleSkillCommand (re-roll prompt).
///
/// Unported utilities:
///   TODO: executeStepHooks (StepHook / StepThrowTeamMateHook infrastructure)
///   TODO: throw roll, range ruler lookup, Bullseye modifier
///   TODO: PassResult routing to scatterPlayerSequence
///   TODO: handleSkillCommand (AbstractStepWithReRoll re-roll infrastructure)
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.ttm.StepThrowTeamMate`.
pub struct StepThrowTeamMate {
    /// Java: state.thrownPlayerId
    pub thrown_player_id: Option<String>,
    /// Java: state.thrownPlayerState
    pub thrown_player_state: Option<PlayerState>,
    /// Java: state.thrownPlayerHasBall
    pub thrown_player_has_ball: bool,
    /// Java: state.passResult
    pub pass_result: Option<PassResult>,
    /// Java: state.kicked (IS_KICKED_PLAYER init param)
    pub kicked: bool,
    /// Java: state.usingBullseye (Boolean tristate)
    pub using_bullseye: Option<bool>,
    // AbstractStepWithReRoll stubs
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
}

impl StepThrowTeamMate {
    pub fn new() -> Self {
        Self {
            thrown_player_id: None,
            thrown_player_state: None,
            thrown_player_has_ball: false,
            pass_result: None,
            kicked: false,
            using_bullseye: None,
            re_rolled_action: None,
            re_roll_source: None,
        }
    }
}

impl Default for StepThrowTeamMate {
    fn default() -> Self { Self::new() }
}

impl Step for StepThrowTeamMate {
    fn id(&self) -> StepId { StepId::ThrowTeamMate }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_USE_SKILL -> handleSkillCommand (re-roll / Bullseye) -> executeStep
        // TODO: detect UseSkill action, update using_bullseye / re-roll fields
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::ThrownPlayerId(v) => { self.thrown_player_id = v.clone(); true }
            StepParameter::ThrownPlayerState(v) => { self.thrown_player_state = Some(*v); true }
            StepParameter::ThrownPlayerHasBall(v) => { self.thrown_player_has_ball = *v; true }
            _ => false,
        }
    }
}

impl StepThrowTeamMate {
    fn execute_step(&self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // TODO: executeStepHooks(this, state) — hooks perform throw roll and routing.
        //
        // Hook logic summary (StepThrowTeamMateHook):
        //   thrower = game.actingPlayer.player
        //   passModifiers = PassModifierFactory.findModifiers(PassContext(...))
        //   minimumRoll = passMechanic.minimumRollThrowTeamMate(thrower, passModifiers)
        //   roll = diceRoller.rollSkill()
        //   successful = DiceInterpreter.isPassSuccessful(roll, minimumRoll)
        //   if successful (and not re-rolling):
        //     state.passResult = ACCURATE
        //   else if re-rolling THROW_TEAM_MATE:
        //     use re-roll or fail -> INACCURATE / FUMBLE
        //   else:
        //     ask for re-roll -> Continue
        //   push scatterPlayerSequence(..., throwScatter=true/false based on passResult)
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
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn start_returns_next_step() {
        let mut game = make_game();
        let mut step = StepThrowTeamMate::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_thrown_player_id() {
        let mut step = StepThrowTeamMate::new();
        assert!(step.set_parameter(&StepParameter::ThrownPlayerId(Some("p1".into()))));
        assert_eq!(step.thrown_player_id.as_deref(), Some("p1"));
    }

    #[test]
    fn set_thrown_player_has_ball() {
        let mut step = StepThrowTeamMate::new();
        assert!(step.set_parameter(&StepParameter::ThrownPlayerHasBall(true)));
        assert!(step.thrown_player_has_ball);
    }

    #[test]
    fn set_unknown_parameter_rejected() {
        let mut step = StepThrowTeamMate::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn handle_command_returns_next_step() {
        let mut game = make_game();
        let mut step = StepThrowTeamMate::new();
        let out = step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }
}
