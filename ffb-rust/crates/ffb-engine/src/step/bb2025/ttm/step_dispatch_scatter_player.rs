use ffb_model::enums::{PassResult, PlayerState};
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepAction, StepId, StepParameter};
use crate::step::generator::bb2025::ScatterPlayer;
use crate::step::generator::bb2025::scatter_player::ScatterPlayerParams;

/// Dispatches the scatter-player sequence after a TTM throw.
///
/// Java start() logic:
///   if passResult==FUMBLE && isKickedPlayer:
///     addReport(ReportKickTeamMateFumble); NEXT_STEP; return
///
///   thrower = game.actingPlayer.player
///   throwerCoordinate = fieldModel.getPlayerCoordinate(thrower)
///   thrownPlayer = game.getPlayerById(thrownPlayerId)
///   scattersSingleDirection = thrownPlayer != null
///     && thrownPlayer.hasUsableSkillProperty(ttmScattersInSingleDirection, oldPlayerState)
///
///   throwScatter:
///     FUMBLE -> false; scattersSingleDirection=false
///     INACCURATE / ACCURATE:
///       if usingBullseye -> throwScatter=false; scattersSingleDirection=false
///       else -> throwScatter=true
///
///   push ScatterPlayer sequence (..., throwScatter, scattersSingleDirection, false, false)
///   publish USING_BULLSEYE=usingBullseye; IS_KICKED_PLAYER=isKickedPlayer; OLD_DEFENDER_STATE=oldPlayerState
///   NEXT_STEP
///
/// Unported utilities:
///   TODO: ScatterPlayer sequence generator (SequenceGenerator.ScatterPlayer.pushSequence)
///   TODO: thrownPlayer.hasUsableSkillProperty(ttmScattersInSingleDirection, oldPlayerState)
///   TODO: game.actingPlayer.getPlayer() / fieldModel.getPlayerCoordinate
///   TODO: ReportKickTeamMateFumble
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.ttm.StepDispatchScatterPlayer`.
pub struct StepDispatchScatterPlayer {
    /// Java: thrownPlayerId
    pub thrown_player_id: Option<String>,
    /// Java: thrownPlayerState
    pub thrown_player_state: Option<PlayerState>,
    /// Java: oldPlayerState
    pub old_player_state: Option<PlayerState>,
    /// Java: thrownPlayerHasBall (init false)
    pub thrown_player_has_ball: bool,
    /// Java: isKickedPlayer (init param IS_KICKED_PLAYER)
    pub is_kicked_player: bool,
    /// Java: usingBullseye
    pub using_bullseye: bool,
    /// Java: passResult (init FUMBLE)
    pub pass_result: PassResult,
}

impl StepDispatchScatterPlayer {
    pub fn new() -> Self {
        Self {
            thrown_player_id: None,
            thrown_player_state: None,
            old_player_state: None,
            thrown_player_has_ball: false,
            is_kicked_player: false,
            using_bullseye: false,
            pass_result: PassResult::Fumble,
        }
    }
}

impl Default for StepDispatchScatterPlayer {
    fn default() -> Self { Self::new() }
}

impl Step for StepDispatchScatterPlayer {
    fn id(&self) -> StepId { StepId::DispatchScatterPlayer }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::ThrownPlayerId(v) => { self.thrown_player_id = v.clone(); true }
            StepParameter::ThrownPlayerState(v) => { self.thrown_player_state = Some(*v); true }
            StepParameter::ThrownPlayerHasBall(v) => { self.thrown_player_has_ball = *v; true }
            StepParameter::PassResultParam(v) => { self.pass_result = *v; true }
            StepParameter::IsKickedPlayer(v) => { self.is_kicked_player = *v; true }
            StepParameter::OldDefenderState(v) => { self.old_player_state = Some(*v); true }
            StepParameter::UsingBullseye(v) => { self.using_bullseye = *v; true }
            _ => false,
        }
    }
}

impl StepDispatchScatterPlayer {
    fn execute_step(&self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: if passResult==FUMBLE && isKickedPlayer: report KickTeamMateFumble; NEXT_STEP
        if self.pass_result == PassResult::Fumble && self.is_kicked_player {
            // DEFERRED: addReport(ReportKickTeamMateFumble) when event system is ported
            return StepOutcome::next();
        }

        // Java: thrower = game.actingPlayer.getPlayer()
        // Java: throwerCoordinate = game.fieldModel.getPlayerCoordinate(thrower)
        let thrower_coordinate = game.acting_player.player_id.as_deref()
            .and_then(|id| game.field_model.player_coordinate(id));

        // Java: scattersSingleDirection = thrownPlayer != null
        //   && thrownPlayer.hasUsableSkillProperty(ttmScattersInSingleDirection, oldPlayerState)
        // hasUsableSkillProperty = hasSkillProperty && state.isStanding() && !state.isDistracted()
        let scatters_single_direction = self.thrown_player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill_property(NamedProperties::TTM_SCATTERS_IN_SINGLE_DIRECTION))
            .unwrap_or(false)
            && self.old_player_state
                .map(|s| s.is_standing() && !s.is_confused())
                .unwrap_or(false);

        // Java: throwScatter / scattersSingleDirection reset based on passResult
        // Java: FUMBLE → throwScatter=false, scattersSingleDirection=false
        // Java: INACCURATE/ACCURATE + bullseye → throwScatter=false, scattersSingleDirection=false
        // Java: INACCURATE/ACCURATE → throwScatter=true
        let (throw_scatter, has_swoop) = match self.pass_result {
            PassResult::Fumble => (false, false),
            PassResult::Inaccurate | PassResult::Complete => {
                if self.using_bullseye { (false, false) }
                else { (true, scatters_single_direction) }
            }
            _ => (false, false),
        };

        let seq = ScatterPlayer::build_sequence(&ScatterPlayerParams {
            thrown_player_id: self.thrown_player_id.clone(),
            thrown_player_state: self.thrown_player_state,
            thrown_player_has_ball: self.thrown_player_has_ball,
            thrown_player_coordinate: thrower_coordinate,
            throw_scatter,
            has_swoop,
        });

        StepOutcome::next()
            .push_seq(seq)
            .publish(StepParameter::UsingBullseye(self.using_bullseye))
            .publish(StepParameter::IsKickedPlayer(self.is_kicked_player))
            .publish(StepParameter::OldDefenderState(
                self.old_player_state.unwrap_or(PlayerState(0)),
            ))
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
    fn fumble_kicked_returns_next_step() {
        let mut game = make_game();
        let mut step = StepDispatchScatterPlayer::new();
        step.pass_result = PassResult::Fumble;
        step.is_kicked_player = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn accurate_pass_pushes_scatter_player_sequence() {
        let mut game = make_game();
        let mut step = StepDispatchScatterPlayer::new();
        step.pass_result = PassResult::Complete;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::InitScatterPlayer);
    }

    #[test]
    fn fumble_non_kicked_pushes_scatter_player_sequence_no_throw_scatter() {
        let mut game = make_game();
        let mut step = StepDispatchScatterPlayer::new();
        step.pass_result = PassResult::Fumble;
        step.is_kicked_player = false;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::InitScatterPlayer);
    }

    #[test]
    fn bullseye_accurate_pass_no_throw_scatter() {
        let mut game = make_game();
        let mut step = StepDispatchScatterPlayer::new();
        step.pass_result = PassResult::Complete;
        step.using_bullseye = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.pushes.len(), 1);
    }

    #[test]
    fn set_pass_result_accepted() {
        let mut step = StepDispatchScatterPlayer::default();
        assert!(step.set_parameter(&StepParameter::PassResultParam(PassResult::Complete)));
        assert_eq!(step.pass_result, PassResult::Complete);
    }

    #[test]
    fn set_using_bullseye_accepted() {
        let mut step = StepDispatchScatterPlayer::default();
        assert!(step.set_parameter(&StepParameter::UsingBullseye(true)));
        assert!(step.using_bullseye);
    }

    #[test]
    fn default_pass_result_is_fumble() {
        let step = StepDispatchScatterPlayer::default();
        assert_eq!(step.pass_result, PassResult::Fumble);
    }
}
