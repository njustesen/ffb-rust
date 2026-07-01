use ffb_model::enums::{PassingDistance, PlayerAction, ReRollSource};
use ffb_model::model::game::Game;
use ffb_model::util::passing::passing_distance;
use ffb_model::util::rng::GameRng;
use ffb_mechanics::bb2020::pass_mechanic::PassMechanic as Bb2020PassMechanic;
use ffb_mechanics::pass_mechanic::PassMechanic;
use ffb_mechanics::pass_result::PassResult;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{CatchScatterThrowInMode, StepId, StepParameter};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.pass.StepPass.
///
/// Main pass step: sets ball/bomb moving, computes passing distance, rolls PA,
/// handles modifying-skill dialog (canAddStrengthToPass), re-roll prompts,
/// and routes to accurate/fumble/missed/saved-fumble labels.
///
/// Needs init params: `GotoLabelOnEnd`, `GotoLabelOnMissedPass`, `GotoLabelOnSavedFumble`.
/// Expects stepParameter `CatcherId` from a preceding step.
/// Publishes: `PassingDistance`, `PassFumble`, `DontDropFumble`, `CatcherId`,
///            `CatchScatterThrowInMode`, `PassResultParam`.
///
/// DEFERRED(pass-modifiers): PassModifierFactory.findModifiers(PassContext) — tacklezone and
///   disturbing-presence modifier counting requires UtilServerGame.passModifiers() (counts adj.
///   tacklezone squares from game.field_model) which is not yet translated.
/// DEFERRED(dialog): modifying-skill dialog (DialogSkillUseParameter for canAddStrengthToPass) and
///   pass-skill re-roll dialog — waiting for dialog infrastructure.
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
        // Java: otherwise → handleSkillCommand(commandUseSkill, passState) [pass reroll dialog]
        match action {
            Action::UseSkill { skill_id, use_skill } => {
                use ffb_model::model::property::named_properties::NamedProperties;
                if skill_id.properties().contains(&NamedProperties::CAN_ADD_STRENGTH_TO_PASS) {
                    self.using_modifying_skill = Some(*use_skill);
                } else {
                    // pass skill re-roll dialog answer
                    self.using_modifying_skill = Some(*use_skill);
                }
            }
            Action::UseReRoll { use_reroll: false } => {
                // Player declined re-roll
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
            StepParameter::GotoLabelOnFailure(v) => { self.goto_label_on_missed_pass = v.clone(); true }
            StepParameter::GotoLabelOnMissedPass(v) => { self.goto_label_on_missed_pass = v.clone(); true }
            StepParameter::GotoLabelOnSuccess(v) => { self.goto_label_on_saved_fumble = v.clone(); true }
            StepParameter::GotoLabelOnSavedFumble(v) => { self.goto_label_on_saved_fumble = v.clone(); true }
            StepParameter::UsingModifyingSkill(v) => { self.using_modifying_skill = Some(*v); true }
            _ => false,
        }
    }
}

impl StepPass {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java guard: if (game.getThrower() == null || game.getThrowerAction() == null) return
        if game.thrower_id.is_none() || game.thrower_action.is_none() {
            return StepOutcome::goto(&self.goto_label_on_end);
        }

        let is_bomb = matches!(
            game.thrower_action,
            Some(PlayerAction::ThrowBomb) | Some(PlayerAction::HailMaryBomb)
        );

        // Java: if (PASS == reRolledAction) ...
        if self.re_rolled_action.as_deref() == Some("PASS") {
            if self.using_modifying_skill == Some(false) || self.using_modifying_skill.is_none() {
                let thrower_id = game.thrower_id.clone().unwrap_or_default();
                if let Some(ref source_name) = self.re_roll_source.clone() {
                    let source = ReRollSource::new(source_name.as_str());
                    if !use_reroll(game, &source, &thrower_id) {
                        // token exhausted → handle failed pass with stored result
                        return self.handle_failed_pass(game);
                    }
                    // Re-roll consumed — clear stored result so we re-roll below
                    self.roll = 0;
                    self.pass_result = None;
                } else {
                    // source == None (player declined) → handle failed pass
                    return self.handle_failed_pass(game);
                }
            }
        }

