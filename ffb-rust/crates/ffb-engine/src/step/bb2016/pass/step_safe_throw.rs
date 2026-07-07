/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.pass.StepSafeThrow`.
///
/// Step in the pass sequence to handle skill SAFE_THROW (BB2016).
/// - If thrower lacks canCancelInterceptions property, skip.
/// - If interceptor can cancel the skill (VeryLongLegs), skip.
/// - Rolls (AgilityMechanic.minimumRollSafeThrow): on success, nullify interceptor → NEXT_STEP.
/// - On failure: set ball/bomb to interceptor coordinate → goto failure.
/// - Re-roll (SAFE_THROW) supported.
///
/// Init parameter: GOTO_LABEL_ON_FAILURE (mandatory).
/// Receives: INTERCEPTOR_ID.
/// Publishes: INTERCEPTOR_ID (null on success).
///
/// SafeThrow skill → canCancelInterceptions property wired.
/// VeryLongLegs interceptor cancel → cancelsCancelInterceptions property wired.
/// AgilityMechanic.minimumRollSafeThrow → wired (bb2016::AgilityMechanic).
/// SafeThrowRoll GameEvent wired.
use ffb_mechanics::bb2016::agility_mechanic::AgilityMechanic as Bb2016AgilityMechanic;
use ffb_mechanics::agility_mechanic::AgilityMechanic as AgilityMechanicTrait;
use ffb_model::enums::ReRollSource;
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_safe_throw_roll::ReportSafeThrowRoll;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

const SAFE_THROW_ACTION: &str = "SAFE_THROW";

/// Java: `StepSafeThrow` (bb2016/pass).
pub struct StepSafeThrow {
    /// Java: `fGotoLabelOnFailure` — mandatory init param.
    goto_label_on_failure: String,
    /// Java: `fInterceptorId`
    interceptor_id: Option<String>,
    /// Java: AbstractStepWithReRoll — re-roll source to consume on next execute.
    re_roll_source: Option<ReRollSource>,
    /// Java: AbstractStepWithReRoll — tracks if we've already rerolled this action.
    re_rolled_action: Option<String>,
}

impl StepSafeThrow {
    pub fn new() -> Self {
        Self {
            goto_label_on_failure: String::new(),
            interceptor_id: None,
            re_roll_source: None,
            re_rolled_action: None,
        }
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // No interceptor → skip to next (nothing to cancel).
        let interceptor_id = match &self.interceptor_id {
            Some(id) if !id.is_empty() => id.clone(),
            _ => return StepOutcome::next(),
        };

        // Java: Skill canForceInterceptionRerollSkill = thrower.getSkillWithProperty(canCancelInterceptions)
        let thrower_has_safe_throw = game.thrower()
            .map(|p| p.has_skill_property(NamedProperties::CAN_CANCEL_INTERCEPTIONS))
            .unwrap_or(false);
        if !thrower_has_safe_throw {
            return StepOutcome::next();
        }

        // VeryLongLegs registers CancelSkillProperty(canCancelInterceptions) →
        // interceptor with VeryLongLegs cancels SafeThrow.
        let interceptor_cancels = game.player(&interceptor_id)
            .map(|p| p.has_skill_property(NamedProperties::CANCELS_CAN_CANCEL_INTERCEPTIONS))
            .unwrap_or(false);
        if interceptor_cancels {
            return StepOutcome::next();
        }

        // Java: if (ReRolledActions.SAFE_THROW == getReRolledAction()) {
        //   if ((getReRollSource() == null) || !useReRoll(this, source, thrower)) doSafeThrow = false }
        let mut do_safe_throw = true;
        let already_rerolled = self.re_rolled_action.as_deref() == Some(SAFE_THROW_ACTION);
        if already_rerolled {
            let thrower_id = game.thrower_id.clone().unwrap_or_default();
            if let Some(ref source) = self.re_roll_source.clone() {
                if !use_reroll(game, source, &thrower_id) {
                    do_safe_throw = false;
                }
            } else {
                do_safe_throw = false;
            }
        }

        if !do_safe_throw {
            return self.fail_safe_throw(game, &interceptor_id);
        }

        // Roll d6 and compare to minimum.
        let roll = rng.d6();
        let minimum_roll = game.thrower()
            .map(|p| Bb2016AgilityMechanic::default().minimum_roll_safe_throw(p))
            .unwrap_or(2);
        let successful = DiceInterpreter::is_skill_roll_successful(roll, minimum_roll);
        let player_id = game.acting_player.player_id.clone().unwrap_or_default();
        let safe_throw_event = GameEvent::SafeThrowRoll { player_id: player_id.clone(), roll, success: successful };

        // Java: getResult().addReport(new ReportSafeThrowRoll(game.getThrowerId(), successful, roll, minimumRoll, reRolled))
        let re_rolled = already_rerolled;
        game.report_list.add(ReportSafeThrowRoll::new(
            game.thrower_id.clone(),
            successful,
            roll,
            minimum_roll,
            re_rolled,
            vec![],
        ));

        // Java: if (!safeThrowSuccessful && getReRolledAction() != SAFE_THROW
        //         && askForReRollIfAvailable(...)) doNextStep = false
        if !successful && !already_rerolled {
            if let Some(prompt) = ask_for_reroll_if_available(game, SAFE_THROW_ACTION, minimum_roll, false) {
                self.re_rolled_action = Some(SAFE_THROW_ACTION.to_string());
                self.re_roll_source = Some(ReRollSource::new("TRR"));
                return StepOutcome::cont().with_event(safe_throw_event).with_prompt(prompt);
            }
        }

        if successful {
            return StepOutcome::next()
                .with_event(safe_throw_event)
                .publish(StepParameter::InterceptorId(None));
        }

        self.fail_safe_throw(game, &interceptor_id).with_event(safe_throw_event)
    }

    fn fail_safe_throw(&self, game: &mut Game, interceptor_id: &str) -> StepOutcome {
        // Java: fieldModel.setBallCoordinate(interceptorCoordinate) / setBombCoordinate if THROW_BOMB
        if let Some(coord) = game.field_model.player_coordinate(interceptor_id) {
            game.field_model.ball_coordinate = Some(coord);
        }
        StepOutcome::goto(&self.goto_label_on_failure)
    }
}

impl Default for StepSafeThrow {
    fn default() -> Self { Self::new() }
}

impl Step for StepSafeThrow {
    fn id(&self) -> StepId { StepId::SafeThrow }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: AbstractStepWithReRoll.handleCommand → if CLIENT_USE_REROLL + declined: clear source
        if let Action::UseReRoll { use_reroll: false } = action {
            self.re_roll_source = None;
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(s) => { self.goto_label_on_failure = s.clone(); true }
            StepParameter::InterceptorId(v)      => { self.interceptor_id = v.clone(); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{PlayerGender, PlayerType, Rules, SkillId};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::report::report_id::ReportId;
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    fn add_player_with_skill(game: &mut Game, team_home: bool, id: &str, skill: SkillId) -> Player {
        let p = Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 4, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![SkillWithValue { skill_id: skill, value: None }],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        };
        if team_home {
            game.team_home.players.push(p.clone());
        } else {
            game.team_away.players.push(p.clone());
        }
        p
    }

    #[test]
    fn id_is_safe_throw() {
        assert_eq!(StepSafeThrow::new().id(), StepId::SafeThrow);
    }

    #[test]
    fn no_interceptor_returns_next() {
        let mut game = make_game();
        let mut step = StepSafeThrow::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn interceptor_but_thrower_lacks_safe_throw_skill_returns_next() {
        let mut game = make_game();
        game.thrower_id = Some("thrower".into());
        add_player_with_skill(&mut game, true, "thrower", SkillId::Block); // no SafeThrow
        let mut step = StepSafeThrow::new();
        step.interceptor_id = Some("interceptor".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn interceptor_with_very_long_legs_cancels_safe_throw() {
        let mut game = make_game();
        game.thrower_id = Some("thrower".into());
        add_player_with_skill(&mut game, true, "thrower", SkillId::SafeThrow);
        add_player_with_skill(&mut game, false, "interceptor", SkillId::VeryLongLegs);
        let mut step = StepSafeThrow::new();
        step.interceptor_id = Some("interceptor".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        // VeryLongLegs cancels SafeThrow → next step without roll
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_goto_label_on_failure() {
        let mut step = StepSafeThrow::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("fail".into())));
        assert_eq!(step.goto_label_on_failure, "fail");
    }

    #[test]
    fn set_parameter_interceptor_id() {
        let mut step = StepSafeThrow::new();
        assert!(step.set_parameter(&StepParameter::InterceptorId(Some("p3".into()))));
        assert_eq!(step.interceptor_id, Some("p3".into()));
    }

    #[test]
    fn set_parameter_interceptor_id_none() {
        let mut step = StepSafeThrow::new();
        step.interceptor_id = Some("p3".into());
        assert!(step.set_parameter(&StepParameter::InterceptorId(None)));
        assert!(step.interceptor_id.is_none());
    }

    #[test]
    fn safe_throw_failure_sets_ball_coordinate() {
        let mut game = make_game();
        game.thrower_id = Some("thrower".into());
        add_player_with_skill(&mut game, true, "thrower", SkillId::SafeThrow);
        add_player_with_skill(&mut game, false, "interceptor", SkillId::Block);
        let interceptor_coord = FieldCoordinate::new(5, 5);
        game.field_model.set_player_coordinate("interceptor", interceptor_coord);
        let mut step = StepSafeThrow::new();
        step.goto_label_on_failure = "fail".into();
        step.interceptor_id = Some("interceptor".into());
        // Simulate already_rerolled so no reroll prompt is issued → pure fail/success
        step.re_rolled_action = Some(SAFE_THROW_ACTION.to_string());
        step.re_roll_source = None; // declined → do_safe_throw = false → fail immediately
        let out = step.start(&mut game, &mut GameRng::new(1));
        // No reroll source → do_safe_throw=false → fail path → ball moved to interceptor
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(game.field_model.ball_coordinate, Some(interceptor_coord));
    }

    #[test]
    fn safe_throw_roll_report_added_on_roll() {
        let mut game = make_game();
        game.thrower_id = Some("thrower".into());
        add_player_with_skill(&mut game, true, "thrower", SkillId::SafeThrow);
        add_player_with_skill(&mut game, false, "interceptor", SkillId::Block);
        game.field_model.set_player_coordinate("interceptor", FieldCoordinate::new(5, 5));
        let mut step = StepSafeThrow::new();
        step.goto_label_on_failure = "fail".into();
        step.interceptor_id = Some("interceptor".into());
        // Let step execute normally (no pre-set reroll state)
        step.start(&mut game, &mut GameRng::new(3));
        assert!(game.report_list.has_report(ReportId::SAFE_THROW_ROLL),
            "should have SAFE_THROW_ROLL report after rolling");
    }

    #[test]
    fn safe_throw_roll_report_added_on_forced_fail_path() {
        let mut game = make_game();
        game.thrower_id = Some("thrower".into());
        add_player_with_skill(&mut game, true, "thrower", SkillId::SafeThrow);
        add_player_with_skill(&mut game, false, "interceptor", SkillId::Block);
        game.field_model.set_player_coordinate("interceptor", FieldCoordinate::new(5, 5));
        let mut step = StepSafeThrow::new();
        step.goto_label_on_failure = "fail".into();
        step.interceptor_id = Some("interceptor".into());
        // already_rerolled = false → rolls, report should be added before re-roll prompt or fail
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::SAFE_THROW_ROLL),
            "SAFE_THROW_ROLL must be added after rolling");
    }

    /// Declining a reroll clears the reroll source so the next execute goes to fail path.
    #[test]
    fn decline_reroll_clears_source() {
        use ffb_model::enums::ReRollSource;
        let mut game = make_game();
        game.thrower_id = Some("thrower".into());
        add_player_with_skill(&mut game, true, "thrower", SkillId::SafeThrow);
        add_player_with_skill(&mut game, false, "interceptor", SkillId::Block);
        game.field_model.set_player_coordinate("interceptor", FieldCoordinate::new(5, 5));
        let mut step = StepSafeThrow::new();
        step.goto_label_on_failure = "fail".into();
        step.interceptor_id = Some("interceptor".into());
        step.re_roll_source = Some(ReRollSource::new("TRR"));
        // Decline reroll → clears source
        step.handle_command(&Action::UseReRoll { use_reroll: false }, &mut game, &mut GameRng::new(0));
        assert!(step.re_roll_source.is_none());
    }
}
