use ffb_model::enums::{ApothecaryMode, PlayerState, PS_FALLING};
use ffb_model::enums::PassResult as ModelPassResult;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::drop_player_context::SteadyFootingContext;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_injury::handle_injury_by_name;
use ffb_mechanics::pass_result::PassResult as MechanicPassResult;
use ffb_mechanics::modifiers::right_stuff_modifier_factory::RightStuffModifierFactory;
use ffb_mechanics::modifiers::right_stuff_context::RightStuffContext;

/// Rolls the Right Stuff landing check for a thrown/kicked player.
// Java executeStep logic:
//   thrownPlayer = game.getPlayerById(fThrownPlayerId)
//   playerCoordinate = fieldModel.getPlayerCoordinate(thrownPlayer)
//
//   // Skip if player landed OOB or in box (trap door)
//   if thrownPlayer != null &&
//      (playerState.base == FALLING || playerCoordinate.isBoxCoordinate()):
//     publish END_TURN=fThrownPlayerHasBall; publish THROWN_PLAYER_COORDINATE=null
//     NEXT_STEP; return
//
//   game.fieldModel.setPlayerState(thrownPlayer, oldPlayerState)
//   publish THROWN_PLAYER_STATE=oldPlayerState
//   if fThrownPlayerHasBall: fieldModel.setBallCoordinate(playerCoordinate)
//
//   fumbledKtm = (passResult==FUMBLE && kickedPlayer)
//   autoFailLanding = oldPlayerState != null && (isProneOrStunned || isDistracted)
//   doRoll = !fDropThrownPlayer && !fumbledKtm && !autoFailLanding
//
//   if doRoll && re-rolling RIGHT_STUFF:
//     if re_roll_source is None || !useReRoll -> doRoll=false
//
//   if doRoll:
//     modifiers = RightStuffModifierFactory.findModifiers(RightStuffContext(...))
//     minimumRoll = agilityMechanic.minimumRollRightStuff(thrownPlayer, modifiers)
//     roll = diceRoller.rollSkill()
//     successful = DiceInterpreter.isSkillRollSuccessful(roll, minimumRoll)
//     // FumbledPlayerLandsSafely override
//     if passResult==FUMBLE && thrower.hasSkillProperty(fumbledPlayerLandsSafely): successful=true
//
//     if successful:
//       spp.addLanding(playerResult)
//       if passResult==ACCURATE: spp.addCompletion(throwerResult)
//       if fThrownPlayerHasBall:
//         if checkTouchdown -> publish END_TURN=true
//       else:
//         if playerCoordinate==ballCoordinate -> publish CATCH_SCATTER_THROW_IN_MODE=SCATTER_BALL
//       publish THROWN_PLAYER_COORDINATE=null
//       GOTO goToOnSuccess
//     else:
//       if not re-rolling:
//         if usingSwoop: setReRoll(RIGHT_STUFF, SWOOP); REPEAT; return
//         else: ask for re-roll(RIGHT_STUFF, minimumRoll)
//
//   if !doRoll:
//     injuryResult = handleInjury(InjuryTypeTTMLanding / InjuryTypeFumbledKtmApoKo, ...)
//     publish STEADY_FOOTING_CONTEXT
//     NEXT_STEP
//
// Unported utilities:
//   TODO: RightStuffModifierFactory / RightStuffContext
//   TODO: AgilityMechanic.minimumRollRightStuff
//   TODO: DiceInterpreter.isSkillRollSuccessful
//   TODO: SppMechanic.addLanding / addCompletion
//   TODO: UtilServerSteps.checkTouchdown
//   TODO: UtilServerInjury.handleInjury (InjuryTypeTTMLanding, InjuryTypeFumbledKtmApoKo)
//   TODO: UtilServerReRoll.useReRoll / askForReRollIfAvailable
//   TODO: FieldCoordinate.isBoxCoordinate
//   TODO: SteadyFootingContext / DeferredCommand (RightStuffCommand)
//
// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.ttm.StepRightStuff`.
pub struct StepRightStuff {
    /// Java: fThrownPlayerHasBall (Boolean tristate — None until set)
    pub thrown_player_has_ball: Option<bool>,
    /// Java: fThrownPlayerId
    pub thrown_player_id: Option<String>,
    /// Java: fDropThrownPlayer
    pub drop_thrown_player: bool,
    /// Java: kickedPlayer (init param IS_KICKED_PLAYER)
    pub kicked_player: bool,
    /// Java: usingSwoop
    pub using_swoop: bool,
    /// Java: passResult
    pub pass_result: Option<ModelPassResult>,
    /// Java: goToOnSuccess (init param GOTO_LABEL_ON_SUCCESS)
    pub goto_on_success: String,
    /// Java: oldPlayerState
    pub old_player_state: Option<PlayerState>,
    // AbstractStepWithReRoll stubs
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
}

