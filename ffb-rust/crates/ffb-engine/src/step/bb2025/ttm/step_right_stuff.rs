use ffb_model::enums::{ApothecaryMode, PlayerState, PS_FALLING, SkillId};
use ffb_model::enums::PassOutcome as ModelPassResult;
use ffb_model::enums::ReRollSource;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::re_rolled_action::ReRolledAction;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::report::report_right_stuff_roll::ReportRightStuffRoll;
use ffb_model::report::report_skill_use::ReportSkillUse;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::drop_player_context::SteadyFootingContext;
use crate::mechanic::spp_calc::SppCalc;
use crate::step::abstract_step_with_re_roll::{ReRollState, find_skill_reroll_source};
use crate::step::framework::{Step, StepOutcome, CatchScatterThrowInMode};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_injury::handle_injury_by_name;
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use crate::step::util_server_steps::check_touchdown;
use ffb_mechanics::pass_result::PassResult as MechanicPassResult;
use ffb_mechanics::modifiers::right_stuff_modifier_factory::RightStuffModifierFactory;
use ffb_mechanics::modifiers::right_stuff_context::RightStuffContext;

/// Rolls the Right Stuff landing check for a thrown/kicked player.
// Java executeStep logic (StepRightStuff bb2020, used unchanged by bb2025):
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
//   doRoll = !fDropThrownPlayer && !fumbledKtm
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
//       spp.addLanding(playerResult)                        // BB2025: landing = 1 SPP
//       if passResult==ACCURATE && !kickedPlayer:
//         spp.addCompletion(throwerResult)
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
//     injuryResult = handleInjury(InjuryTypeTTMLanding / InjuryTypeFumbledKtm, ...)
//     publish INJURY_RESULT, dropPlayer params
//     publish THROWN_PLAYER_COORDINATE=null
//     NEXT_STEP
//
// Mirrors Java `com.fumbbl.ffb.server.step.bb2020.ttm.StepRightStuff` (used for BB2025 too).
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
    /// AbstractStepWithReRoll state
    pub re_roll_state: ReRollState,
    /// Cached dice roll (0 = not yet rolled — cleared to 0 on re-roll)
    pub roll: i32,
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
            re_roll_state: ReRollState::new(),
            roll: 0,
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
        match action {
            Action::UseSkill { skill_id, use_skill: true } => {
                // Java: CLIENT_USE_SKILL with ttmScattersInSingleDirection skill:
                //   setReRolledAction(RIGHT_STUFF); setReRollSource(SWOOP)
                if skill_id.properties().contains(&NamedProperties::TTM_SCATTERS_IN_SINGLE_DIRECTION) {
                    self.re_roll_state.set_re_rolled_action(ReRolledAction::new("RIGHT_STUFF"));
                    self.re_roll_state.set_re_roll_source(ReRollSource::new("SWOOP"));
                }
            }
            Action::UseReRoll { use_reroll: true } => {
                // Java: super.handleCommand(pReceivedCommand) → AbstractStepWithReRoll sets source+action
                // re_roll_source was already set in execute_step before asking
            }
            Action::UseReRoll { use_reroll: false } => {
                // Java: declined re-roll → clear source so useReRoll returns false
                self.re_roll_state.re_roll_source = None;
            }
            _ => {}
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
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
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
        // Java: doRoll = !dropThrownPlayer && !fumbledKtm
        let mut do_roll = !self.drop_thrown_player && !fumbled_ktm;

        // Java: if (doRoll && reRolledAction == RIGHT_STUFF) {
        //         if (source == null || !useReRoll) doRoll = false; }
        let already_rerolled = self.re_roll_state.re_rolled_action
            .as_ref()
            .map(|a| a.name == "RIGHT_STUFF")
            .unwrap_or(false);
        if do_roll && already_rerolled {
            let consumed = self.re_roll_state.re_roll_source
                .as_ref()
                .map(|s| use_reroll(game, s, &player_id))
                .unwrap_or(false);
            if !consumed {
                do_roll = false;
            }
        }

        if do_roll {
            let minimum_roll = if let Some(player) = game.player(&player_id) {
                let factory = RightStuffModifierFactory::for_rules(game.rules);
                let mechanic_pass_result = self.pass_result.map(|r| match r {
                    ModelPassResult::Fumble | ModelPassResult::MissedCatch => MechanicPassResult::FUMBLE,
                    ModelPassResult::Inaccurate => MechanicPassResult::INACCURATE,
                    ModelPassResult::WildlyInaccurate => MechanicPassResult::WILDLY_INACCURATE,
                    ModelPassResult::Complete | ModelPassResult::Caught => MechanicPassResult::ACCURATE,
                });
                let ctx = RightStuffContext::new_full(game, player, mechanic_pass_result, None);
                let mods = factory.find_applicable(&ctx);
                RightStuffModifierFactory::minimum_roll(player.agility_with_modifiers(), &mods)
            } else {
                4
            };

            if self.roll == 0 {
                self.roll = rng.d6();
            }
            let mut successful = DiceInterpreter::is_skill_roll_successful(self.roll, minimum_roll);

            // Java: if passResult==FUMBLE && thrower.hasSkillProperty(fumbledPlayerLandsSafely): successful=true
            let thrower_lands_safely = if self.pass_result == Some(ModelPassResult::Fumble) {
                game.thrower_id.as_deref()
                    .and_then(|id| game.player(id))
                    .map(|p| p.has_skill_property(NamedProperties::FUMBLED_PLAYER_LANDS_SAFELY))
                    .unwrap_or(false)
            } else {
                false
            };
            if thrower_lands_safely {
                // Java: addReport(new ReportSkillUse(game.getThrowerId(), thrower.getSkillWithProperty(fumbledPlayerLandsSafely), true, FUMBLED_PLAYER_LANDS_SAFELY));
                game.report_list.add(ReportSkillUse::new(
                    game.thrower_id.clone(),
                    SkillId::Reliable,
                    true,
                    SkillUse::FUMBLED_PLAYER_LANDS_SAFELY,
                ));
                successful = true;
            } else {
                // Java: getResult().addReport(new ReportRightStuffRoll(fThrownPlayerId, successful, roll, minimumRoll, reRolled, modifiers));
                let re_rolled = already_rerolled && self.re_roll_state.re_roll_source.is_some();
                game.report_list.add(ReportRightStuffRoll::new(
                    self.thrown_player_id.clone(),
                    successful,
                    self.roll,
                    minimum_roll,
                    re_rolled,
                    vec![],
                ));
            }

            if successful {
                // Java: spp.addLanding(playerResult)
                // BB2025: landing_spp = 1
                let is_home_player = game.team_home.player(&player_id).is_some();
                let team_result = if is_home_player {
                    &mut game.game_result.home
                } else {
                    &mut game.game_result.away
                };
                let pr = team_result.player_results.entry(player_id.clone()).or_default();
                pr.landings += 1;
                pr.spp_gained += SppCalc::landing_spp(game.rules);

                // Java: if passResult==ACCURATE && !kickedPlayer: spp.addCompletion(throwerResult)
                if self.pass_result == Some(ModelPassResult::Complete) && !self.kicked_player {
                    if let Some(thrower_id) = game.thrower_id.clone() {
                        let is_home_thrower = game.team_home.player(&thrower_id).is_some();
                        let thrower_team_id = if is_home_thrower {
                            game.team_home.id.clone()
                        } else {
                            game.team_away.id.clone()
                        };
                        let has_prayer_bonus = game.prayer_state
                            .get_additional_completion_spp_teams()
                            .contains(&thrower_team_id);
                        let team_result = if is_home_thrower {
                            &mut game.game_result.home
                        } else {
                            &mut game.game_result.away
                        };
                        let tpr = team_result.player_results.entry(thrower_id.clone()).or_default();
                        tpr.completions += 1;
                        tpr.spp_gained += SppCalc::completion_spp();
                        if has_prayer_bonus {
                            tpr.completions_with_additional_spp += 1;
                        }
                    }
                }

                let has_ball = self.thrown_player_has_ball.unwrap_or(false);
                let player_coord_after = game.field_model.player_coordinate(&player_id);
                let mut out = StepOutcome::goto(&self.goto_on_success)
                    .publish(StepParameter::ThrownPlayerCoordinate(None));
                if has_ball {
                    if check_touchdown(game) {
                        out = out.publish(StepParameter::EndTurn(true));
                    }
                } else {
                    let ball_coord = game.field_model.ball_coordinate;
                    if player_coord_after.is_some() && player_coord_after == ball_coord {
                        game.field_model.ball_moving = true;
                        out = out.publish(StepParameter::CatchScatterThrowInMode(
                            CatchScatterThrowInMode::ScatterBall));
                    }
                }
                return out;
            }

            // Java: failure branch
            // if (getReRolledAction() != ReRolledActions.RIGHT_STUFF):
            //   setReRolledAction(RIGHT_STUFF)
            //   doRoll = UtilServerReRoll.askForReRollIfAvailable(...)
            if !already_rerolled {
                self.re_roll_state.set_re_rolled_action(ReRolledAction::new("RIGHT_STUFF"));

                // Java: usingSwoop path → setReRoll(RIGHT_STUFF, SWOOP); pushCurrentStep; NEXT_STEP
                // In Rust: set re-roll state to SWOOP and Repeat (re-enter executeStep)
                if self.using_swoop {
                    self.re_roll_state.set_re_roll_source(ReRollSource::new("SWOOP"));
                    self.roll = 0;
                    return self.execute_step(game, rng);
                }

                // Skill re-roll (auto-use)
                let skill_source = find_skill_reroll_source(game, "RIGHT_STUFF");
                if let Some(source) = skill_source {
                    use_reroll(game, &source, &player_id);
                    self.re_roll_state.re_roll_source = Some(source);
                    self.roll = 0;
                    return self.execute_step(game, rng);
                }

                // TRR offer
                if let Some(prompt) = ask_for_reroll_if_available(game, "RIGHT_STUFF", minimum_roll, false) {
                    self.re_roll_state.re_roll_source = Some(ReRollSource::new("TRR"));
                    self.roll = 0;
                    return StepOutcome::cont().with_prompt(prompt);
                }
            }
            // Re-roll exhausted or declined — fall through to injury
        }

        // Java: !doRoll → handleInjury; publish INJURY_RESULT; dropPlayer params; NEXT_STEP
        // Java: injuryType = fumbledKtm ? new InjuryTypeFumbledKtm() : new InjuryTypeTTMLanding()
        let injury_type = if fumbled_ktm { "InjuryTypeFumbledKtm" } else { "InjuryTypeTTMLanding" };
        let coord = game.field_model.player_coordinate(&player_id)
            .unwrap_or(ffb_model::types::FieldCoordinate::new(0, 0));
        // Java: actingPlayer.getPlayer() is the attacker (thrower); thrownPlayer is defender-role
        let attacker_id: Option<String> = game.acting_player.player_id.clone();
        let injury_result = handle_injury_by_name(
            game, rng, injury_type,
            attacker_id.as_deref(), &player_id,
            coord, None, None, ApothecaryMode::ThrownPlayer,
        );
        let ctx = SteadyFootingContext::from_injury_result(injury_result);
        StepOutcome::next()
            .publish(StepParameter::SteadyFootingContext(Box::new(ctx)))
            .publish(StepParameter::ThrownPlayerCoordinate(None))
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
        assert_eq!(step.re_roll_state.re_rolled_action.as_ref().map(|a| a.name.as_str()), Some("RIGHT_STUFF"));
        assert_eq!(step.re_roll_state.re_roll_source.as_ref().map(|s| s.name.as_str()), Some("SWOOP"));
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

    #[test]
    fn use_reroll_decline_clears_source() {
        let mut game = make_game();
        let mut step = StepRightStuff::new("ok".into());
        step.re_roll_state.set_re_roll_source(ReRollSource::new("TRR"));
        step.handle_command(&Action::UseReRoll { use_reroll: false }, &mut game, &mut GameRng::new(0));
        assert!(step.re_roll_state.re_roll_source.is_none());
    }

    // ── report wiring ─────────────────────────────────────────────────────────

    #[test]
    fn right_stuff_roll_report_added_on_normal_roll() {
        use ffb_model::report::report_id::ReportId;
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PS_STANDING, PlayerState};
        use ffb_model::types::FieldCoordinate;
        let mut game = make_game();
        let mut p = Player::default();
        p.id = "p1".into();
        p.agility = 3;
        game.team_home.players.push(p);
        let coord = FieldCoordinate::new(5, 5);
        game.field_model.set_player_coordinate("p1", coord);
        game.field_model.set_player_state("p1", PlayerState::new(PS_STANDING));
        game.home_playing = true;

        let mut step = StepRightStuff::new("success".into());
        step.thrown_player_id = Some("p1".into());
        step.thrown_player_has_ball = Some(false);
        step.old_player_state = Some(PlayerState::new(PS_STANDING));
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::RIGHT_STUFF_ROLL)
            || game.report_list.has_report(ReportId::SKILL_USE),
            "either RIGHT_STUFF_ROLL or SKILL_USE report must be added");
    }

    #[test]
    fn right_stuff_roll_report_present_when_roll_happens() {
        use ffb_model::report::report_id::ReportId;
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PS_STANDING, PlayerState};
        use ffb_model::types::FieldCoordinate;
        // Seed that produces roll=6 (success)
        let mut seed = 0u64;
        loop {
            if GameRng::new(seed).d6() == 6 { break; }
            seed += 1;
        }
        let mut game = make_game();
        game.home_playing = true;
        let mut p = Player::default();
        p.id = "p1".into();
        p.agility = 3;
        game.team_home.players.push(p);
        let coord = FieldCoordinate::new(10, 7);
        game.field_model.set_player_coordinate("p1", coord);
        game.field_model.set_player_state("p1", PlayerState::new(PS_STANDING));

        let mut step = StepRightStuff::new("success".into());
        step.thrown_player_id = Some("p1".into());
        step.thrown_player_has_ball = Some(false);
        step.old_player_state = Some(PlayerState::new(PS_STANDING));
        step.start(&mut game, &mut GameRng::new(seed));
        assert!(game.report_list.has_report(ReportId::RIGHT_STUFF_ROLL),
            "RIGHT_STUFF_ROLL report must be added on a normal d6 roll (no FumbledPlayerLandsSafely)");
    }

    #[test]
    fn minimum_roll_uses_agility_with_modifiers_not_raw_agility() {
        // Java: AgilityMechanic.minimumRollRightStuff(Player, modifiers) uses
        // player.getAgilityWithModifiers(), not the raw AG stat. A temporary AG
        // penalty (e.g. from a spell/skill effect) must lower the effective
        // agility used for the Right Stuff minimum-roll calculation.
        use ffb_model::model::player::{Player, STAT_AG};
        use ffb_model::enums::{PS_STANDING, PlayerState};
        use ffb_model::types::FieldCoordinate;
        let mut game = make_game();
        let mut p = Player::default();
        p.id = "p1".into();
        p.agility = 6;
        // Raw agility 6 → minimum roll 6 (fails on a roll of 5).
        // Effective agility (with -2 penalty) is 4 → minimum roll 4 (succeeds on a roll of 5).
        p.add_temporary_stat_mod("test", STAT_AG, -2);
        game.team_home.players.push(p);
        let coord = FieldCoordinate::new(10, 7);
        game.field_model.set_player_coordinate("p1", coord);
        game.field_model.set_player_state("p1", PlayerState::new(PS_STANDING));

        let mut step = StepRightStuff::new("success".into());
        step.thrown_player_id = Some("p1".into());
        step.thrown_player_has_ball = Some(false);
        step.old_player_state = Some(PlayerState::new(PS_STANDING));
        step.roll = 5;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel,
            "roll of 5 must succeed against the agility-with-modifiers threshold of 4, \
             not fail against the raw-agility threshold of 6");
    }
}