        // Java: if THROW_BOMB → setBombMoving(true), set originalBombardier if not set
        // Java: else → setBallMoving(true)
        if is_bomb {
            game.field_model.bomb_moving = true;
        } else {
            game.field_model.ball_moving = true;
        }

        let thrower_id = game.thrower_id.clone().unwrap();
        let thrower_coord = game.field_model.player_coordinate(&thrower_id);

        // Java: PassMechanic.findPassingDistance(game, throwerCoord, passCoordinate, false)
        let passing_dist: Option<PassingDistance> = thrower_coord.and_then(|tc| {
            game.pass_coordinate.and_then(|pc| passing_distance(tc, pc))
        });

        // Java: PassModifierFactory.findModifiers(new PassContext(game, thrower, passingDistance, false))
        // DEFERRED(pass-modifiers): PassContext tacklezone + disturbing-presence counting requires
        //   UtilServerGame.passModifiers() (counts adj. tacklezone squares) — not yet translated.
        //   BB2020 also has Very Sunny weather modifier and skill-based modifiers; currently empty.
        let pass_modifiers: Vec<ffb_mechanics::modifiers::PassModifier> = Vec::new();

        // Java: usingModifyingSkill != null && usingModifyingSkill → use stat-based modifier path
        // Java: else → standard roll path (state.setThrowerCoordinate, publishParameter PASSING_DISTANCE)
        if let Some(true) = self.using_modifying_skill {
            // Java: if (minimumRoll == 0) minimumRoll = mechanic.minimumRoll(...)
            if self.minimum_roll == 0 {
                let mechanic = Bb2020PassMechanic::new();
                if let Some(thrower) = game.thrower() {
                    let minimum = passing_dist.and_then(|dist| {
                        mechanic.minimum_roll_simple(thrower, dist, &pass_modifiers)
                    });
                    self.minimum_roll = minimum.unwrap_or(0);
                }
            }
            // Java: if (roll == 0) roll = minimumRoll > 0 ? rollSkill() : 0
            if self.roll == 0 {
                self.roll = if self.minimum_roll > 0 { rng.d6() } else { 0 };
            }
            // Java: state.setResult(mechanic.evaluatePass(thrower, roll, dist, modifiers, isBomb, statBasedModifier))
            if self.pass_result.is_none() {
                let result = if let Some(thrower) = game.thrower() {
                    if let Some(dist) = passing_dist {
                        let mechanic = Bb2020PassMechanic::new();
                        mechanic.evaluate_pass_simple(thrower, self.roll, dist, &pass_modifiers, is_bomb)
                    } else {
                        PassResult::FUMBLE
                    }
                } else {
                    PassResult::FUMBLE
                };
                self.pass_result = Some(result);
            }
        } else {
            // Standard path
            // Java: state.setThrowerCoordinate(throwerCoordinate)
            // Java: publishParameter(from(PASSING_DISTANCE, passingDistance))
            // Java: minimumRoll = mechanic.minimumRoll(thrower, dist, modifiers)
            // Java: roll = minimumRollO.isPresent() ? rollSkill() : 0
            if self.roll == 0 {
                let mechanic = Bb2020PassMechanic::new();
                if let Some(thrower) = game.thrower() {
                    let minimum = passing_dist.and_then(|dist| {
                        mechanic.minimum_roll_simple(thrower, dist, &pass_modifiers)
                    });
                    self.minimum_roll = minimum.unwrap_or(0);
                }
                self.roll = if self.minimum_roll > 0 { rng.d6() } else { 0 };
            }
            // Java: state.setResult(mechanic.evaluatePass(thrower, roll, dist, modifiers, isBomb))
            if self.pass_result.is_none() {
                let result = if let Some(thrower) = game.thrower() {
                    if let Some(dist) = passing_dist {
                        let mechanic = Bb2020PassMechanic::new();
                        mechanic.evaluate_pass_simple(thrower, self.roll, dist, &pass_modifiers, is_bomb)
                    } else {
                        PassResult::FUMBLE
                    }
                } else {
                    PassResult::FUMBLE
                };
                self.pass_result = Some(result);
            }
        }