impl StepRightStuff {
    pub fn new(goto_on_success: String) -> Self {
        Self {
            thrown_player_has_ball: None,
            thrown_player_id: None,
            drop_thrown_player: false,
            kicked_player: false,
            using_swoop: false,
            pass_result: None,
            goto_on_success,
            old_player_state: None,
            re_rolled_action: None,
            re_roll_source: None,
        }
    }
}

impl Default for StepRightStuff {
    fn default() -> Self { Self::new(String::new()) }
}

impl Step for StepRightStuff {
    fn id(&self) -> StepId { StepId::RightStuff }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_USE_SKILL with ttmScattersInSingleDirection -> set re-roll to SWOOP -> executeStep
        // Java: CLIENT_USE_SKILL with ttmScattersInSingleDirection skill:
        //   setReRolledAction(RIGHT_STUFF); setReRollSource(SWOOP)
        if let Action::UseSkill { skill_id, use_skill: true } = action {
            // Java: verify skill has ttmScattersInSingleDirection property
            if skill_id.properties().contains(&NamedProperties::TTM_SCATTERS_IN_SINGLE_DIRECTION) {
                self.re_rolled_action = Some("RIGHT_STUFF".into());
                self.re_roll_source = Some("SWOOP".into());
            }
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::ThrownPlayerHasBall(v) => { self.thrown_player_has_ball = Some(*v); true }
            StepParameter::ThrownPlayerId(v) => { self.thrown_player_id = v.clone(); true }
            StepParameter::DropThrownPlayer(v) => { self.drop_thrown_player = *v; true }
            StepParameter::PassResultParam(v) => { self.pass_result = Some(*v); true }
            StepParameter::OldDefenderState(v) => { self.old_player_state = Some(*v); true }
            StepParameter::UsingSwoop(v) => { self.using_swoop = *v; true }
            _ => false,
        }
    }
}

