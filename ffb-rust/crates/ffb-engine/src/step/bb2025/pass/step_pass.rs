use ffb_model::enums::{PassingDistance, PlayerAction, ReRollSource};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::report::mixed::report_pass_roll::ReportPassRoll;
use ffb_mechanics::modifiers::modifier_type::ModifierType;
use ffb_mechanics::modifiers::pass_context::PassContext;
use ffb_mechanics::modifiers::pass_modifier::PassModifier;
use ffb_mechanics::modifiers::pass_modifier_factory::PassModifierFactory;
use ffb_mechanics::pass_mechanic::PassMechanic;
use ffb_mechanics::pass_result::PassResult;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{CatchScatterThrowInMode, StepId, StepParameter};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use crate::step::abstract_step_with_re_roll::find_skill_reroll_source;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.pass.StepPass.
///
/// Main pass step: sets ball/bomb moving, computes passing distance, rolls agility (PA),
/// handles Safe Pass dialog, re-roll prompts, and routes to accurate/fumble/missed labels.
///
/// Needs init params: `GotoLabelOnEnd`, `GotoLabelOnMissedPass`, `GotoLabelOnSavedFumble`.
/// Expects stepParameter `CatcherId` from a preceding step.
/// Publishes: `PassingDistance`, `PassFumble`, `DontDropFumble`, `CatcherId`,
///            `CatchScatterThrowInMode`, `PassResultParam`.
///
/// client-only: re-roll dialog — headless uses auto-reroll via AbstractStepWithReRoll.
/// client-only: Safe Pass (dontDropFumbles) dialog — headless auto-skips.
/// client-only: usingModifyingSkill dialog (canAddStrengthToPass) — headless auto-declines skill use.
pub struct StepPass {
    /// Java: goToLabelOnEnd (init param, mandatory)
    pub goto_label_on_end: String,
    /// Java: goToLabelOnSavedFumble (init param, mandatory)
    pub goto_label_on_saved_fumble: String,
    /// Java: goToLabelOnMissedPass (init param, mandatory)
    pub goto_label_on_missed_pass: String,
    /// Java: PassState.catcherId — set via CatcherId parameter
    pub catcher_id: Option<String>,
    /// Java: usingModifyingSkill (Boolean tristate — null=not asked, true/false=answered)
    pub using_modifying_skill: Option<bool>,
    /// Java: usingSafePass (Boolean tristate — null=not asked, true/false=answered)
    pub using_safe_pass: Option<bool>,
    /// Java: roll
    pub roll: i32,
    /// Java: minimumRoll
    pub minimum_roll: i32,
    /// Java: PassState.result — the PassResult from evaluatePass()
    pub pass_result: Option<PassResult>,
    // AbstractStepWithReRoll fields
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
}

impl StepPass {
    pub fn new(
        goto_label_on_end: String,
        goto_label_on_missed_pass: String,
        goto_label_on_saved_fumble: String,
    ) -> Self {
        Self {
            goto_label_on_end,
            goto_label_on_saved_fumble,
            goto_label_on_missed_pass,
            catcher_id: None,
            using_modifying_skill: None,
            using_safe_pass: None,
            roll: 0,
            minimum_roll: 0,
            pass_result: None,
            re_rolled_action: None,
            re_roll_source: None,
        }
    }
}

impl Step for StepPass {
    fn id(&self) -> StepId { StepId::Pass }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_USE_SKILL → canAddStrengthToPass → usingModifyingSkill = isSkillUsed()
        // Java: CLIENT_USE_SKILL → dontDropFumbles      → usingSafePass = isSkillUsed()
        // Java: otherwise → handleSkillCommand(commandUseSkill, passState) [pass reroll dialog]
        match action {
            Action::UseSkill { skill_id, use_skill } => {
                // Java: route by skill property: canAddStrengthToPass → usingModifyingSkill
                //                                dontDropFumbles       → usingSafePass
                use ffb_model::model::property::named_properties::NamedProperties;
                if skill_id.properties().contains(&NamedProperties::DONT_DROP_FUMBLES) {
                    self.using_safe_pass = Some(*use_skill);
                } else {
                    self.using_modifying_skill = Some(*use_skill);
                }
            }
            Action::UseReRoll { use_reroll: false } => {
                // Player declined re-roll — keep re_rolled_action set so we don't re-offer.
                self.re_roll_source = None;
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            // Java: CATCHER_ID → passState.setCatcherId(value)
            StepParameter::CatcherId(v) => { self.catcher_id = v.clone(); true }
            StepParameter::GotoLabelOnEnd(v) => { self.goto_label_on_end = v.clone(); true }
            // Java init key GOTO_LABEL_ON_MISSED_PASS maps to GotoLabelOnFailure variant
            StepParameter::GotoLabelOnFailure(v) => { self.goto_label_on_missed_pass = v.clone(); true }
            StepParameter::GotoLabelOnMissedPass(v) => { self.goto_label_on_missed_pass = v.clone(); true }
            // Java init key GOTO_LABEL_ON_SAVED_FUMBLE maps to GotoLabelOnSuccess variant
            StepParameter::GotoLabelOnSuccess(v) => { self.goto_label_on_saved_fumble = v.clone(); true }
            StepParameter::GotoLabelOnSavedFumble(v) => { self.goto_label_on_saved_fumble = v.clone(); true }
            StepParameter::UsingModifyingSkill(v) => { self.using_modifying_skill = Some(*v); true }
            _ => false,
        }
    }