        let result = self.pass_result.unwrap();
        let already_rerolled = self.re_rolled_action.is_some();

        // Java: if (PassResult.FUMBLE == state.getResult()) → publishParameter(DONT_DROP_FUMBLE, false)
        // Java: else if (PassResult.SAVED_FUMBLE == state.getResult()) → publishParameter(DONT_DROP_FUMBLE, true)

        match result {
            PassResult::ACCURATE => {
                // Java: getResult().setNextAction(StepAction.GOTO_LABEL, goToLabelOnEnd)
                // Java: if THROW_BOMB → setBombCoordinate(passCoord) else → setBallCoordinate(passCoord)
                if let Some(pass_coord) = game.pass_coordinate {
                    if is_bomb {
                        game.field_model.bomb_coordinate = Some(pass_coord);
                    } else {
                        game.field_model.ball_coordinate = Some(pass_coord);
                    }
                }
                let label = self.goto_label_on_end.clone();
                StepOutcome::goto(&label)
                    .publish(StepParameter::PassResultParam(ffb_model::enums::PassResult::Complete))
            }
            PassResult::SAVED_FUMBLE => {
                // Java: handleFailedPass → SAVED_FUMBLE branch
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
                    .publish(StepParameter::PassResultParam(ffb_model::enums::PassResult::Fumble))
            }
            PassResult::FUMBLE => {
                // Java: mechanic.eligibleToReRoll → askForReRollIfAvailable
                if !already_rerolled {
                    if let Some(prompt) = ask_for_reroll_if_available(game, "PASS", self.minimum_roll, true) {
                        self.re_rolled_action = Some("PASS".into());
                        self.re_roll_source = Some("TRR".into());
                        return StepOutcome::cont().with_prompt(prompt);
                    }
                }
                self.handle_failed_pass_fumble(game, thrower_coord, is_bomb)
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
                self.handle_failed_pass_missed(game, is_bomb)
            }
        }
    }

    /// Java: handleFailedPass() — dispatch based on stored pass_result
    fn handle_failed_pass(&mut self, game: &mut Game) -> StepOutcome {
        let result = self.pass_result.unwrap_or(PassResult::FUMBLE);
        let is_bomb = matches!(
            game.thrower_action,
            Some(PlayerAction::ThrowBomb) | Some(PlayerAction::HailMaryBomb)
        );
        let thrower_id = game.thrower_id.clone();
        let thrower_coord = thrower_id.as_deref().and_then(|id| game.field_model.player_coordinate(id));

        match result {
            PassResult::SAVED_FUMBLE => {
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
            }
            PassResult::FUMBLE => {
                self.handle_failed_pass_fumble(game, thrower_coord, is_bomb)
            }
            _ => {
                self.handle_failed_pass_missed(game, is_bomb)
            }
        }
    }

    /// Java: handleFailedPass() FUMBLE branch
    fn handle_failed_pass_fumble(&self, game: &mut Game, thrower_coord: Option<ffb_model::types::FieldCoordinate>, is_bomb: bool) -> StepOutcome {
        // Java: if THROW_BOMB → setBombCoordinate(throwerCoordinate)
        // Java: else → setBallCoordinate(throwerCoordinate); publishParameter(CATCH_SCATTER_THROW_IN_MODE, SCATTER_BALL)
        // Java: publishParameter(CATCHER_ID, null); setNextAction(NEXT_STEP)
        if is_bomb {
            if let Some(tc) = thrower_coord {
                game.field_model.bomb_coordinate = Some(tc);
            }
        } else {
            if let Some(tc) = thrower_coord {
                game.field_model.ball_coordinate = Some(tc);
            }
        }
        StepOutcome::next()
            .publish(StepParameter::PassFumble(true))
            .publish(StepParameter::DontDropFumble(false))
            .publish(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ScatterBall))
            .publish(StepParameter::CatcherId(None))
            .publish(StepParameter::PassResultParam(ffb_model::enums::PassResult::Fumble))
    }

    /// Java: handleFailedPass() missed/inaccurate branch
    fn handle_failed_pass_missed(&self, game: &mut Game, is_bomb: bool) -> StepOutcome {
        // Java: if THROW_BOMB → setBombCoordinate(passCoordinate)
        // Java: else → setBallCoordinate(passCoordinate)
        // Java: publishParameter(CATCHER_ID, null); setNextAction(GOTO_LABEL, goToLabelOnMissedPass)
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
            .publish(StepParameter::PassResultParam(ffb_model::enums::PassResult::Inaccurate))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{Rules, PlayerAction};
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2020)
    }

    fn make_step() -> StepPass {
        StepPass::new("end".into(), "missed".into(), "saved_fumble".into())
    }

    fn make_game_with_thrower(pa: i32) -> Game {
        let mut home = test_team("home", 0);
        let away = test_team("away", 0);
        let mut thrower = ffb_model::model::player::Player::default();
        thrower.id = "t1".into();
        thrower.passing = pa;
        home.players.push(thrower);
        let mut game = Game::new(home, away, Rules::Bb2020);
        game.thrower_id = Some("t1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        game.field_model.set_player_coordinate("t1", FieldCoordinate::new(1, 7));
        game.pass_coordinate = Some(FieldCoordinate::new(4, 7));
        game
    }

    #[test]
    fn no_thrower_goes_to_end_label() {
        let mut game = make_game();
        let mut step = make_step();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end"));
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

    #[test]
    fn fumble_pa_zero_publishes_pass_fumble_true() {
        let mut game = make_game_with_thrower(0);
        let mut step = make_step();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        let fumble = out.published.iter().find(|p| matches!(p, StepParameter::PassFumble(true)));
        assert!(fumble.is_some(), "expected PassFumble(true) published for PA=0");
    }

    #[test]
    fn forced_accurate_roll_goes_to_end_label() {
        let mut game = make_game_with_thrower(3);
        let mut step = make_step();
        step.roll = 6;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end"));
    }

    #[test]
    fn forced_inaccurate_roll_goes_to_missed_label() {
        let mut game = make_game_with_thrower(4);
        let mut step = make_step();
        step.roll = 2;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("missed"));
    }

    #[test]
    fn bomb_action_sets_bomb_moving() {
        let mut game = make_game_with_thrower(3);
        game.thrower_action = Some(PlayerAction::ThrowBomb);
        let mut step = make_step();
        step.roll = 6;
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.field_model.bomb_coordinate, Some(FieldCoordinate::new(4, 7)));
    }

    #[test]
    fn accurate_pass_places_ball_at_pass_coordinate() {
        let mut game = make_game_with_thrower(3);
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
        let mut step = make_step();
        step.pass_result = Some(PassResult::FUMBLE);
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.field_model.ball_coordinate, Some(FieldCoordinate::new(1, 7)));
    }

    #[test]
    fn wildly_inaccurate_goes_to_missed_label() {
        // BB2020: roll=1 when PA>1 gives WILDLY_INACCURATE (roll_mod <= 1)
        let mut game = make_game_with_thrower(4);
        let mut step = make_step();
        // Force WILDLY_INACCURATE via pre-set result
        step.pass_result = Some(PassResult::WILDLY_INACCURATE);
        step.minimum_roll = 3;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("missed"));
    }
}
