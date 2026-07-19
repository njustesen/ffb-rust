/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.foul.StepEjectPlayer`.
///
/// Removes the spotted fouler from the field (puts them in the box) and ends the turn.
/// If the fouler had the ball, also scatters it.
///
/// Java: `executeStepHooks(this, state)` is called unconditionally at the top of
/// `executeStep()` — its return value is discarded (void call), but its side effects
/// (SneakyGit's BB2025 `SneakyGitEjectPlayerModifier`, which may change the ejected
/// player's state to BANNED/KNOCKED_OUT and set the send-to-box reason/turn/half) must
/// land on `game` before `UtilBox.putPlayerIntoBox` runs.
/// `UtilServerGame.updatePlayerStateDependentProperties` is deferred.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_box::UtilBox;
use crate::action::Action;
use crate::skill_behaviour::bb2025::sneaky_git_behaviour::StepEjectPlayerHookState;
use crate::skill_behaviour::dispatch;
use crate::step::framework::{CatchScatterThrowInMode, Step, StepOutcome, StepId, StepParameter};
use crate::util::util_server_game::UtilServerGame;

/// Java: `StepEjectPlayer` (mixed/foul, BB2020 + BB2025).
pub struct StepEjectPlayer {
    /// Java: `state.gotoLabelOnEnd`
    goto_label_on_end: String,
    /// Java: `state.foulerHasBall`
    fouler_has_ball: Option<bool>,
    /// Java: `state.argueTheCallSuccessful`
    argue_the_call_successful: Option<bool>,
    /// Java: `state.officiousRef`
    officious_ref: bool,
}

impl StepEjectPlayer {
    pub fn new() -> Self {
        Self {
            goto_label_on_end: String::new(),
            fouler_has_ball: None,
            argue_the_call_successful: None,
            officious_ref: false,
        }
    }

    fn execute_step(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let mut hook_state = StepEjectPlayerHookState {
            argue_the_call_successful: self.argue_the_call_successful,
            officious_ref: self.officious_ref,
        };
        dispatch::execute_step_hooks(game, rng, StepId::EjectPlayer, &mut hook_state);

        if let Some(player_id) = game.acting_player.player_id.clone() {
            UtilBox::put_player_into_box(game, &player_id);
        }
        UtilBox::refresh_boxes(game);
        UtilServerGame::update_player_state_dependent_properties(game);

        if self.fouler_has_ball == Some(true) {
            // Java: setNextAction(StepAction.NEXT_STEP)
            StepOutcome::next()
                .publish(StepParameter::EndTurn(true))
                .publish(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ScatterBall))
        } else {
            // Java: setNextAction(StepAction.GOTO_LABEL, state.gotoLabelOnEnd)
            StepOutcome::goto(&self.goto_label_on_end)
                .publish(StepParameter::EndTurn(true))
        }
    }
}

impl Default for StepEjectPlayer {
    fn default() -> Self { Self::new() }
}

impl Step for StepEjectPlayer {
    fn id(&self) -> StepId { StepId::EjectPlayer }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(v)         => { self.goto_label_on_end = v.clone(); false }
            StepParameter::FoulerHasBall(v)           => { self.fouler_has_ball = Some(*v); true }
            StepParameter::ArgueTheCallSuccessful(v)  => { self.argue_the_call_successful = Some(*v); true }
            // Java: `init()` sets `state.officiousRef` and calls `consume(parameter)`.
            StepParameter::OfficiousRef(v)            => { self.officious_ref = *v; true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{Rules, PS_BANNED};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;
    use ffb_model::types::{FieldCoordinate, BAN_HOME_X};

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn make_game_with_fouler(player_id: &str, state_base: u32) -> Game {
        let mut team = test_team("home", 0);
        team.players.push(Player {
            id: player_id.into(),
            name: player_id.into(),
            nr: 99,
            ..Default::default()
        });
        let mut game = Game::new(team, test_team("away", 0), Rules::Bb2025);
        game.acting_player.player_id = Some(player_id.into());
        game.field_model.set_player_coordinate(player_id, FieldCoordinate::new(5, 5));
        game.field_model.set_player_state(player_id, ffb_model::enums::PlayerState::new(state_base));
        game
    }

    #[test]
    fn id_is_eject_player() {
        assert_eq!(StepEjectPlayer::new().id(), StepId::EjectPlayer);
    }

    #[test]
    fn always_publishes_end_turn() {
        let mut step = StepEjectPlayer::new();
        step.set_parameter(&StepParameter::GotoLabelOnEnd("end".into()));
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        let has_end_turn = outcome.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true)));
        assert!(has_end_turn);
    }

    #[test]
    fn without_ball_gotos_label() {
        let mut step = StepEjectPlayer::new();
        step.set_parameter(&StepParameter::GotoLabelOnEnd("end".into()));
        step.set_parameter(&StepParameter::FoulerHasBall(false));
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, StepAction::GotoLabel));
    }

    #[test]
    fn with_ball_next_step_and_scatter() {
        let mut step = StepEjectPlayer::new();
        step.set_parameter(&StepParameter::GotoLabelOnEnd("end".into()));
        step.set_parameter(&StepParameter::FoulerHasBall(true));
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, StepAction::NextStep));
        let has_scatter = outcome.published.iter().any(|p| {
            matches!(p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ScatterBall))
        });
        assert!(has_scatter);
    }

    #[test]
    fn fouler_has_ball_param_consumed() {
        let mut step = StepEjectPlayer::new();
        // set_parameter returns true → consumed
        let consumed = step.set_parameter(&StepParameter::FoulerHasBall(true));
        assert!(consumed);
    }

    #[test]
    fn goto_label_on_end_not_consumed() {
        let mut step = StepEjectPlayer::new();
        // Java: init() stores it but does NOT consume → return false
        let consumed = step.set_parameter(&StepParameter::GotoLabelOnEnd("end".into()));
        assert!(!consumed);
    }

    #[test]
    fn sneaky_git_hook_actually_bans_the_fouler() {
        // Java: executeStepHooks(this, state) is called unconditionally at the top of
        // executeStep() — its side effect (SneakyGitEjectPlayerModifier setting the
        // ejected player's state to BANNED and recording send-to-box bookkeeping) was
        // previously never invoked at all in the Rust translation (stale "no hooks
        // registered for this step in practice" comment). Without this wiring nothing in
        // this file ever actually changes the fouler's PlayerState to BANNED.
        use ffb_model::enums::PS_STANDING;
        let mut step = StepEjectPlayer::new();
        step.set_parameter(&StepParameter::GotoLabelOnEnd("end".into()));
        let mut game = make_game_with_fouler("fouler1", PS_STANDING);
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        let state = game.field_model.player_state("fouler1").expect("fouler state must exist");
        assert_eq!(state.base(), PS_BANNED);
        let pr = game.game_result.home.player_result("fouler1").expect("player result must exist");
        assert_eq!(pr.send_to_box_reason, Some(ffb_model::enums::SendToBoxReason::FoulBan));
    }

    /// Regression test for the missing `StepParameter::OfficiousRef` wiring. Java's
    /// `StepApplyKickoffResult` (bb2020) pushes this `EjectPlayer` step with
    /// `from(StepParameterKey.OFFICIOUS_REF, true)` when the Officious Ref kickoff event
    /// ejects a player. Before this fix, no `StepParameter::OfficiousRef` variant even
    /// existed, so `state.officious_ref` was always `false` and
    /// `SneakyGitEjectPlayerModifier` could never record `SendToBoxReason::OficiousRef` —
    /// every ejection was misreported as a foul ban, even ones caused by the ref event.
    #[test]
    fn officious_ref_param_sets_send_to_box_reason() {
        use ffb_model::enums::PS_STANDING;
        let mut step = StepEjectPlayer::new();
        step.set_parameter(&StepParameter::GotoLabelOnEnd("end".into()));
        step.set_parameter(&StepParameter::OfficiousRef(true));
        let mut game = make_game_with_fouler("fouler1", PS_STANDING);
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        let pr = game.game_result.home.player_result("fouler1").expect("player result must exist");
        assert_eq!(pr.send_to_box_reason, Some(ffb_model::enums::SendToBoxReason::OficiousRef));
    }

    #[test]
    fn officious_ref_param_is_consumed() {
        let mut step = StepEjectPlayer::new();
        let consumed = step.set_parameter(&StepParameter::OfficiousRef(true));
        assert!(consumed);
    }

    #[test]
    fn puts_fouler_into_box_on_execute() {
        let mut step = StepEjectPlayer::new();
        step.set_parameter(&StepParameter::GotoLabelOnEnd("end".into()));
        let mut game = make_game_with_fouler("fouler1", PS_BANNED);
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        let coord = game.field_model.player_coordinate("fouler1").expect("fouler should be boxed");
        assert_eq!(coord.x, BAN_HOME_X, "banned fouler should land in ban box");
    }
}