impl StepRightStuff {
    fn execute_step(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: thrownPlayer = game.getPlayerById(fThrownPlayerId)
        let player_id = match self.thrown_player_id.as_deref() {
            Some(id) if game.player(id).is_some() => id.to_string(),
            _ => return StepOutcome::next(),
        };

        let player_coord = game.field_model.player_coordinate(&player_id);
        let player_state = game.field_model.player_state(&player_id).unwrap_or_default();

        // Java: if state.base==FALLING || coord.isBoxCoordinate() → publish + NEXT_STEP
        let is_falling = player_state.base() == PS_FALLING;
        let is_box = player_coord.map(|c| c.is_box_coordinate()).unwrap_or(false);
        if is_falling || is_box {
            let end_turn = self.thrown_player_has_ball.unwrap_or(false);
            return StepOutcome::next()
                .publish(StepParameter::EndTurn(end_turn))
                .publish(StepParameter::ThrownPlayerCoordinate(None));
        }

        // Java: setPlayerState(thrownPlayer, oldPlayerState)
        let old_state = self.old_player_state.unwrap_or_default();
        game.field_model.set_player_state(&player_id, old_state);

        // Java: if fThrownPlayerHasBall: setBallCoordinate(playerCoordinate)
        if self.thrown_player_has_ball == Some(true) {
            game.field_model.ball_coordinate = player_coord;
        }

        // Java: fumbledKtm = (passResult==FUMBLE && kickedPlayer)
        let fumbled_ktm = self.pass_result == Some(ModelPassResult::Fumble) && self.kicked_player;
        // Java: autoFailLanding = oldPlayerState.isProneOrStunned || isDistracted
        let auto_fail = old_state.is_prone_or_stunned() || old_state.is_distracted();
        // Java: doRoll = !dropThrownPlayer && !fumbledKtm && !autoFailLanding
        let do_roll = !self.drop_thrown_player && !fumbled_ktm && !auto_fail;

        if do_roll {
            let roll = rng.d6();
            let minimum_roll = if let Some(player) = game.player(&player_id) {
                let factory = RightStuffModifierFactory::for_rules(game.rules);
                let mechanic_pass_result = self.pass_result.map(|r| match r {
                    ModelPassResult::Fumble | ModelPassResult::MissedCatch => MechanicPassResult::FUMBLE,
                    ModelPassResult::Inaccurate => MechanicPassResult::INACCURATE,
                    ModelPassResult::WildlyInaccurate => MechanicPassResult::WILDLY_INACCURATE,
                    _ => MechanicPassResult::ACCURATE,
                });
                let ctx = RightStuffContext::new_full(game, player, mechanic_pass_result, None);
                let mods = factory.find_applicable(&ctx);
                RightStuffModifierFactory::minimum_roll(player.agility as i32, &mods)
            } else {
                4
            };
            let successful = DiceInterpreter::is_skill_roll_successful(roll, minimum_roll);
            if successful {
                // TODO: SppMechanic.addLanding; checkTouchdown
                return StepOutcome::goto(&self.goto_on_success)
                    .publish(StepParameter::ThrownPlayerCoordinate(None));
            }
            // TODO: re-roll handling (useReRoll / askForReRoll / usingSwoop REPEAT path)
        }

        // Java: !doRoll or failed roll → handleInjury; NEXT_STEP
        // Java: injuryType = fumbledKtm ? new InjuryTypeFumbledKtmApoKo() : new InjuryTypeTTMLanding()
        let injury_type = if fumbled_ktm { "InjuryTypeFumbledKtmApoKo" } else { "InjuryTypeTTMLanding" };
        let player_coord = game.field_model.player_coordinate(&player_id)
            .unwrap_or(ffb_model::types::FieldCoordinate::new(0, 0));
        let injury_result = handle_injury_by_name(
            game, rng, injury_type,
            None, &player_id,
            player_coord, None, None, ApothecaryMode::Defender,
        );
        // Java: new SteadyFootingContext(injuryResult, deferredCommands) — InjuryResult variant
        let ctx = SteadyFootingContext::from_injury_result(injury_result);
        StepOutcome::next().publish(StepParameter::SteadyFootingContext(Box::new(ctx)))
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
        let mut step = StepRightStuff::new("success".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_thrown_player_has_ball() {
        let mut step = StepRightStuff::default();
        assert!(step.set_parameter(&StepParameter::ThrownPlayerHasBall(true)));
        assert_eq!(step.thrown_player_has_ball, Some(true));
    }

    #[test]
    fn set_drop_thrown_player() {
        let mut step = StepRightStuff::default();
        assert!(step.set_parameter(&StepParameter::DropThrownPlayer(true)));
        assert!(step.drop_thrown_player);
    }

    #[test]
    fn set_using_swoop() {
        let mut step = StepRightStuff::default();
        assert!(step.set_parameter(&StepParameter::UsingSwoop(true)));
        assert!(step.using_swoop);
    }

    #[test]
    fn use_skill_command_sets_reroll_fields() {
        let mut game = make_game();
        let mut step = StepRightStuff::new("ok".into());
        use ffb_mechanics::skills::SkillId;
        step.handle_command(&Action::UseSkill { skill_id: SkillId::Swoop, use_skill: true }, &mut game, &mut GameRng::new(0));
        assert_eq!(step.re_rolled_action.as_deref(), Some("RIGHT_STUFF"));
        assert_eq!(step.re_roll_source.as_deref(), Some("SWOOP"));
    }

    #[test]
    fn falling_player_publishes_end_turn_and_returns_next_step() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PS_FALLING, PS_STANDING, PlayerState};
        use ffb_model::types::FieldCoordinate;
        let mut game = make_game();
        let mut p = Player::default();
        p.id = "p1".into();
        game.team_home.players.push(p);
        let coord = FieldCoordinate::new(5, 5);
        game.field_model.set_player_coordinate("p1", coord);
        game.field_model.set_player_state("p1", PlayerState::new(PS_FALLING));
        let mut step = StepRightStuff::new("success".into());
        step.thrown_player_id = Some("p1".into());
        step.thrown_player_has_ball = Some(true);
        step.old_player_state = Some(PlayerState::new(PS_STANDING));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::ThrownPlayerCoordinate(None))));
    }

    #[test]
    fn missing_player_returns_next_step() {
        let mut game = make_game();
        let mut step = StepRightStuff::new("success".into());
        step.thrown_player_id = Some("nonexistent".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }
}