    /// Java `StepPass.setParameter` returns TRUE for CATCHER_ID — i.e. it CONSUMES the key into
    /// `PassState.catcherId` and the parameter travels no further. Rust accepted it without
    /// consuming, so the intended receiver kept flowing downstream to `StepCatchScatterThrowIn`,
    /// whose `if catcher_id.is_none() { catcher_id = player_under_ball }` then never fired. On a
    /// DEFLECTED pass that resolved the catch for the RECEIVER instead of the deflector standing
    /// under the ball, and the ball landed on the receiver's square (dark_elf bb2020 seed 21 i=95:
    /// `catcher_id=away_09` while `under_ball=home_02`, ball 22,7 in Rust vs 12,7 in Java).
    fn consumes_parameter(&self, param: &StepParameter) -> bool {
        matches!(param, StepParameter::CatcherId(_))
    }
}

impl StepPass {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java guard: if thrower or throwerAction is null → return (no-op).
        // Java's `return;` here leaves StepResult's default nextAction (CONTINUE) untouched —
        // it does NOT jump to goToLabelOnEnd. Match that exactly: stay put / wait.
        if game.thrower_id.is_none() || game.thrower_action.is_none() {
            return StepOutcome::cont();
        }

        // Java: if (PASS == reRolledAction) { if (source == null || !useReRoll) proceed with stored result }
        //       else → clear roll + result → re-roll below
        if self.re_rolled_action.as_deref() == Some("PASS") {
            let thrower_id = game.thrower_id.clone().unwrap_or_default();
            if let Some(ref source_name) = self.re_roll_source.clone() {
                let source = ReRollSource::new(source_name.as_str());
                if use_reroll(game, &source, &thrower_id) {
                    // Re-roll consumed — clear stored roll so we re-roll below
                    self.roll = 0;
                    self.pass_result = None;
                }
                // else: token exhausted → fall through with stored result
            }
            // source == None (player declined) → fall through with stored result
        }

        let is_bomb = matches!(
            game.thrower_action,
            Some(PlayerAction::ThrowBomb) | Some(PlayerAction::HailMaryBomb)
        );

        // Java: set ball/bomb moving flag
        if is_bomb {
            game.field_model.bomb_moving = true;
            if game.original_bombardier.is_none() {
                game.original_bombardier = game.thrower_id.clone();
            }
        } else {
            game.field_model.ball_moving = true;
        }

        // Java: throwerCoordinate = fieldModel.getPlayerCoordinate(thrower)
        let thrower_id = game.thrower_id.clone().unwrap();
        let thrower_coord = game.field_model.player_coordinate(&thrower_id);

        // Java: PassMechanic.findPassingDistance(game, throwerCoord, passCoordinate, false) — use the
        // EDITION's throwing_range_table (via the mechanic), NOT the hardcoded BB2020 `passing_distance`
        // util. bb2016's range table is shorter at the far corners (e.g. dx=13,dy=2 is out-of-range in
        // bb2016 but LongBomb in bb2020), so a long pass bb2020 would call LongBomb is OUT OF RANGE in
        // bb2016 → findPassingDistance returns None → thrown with NO accuracy roll (StepPass rolls 0
        // dice). amazon bb2016 seed56 i=170: home_03's (14,7)->(1,9) pass — Java (out of range) rolled
        // 0 dice, Rust (shared bb2020 table said LongBomb) rolled the accuracy die → 2-die desync.
        // find_passing_distance also applies the Blizzard/TTM Long(Bomb) out-of-range gate (1:1 Java).
        let passing_dist: Option<PassingDistance> =
            crate::mechanic::pass_mechanic_for(game.rules)
                .find_passing_distance(game, thrower_coord, game.pass_coordinate, false);

