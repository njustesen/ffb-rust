use ffb_model::enums::{ApothecaryMode, PlayerState, PS_FALLING, PS_MOVING, PS_HIT_ON_GROUND, ReRollSource};
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use ffb_mechanics::mechanics::is_skill_roll_successful;
use crate::action::Action;
use crate::drop_player_context::SteadyFootingContext;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

/// Minimum d6 roll required for Steady Footing (Java: MINMUM_ROLL = 6).
const MINIMUM_ROLL: i32 = 6;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.shared.StepSteadyFooting.
///
/// Resolves the Steady Footing skill (canAvoidFallingDown property). Cancels a fall-down
/// result on a roll of 6+ (natural-6 rule from DiceInterpreter.isSkillRollSuccessful).
///
/// Init parameters: GOTO_LABEL_ON_FAILURE, GOTO_LABEL_ON_SUCCESS, APOTHECARY_MODE.
/// Runtime parameters: STEADY_FOOTING_CONTEXT, OLD_DEFENDER_STATE, ATTACKER_ALREADY_DOWN,
///   PUSHED_ON_BALL, BALL_KNOCKED_LOOSE.
///
/// Java: useSkill (Boolean tristate null/true/false):
///   null  → show dialog, wait (CONTINUE)
///   false → skip skill, fail()
///   true  → roll
pub struct StepSteadyFooting {
    /// Java: useSkill (Boolean tristate). None = not yet answered.
    pub use_skill: Option<bool>,
    /// Java: goToLabelOnFailure (init param)
    pub goto_label_on_failure: String,
    /// Java: goToLabelOnSuccess (init param)
    pub goto_label_on_success: String,
    /// Java: apothecaryMode (init param)
    pub apothecary_mode: Option<ApothecaryMode>,
    /// Java: context (SteadyFootingContext)
    pub context: Option<Box<SteadyFootingContext>>,
    /// Java: oldDefenderState
    pub old_defender_state: Option<PlayerState>,
    /// Java: skip — set when ATTACKER_ALREADY_DOWN is published
    pub skip: bool,
    /// Java: playerId — resolved from context
    pub player_id: Option<String>,
    /// Java: removeCatchMode (default true; flipped by PUSHED_ON_BALL / BALL_KNOCKED_LOOSE)
    pub remove_catch_mode: bool,
    // AbstractStepWithReRoll stubs
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
}

impl StepSteadyFooting {
    pub fn new(goto_label_on_failure: String, goto_label_on_success: String) -> Self {
        Self {
            use_skill: None,
            goto_label_on_failure,
            goto_label_on_success,
            apothecary_mode: None,
            context: None,
            old_defender_state: None,
            skip: false,
            player_id: None,
            remove_catch_mode: true,
            re_rolled_action: None,
            re_roll_source: None,
        }
    }
}

impl Default for StepSteadyFooting {
    fn default() -> Self { Self::new(String::new(), String::new()) }
}

impl Step for StepSteadyFooting {
    fn id(&self) -> StepId { StepId::SteadyFooting }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::UseSkill { use_skill, .. } => {
                self.use_skill = Some(*use_skill);
            }
            Action::UseReRoll { use_reroll: false } => {
                self.re_roll_source = None;
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::SteadyFootingContext(ctx) => {
                // Java: only accept if apothecaryMode == null || apothecaryMode == ctx.getApothecaryMode()
                let ctx_apo = ctx.get_apothecary_mode();
                if self.apothecary_mode.is_none() || self.apothecary_mode == Some(ctx_apo) {
                    // Java: derive playerId from whichever sub-field is set
                    self.player_id = ctx.get_player_id().map(str::to_owned);
                    self.context = Some(ctx.clone());
                    return true;
                }
                false
            }
            StepParameter::OldDefenderState(v) => {
                if self.apothecary_mode == Some(ApothecaryMode::Defender) {
                    self.old_defender_state = Some(*v);
                    return true;
                }
                false
            }
            StepParameter::AttackerAlreadyDown(v) => {
                // Java: ATTACKER_ALREADY_DOWN only consumed when apothecaryMode == ATTACKER
                if self.apothecary_mode == Some(ApothecaryMode::Attacker) {
                    self.skip = *v;
                    return true;
                }
                false
            }
            StepParameter::PushedOnBall(v) => {
                self.remove_catch_mode = !v;
                true
            }
            StepParameter::BallKnockedLoose(v) => {
                self.remove_catch_mode = !v;
                true
            }
            StepParameter::PlayerId(v) => {
                self.player_id = Some(v.clone());
                true
            }
            StepParameter::ApothecaryMode(v) => {
                self.apothecary_mode = Some(*v);
                true
            }
            StepParameter::GotoLabelOnFailure(v) => {
                self.goto_label_on_failure = v.clone();
                true
            }
            StepParameter::GotoLabelOnSuccess(v) => {
                self.goto_label_on_success = v.clone();
                true
            }
            _ => false,
        }
    }
}

