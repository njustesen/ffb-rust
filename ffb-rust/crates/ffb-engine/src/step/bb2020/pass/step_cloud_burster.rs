/// 1:1 translation of the `StepCloudBurster` static nested class inside
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.CloudBursterBehaviour`.
///
/// Unlike most skill-hook audit items, `CloudBursterBehaviour` does not register a
/// `StepModifier` hook — it registers a whole standalone step
/// (`registerStep(StepId.CLOUD_BURSTER, StepCloudBurster.class)`), annotated
/// `@StepHook(HookPoint.PASS_INTERCEPT)`. Java's `bb2020.Pass` generator inserts it via
/// `sequence.insertHooks(StepHook.HookPoint.PASS_INTERCEPT, ...)` right after `INTERCEPT` and
/// before `RESOLVE_PASS`, forwarding `GOTO_LABEL_ON_FAILURE = RESOLVE_PASS`.
///
/// NOTE: mirroring the precedent set by `StepSafeThrow` (bb2016, the other
/// `PASS_INTERCEPT` hook step — see its generator file comment "insertHooks skipped —
/// StepHooks not yet ported"), this step exists as a fully-implemented, unit-tested step
/// file but is **not yet wired into `generator/bb2020/pass.rs`**. Wiring the hook-insertion
/// mechanism generically is a separate concern from porting the step body itself.
///
/// Flow (Java `executeStep`):
///  1. Guard: `!state.isDeflectionSuccessful()` or no thrower or no interceptor → goto failure.
///  2. `canForceInterceptionRerollSkill = thrower.getSkillWithProperty(canForceInterceptionRerollOfLongPasses)`.
///  3. `passingDistance = PassMechanic.findPassingDistance(game, throwerCoordinate, passCoordinate, false)`.
///  4. `useCloudBurster = skill != null && !cancelsSkill(interceptor, skill) && distance in {LONG_PASS, LONG_BOMB}`.
///  5. If true: report `ReportCloudBurster`, reset `deflectionSuccessful = false`, push a fresh
///     `INTERCEPT` step (with `GOTO_LABEL_ON_FAILURE` forwarded) → `NEXT_STEP`.
///  6. Else: goto failure.
///
/// Java quirk reproduced: `cancelsSkill(interceptor, skill)` is translated (matching the
/// `StepSafeThrow`/`VeryLongLegs` precedent) as `interceptor.has_skill_property(CANCELS_...)`
/// rather than resolving an actual `Skill` instance — the only skill in the game data that
/// registers `CancelSkillProperty(canForceInterceptionRerollOfLongPasses)` is BB2020
/// `VeryLongLegs`.
///
/// Java quirk reproduced (architectural): Java's `PassState` is a single mutable object shared
/// by reference across every step in the pass sequence, so re-pushing a fresh `INTERCEPT` step
/// after `state.setDeflectionSuccessful(false)` still sees the *same* `interceptorId`/
/// `interceptorChosen` already set from the first interception attempt (a genuine re-roll of the
/// same interceptor, no new dialog). Rust's per-step-instance fields (an explicitly accepted
/// idiom divergence per `step/framework.rs`'s header comment) mean the freshly constructed
/// `StepIntercept` only receives `GOTO_LABEL_ON_FAILURE` — exactly the one parameter Java's
/// `StepParameterSet` actually carries when re-pushing (`params.add(StepParameter.from(
/// GOTO_LABEL_ON_FAILURE, fGotoLabelOnFailure))`) — so the literal parameter-forwarding is a
/// faithful 1:1 port; only the *reuse of the previously-chosen interceptor* is not observable
/// without a shared `PassState`-equivalent.
///
/// Init parameter: `GOTO_LABEL_ON_FAILURE` (mandatory).
/// Consumes (from earlier steps in the pass sequence): `InterceptorId`, `DeflectionSuccessful`.
use ffb_mechanics::bb2020::pass_mechanic::PassMechanic as Bb2020PassMechanic;
use ffb_mechanics::pass_mechanic::PassMechanic as PassMechanicTrait;
use ffb_model::enums::PassingDistance;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::report::mixed::report_cloud_burster::ReportCloudBurster;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter, SequenceStep};

/// Java: `StepCloudBurster` (nested inside `CloudBursterBehaviour`, bb2020).
pub struct StepCloudBurster {
    /// Java: `fGotoLabelOnFailure` — mandatory init param.
    goto_label_on_failure: String,
    /// Java: `PassState.interceptorId` — set by the preceding `StepIntercept`.
    interceptor_id: Option<String>,
    /// Java: `PassState.deflectionSuccessful` — set by the preceding `StepIntercept`.
    deflection_successful: bool,
}

impl StepCloudBurster {
    pub fn new(goto_label_on_failure: String) -> Self {
        Self {
            goto_label_on_failure,
            interceptor_id: None,
            deflection_successful: false,
        }
    }

    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        let label = self.goto_label_on_failure.clone();

        // Java: if (!state.isDeflectionSuccessful() || (game.getThrower() == null) || (interceptor == null))
        let interceptor = self.interceptor_id.as_deref().and_then(|id| game.player(id));
        if !self.deflection_successful || game.thrower().is_none() || interceptor.is_none() {
            return StepOutcome::goto(&label);
        }

        // Java: Skill canForceInterceptionRerollSkill = game.getThrower().getSkillWithProperty(
        //   NamedProperties.canForceInterceptionRerollOfLongPasses)
        let can_force_interception_reroll_skill = game.thrower()
            .and_then(|t| t.skill_id_with_property(NamedProperties::CAN_FORCE_INTERCEPTION_REROLL_OF_LONG_PASSES));

        // Java: FieldCoordinate throwerCoordinate = game.getFieldModel().getPlayerCoordinate(game.getThrower());
        //       PassMechanic mechanic = ...; PassingDistance passingDistance = mechanic.findPassingDistance(
        //         game, throwerCoordinate, game.getPassCoordinate(), false);
        let thrower_coordinate = game.thrower_id.as_deref()
            .and_then(|id| game.field_model.player_coordinate(id));
        let mechanic = Bb2020PassMechanic::new();
        let passing_distance = mechanic.find_passing_distance(game, thrower_coordinate, game.pass_coordinate, false);

        // Java: !UtilCards.cancelsSkill(interceptor, canForceInterceptionRerollSkill)
        // (only BB2020 VeryLongLegs registers CancelSkillProperty(canForceInterceptionRerollOfLongPasses))
        let interceptor_cancels = interceptor
            .map(|p| p.has_skill_property(NamedProperties::CANCELS_CAN_FORCE_INTERCEPTION_REROLL_OF_LONG_PASSES))
            .unwrap_or(false);

        // Java: NamedProperties.canForceInterceptionRerollOfLongPasses.appliesToContext(...) —
        // the property's appliesToContext override checks distance is LONG_PASS or LONG_BOMB.
        let applies_to_distance = matches!(
            passing_distance,
            Some(PassingDistance::LongPass) | Some(PassingDistance::LongBomb)
        );

        let use_cloud_burster = can_force_interception_reroll_skill.is_some()
            && !interceptor_cancels
            && applies_to_distance;

        if use_cloud_burster {
            // Java: getResult().addReport(new ReportCloudBurster(game.getThrowerId(), state.getInterceptorId(),
            //   game.getThrower().getTeam().getId()))
            let thrower_id = game.thrower_id.clone();
            let thrower_team_id = thrower_id.as_deref()
                .and_then(|id| game.player_team_id(id))
                .map(str::to_string);
            game.report_list.add(ReportCloudBurster::new(
                thrower_id,
                self.interceptor_id.clone(),
                thrower_team_id,
            ));

            // Java: state.setDeflectionSuccessful(false);
            self.deflection_successful = false;

            // Java: StepParameterSet params = new StepParameterSet();
            //       params.add(StepParameter.from(GOTO_LABEL_ON_FAILURE, fGotoLabelOnFailure));
            //       IStep interceptStep = getGameState().getStepFactory().create(StepId.INTERCEPT, null, params);
            //       getGameState().getStepStack().push(interceptStep);
            //       getResult().setNextAction(StepAction.NEXT_STEP);
            StepOutcome::next().push_seq(vec![
                SequenceStep::with_params(StepId::Intercept, vec![StepParameter::GotoLabelOnFailure(label)]),
            ])
        } else {
            // Java: getResult().setNextAction(StepAction.GOTO_LABEL, fGotoLabelOnFailure);
            StepOutcome::goto(&label)
        }
    }
}

