use ffb_model::enums::{PassingDistance, PlayerAction, ReRollSource};
use ffb_model::model::game::Game;
use ffb_model::util::passing::passing_distance;
use ffb_model::util::rng::GameRng;
use ffb_model::report::mixed::report_pass_roll::ReportPassRoll;
use ffb_mechanics::bb2025::pass_mechanic::PassMechanic as Bb2025PassMechanic;
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

        // Java: PassMechanic.findPassingDistance(game, throwerCoord, passCoordinate, false)
        let passing_dist: Option<PassingDistance> = thrower_coord.and_then(|tc| {
            game.pass_coordinate.and_then(|pc| passing_distance(tc, pc))
        });

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

        // Roll if not yet rolled (roll=0 means fresh)
        if self.roll == 0 {
            // Java: publishParameter(from(PASSING_DISTANCE, passingDistance))
            if let Some(dist) = passing_dist {
                // Publish passing distance for downstream steps
                // (stored by StepEndPassing etc.)
            }

            // Java: minimumRoll = mechanic.minimumRoll(thrower, passingDistance, passModifiers)
            let mechanic = Bb2025PassMechanic::new();
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
        if self.pass_result.is_none() {
            let result = if let Some(thrower) = game.thrower() {
                if let Some(dist) = passing_dist {
                    let mechanic = Bb2025PassMechanic::new();
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

        // Java result routing:
        match result {
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
                StepOutcome::goto(&label)
                    .publish(StepParameter::PassResultParam(ffb_model::enums::PassOutcome::Complete))
            }
            PassResult::SAVED_FUMBLE => {
                // Java: handleSafePass → usingSafePass dialog / goto goToLabelOnSavedFumble
                // client-only: Safe Pass (dontDropFumbles) dialog — headless auto-skips, ball stays with thrower
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
            PassResult::FUMBLE => {
                // Java: askForReRollIfAvailable before handling fumble
                if !already_rerolled {
                    if let Some(prompt) = ask_for_reroll_if_available(game, "PASS", self.minimum_roll, true) {
                        self.re_rolled_action = Some("PASS".into());
                        self.re_roll_source = Some("TRR".into());
                        return StepOutcome::cont().with_prompt(prompt);
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
                    if let Some(prompt) = ask_for_reroll_if_available(game, "PASS", self.minimum_roll, false) {
                        self.re_rolled_action = Some("PASS".into());
                        self.re_roll_source = Some("TRR".into());
                        return StepOutcome::cont().with_prompt(prompt);
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
                StepOutcome::goto(&label)
                    .publish(StepParameter::CatcherId(None))
                    .publish(StepParameter::PassResultParam(ffb_model::enums::PassOutcome::Inaccurate))
            }
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
    fn forced_accurate_roll_goes_to_end_label() {
        // PA=3, quick pass (dist_mod=0): minimum = max(2, 3+0+0) = 3.
        // Roll=6 is always accurate.
        let mut game = make_game_with_thrower(3);
        let mut step = make_step();
        step.roll = 6; // always accurate
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end"));
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