impl StepSteadyFooting {
    /// Java: executeStep()
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: if (context == null) → NEXT_STEP
        if self.context.is_none() {
            return StepOutcome::next();
        }

        let player_id = match &self.player_id {
            Some(id) => id.clone(),
            None => return self.fail(game),
        };

        // Java: Player player = game.getPlayerById(playerId)
        // Java: Optional<Skill> skill = UtilCards.getSkillWithProperty(player, canAvoidFallingDown)
        // Java: UtilCards.getSkillWithProperty(player, NamedProperties.canAvoidFallingDown)
        let has_steady_footing = game.player(&player_id)
            .map(|p| p.has_skill_property(NamedProperties::CAN_AVOID_FALLING_DOWN))
            .unwrap_or(false);

        // Java: PlayerState playerState = game.getFieldModel().getPlayerState(player)
        let player_state = game.field_model.player_state(&player_id)
            .unwrap_or_else(|| PlayerState::new(PS_HIT_ON_GROUND));

        // Java: if (skip || !skill.isPresent() || playerState.isHypnotized() ||
        //           playerState.isConfused() || playerState.getBase() == HIT_ON_GROUND)
        if self.skip
            || !has_steady_footing
            || player_state.is_hypnotized()
            || player_state.is_confused()
            || player_state.base() == PS_HIT_ON_GROUND
        {
            return self.fail(game);
        }

        // Java: if (useSkill == null) → show dialog, CONTINUE
        if self.use_skill.is_none() {
            // client-only: publish dialog prompt for canAvoidFallingDown skill use
            // For now: auto-accept (use_skill = true) so the random agent rolls.
            self.use_skill = Some(true);
        }

        let re_rolled = self.re_rolled_action.as_deref() == Some("STEADY_FOOTING")
            && self.re_roll_source.is_some();

        // Java: if (!reRolled) addReport(ReportSkillUse(playerId, SteadyFooting, useSkill, AVOID_FALLING))
        if !re_rolled {
            use ffb_model::enums::SkillId;
            use ffb_model::model::skill_use::SkillUse;
            use ffb_model::report::report_skill_use::ReportSkillUse;
            game.report_list.add(ReportSkillUse::new(
                Some(player_id.clone()), SkillId::SteadyFooting,
                self.use_skill.unwrap_or(false), SkillUse::AVOID_FALLING,
            ));
        }

        // Java: if (!useSkill) → fail()
        if !self.use_skill.unwrap_or(false) {
            return self.fail(game);
        }

        // Java: if (STEADY_FOOTING == reRolledAction) { if (source == null || !useReRoll) → fail() }
        if self.re_rolled_action.as_deref() == Some("STEADY_FOOTING") {
            if let Some(ref source_name) = self.re_roll_source.clone() {
                let source = ReRollSource::new(source_name.as_str());
                if !use_reroll(game, &source, &player_id) {
                    return self.fail(game);
                }
            } else {
                return self.fail(game); // player declined
            }
        }

        let roll = rng.d6();
        let successful = is_skill_roll_successful(roll, MINIMUM_ROLL);

        if successful {
            return self.succeed(game, &player_id, player_state);
        }

        // Java: if (!reRolled && askForReRollIfAvailable(...)) { setReRolledAction; CONTINUE }
        if !re_rolled {
            if let Some(prompt) = ask_for_reroll_if_available(game, "STEADY_FOOTING", MINIMUM_ROLL, false) {
                self.re_rolled_action = Some("STEADY_FOOTING".into());
                self.re_roll_source = Some("TRR".into());
                return StepOutcome::cont().with_prompt(prompt);
            }
        }

