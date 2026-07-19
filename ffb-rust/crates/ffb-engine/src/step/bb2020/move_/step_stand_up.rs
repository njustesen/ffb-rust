use ffb_model::enums::{PS_PRONE, PlayerAction, PlayerState};
use ffb_model::enums::ReRollSource;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use ffb_model::model::target_selection_state::TargetSelectionState;
use crate::step::abstract_step_with_re_roll::ReRollState;
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

/// Minimum MA to stand up for free. Java: Constant.MINIMUM_MOVE_TO_STAND_UP = 3.
const MINIMUM_MOVE_TO_STAND_UP: i32 = 3;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.move.StepStandUp.
///
/// BB2020 logic is identical to BB2025.
///
/// Stand-up sequence for a prone player.
/// If MA ≥ 3 or canStandUpForFree: free stand-up, NEXT_STEP.
/// Otherwise: roll d6 ≥ 4.  Success → NEXT_STEP; Failure → END_PLAYER_ACTION + GOTO failure label.
///
/// Init params: GOTO_LABEL_ON_FAILURE (mandatory).
///
/// allowStandUpAssists → findStandUpAssists wired.
/// TargetSelectionState.failed() wired in Blitz/BlitzMove/KickEmBlitz branch of handleFailedStandUp.
pub struct StepStandUp {
    /// Java: fGotoLabelOnFailure
    pub goto_label_on_failure: String,
    /// Java: AbstractStepWithReRoll fields
    pub re_roll_state: ReRollState,
    /// Persisted roll for re-roll path
    roll: i32,
}

impl StepStandUp {
    pub fn new(goto_label_on_failure: String) -> Self {
        Self { goto_label_on_failure, re_roll_state: ReRollState::new(), roll: 0 }
    }
}

impl Step for StepStandUp {
    fn id(&self) -> StepId { StepId::StandUp }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if let Action::UseReRoll { use_reroll: false } = action {
            self.re_roll_state.re_roll_source = None;
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(v) => { self.goto_label_on_failure = v.clone(); true }
            _ => false,
        }
    }
}

impl StepStandUp {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        game.turn_data_mut().turn_started = true;

        let already_rerolled = self.re_roll_state.re_rolled_action
            .as_ref().map(|a| a.name == "STAND_UP").unwrap_or(false);

        if !already_rerolled {
            if !game.acting_player.standing_up {
                return StepOutcome::next();
            }
            if game.acting_player.has_moved {
                return StepOutcome::next();
            }
        }

        game.concession_possible = false;