        // Java: PassModifierFactory.findModifiers(new PassContext(game, thrower, passingDistance, false))
        let pass_modifier_total: i32 = {
            if let (Some(thrower), Some(dist)) = (game.thrower(), passing_dist) {
                let factory = PassModifierFactory::for_rules(game.rules);
                let ctx = PassContext::new(game, thrower, dist, false);
                let collection_total: i32 = factory.find_modifiers(&ctx).iter().map(|m| m.get_modifier()).sum();
                let skill_total: i32 = factory.find_skill_modifiers(&ctx).iter().map(|m| m.get_modifier()).sum();
                let card_total: i32 = factory.find_card_modifiers(&ctx).iter().map(|m| m.get_modifier()).sum();
                collection_total + skill_total + card_total
            } else {
                0
            }
        };
        let pass_modifiers: Vec<PassModifier> = if pass_modifier_total != 0 {
            vec![PassModifier::new("pass_mods", pass_modifier_total, ModifierType::REGULAR)]
        } else {
            vec![]
        };

        if std::env::var("FFB_TRACE").is_ok() {
            eprintln!("RUST_STEPPASS thrower={:?} action={:?} pass_coord={:?} thrower_coord={:?} dist={:?} roll={}",
                game.thrower_id, game.thrower_action, game.pass_coordinate, thrower_coord, passing_dist, self.roll);
        }
        // Roll if not yet rolled (roll=0 means fresh)
        if self.roll == 0 {
            // Java: publishParameter(from(PASSING_DISTANCE, passingDistance))
            if let Some(dist) = passing_dist {
                // Publish passing distance for downstream steps
                // (stored by StepEndPassing etc.)
            }

            // Java: minimumRoll = mechanic.minimumRoll(thrower, passingDistance, passModifiers)
            // Edition-aware: BB2016 classifies the pass roll (accurate/inaccurate/fumble) by a
            // different range/AG table than BB2020/BB2025 — a hardcoded Bb2025PassMechanic mis-graded
            // BB2016 passes (amazon bb2016 seed4 i=136: pass d6=3 → Java INACCURATE, Rust FUMBLE →
            // different scatter chain + die count).
            let mechanic = crate::mechanic::pass_mechanic_for(game.rules);
            if let Some(thrower) = game.thrower() {
                let minimum = passing_dist.and_then(|dist| {
                    mechanic.minimum_roll_simple(thrower, dist, &pass_modifiers)
                });
                self.minimum_roll = minimum.unwrap_or(0);
            }

            // Java: roll = minimumRollO.isPresent() ? getDiceRoller().rollSkill() : 0
            self.roll = if self.minimum_roll > 0 { rng.d6() } else { 0 };

            // Java: state.setThrowerCoordinate(throwerCoordinate)
            // (stored in pass state for ScatterBall at thrower coord on fumble)
        }

        // Java: state.setResult(mechanic.evaluatePass(thrower, roll, passingDistance, passModifiers, isBomb))
        // A declined re-roll re-enters this step with the stored roll/result — only a
        // freshly evaluated roll counts as a newly resolved roll for event emission.
        let freshly_resolved = self.pass_result.is_none();
        if self.pass_result.is_none() {
            let result = if let Some(thrower) = game.thrower() {
                if let Some(dist) = passing_dist {
                    let mechanic = crate::mechanic::pass_mechanic_for(game.rules);
                    mechanic.evaluate_pass_simple(thrower, self.roll, dist, &pass_modifiers, is_bomb)
                } else {
                    // No passing distance → auto-fumble
                    PassResult::FUMBLE
                }
            } else {
                PassResult::FUMBLE
            };
            self.pass_result = Some(result);
        }

        let result = self.pass_result.unwrap();
        let already_rerolled = self.re_rolled_action.is_some();
        // An OUT-OF-RANGE pass (findPassingDistance → None) is never thrown: no accuracy roll (roll=0
        // above) and — unlike a rolled FUMBLE — the ball is NOT scattered. Java's out-of-range pass
        // leaves the ball on the thrower's square and simply turns the drive over (0 dice total).
        // Rust classified None as PassResult::FUMBLE, which scattered the ball (1 extra d8) →
        // amazon bb2016 seed56 i=170: home_03's (14,7)->(1,9) LongBomb was out of range in bb2016;
        // Java kept the ball at (14,7), Rust scattered it to (15,6). Handle the None case as a
        // no-scatter turnover before the FUMBLE scatter path.
        let out_of_range = passing_dist.is_none();

        // Java: getResult().addReport(new ReportPassRoll(game.getThrowerId(), roll, minimumRoll, reRolled,
        //   passModifiers, passingDistance, isBomb, state.getResult(), false, statBasedRollModifier))
        {
            let re_rolled = self.re_rolled_action.is_some() && self.re_roll_source.is_some();
            let pass_result_name = self.pass_result.map(|r| r.get_name().to_string());
            let successful = self.pass_result == Some(PassResult::ACCURATE);
            let dist_name = passing_dist.map(|d| format!("{:?}", d));
            game.report_list.add(ReportPassRoll::new(
                game.thrower_id.clone(),
                successful,
                self.roll,
                self.minimum_roll,
                re_rolled,
                vec![],
                dist_name,
                is_bomb,
                pass_result_name,
                false,
                None,
            ));
        }