        self.fail(game)
    }

    /// Java: success path — undoes fall, adjusts player state, routes to success label.
    fn succeed(&self, game: &mut Game, player_id: &str, player_state: PlayerState) -> StepOutcome {
        let mut out = StepOutcome::next();

        // Java: if (game.getActingTeam().hasPlayer(player))
        //         publishParameter(END_TURN, false)
        //         publishParameter(END_PLAYER_ACTION, false)
        if game.is_active_team_player(player_id) {
            out = out
                .publish(StepParameter::EndTurn(false))
                .publish(StepParameter::EndPlayerAction(false));
        }

        // Java: if (removeCatchMode && UtilPlayer.hasBall(game, player))
        //         publishParameter(CATCH_SCATTER_THROW_IN_MODE, null)
        // Requires StepParameter::CatchScatterThrowInMode(Option<...>) — type change deferred.
        let _ = UtilPlayer::has_ball(game, player_id); // suppress unused warning; wiring deferred

        // Java: if (oldDefenderState != null) setPlayerState(player, oldDefenderState)
        let new_state = if let Some(ods) = self.old_defender_state {
            game.field_model.set_player_state(player_id, ods);
            ods
        } else {
            player_state
        };

        // Java: if (apothecaryMode == ATTACKER && playerState.getBase() == FALLING)
        //         setPlayerState(player, state.changeBase(MOVING))
        if self.apothecary_mode == Some(ApothecaryMode::Attacker)
            && new_state.base() == PS_FALLING
        {
            let fixed = new_state.change_base(PS_MOVING);
            game.field_model.set_player_state(player_id, fixed);
        }

        // Java: if (StringTool.isProvided(goToLabelOnSuccess)) GOTO_LABEL else NEXT_STEP
        if !self.goto_label_on_success.is_empty() {
            StepOutcome::goto(&self.goto_label_on_success)
                .publish(StepParameter::EndTurn(false))
                .publish(StepParameter::EndPlayerAction(false))
        } else {
            out
        }
    }

    /// Java: fail() — routes to failure label, republishes context-derived parameters.
    fn fail(&self, game: &mut Game) -> StepOutcome {
        let mut out = if !self.goto_label_on_failure.is_empty() {
            StepOutcome::goto(&self.goto_label_on_failure)
        } else {
            StepOutcome::next()
        };

        // Java: context.getDeferredCommands().forEach(cmd → cmd.execute(this))
        if let Some(ctx) = &self.context {
            for param in ctx.execute_deferred_commands(game) {
                out = out.publish(param);
            }
        }

        if let Some(ctx) = &self.context {
            // Java: if (context.getDropPlayerContext() != null) publishParameter(DROP_PLAYER_CONTEXT, ...)
            if let Some(dpc) = ctx.drop_player_context() {
                out = out.publish(StepParameter::DropPlayerContext(Box::new(dpc.clone())));
            }
            // Java: if (context.getInjuryResult() != null) publishParameter(INJURY_RESULT, ...)
            if let Some(r) = ctx.injury_result() {
                out = out.publish(StepParameter::InjuryResult(Box::new(r.clone())));
            }
            // Java: if (context.getInjuryType() != null) publishParameter(INJURY_TYPE, ...)
            if let Some(name) = ctx.injury_type_name() {
                out = out.publish(StepParameter::InjuryTypeName(name.to_owned()));
            }
        }

        // Java: if (apothecaryMode == ATTACKER) publishParameter(ATTACKER_ALREADY_DOWN, true)
        if self.apothecary_mode == Some(ApothecaryMode::Attacker) {
            out = out.publish(StepParameter::AttackerAlreadyDown(true));
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drop_player_context::{DropPlayerContext, SteadyFootingContext};
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{ApothecaryMode, Rules, SkillId, PS_FALLING, PS_STANDING,
                           PlayerType, PlayerGender};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    fn bare_player(id: &str) -> Player {
        Player {
            id: id.into(), name: id.into(), nr: 0, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 3, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        }
    }

    fn add_player_with_steady_footing(game: &mut Game, pid: &str) {
        let mut p = bare_player(pid);
        p.starting_skills.push(SkillWithValue::new(SkillId::SteadyFooting));
        game.team_home.players.push(p);
        game.field_model.set_player_state(pid, PlayerState::new(PS_FALLING));
    }

    /// Helper: create a minimal SteadyFootingContext with the given player id.
    fn make_context(player_id: &str) -> Box<SteadyFootingContext> {
        let dpc = DropPlayerContext { player_id: Some(player_id.to_owned()), ..DropPlayerContext::new() };
        Box::new(SteadyFootingContext::from_drop_player(dpc))
    }

    /// No context → always NEXT_STEP without rolling.
    #[test]
    fn no_context_returns_next_step() {
        let mut game = make_game();
        let mut step = StepSteadyFooting::new("fail".into(), "success".into());
        assert!(step.context.is_none());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    /// No player_id with context → fail path → goto failure label.
    #[test]
    fn context_no_player_id_goes_to_failure_label() {
        let mut game = make_game();
        let mut step = StepSteadyFooting::new("fail_label".into(), "ok_label".into());
        // Context with no player_id
        step.context = Some(Box::new(SteadyFootingContext::from_drop_player(DropPlayerContext::new())));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail_label"));
    }

    /// Player has SteadyFooting + context set; minimum roll is 6, so only roll=6 succeeds.
    #[test]
    fn success_on_roll_six_returns_next_step() {
        for seed in 0..1000u64 {
            let mut rng = GameRng::new(seed);
            if rng.d6() == 6 {
                let mut g = make_game();
                add_player_with_steady_footing(&mut g, "p1");
                let mut s = StepSteadyFooting::new(String::new(), String::new());
                s.context = Some(make_context("p1"));
                s.player_id = Some("p1".into());
                s.use_skill = Some(true);
                let out = s.start(&mut g, &mut GameRng::new(seed));
                assert_eq!(out.action, StepAction::NextStep, "seed={seed}");
                return;
            }
        }
        panic!("could not find a seed producing roll=6");
    }

    /// Player has SteadyFooting + context; roll < 6 → fail path (goto failure label).
    #[test]
    fn failure_on_low_roll_goes_to_failure_label() {
        for seed in 0..200u64 {
            let mut rng = GameRng::new(seed);
            if rng.d6() < 6 {
                let mut g = make_game();
                add_player_with_steady_footing(&mut g, "p1");
                let mut s = StepSteadyFooting::new("fail_lbl".into(), "ok_lbl".into());
                s.context = Some(make_context("p1"));
                s.player_id = Some("p1".into());
                s.use_skill = Some(true);
                let out = s.start(&mut g, &mut GameRng::new(seed));
                assert_eq!(out.action, StepAction::GotoLabel, "seed={seed}");
                assert_eq!(out.goto_label.as_deref(), Some("fail_lbl"));
                return;
            }
        }
        panic!("could not find a seed producing roll < 6");
    }

    /// skip=true → fail path immediately.
    #[test]
    fn skip_true_fails_immediately() {
        let mut game = make_game();
        add_player_with_steady_footing(&mut game, "p1");

        let mut step = StepSteadyFooting::new("fail_skip".into(), "ok".into());
        step.context = Some(make_context("p1"));
        step.player_id = Some("p1".into());
        step.skip = true;

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail_skip"));
    }

    /// No SteadyFooting skill → fail path.
    #[test]
    fn no_skill_fails() {
        let mut game = make_game();
        let p = bare_player("p1"); // no skills
        game.team_home.players.push(p);
        game.field_model.set_player_state("p1", PlayerState::new(PS_FALLING));

        let mut step = StepSteadyFooting::new("fail".into(), "ok".into());
        step.context = Some(make_context("p1"));
        step.player_id = Some("p1".into());

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    /// use_skill = false → fail path.
    #[test]
    fn use_skill_false_fails() {
        let mut game = make_game();
        add_player_with_steady_footing(&mut game, "p1");

        let mut step = StepSteadyFooting::new("fail".into(), "ok".into());
        step.context = Some(make_context("p1"));
        step.player_id = Some("p1".into());
        step.use_skill = Some(false);

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    /// Success: acting team player → publishes EndTurn(false) + EndPlayerAction(false).
    #[test]
    fn success_publishes_end_turn_false_for_active_player() {
        for seed in 0..1000u64 {
            let mut rng = GameRng::new(seed);
            if rng.d6() == 6 {
                let mut g = make_game();
                add_player_with_steady_footing(&mut g, "p1");
                g.acting_player.player_id = Some("p1".into());

                let mut s = StepSteadyFooting::new(String::new(), String::new());
                s.context = Some(make_context("p1"));
                s.player_id = Some("p1".into());
                s.use_skill = Some(true);

                let out = s.start(&mut g, &mut GameRng::new(seed));
                assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(false))),
                    "seed={seed}");
                assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(false))),
                    "seed={seed}");
                return;
            }
        }
        panic!("no seed rolls 6");
    }

    /// set_parameter: OldDefenderState accepted only when apothecary_mode == Defender.
    #[test]
    fn set_parameter_old_defender_state_requires_defender_mode() {
        let mut step = StepSteadyFooting::default();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        assert!(step.set_parameter(&StepParameter::OldDefenderState(PlayerState::new(PS_STANDING))));
        assert!(step.old_defender_state.is_some());
    }

    #[test]
    fn set_parameter_old_defender_state_rejected_without_mode() {
        let mut step = StepSteadyFooting::default();
        assert!(!step.set_parameter(&StepParameter::OldDefenderState(PlayerState::new(PS_STANDING))));
        assert!(step.old_defender_state.is_none());
    }

    /// set_parameter: PlayerId accepted.
    #[test]
    fn set_parameter_player_id_accepted() {
        let mut step = StepSteadyFooting::default();
        assert!(step.set_parameter(&StepParameter::PlayerId("p42".into())));
        assert_eq!(step.player_id.as_deref(), Some("p42"));
    }

    /// set_parameter: SteadyFootingContext accepted when apothecary_mode matches.
    #[test]
    fn set_parameter_steady_footing_context_accepted() {
        let mut step = StepSteadyFooting::default();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        let ctx = SteadyFootingContext::from_drop_player(DropPlayerContext {
            player_id: Some("p1".into()),
            apothecary_mode: Some(ApothecaryMode::Defender),
            ..DropPlayerContext::new()
        });
        assert!(step.set_parameter(&StepParameter::SteadyFootingContext(Box::new(ctx))));
        assert!(step.context.is_some());
        assert_eq!(step.player_id.as_deref(), Some("p1"));
    }

    #[test]
    fn set_parameter_steady_footing_context_rejected_wrong_mode() {
        let mut step = StepSteadyFooting::default();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        let ctx = SteadyFootingContext::from_drop_player(DropPlayerContext {
            apothecary_mode: Some(ApothecaryMode::Attacker),
            ..DropPlayerContext::new()
        });
        assert!(!step.set_parameter(&StepParameter::SteadyFootingContext(Box::new(ctx))));
        assert!(step.context.is_none());
    }

    /// fail() publishes DROP_PLAYER_CONTEXT when context holds one.
    #[test]
    fn fail_publishes_drop_player_context() {
        let mut game = make_game();
        let p = bare_player("p1");
        game.team_home.players.push(p);
        game.field_model.set_player_state("p1", PlayerState::new(PS_FALLING));

        let mut step = StepSteadyFooting::new(String::new(), String::new());
        step.context = Some(make_context("p1"));
        step.player_id = Some("p1".into()); // no skill → fail
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::DropPlayerContext(_))));
    }

    /// Success path with goto_label_on_success set → GotoLabel action.
    #[test]
    fn success_with_goto_label_returns_goto_label() {
        for seed in 0..1000u64 {
            let mut rng = GameRng::new(seed);
            if rng.d6() == 6 {
                let mut g = make_game();
                add_player_with_steady_footing(&mut g, "p1");

                let mut s = StepSteadyFooting::new("fail".into(), "success_label".into());
                s.context = Some(make_context("p1"));
                s.player_id = Some("p1".into());
                s.use_skill = Some(true);

                let out = s.start(&mut g, &mut GameRng::new(seed));
                assert_eq!(out.action, StepAction::GotoLabel, "seed={seed}");
                assert_eq!(out.goto_label.as_deref(), Some("success_label"));
                return;
            }
        }
        panic!("no seed rolls 6");
    }

    /// Attacker apothecary mode: FALLING state gets corrected to MOVING on success.
    #[test]
    fn attacker_mode_corrects_falling_to_moving_on_success() {
        for seed in 0..1000u64 {
            let mut rng = GameRng::new(seed);
            if rng.d6() == 6 {
                let mut g = make_game();
                add_player_with_steady_footing(&mut g, "p1");
                g.field_model.set_player_state("p1", PlayerState::new(PS_FALLING));

                let mut s = StepSteadyFooting::new(String::new(), String::new());
                s.context = Some(make_context("p1"));
                s.player_id = Some("p1".into());
                s.use_skill = Some(true);
                s.apothecary_mode = Some(ApothecaryMode::Attacker);

                s.start(&mut g, &mut GameRng::new(seed));

                let state = g.field_model.player_state("p1").unwrap();
                assert_eq!(state.base(), PS_MOVING, "seed={seed}");
                return;
            }
        }
        panic!("no seed rolls 6");
    }
}