impl Default for StepCloudBurster {
    fn default() -> Self { Self::new(String::new()) }
}

impl Step for StepCloudBurster {
    fn id(&self) -> StepId { StepId::CloudBurster }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: public void start() { super.start(); executeStep(); }
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: handleCommand delegates to super, then executeStep() if EXECUTE_STEP.
        // This step has no client commands of its own (super's default handling never
        // returns EXECUTE_STEP without a matching command), so execute_step is not
        // re-invoked from handle_command in practice; kept for Step trait completeness.
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(v) => { self.goto_label_on_failure = v.clone(); true }
            StepParameter::InterceptorId(v) => { self.interceptor_id = v.clone(); true }
            StepParameter::DeflectionSuccessful(v) => { self.deflection_successful = *v; true }
            _ => false,
        }
    }
}

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
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
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
            is_big_guy: false,
            ..Default::default()
        };
        if team_home {
            game.team_home.players.push(p.clone());
        } else {
            game.team_away.players.push(p.clone());
        }
        p
    }

    /// Sets up a thrower with CloudBurster and an interceptor at LONG_PASS range.
    fn setup_long_pass(game: &mut Game, interceptor_skill: Option<SkillId>) {
        game.thrower_id = Some("thrower".into());
        add_player_with_skill(game, true, "thrower", SkillId::CloudBurster);
        if let Some(skill) = interceptor_skill {
            add_player_with_skill(game, false, "interceptor", skill);
        } else {
            add_player_with_skill(game, false, "interceptor", SkillId::Block);
        }
        game.field_model.set_player_coordinate("thrower", FieldCoordinate::new(1, 1));
        // LONG_PASS distance in the BB2020 throwing-range table (delta far enough to be long).
        game.pass_coordinate = Some(FieldCoordinate::new(10, 1));
    }

    #[test]
    fn id_is_cloud_burster() {
        assert_eq!(StepCloudBurster::new(String::new()).id(), StepId::CloudBurster);
    }

    #[test]
    fn deflection_not_successful_goes_to_failure() {
        let mut game = make_game();
        setup_long_pass(&mut game, None);
        let mut step = StepCloudBurster::new("fail".into());
        step.interceptor_id = Some("interceptor".into());
        step.deflection_successful = false;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    #[test]
    fn no_thrower_goes_to_failure() {
        let mut game = make_game();
        add_player_with_skill(&mut game, false, "interceptor", SkillId::Block);
        let mut step = StepCloudBurster::new("fail".into());
        step.interceptor_id = Some("interceptor".into());
        step.deflection_successful = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    #[test]
    fn no_interceptor_goes_to_failure() {
        let mut game = make_game();
        game.thrower_id = Some("thrower".into());
        add_player_with_skill(&mut game, true, "thrower", SkillId::CloudBurster);
        let mut step = StepCloudBurster::new("fail".into());
        step.interceptor_id = None;
        step.deflection_successful = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    #[test]
    fn has_skill_and_applies_reports_and_pushes_intercept() {
        let mut game = make_game();
        setup_long_pass(&mut game, None);
        let mut step = StepCloudBurster::new("fail".into());
        step.interceptor_id = Some("interceptor".into());
        step.deflection_successful = true;
        let out = step.start(&mut game, &mut GameRng::new(0));

        assert_eq!(out.action, StepAction::NextStep);
        assert!(game.report_list.has_report(ReportId::CLOUD_BURSTER),
            "expected ReportCloudBurster to be added");
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0].len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::Intercept);
        // deflection_successful reset to false (state.setDeflectionSuccessful(false))
        assert!(!step.deflection_successful);
    }

    #[test]
    fn has_skill_but_cancelled_goes_to_failure() {
        let mut game = make_game();
        setup_long_pass(&mut game, Some(SkillId::VeryLongLegs));
        let mut step = StepCloudBurster::new("fail".into());
        step.interceptor_id = Some("interceptor".into());
        step.deflection_successful = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
        assert!(!game.report_list.has_report(ReportId::CLOUD_BURSTER));
    }

    #[test]
    fn wrong_distance_does_not_apply_goes_to_failure() {
        let mut game = make_game();
        game.thrower_id = Some("thrower".into());
        add_player_with_skill(&mut game, true, "thrower", SkillId::CloudBurster);
        add_player_with_skill(&mut game, false, "interceptor", SkillId::Block);
        game.field_model.set_player_coordinate("thrower", FieldCoordinate::new(1, 1));
        // Adjacent square → QUICK_PASS distance, not LONG_PASS/LONG_BOMB.
        game.pass_coordinate = Some(FieldCoordinate::new(2, 1));
        let mut step = StepCloudBurster::new("fail".into());
        step.interceptor_id = Some("interceptor".into());
        step.deflection_successful = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
        assert!(!game.report_list.has_report(ReportId::CLOUD_BURSTER));
    }

    #[test]
    fn thrower_without_cloud_burster_skill_goes_to_failure() {
        let mut game = make_game();
        setup_long_pass(&mut game, None);
        // Override thrower with no CloudBurster skill.
        game.team_home.players.clear();
        add_player_with_skill(&mut game, true, "thrower", SkillId::Block);
        let mut step = StepCloudBurster::new("fail".into());
        step.interceptor_id = Some("interceptor".into());
        step.deflection_successful = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    #[test]
    fn set_parameter_goto_label_on_failure() {
        let mut step = StepCloudBurster::new("old".into());
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("new".into())));
        assert_eq!(step.goto_label_on_failure, "new");
    }

    #[test]
    fn set_parameter_interceptor_id() {
        let mut step = StepCloudBurster::new(String::new());
        assert!(step.set_parameter(&StepParameter::InterceptorId(Some("p1".into()))));
        assert_eq!(step.interceptor_id, Some("p1".into()));
    }

    #[test]
    fn set_parameter_deflection_successful() {
        let mut step = StepCloudBurster::new(String::new());
        assert!(step.set_parameter(&StepParameter::DeflectionSuccessful(true)));
        assert!(step.deflection_successful);
    }

    #[test]
    fn set_parameter_unrelated_returns_false() {
        let mut step = StepCloudBurster::new(String::new());
        assert!(!step.set_parameter(&StepParameter::CardId(None)));
    }

    #[test]
    fn handle_command_executes_step() {
        let mut game = make_game();
        setup_long_pass(&mut game, None);
        let mut step = StepCloudBurster::new("fail".into());
        step.interceptor_id = Some("interceptor".into());
        step.deflection_successful = false;
        let out = step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn default_impl_has_empty_label() {
        let step = StepCloudBurster::default();
        assert_eq!(step.goto_label_on_failure, "");
    }
}