        // Emit one GameEvent per resolved roll (monolith parity: initial roll and
        // re-rolled resolution each produce their own PassRoll event; a declined
        // re-roll reuses the stored result and emits nothing new).
        let roll_event = if freshly_resolved {
            use ffb_model::enums::PassOutcome;
            let re_rolled = self.re_rolled_action.is_some() && self.re_roll_source.is_some();
            Some(ffb_model::events::GameEvent::PassRoll {
                player_id: thrower_id.clone(),
                target: self.minimum_roll,
                distance: passing_dist.unwrap_or(PassingDistance::LongBomb),
                roll: self.roll,
                result: match result {
                    PassResult::ACCURATE => PassOutcome::Complete,
                    PassResult::INACCURATE => PassOutcome::Inaccurate,
                    PassResult::WILDLY_INACCURATE => PassOutcome::WildlyInaccurate,
                    PassResult::FUMBLE | PassResult::SAVED_FUMBLE => PassOutcome::Fumble,
                },
                rerolled: re_rolled,
            })
        } else {
            None
        };

        // Java result routing:
        let outcome = match result {
            PassResult::ACCURATE => {
                // Java: fieldModel.setBallCoordinate(game.getPassCoordinate()) [or setBombCoordinate]
                if let Some(pass_coord) = game.pass_coordinate {
                    if is_bomb {
                        game.field_model.bomb_coordinate = Some(pass_coord);
                    } else {
                        game.field_model.ball_coordinate = Some(pass_coord);
                    }
                }
                let label = self.goto_label_on_end.clone();
                // Java StepResolvePass keys the accurate-catch routing off the shared PassState.result
                // (== ACCURATE). Rust's StepResolvePass instead reads a `PassAccurate` bool it expects
                // StepPass to publish — but StepPass only published PassResultParam, so pass_accurate
                // stayed false and every accurate pass fell through to the missed/inaccurate branch
                // (CatchScatter, +1 to catch). Publish PassAccurate(true) so the receiver actually
                // catches an accurate pass (seed 23 i=158). Bombs route through their own accurate
                // branch, which does not read PassAccurate, so only publish it for a real ball pass.
                StepOutcome::goto(&label)
                    .publish(StepParameter::PassResultParam(ffb_model::enums::PassOutcome::Complete))
                    .publish(StepParameter::PassAccurate(!is_bomb))
            }
            PassResult::SAVED_FUMBLE => {
                // Java: SAVED_FUMBLE routes through the SAME re-roll offers as FUMBLE — the
                // reroll block precedes handleFailedPass/handleSafePass, so a Pass-skill
                // re-roll can roll the fumble away entirely before Safe Pass is consulted
                // (amazon seed 34 i=45: roll 1 → auto Pass re-roll 5 → ACCURATE → catch).
                if !already_rerolled {
                    if let Some(source) = find_skill_reroll_source(game, "PASS") {
                        self.re_rolled_action = Some("PASS".into());
                        self.re_roll_source = Some(source.name.clone());
                        return self.execute_step(game, rng);
                    }
                    if let Some(prompt) = ask_for_reroll_if_available(game, "PASS", self.minimum_roll, true) {
                        self.re_rolled_action = Some("PASS".into());
                        self.re_roll_source = Some("TRR".into());
                        let mut out = StepOutcome::cont().with_prompt(prompt);
                        if let Some(ev) = roll_event { out = out.with_event(ev); }
                        return out;
                    }
                }
                // Java: handleSafePass → usingSafePass dialog (ParityRunner SKILL_USE =
                // always use, 0 rng) → markSkillUsed(safePass), ball stays with the
                // thrower, goto goToLabelOnSavedFumble.
                if is_bomb {
                    game.field_model.bomb_coordinate = None;
                    game.field_model.bomb_moving = false;
                } else {
                    if let Some(tc) = thrower_coord {
                        game.field_model.ball_coordinate = Some(tc);
                    }
                    game.field_model.ball_moving = false;
                }
                let label = self.goto_label_on_saved_fumble.clone();
                StepOutcome::goto(&label)
                    .publish(StepParameter::PassFumble(false))
                    .publish(StepParameter::DontDropFumble(true))
                    .publish(StepParameter::PassResultParam(ffb_model::enums::PassOutcome::Fumble))
            }
            // Out-of-range pass: never thrown. Ball stays on the thrower's square, NO scatter, NO
            // re-roll (there was no roll), drive turns over. Matches Java's null-passingDistance path.
            PassResult::FUMBLE if out_of_range => {
                if let Some(tc) = thrower_coord {
                    if is_bomb {
                        game.field_model.bomb_coordinate = Some(tc);
                        game.field_model.bomb_moving = false;
                    } else {
                        game.field_model.ball_coordinate = Some(tc);
                        game.field_model.ball_moving = false;
                    }
                }
                StepOutcome::next()
                    .publish(StepParameter::PassFumble(true))
                    .publish(StepParameter::DontDropFumble(false))
                    .publish(StepParameter::CatcherId(None))
                    .publish(StepParameter::PassResultParam(ffb_model::enums::PassOutcome::Fumble))
            }
            PassResult::FUMBLE => {
                // Java: askForReRollIfAvailable before handling fumble
                if !already_rerolled {
                    // A FREE single-use SKILL re-roll (e.g. Pass) is offered by Java as a SKILL_USE
                    // that ParityRunner ALWAYS uses — mirroring the engine's auto-use of Sure
                    // Hands/Catch. Auto-use it here (no prompt): the pass die re-rolls once. Only a
                    // TEAM re-roll is offered to the agent (which declines it deterministically).
                    if let Some(source) = find_skill_reroll_source(game, "PASS") {
                        self.re_rolled_action = Some("PASS".into());
                        self.re_roll_source = Some(source.name.clone());
                        // Re-enter: the top-of-function re-roll gate consumes the skill token via
                        // use_reroll (marks it used), clears roll/result and re-rolls; already_rerolled
                        // then blocks a second offer so the re-rolled result stands.
                        return self.execute_step(game, rng);
                    }
                    if let Some(prompt) = ask_for_reroll_if_available(game, "PASS", self.minimum_roll, true) {
                        self.re_rolled_action = Some("PASS".into());
                        self.re_roll_source = Some("TRR".into());
                        let mut out = StepOutcome::cont().with_prompt(prompt);
                        if let Some(ev) = roll_event { out = out.with_event(ev); }
                        return out;
                    }
                }
                if let Some(tc) = thrower_coord {
                    if is_bomb {
                        game.field_model.bomb_coordinate = Some(tc);
                    } else {
                        game.field_model.ball_coordinate = Some(tc);
                        game.field_model.ball_moving = false;
                    }
                }
                StepOutcome::next()
                    .publish(StepParameter::PassFumble(true))
                    .publish(StepParameter::DontDropFumble(false))
                    .publish(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ScatterBall))
                    .publish(StepParameter::CatcherId(None))
                    .publish(StepParameter::PassResultParam(ffb_model::enums::PassOutcome::Fumble))
            }
            PassResult::INACCURATE | PassResult::WILDLY_INACCURATE => {
                // Java: askForReRollIfAvailable before routing to missed pass
                if !already_rerolled {
                    // Free single-use SKILL re-roll (Pass): auto-use it (see the FUMBLE branch).
                    if let Some(source) = find_skill_reroll_source(game, "PASS") {
                        self.re_rolled_action = Some("PASS".into());
                        self.re_roll_source = Some(source.name.clone());
                        return self.execute_step(game, rng);
                    }
                    if let Some(prompt) = ask_for_reroll_if_available(game, "PASS", self.minimum_roll, false) {
                        self.re_rolled_action = Some("PASS".into());
                        self.re_roll_source = Some("TRR".into());
                        let mut out = StepOutcome::cont().with_prompt(prompt);
                        if let Some(ev) = roll_event { out = out.with_event(ev); }
                        return out;
                    }
                }
                if let Some(pass_coord) = game.pass_coordinate {
                    if is_bomb {
                        game.field_model.bomb_coordinate = Some(pass_coord);
                    } else {
                        game.field_model.ball_coordinate = Some(pass_coord);
                    }
                }
                let label = self.goto_label_on_missed_pass.clone();
                // Java stores the PassResult itself in PassState; BB2020's StepMissedPass reads it
                // back to choose between the deviate (WILDLY_INACCURATE) and 3x1-scatter arms.
                // Flattening both to Inaccurate here made BB2020 always take the scatter arm.
                let outcome_param = if self.pass_result == Some(PassResult::WILDLY_INACCURATE) {
                    ffb_model::enums::PassOutcome::WildlyInaccurate
                } else {
                    ffb_model::enums::PassOutcome::Inaccurate
                };
                StepOutcome::goto(&label)
                    .publish(StepParameter::CatcherId(None))
                    .publish(StepParameter::PassResultParam(outcome_param))
            }
        };
        match roll_event {
            Some(ev) => outcome.with_event(ev),
            None => outcome,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::action::Action;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::Rules;
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    fn make_step() -> StepPass {
        StepPass::new("end".into(), "missed".into(), "saved_fumble".into())
    }

    #[test]
    fn no_thrower_stays_put_matching_java_implicit_continue_default() {
        // Bug fix regression: Java's `if (thrower == null || throwerAction == null) return;`
        // leaves StepResult's default nextAction (CONTINUE) untouched — it does NOT jump to
        // goToLabelOnEnd. The step must wait, not skip ahead.
        let mut game = make_game();
        let mut step = make_step();
        // thrower_id is None by default
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
    }

    /// Java's `StepPass.setParameter` returns TRUE for CATCHER_ID — it CONSUMES the key into
    /// `PassState.catcherId`, so the parameter never reaches `StepCatchScatterThrowIn`. Rust merely
    /// accepted it, and the leaked receiver id then resolved a DEFLECTED catch for the receiver
    /// instead of the deflector under the ball (dark_elf bb2020 seed 21).
    #[test]
    fn catcher_id_is_consumed_so_it_cannot_leak_downstream() {
        let step = make_step();
        assert!(step.consumes_parameter(&StepParameter::CatcherId(Some("p1".into()))),
            "CATCHER_ID stops at StepPass, as in Java");
        assert!(!step.consumes_parameter(&StepParameter::GotoLabelOnEnd("x".into())),
            "other keys are not consumed here");
    }

    #[test]
    fn set_parameter_catcher_id_accepted() {
        let mut step = make_step();
        assert!(step.set_parameter(&StepParameter::CatcherId(Some("p1".into()))));
        assert_eq!(step.catcher_id.as_deref(), Some("p1"));
    }

    #[test]
    fn set_parameter_goto_label_on_missed_pass_accepted() {
        let mut step = make_step();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnMissedPass("mp".into())));
        assert_eq!(step.goto_label_on_missed_pass.as_str(), "mp");
    }

    #[test]
    fn set_parameter_goto_label_on_saved_fumble_accepted() {
        let mut step = make_step();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnSavedFumble("sf".into())));
        assert_eq!(step.goto_label_on_saved_fumble.as_str(), "sf");
    }

    /// Helper: build a game with a thrower player that has specific PA stats
    fn make_game_with_thrower(pa: i32) -> Game {
        let mut home = test_team("home", 0);
        let away = test_team("away", 0);
        let mut thrower = ffb_model::model::player::Player::default();
        thrower.id = "t1".into();
        thrower.passing = pa;
        home.players.push(thrower);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.thrower_id = Some("t1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        // Place thrower at (1,7), target at (4,7) → dx=3 → QuickPass
        game.field_model.set_player_coordinate("t1", FieldCoordinate::new(1, 7));
        game.pass_coordinate = Some(FieldCoordinate::new(4, 7));
        game
    }

    #[test]
    fn fumble_pa_zero_publishes_pass_fumble_true() {
        // PA=0 means no passing ability → auto-fumble in BB2025
        let mut game = make_game_with_thrower(0);
        let mut step = make_step();
        let out = step.start(&mut game, &mut GameRng::new(0));
        // PA=0 → evaluatePass returns FUMBLE → NextStep
        assert_eq!(out.action, StepAction::NextStep);
        let fumble = out.published.iter().find(|p| matches!(p, StepParameter::PassFumble(true)));
        assert!(fumble.is_some(), "expected PassFumble(true) published for PA=0");
    }

    #[test]
    fn bb2016_out_of_range_pass_keeps_ball_and_does_not_scatter() {
        // A bb2016 pass whose target is OUT OF RANGE (findPassingDistance → None) is never thrown:
        // no accuracy roll AND no scatter — the ball stays on the thrower's square and the drive
        // turns over. (14,7)->(1,9) is dx=13,dy=2 = out of range in the bb2016 range table (LongBomb
        // in bb2020). amazon bb2016 seed56 i=170: Java kept the ball at (14,7); Rust used to FUMBLE +
        // scatter it (1 extra d8).
        let mut home = test_team("home", 0);
        let away = test_team("away", 0);
        let mut thrower = ffb_model::model::player::Player::default();
        thrower.id = "t1".into();
        thrower.passing = 3;
        home.players.push(thrower);
        let mut game = Game::new(home, away, Rules::Bb2016);
        game.thrower_id = Some("t1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        game.field_model.set_player_coordinate("t1", FieldCoordinate::new(14, 7));
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(14, 7));
        game.pass_coordinate = Some(FieldCoordinate::new(1, 9)); // dx=13, dy=2 → out of range (bb2016)
        let mut step = make_step();
        let mut rng = GameRng::new(0);
        let start_calls = rng.call_count;
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(rng.call_count - start_calls, 0, "an out-of-range pass rolls NO dice (no accuracy, no scatter)");
        assert!(!out.published.iter().any(|p| matches!(p, StepParameter::CatchScatterThrowInMode(_))),
            "an out-of-range pass must NOT publish a ScatterBall mode (ball stays at the thrower)");
        assert_eq!(game.field_model.ball_coordinate, Some(FieldCoordinate::new(14, 7)),
            "the ball must remain on the thrower's square");
    }

    #[test]
    fn forced_fumble_roll_goes_to_next_step() {
        // Force roll=1 which is always a fumble in BB2025 (natural 1)
        let mut game = make_game_with_thrower(3);
        let mut step = make_step();
        step.roll = 1; // force natural 1 → FUMBLE
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        let fumble = out.published.iter().find(|p| matches!(p, StepParameter::PassFumble(true)));
        assert!(fumble.is_some(), "expected PassFumble(true) on natural 1");
    }

    #[test]
    fn fumble_auto_uses_free_pass_skill_reroll_without_prompt() {
        // Regression (docs/PARITY_TTM.md "FRONTIER (human) — seed 4 step 174"): a thrower with the
        // Pass skill that fumbles/misses has a FREE single-use skill re-roll. Java offers it as a
        // SKILL_USE that ParityRunner ALWAYS uses (mirroring the engine's auto-use of Sure Hands /
        // Catch), so the engine must AUTO-USE it — re-rolling the pass die once and marking the Pass
        // skill used — WITHOUT emitting a decline-able ReRollOffer (which the agent would decline,
        // skipping the re-roll die and desyncing from Java).
        use ffb_model::enums::{SkillId, TurnMode};
        use ffb_model::model::skill_def::SkillWithValue;
        use ffb_model::prompts::AgentPrompt;
        let mut game = make_game_with_thrower(3);
        game.team_home.player_mut("t1").unwrap()
            .starting_skills.push(SkillWithValue { skill_id: SkillId::Pass, value: None });
        game.acting_player.player_id = Some("t1".into());
        game.turn_mode = TurnMode::Regular;

        let mut step = make_step();
        step.roll = 1; // force the first pass roll to a natural 1 → FUMBLE
        let out = step.start(&mut game, &mut GameRng::new(0));

        assert!(!matches!(out.prompt, Some(AgentPrompt::ReRollOffer { .. })),
            "the free Pass skill re-roll must be auto-used, not offered as a decline-able ReRollOffer");
        assert!(game.player("t1").unwrap().used_skills.contains(&SkillId::Pass),
            "the Pass skill must be marked used after the auto re-roll");
        assert_eq!(step.re_rolled_action.as_deref(), Some("PASS"),
            "re_rolled_action records the single PASS re-roll (blocks a second offer)");
    }

    #[test]
    fn forced_accurate_roll_goes_to_end_label() {
        // PA=3, quick pass (dist_mod=0): minimum = max(2, 3+0+0) = 3.
        // Roll=6 is always accurate.
        let mut game = make_game_with_thrower(3);
        let mut step = make_step();
        step.roll = 6; // always accurate
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end"));
        // Must publish PassAccurate(true) so StepResolvePass routes the accurate catch
        // (CatchAccuratePass) instead of falling through to the missed/scatter branch.
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::PassAccurate(true))),
            "accurate ball pass publishes PassAccurate(true)");
    }

    #[test]
    fn forced_inaccurate_roll_goes_to_missed_label() {
        // PA=4, quick pass (dist_mod=0): effective = roll - 0 = 2;
        // 2 is neither ≥4 (accurate) nor ≤1 (fumble) → INACCURATE → missed pass
        // Use roll=2: effective = 2 - 0 = 2, not ≥ PA(4), not ≤ 1 → INACCURATE
        let mut game = make_game_with_thrower(4);
        let mut step = make_step();
        step.roll = 2; // INACCURATE for PA=4 quick pass
        let out = step.start(&mut game, &mut GameRng::new(0));
        // INACCURATE → goto missed label
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("missed"));
    }

    #[test]
    fn roll_4_accurate_goes_to_end_label() {
        let mut game = make_game();
        game.thrower_id = Some("t1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        let mut step = make_step();
        step.roll = 4; // legacy path: roll >= 4 used to be accurate stub
        // With real mechanics and no thrower player → fumble (pa=0)
        // But if we set pass_result directly, we can test the routing
        step.pass_result = Some(PassResult::ACCURATE);
        step.minimum_roll = 4; // skip re-roll
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end"));
    }

    #[test]
    fn roll_1_publishes_pass_fumble_true() {
        let mut game = make_game();
        game.thrower_id = Some("t1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        let mut step = make_step();
        step.pass_result = Some(PassResult::FUMBLE);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        let fumble = out.published.iter().find(|p| matches!(p, StepParameter::PassFumble(true)));
        assert!(fumble.is_some(), "expected PassFumble(true) published");
    }

    #[test]
    fn bomb_action_sets_bomb_moving() {
        let mut game = make_game_with_thrower(3);
        game.thrower_action = Some(PlayerAction::ThrowBomb);
        let mut step = make_step();
        step.roll = 6;
        step.start(&mut game, &mut GameRng::new(0));
        // For accurate bomb: bomb_coordinate set to pass_coordinate
        assert_eq!(game.field_model.bomb_coordinate, Some(FieldCoordinate::new(4, 7)));
    }

    #[test]
    fn accurate_pass_places_ball_at_pass_coordinate() {
        let mut game = make_game_with_thrower(3);
        game.pass_coordinate = Some(FieldCoordinate::new(4, 7));
        let mut step = make_step();
        step.pass_result = Some(PassResult::ACCURATE);
        step.minimum_roll = 3;
        step.roll = 6;
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.field_model.ball_coordinate, Some(FieldCoordinate::new(4, 7)));
    }

    #[test]
    fn fumble_places_ball_at_thrower_coordinate() {
        let mut game = make_game_with_thrower(3);
        game.field_model.set_player_coordinate("t1", FieldCoordinate::new(1, 7));
        let mut step = make_step();
        step.pass_result = Some(PassResult::FUMBLE);
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.field_model.ball_coordinate, Some(FieldCoordinate::new(1, 7)));
    }

    #[test]
    fn pa_3_quick_pass_roll_3_is_accurate() {
        // PA=3, quick pass (dist_mod=0): effective = roll - 0 = roll;
        // roll=3: effective=3, 3 >= PA(3) → ACCURATE
        let mut game = make_game_with_thrower(3);
        let mut step = make_step();
        step.roll = 3;
        let out = step.start(&mut game, &mut GameRng::new(0));
        // effective = 3 - 0 = 3 >= PA(3) → ACCURATE → goto end
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end"));
    }

    #[test]
    fn pa_3_quick_pass_roll_2_is_inaccurate() {
        // PA=3, quick pass (dist_mod=0): effective = 2 - 0 = 2;
        // 2 is not ≥ 3 (accurate) and not ≤ 1 (fumble) → INACCURATE
        let mut game = make_game_with_thrower(3);
        let mut step = make_step();
        step.roll = 2;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("missed"));
    }

    #[test]
    fn pass_result_param_published_on_accurate() {
        let mut game = make_game_with_thrower(3);
        let mut step = make_step();
        step.roll = 6; // accurate
        let out = step.start(&mut game, &mut GameRng::new(0));
        let param = out.published.iter().find(|p| {
            matches!(p, StepParameter::PassResultParam(ffb_model::enums::PassOutcome::Complete))
        });
        assert!(param.is_some(), "expected PassResultParam(Complete) published for accurate pass");
    }

    #[test]
    fn pass_result_param_published_on_fumble() {
        let mut game = make_game_with_thrower(3);
        let mut step = make_step();
        step.roll = 1; // natural 1 → fumble
        let out = step.start(&mut game, &mut GameRng::new(0));
        let param = out.published.iter().find(|p| {
            matches!(p, StepParameter::PassResultParam(ffb_model::enums::PassOutcome::Fumble))
        });
        assert!(param.is_some(), "expected PassResultParam(Fumble) published");
    }

    #[test]
    fn inaccurate_with_trr_offers_reroll_prompt() {
        // PA=3, roll=2 → INACCURATE; TRR available → should offer re-roll
        let mut game = make_game_with_thrower(3);
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        let mut step = make_step();
        step.roll = 2; // force INACCURATE
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue, "TRR available → offer re-roll");
        assert!(out.prompt.is_some());
        assert_eq!(step.re_rolled_action.as_deref(), Some("PASS"));
    }

    #[test]
    fn accurate_pass_emits_report_pass_roll() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game_with_thrower(3);
        let mut step = make_step();
        step.roll = 6; // accurate
        step.start(&mut game, &mut GameRng::new(0));
        assert!(
            game.report_list.has_report(ReportId::PASS_ROLL),
            "expected ReportPassRoll in report_list after an accurate pass"
        );
    }

    #[test]
    fn fumble_emits_report_pass_roll() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game_with_thrower(3);
        let mut step = make_step();
        step.roll = 1; // natural 1 → fumble
        step.start(&mut game, &mut GameRng::new(0));
        assert!(
            game.report_list.has_report(ReportId::PASS_ROLL),
            "expected ReportPassRoll in report_list after a fumble"
        );
    }

    #[test]
    fn decline_pass_reroll_goes_to_missed_label() {
        let mut game = make_game_with_thrower(3);
        game.home_playing = true;
        let mut step = make_step();
        step.roll = 2; // force INACCURATE result
        step.pass_result = Some(PassResult::INACCURATE);
        step.re_rolled_action = Some("PASS".into());
        step.re_roll_source = Some("TRR".into());
        // Decline
        let out = step.handle_command(
            &Action::UseReRoll { use_reroll: false },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("missed"));
    }
}