        let roll_stand_up_needed = game.acting_player.player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| {
                p.movement_with_modifiers() < MINIMUM_MOVE_TO_STAND_UP
                    && !p.has_skill_property(NamedProperties::CAN_STAND_UP_FOR_FREE)
            })
            .unwrap_or(true);

        if !roll_stand_up_needed {
            game.acting_player.has_moved = true;
            game.acting_player.standing_up = false;
            return StepOutcome::next();
        }

        if already_rerolled {
            let pid = game.acting_player.player_id.as_deref().unwrap_or("").to_owned();
            let source_opt = self.re_roll_state.re_roll_source.clone();
            let consumed = source_opt
                .as_ref()
                .map(|s| use_reroll(game, s, &pid))
                .unwrap_or(false);
            if !consumed {
                return self.fail_stand_up(game);
            }
        }

        if self.roll == 0 {
            self.roll = rng.d6();
        }

        let modifier = game.acting_player.player_id.as_deref()
            .and_then(|id| game.player(id))
            .filter(|p| p.has_skill_property(NamedProperties::ALLOW_STAND_UP_ASSISTS))
            .map(|_| {
                let id = game.acting_player.player_id.as_deref().unwrap_or("");
                UtilPlayer::find_stand_up_assists(game, id)
            })
            .unwrap_or(0);
        let successful = DiceInterpreter::is_stand_up_successful(self.roll, modifier);

        // Java line 110-112: boolean reRolled = ...; addReport(new ReportStandUpRoll(...))
        {
            use ffb_model::report::report_stand_up_roll::ReportStandUpRoll;
            let re_rolled = self.re_roll_state.re_rolled_action.as_ref()
                .map(|a| a.name == "STAND_UP").unwrap_or(false)
                && self.re_roll_state.re_roll_source.is_some();
            game.report_list.add(ReportStandUpRoll::new(
                game.acting_player.player_id.clone(),
                successful,
                self.roll,
                modifier,
                re_rolled,
            ));
        }

        if successful {
            game.acting_player.has_moved = true;
            game.acting_player.standing_up = false;
            // Java (bb2020): if (playerState.isRooted()) → GOTO_LABEL(fGotoLabelOnFailure)
            // else NEXT_STEP. (bb2025 sibling uses isPinned() instead — BB2020 checks
            // rooted, not pinned; this branch was previously missing entirely here.)
            let is_rooted = game.acting_player.player_id.as_deref()
                .and_then(|id| game.field_model.player_state(id))
                .map(|s| s.is_rooted())
                .unwrap_or(false);
            if is_rooted {
                let label = self.goto_label_on_failure.clone();
                StepOutcome::goto(&label)
            } else {
                StepOutcome::next()
            }
        } else {
            if already_rerolled {
                return self.fail_stand_up(game);
            }
            let minimum_roll = i32::max(2, 4 - modifier);
            if let Some(prompt) = ask_for_reroll_if_available(game, "STAND_UP", minimum_roll, false) {
                use ffb_model::model::re_rolled_action::ReRolledAction;
                self.re_roll_state.re_rolled_action = Some(ReRolledAction::new("STAND_UP"));
                self.re_roll_state.re_roll_source = Some(ReRollSource::new("TRR"));
                self.roll = 0;
                return StepOutcome::cont().with_prompt(prompt);
            }
            self.fail_stand_up(game)
        }
    }

    fn fail_stand_up(&self, game: &mut Game) -> StepOutcome {
        if let Some(pid) = game.acting_player.player_id.clone() {
            game.field_model.set_player_state(&pid, PlayerState::new(PS_PRONE));
        }
        self.handle_failed_stand_up(game);
        let label = self.goto_label_on_failure.clone();
        StepOutcome::goto(&label)
            .publish(StepParameter::EndPlayerAction(true))
    }

    /// Java: handleFailedStandUp(Game, ActingPlayer) — marks the per-turn action flags
    /// for the relevant action type so the client knows it was consumed.
    fn handle_failed_stand_up(&self, game: &mut Game) {
        let player_action = game.acting_player.player_action;
        match player_action {
            Some(PlayerAction::Blitz)
            | Some(PlayerAction::BlitzMove)
            | Some(PlayerAction::KickEmBlitz) => {
                game.turn_data_mut().blitz_used = true;
                // Java: if (getFieldModel().getTargetSelectionState() != null) → .failed()
                if let Some(ref mut ts) = game.field_model.target_selection_state {
                    ts.failed();
                }
            }
            Some(PlayerAction::KickTeamMate)
            | Some(PlayerAction::KickTeamMateMove) => {
                game.turn_data_mut().ktm_used = true;
            }
            Some(PlayerAction::Pass)
            | Some(PlayerAction::PassMove)
            | Some(PlayerAction::ThrowTeamMate)
            | Some(PlayerAction::ThrowTeamMateMove) => {
                game.turn_data_mut().pass_used = true;
            }
            Some(PlayerAction::HandOver)
            | Some(PlayerAction::HandOverMove) => {
                game.turn_data_mut().hand_over_used = true;
            }
            Some(PlayerAction::Foul)
            | Some(PlayerAction::FoulMove) => {
                let pid = game.acting_player.player_id.clone();
                let allows_extra_foul = pid.as_deref()
                    .and_then(|id| game.player(id))
                    .map(|p| p.has_skill_property(NamedProperties::ALLOWS_ADDITIONAL_FOUL))
                    .unwrap_or(false);
                if !allows_extra_foul {
                    game.turn_data_mut().foul_used = true;
                }
            }
            Some(PlayerAction::SecureTheBall) => {
                game.turn_data_mut().secure_the_ball_used = true;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{Rules, TurnMode};
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn not_standing_up_returns_next_step_immediately() {
        let mut game = make_game();
        game.acting_player.standing_up = false;
        let mut step = StepStandUp::new("fail".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.is_empty());
    }

    #[test]
    fn already_moved_returns_next_step_immediately() {
        let mut game = make_game();
        game.acting_player.standing_up = true;
        game.acting_player.has_moved = true;
        let mut step = StepStandUp::new("fail".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn failure_goes_to_failure_label_with_end_player_action() {
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        game.acting_player.standing_up = true;
        let mut step = StepStandUp::new("fail".into());
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))));
    }

    #[test]
    fn failure_with_trr_offers_reroll_prompt() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        game.acting_player.standing_up = true;
        let mut step = StepStandUp::new("fail".into());
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
        assert!(out.prompt.is_some());
    }

    #[test]
    fn decline_reroll_goes_to_failure_label() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        game.acting_player.standing_up = true;
        let mut step = StepStandUp::new("fail".into());
        step.roll = 1;
        let _offer = step.start(&mut game, &mut GameRng::new(0));
        let out = step.handle_command(&Action::UseReRoll { use_reroll: false }, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn set_parameter_goto_label_on_failure_accepted() {
        let mut step = StepStandUp::new("old".into());
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("new".into())));
        assert_eq!(step.goto_label_on_failure, "new");
    }

    #[test]
    fn roll_emits_stand_up_roll_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        game.acting_player.standing_up = true;
        let mut step = StepStandUp::new("fail".into());
        step.roll = 4;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::STAND_UP_ROLL));
    }

    #[test]
    fn successful_stand_up_while_rooted_goes_to_failure_label() {
        // Java StepStandUp (bb2020) executeStep(): on a *successful* stand-up roll,
        // if playerState.isRooted() the step still routes to GOTO_LABEL(fGotoLabelOnFailure)
        // instead of NEXT_STEP (a rooted player cannot actually act after standing up).
        // This branch was entirely missing from the Rust translation (which unconditionally
        // returned NextStep on success, and instead had an unrelated, non-Java `is_pinned`
        // early-return that doesn't exist anywhere in the Java source).
        use ffb_model::types::FieldCoordinate;
        let mut game = make_game();
        game.acting_player.standing_up = true;
        game.acting_player.player_id = Some("p1".into());
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        let rooted_state = ffb_model::enums::PlayerState::new(ffb_model::enums::PS_STANDING).change_rooted(true);
        game.field_model.set_player_state("p1", rooted_state);
        let mut step = StepStandUp::new("fail".into());
        step.roll = 6; // guaranteed success
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
        // has_moved/standing_up are still updated as on any success
        assert!(game.acting_player.has_moved);
        assert!(!game.acting_player.standing_up);
    }

    #[test]
    fn successful_stand_up_while_not_rooted_returns_next_step() {
        let mut game = make_game();
        game.acting_player.standing_up = true;
        let mut step = StepStandUp::new("fail".into());
        step.roll = 6;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn failed_roll_still_emits_stand_up_roll_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        game.acting_player.standing_up = true;
        let mut step = StepStandUp::new("fail".into());
        step.roll = 1;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::STAND_UP_ROLL));
    }
}
