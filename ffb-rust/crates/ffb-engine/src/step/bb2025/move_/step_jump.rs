use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::enums::ReRollSource;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::drop_player_context::SteadyFootingContext;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::abstract_step_with_re_roll::ReRollState;
use crate::step::abstract_step_with_re_roll::find_skill_reroll_source;
use crate::step::util_server_re_roll::{ask_for_reroll_if_available_ignoring, use_reroll};
use ffb_mechanics::bb2025::jump_mechanic::JumpMechanic;
use ffb_mechanics::jump_mechanic::JumpMechanic as JumpMechanicTrait;
use ffb_mechanics::modifiers::jump_modifier_factory::JumpModifierFactory;
use ffb_mechanics::modifiers::jump_context::JumpContext;
use crate::util::util_server_player_move::UtilServerPlayerMove;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.move.StepJump.
///
/// Resolves a jump (Leap / VLL): agility roll with tackle-zone modifiers.
/// On success: clears jumping flag, publishes JUMPED(true), NEXT_STEP.
/// On failure: clears jumping flag, publishes COORDINATE_FROM + STEADY_FOOTING_CONTEXT,
///             GoTos failure label.
///
/// If `!actingPlayer.isJumping()` or `!mechanic.canStillJump(game, actingPlayer)` → NEXT_STEP.
///
/// Init params: GOTO_LABEL_ON_FAILURE (mandatory), MOVE_START (optional).
///
/// Re-roll: TRR offered via ReRollOffer prompt on first failure.
/// (Skill re-roll for JUMP not yet in the reroll-property mapping.)
///
/// canStillJump (JumpMechanic) is wired; minimum roll uses player agility (BB2025: ag + modifiers).
/// DiceInterpreter.is_skill_roll_successful used for success check.
///
/// Skill re-roll for JUMP (Leap canRerollJump source) → wired via find_skill_reroll_source.
/// JumpModifierFactory wired: TACKLEZONE (max of from/to zones) and PREHENSILE_TAIL modifiers applied.
/// client-only: DivingTackle dialog (checkDivingTackle, usingDivingTackle) — client-side.
/// client-only: canChooseToIgnoreJumpModifierAfterRoll skill dialog — client-side.
/// Bounding Leap (canIgnoreJumpModifiers / useIgnoreModifierSkill) ported 2026-11-20 (BACKLOG §M 2b(f)).
/// handleFailure: COORDINATE_FROM(null if roll==1), updatePlayerAndBallPosition, updateMoveSquares — all implemented.
/// checkDivingTackle / usingDivingTackle: a successful leap away from an opposing Diving Tackler
/// (canAttemptToTackleJumpingPlayer) offers the leaper a re-roll, then asks the opposing coach.
/// canChooseToIgnoreJumpModifierAfterRoll (Swift As The Breeze) is offered like Java.
/// fSecondGoForIt: handled in StepGoForIt (second_go_for_it field there triggers repeat for jumping players).
pub struct StepJump {
    /// Java: goToLabelOnFailure
    pub goto_label_on_failure: String,
    /// Java: moveStart
    pub move_start: Option<FieldCoordinate>,
    /// Java: roll
    pub roll: i32,
    /// Java: usingDivingTackle (Boolean tristate)
    pub using_diving_tackle: Option<bool>,
    /// Java: alreadyReported
    pub already_reported: bool,
    /// Java: useIgnoreModifierAfterRollSkill (Boolean tristate)
    pub use_ignore_modifier_after_roll_skill: Option<bool>,
    /// Java: useIgnoreModifierSkill
    pub use_ignore_modifier_skill: bool,
    /// Java: dtRerollAsked
    pub dt_reroll_asked: bool,
    /// Java: AbstractStepWithReRoll fields
    pub re_roll_state: ReRollState,
    /// The DIVING_TACKLE player choice is open.
    waiting_for_diving_tackle: bool,
    /// leap() ran once (Java `status != null`): later passes re-read the roll.
    leapt: bool,
    /// Java `publishParameter(USING_DIVING_TACKLE, true)` -- carried by the next outcome.
    publish_using_diving_tackle: bool,
}

impl StepJump {
    pub fn new(goto_label_on_failure: String) -> Self {
        Self {
            goto_label_on_failure,
            move_start: None,
            roll: 0,
            using_diving_tackle: None,
            already_reported: false,
            use_ignore_modifier_after_roll_skill: None,
            use_ignore_modifier_skill: false,
            dt_reroll_asked: false,
            re_roll_state: ReRollState::new(),
            waiting_for_diving_tackle: false,
            leapt: false,
            publish_using_diving_tackle: false,
        }
    }
}

impl Step for StepJump {
    fn id(&self) -> StepId { StepId::Jump }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::UseReRoll { use_reroll: false } => {
                self.re_roll_state.re_roll_source = None;
            }
            // Java CLIENT_PLAYER_CHOICE (DIVING_TACKLE): `usingDivingTackle = isProvided(playerId);
            // game.setDefenderId(playerId)`.
            Action::PlayerChoice { player_id, mode, .. } if mode == "DIVING_TACKLE" => {
                let chosen = player_id.clone().filter(|p| !p.is_empty());
                self.using_diving_tackle = Some(chosen.is_some());
                game.defender_id = chosen;
                self.waiting_for_diving_tackle = false;
            }
            Action::SelectPlayer { player_id } if self.waiting_for_diving_tackle => {
                let chosen = Some(player_id.clone()).filter(|p| !p.is_empty());
                self.using_diving_tackle = Some(chosen.is_some());
                game.defender_id = chosen;
                self.waiting_for_diving_tackle = false;
            }
            // Java CLIENT_USE_SKILL with canChooseToIgnoreJumpModifierAfterRoll.
            Action::UseSkill { skill_id, use_skill }
                if skill_id.properties().contains(&ffb_model::model::property::named_properties::NamedProperties::CAN_CHOOSE_TO_IGNORE_JUMP_MODIFIER_AFTER_ROLL) =>
            {
                self.use_ignore_modifier_after_roll_skill = Some(*use_skill);
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(v) => { self.goto_label_on_failure = v.clone(); true }
            StepParameter::MoveStart(v) => { self.move_start = Some(*v); true }
            StepParameter::UsingDivingTackle(v) => { self.using_diving_tackle = Some(*v); true }
            _ => false,
        }
    }
}

/// Java `ActionStatus` as `StepJump.leap()` returns it (WAITING_FOR_RE_ROLL carries the prompt).
enum LeapStatus {
    Success,
    Failure,
    WaitingForReRoll(ffb_model::prompts::AgentPrompt),
}

impl StepJump {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: doLeap = actingPlayer.isJumping() && mechanic.canStillJump(game, actingPlayer)
        let mechanic = JumpMechanic::new();
        let do_leap = game.acting_player.jumping
            && mechanic.can_still_jump(game, &game.acting_player.clone());

        if !do_leap {
            return StepOutcome::next();
        }

        let already_rerolled = self.re_roll_state.re_rolled_action
            .as_ref().map(|a| a.name == "JUMP").unwrap_or(false);

        // Java: if (JUMP == reRolledAction && !useIgnoreModifierAfterRollSkill && usingDivingTackle==null)
        //         if (!dtRerollAsked && (source==null || !useReRoll(...))) { handleFailure; doLeap=false }
        // After the Diving Tackle re-roll offer (dtRerollAsked) Java spends nothing here: an accepted
        // offer re-rolls in leap() (`reRolled`), a declined one keeps the roll.
        if already_rerolled && self.use_ignore_modifier_after_roll_skill != Some(true)
            && self.using_diving_tackle.is_none() && self.dt_reroll_asked
        {
            if self.re_roll_state.re_roll_source.is_some() {
                self.roll = 0;
            }
        } else if already_rerolled && self.use_ignore_modifier_after_roll_skill != Some(true)
            && self.using_diving_tackle.is_none()
        {
            let pid = game.acting_player.player_id.as_deref().unwrap_or("").to_owned();
            let source_opt = self.re_roll_state.re_roll_source.clone();
            let consumed = source_opt
                .as_ref()
                .map(|s| use_reroll(game, s, &pid, rng))
                .unwrap_or(false);
            if !consumed {
                return self.handle_failure(game);
            }
            // Java NEVER clears `roll` when it OFFERS a re-roll: `doRoll` is `reRolled || ...`,
            // so a fresh die is drawn only once a re-roll has actually been CONSUMED. Zeroing it
            // at the offer made a DECLINED offer reach the failure path with `roll == 0`, and the
            // `roll > 1` test there is what decides whether a failed leaper lands on the target
            // square or is put back where he started
            // (`updatePlayerAndBallPosition(player, moveStart)`). Declining a jump re-roll
            // therefore teleported the faller back onto his own square - slann bb2025 seed 22,
            // where Java stunned him at (12,4) and Rust at (13,2).
            self.roll = 0;
        }

        // Java (bb2020 :173, bb2025 :178): Bounding Leap. A jump declared with the skill
        // (StepInitMoving / StepInitSelecting `canIgnoreJumpModifiers` arm) sets
        // `jumpsWithoutModifiers`; the step then reports the use, switches Diving Tackle off and
        // rolls without modifiers.
        self.use_ignore_modifier_skill = game.acting_player.jumps_without_modifiers;
        if self.use_ignore_modifier_skill {
            let pid = game.acting_player.player_id.clone();
            let skill = pid.as_deref().and_then(|id| game.player(id)).and_then(|p| {
                ffb_model::util::util_cards::UtilCards::get_unused_skill_with_property(
                    p, ffb_model::model::property::named_properties::NamedProperties::CAN_IGNORE_JUMP_MODIFIERS)
            });
            match skill {
                None => self.use_ignore_modifier_skill = false,
                Some(skill_id) => {
                    self.using_diving_tackle = Some(false);
                    game.report_list.add(ffb_model::report::report_skill_use::ReportSkillUse::new(
                        pid,
                        skill_id,
                        true,
                        ffb_model::model::skill_use::SkillUse::PASS_JUMP_WITHOUT_MODIFIERS,
                    ));
                }
            }
        }

        let mut events = Vec::new();
        let out = match self.leap(game, rng, &mut events) {
            LeapStatus::Success => {
                game.acting_player.set_jumping(false);
                StepOutcome::next()
                    .publish(StepParameter::Jumped(true))
                    .with_events(events)
            }
            LeapStatus::WaitingForReRoll(prompt) => {
                StepOutcome::cont().with_prompt(prompt).with_events(events)
            }
            LeapStatus::Failure => {
                let mut out = self.handle_failure(game);
                let mut all = events;
                all.append(&mut out.events);
                out.events = all;
                out
            }
        };
        if std::mem::take(&mut self.publish_using_diving_tackle) {
            out.publish(StepParameter::UsingDivingTackle(true))
        } else {
            out
        }
    }

    /// Java `StepJump.leap()` (bb2020 + bb2025 share the shape used here).
    fn leap(
        &mut self,
        game: &mut Game,
        rng: &mut GameRng,
        events: &mut Vec<ffb_model::events::GameEvent>,
    ) -> LeapStatus {
        // Java: reRolled = (reRolledAction == JUMP) && (reRollSource != null)
        let re_rolled = self.re_roll_state.re_rolled_action
            .as_ref().map(|a| a.name == "JUMP").unwrap_or(false)
            && self.re_roll_state.re_roll_source.is_some();

        // Java `doRoll`: a fresh die on the first leap and after an accepted re-roll; the Diving
        // Tackle / modifier-ignoring answers re-read the same roll.
        let do_roll = self.roll == 0 || !self.leapt;
        self.leapt = true;
        if self.roll == 0 {
            self.roll = rng.d6();
        }

        let player_id = game.acting_player.player_id.clone();
        let agility = player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.agility_with_modifiers())
            .unwrap_or(3);

        // Java leap(): an accepted Diving Tackle (usingDivingTackle) reports STOP_OPPONENT once,
        // publishes USING_DIVING_TACKLE and adds the skill's +2 jump modifier.
        let mut diving_tackle = false;
        if self.using_diving_tackle == Some(true) {
            let dt_skill = game.defender_id.as_deref().and_then(|d| game.player(d)).and_then(|p|
                p.skill_id_with_property_in(game.rules,
                    ffb_model::model::property::named_properties::NamedProperties::CAN_ATTEMPT_TO_TACKLE_JUMPING_PLAYER));
            if let Some(skill) = dt_skill {
                diving_tackle = true;
                if !self.already_reported {
                    self.already_reported = true;
                    self.publish_using_diving_tackle = true;
                    game.report_list.add(ffb_model::report::report_skill_use::ReportSkillUse::new(
                        game.defender_id.clone(), skill, true, ffb_model::model::skill_use::SkillUse::STOP_OPPONENT));
                }
            }
        }

        // Java: JumpModifierFactory.findModifiers(JumpContext(game, player, moveStart, to)),
        // skipped entirely when the Bounding Leap is in use (`if (!useIgnoreModifierSkill)`).
        // from = move_start (origin of leap); to = current player coordinate.
        let (modifier_total, modifier_names) = if self.use_ignore_modifier_skill {
            (0, vec![])
        } else {
            self.jump_modifiers(game, diving_tackle)
        };
        let minimum_roll = (agility + modifier_total).max(2);

        // Java: `successfulWithoutModifiers = isSkillRollSuccessful(roll, minimumRollJump(player,
        // emptySet()))`; only then an UNUSED canChooseToIgnoreJumpModifierAfterRoll skill counts.
        let successful_without_modifiers = DiceInterpreter::is_skill_roll_successful(self.roll, agility.max(2));
        let after_roll_skill: Option<ffb_model::enums::SkillId> = if successful_without_modifiers {
            player_id.as_deref().and_then(|pid| game.player(pid)).and_then(|p| p.all_skill_ids().find(|id| {
                id.properties().contains(&ffb_model::model::property::named_properties::NamedProperties::CAN_CHOOSE_TO_IGNORE_JUMP_MODIFIER_AFTER_ROLL)
                    && !p.used_skills.contains(id) && !game.acting_player.used_skills.contains(id)
            }))
        } else {
            None
        };
        if let (Some(skill), Some(true)) = (after_roll_skill, self.use_ignore_modifier_after_roll_skill) {
            if let Some(pid) = player_id.as_deref() {
                crate::step::util_server_steps::mark_skill_used(game, pid, skill);
                game.acting_player.used_skills.insert(skill);
                game.report_list.add(ffb_model::report::report_skill_use::ReportSkillUse::new(
                    Some(pid.to_string()), skill, true, ffb_model::model::skill_use::SkillUse::PASS_JUMP_WITHOUT_MODIFIERS));
            }
            return LeapStatus::Success;
        }
        let successful = DiceInterpreter::is_skill_roll_successful(self.roll, minimum_roll);

        // Java: if (doRoll) getResult().addReport(new ReportJumpRoll(playerId, successful, roll, minRoll, reRolled, modifiers))
        if do_roll {
            use ffb_model::report::report_jump_roll::ReportJumpRoll;
            game.report_list.add(ReportJumpRoll::new(
                player_id.clone(),
                successful,
                self.roll,
                minimum_roll,
                re_rolled,
                modifier_names,
            ));
        }

        // `GameEvent::JumpRoll` existed in `ffb-model` with a coverage-report consumer and a wire
        // encoder but NO PRODUCER anywhere in the engine, so `jumpRoll` would have read 0 in every
        // harvest even once the agent learned to declare a jump (BACKLOG E12). Emitted here on the
        // same rule as `StepMoveDodge`'s `DodgeRoll`: one event per RESOLVED roll, the re-rolled
        // resolution producing its own.
        events.push(ffb_model::events::GameEvent::JumpRoll {
            player_id: player_id.clone().unwrap_or_default(),
            target: minimum_roll,
            roll: self.roll,
            success: successful,
        });

        let mut status = if successful { LeapStatus::Success } else { LeapStatus::Failure };
        if successful && self.using_diving_tackle.is_none() {
            status = self.check_diving_tackle(game);
        }
        if !successful {
            // Java: skill = getUnusedSkillWithProperty(actingPlayer, canIgnoreJumpModifiers);
            //       ignoreSkills = {skill}
            let bounding_leap = player_id.as_deref().and_then(|id| game.player(id)).and_then(|p| {
                ffb_model::util::util_cards::UtilCards::get_unused_skill_with_property(
                    p, ffb_model::model::property::named_properties::NamedProperties::CAN_IGNORE_JUMP_MODIFIERS)
            });
            let already_rerolled = self.re_roll_state.re_rolled_action
                .as_ref().map(|a| a.name == "JUMP").unwrap_or(false);
            if !already_rerolled {
                use ffb_model::model::re_rolled_action::ReRolledAction;
                self.re_roll_state.re_rolled_action = Some(ReRolledAction::new("JUMP"));

                // Java: skillReRollSource = UtilCards.getUnusedRerollSource(actingPlayer, JUMP);
                //   if (skillReRollSource != null && (skill == null
                //       || skill.getRerollSource(JUMP) != skillReRollSource || useIgnoreModifierSkill))
                //     status = leap();          // the skill's own re-roll: no useReRoll, no dialog
                //   else askForReRollIfAvailable(..., ignoreSkills)
                // Bounding Leap is the only skill registering a JUMP re-roll source, so an unused
                // Bounding Leap NOT declared for this jump is never spent as a re-roll: the coach is
                // offered the team re-roll instead, with the skill left out of the dialog.
                let skill_source = find_skill_reroll_source(game, "JUMP");
                let source_is_bounding_leap = match (&skill_source, bounding_leap) {
                    (Some(src), Some(bl)) => {
                        bl.reroll_sources_for(game.rules).iter().any(|(a, _)| *a == "JUMP")
                            && src.name == format!("{:?}", bl)
                    }
                    _ => false,
                };
                if skill_source.is_some()
                    && (bounding_leap.is_none() || !source_is_bounding_leap || self.use_ignore_modifier_skill)
                {
                    self.roll = 0;
                    status = self.leap(game, rng, events);
                } else {
                    let ignore: Vec<ffb_model::enums::SkillId> = bounding_leap.into_iter().collect();
                    if let Some(prompt) = ask_for_reroll_if_available_ignoring(game, "JUMP", minimum_roll, false, &ignore) {
                        // Java askForReRollIfAvailable(..., modificationSkill = the after-roll skill).
                        if let Some(skill) = after_roll_skill {
                            let acting = game.acting_player.player_id.clone();
                            crate::step::util_server_re_roll::show_re_roll_properties_dialog(
                                game, acting.as_deref(), "JUMP", minimum_roll, false, None, Some(skill), Vec::new());
                        }
                        self.re_roll_state.re_roll_source = Some(ReRollSource::new("TRR"));
                        status = LeapStatus::WaitingForReRoll(prompt);
                    }
                }
            } else if let (None, Some(skill)) = (self.use_ignore_modifier_after_roll_skill, after_roll_skill) {
                // Java: showDialog(new DialogSkillUseParameter(pid, skill, 0)); WAITING_FOR_SKILL_USE.
                let pid = player_id.clone().unwrap_or_default();
                crate::util::util_server_dialog::UtilServerDialog::show_dialog(game,
                    ffb_model::dialog::dialog_parameter_factory::DialogParameter::SkillUse(
                        ffb_model::dialog::dialog_skill_use_parameter::DialogSkillUseParameter {
                            player_id: Some(pid.clone()), skill: Some(skill), minimum_roll: 0, ..Default::default()
                        }), false);
                return LeapStatus::WaitingForReRoll(ffb_model::prompts::AgentPrompt::SkillUse {
                    player_id: pid, skill_id: skill as u16, skill_name: format!("{skill:?}"),
                });
            }
        }
        // Java: if (useIgnoreModifierSkill) actingPlayer.markSkillUsed(canIgnoreJumpModifiers)
        if self.use_ignore_modifier_skill {
            if let Some(pid) = player_id.as_deref() {
                let skill = game.player(pid).and_then(|p| {
                    p.all_skill_ids().find(|id| id.properties_for(game.rules)
                        .contains(&ffb_model::model::property::named_properties::NamedProperties::CAN_IGNORE_JUMP_MODIFIERS))
                });
                if let Some(skill_id) = skill {
                    crate::step::util_server_steps::mark_skill_used(game, pid, skill_id);
                }
            }
        }
        status
    }

    /// The leap's jump modifiers (JumpModifierFactory over JumpContext(game, player, moveStart, to))
    /// as total and names; `diving_tackle` adds the Diving Tackle modifier (+2) -- Java's
    /// `context.addModifierValue(2)` counts it towards the sum-dependent skill modifiers too.
    fn jump_modifiers(&self, game: &Game, diving_tackle: bool) -> (i32, Vec<String>) {
        let player_id = game.acting_player.player_id.clone();
        let (Some(pid), Some(from)) = (player_id.as_deref(), self.move_start) else { return (0, vec![]) };
        let Some(player) = game.player(pid) else { return (0, vec![]) };
        let to = game.field_model.player_coordinate(pid).unwrap_or(from);
        let ctx = JumpContext::new(game, player, from, to);
        let factory = JumpModifierFactory::for_rules(game.rules);
        let mods = factory.find_applicable(&ctx);
        let dt = if diving_tackle { 2 } else { 0 };
        let accumulated: i32 = mods.iter().map(|m| m.get_modifier()).sum::<i32>() + dt;
        let count = mods.len() as i32 + i32::from(diving_tackle);
        let skill_mods = factory.find_skill_modifiers(&ctx, accumulated, count);
        let all: Vec<&ffb_mechanics::modifiers::jump_modifier::JumpModifier> = mods.iter().copied().chain(skill_mods.iter()).collect();
        let mut total: i32 = all.iter().map(|m| m.get_modifier()).sum();
        let mut names: Vec<String> = all.iter().map(|m| m.get_name().to_owned()).collect();
        if diving_tackle {
            total += 2;
            names.push("Diving Tackle".into());
        }
        (total, names)
    }

    /// Java `checkDivingTackle`: after a successful leap, an opposing Diving Tackler next to the
    /// square left could still make it fail (+2). If it would not, ReportSkillUse(WOULD_NOT_HELP);
    /// otherwise the leaper is first offered a re-roll ("Diving Tackle can make this jump fail"),
    /// then the opposing coach is asked (DialogPlayerChoiceParameter DIVING_TACKLE).
    fn check_diving_tackle(&mut self, game: &mut Game) -> LeapStatus {
        use ffb_model::model::property::named_properties::NamedProperties;
        let Some(pid) = game.acting_player.player_id.clone() else { return LeapStatus::Success };
        let Some(from) = self.move_start else { return LeapStatus::Success };
        let ignores_dt = game.player(&pid).is_some_and(|p|
            p.has_skill_to_cancel_property_in(game.rules, NamedProperties::CAN_ATTEMPT_TO_TACKLE_JUMPING_PLAYER));
        if ignores_dt {
            return LeapStatus::Success;
        }
        let to = game.field_model.player_coordinate(&pid).unwrap_or(from);
        let tacklers = ffb_model::util::util_player::UtilPlayer::find_eligible_jump_tacklers(game, from, to);
        let Some(first) = tacklers.first() else { return LeapStatus::Success };
        let Some(skill) = game.player(first).and_then(|p|
            p.skill_id_with_property_in(game.rules, NamedProperties::CAN_ATTEMPT_TO_TACKLE_JUMPING_PLAYER))
        else {
            return LeapStatus::Success;
        };
        let agility = game.player(&pid).map(|p| p.agility_with_modifiers()).unwrap_or(3);
        let (total, _) = self.jump_modifiers(game, true);
        let minimum_roll = (agility + total).max(2);
        if DiceInterpreter::is_skill_roll_successful(self.roll, minimum_roll) {
            game.report_list.add(ffb_model::report::report_skill_use::ReportSkillUse::new(
                None, skill, false, ffb_model::model::skill_use::SkillUse::WOULD_NOT_HELP));
            return LeapStatus::Success;
        }
        let already_rerolled = self.re_roll_state.re_rolled_action.as_ref().is_some_and(|a| a.name == "JUMP");
        if !self.dt_reroll_asked && !already_rerolled {
            // Java: askForReRollIfAvailable(gameState, actingPlayer.getPlayer(), JUMP, minimumRoll,
            // false, null, null, null, null, ["Diving Tackle can make this jump fail. Reroll the jump now?"])
            if let Some(prompt) = crate::step::util_server_re_roll::ask_for_reroll_if_available_for(
                game, Some(&pid), "JUMP", minimum_roll, false)
            {
                self.dt_reroll_asked = true;
                use ffb_model::model::re_rolled_action::ReRolledAction;
                self.re_roll_state.re_rolled_action = Some(ReRolledAction::new("JUMP"));
                self.re_roll_state.re_roll_source = Some(ReRollSource::new("TRR"));
                return LeapStatus::WaitingForReRoll(prompt);
            }
        }
        let team_id = if game.home_playing { game.team_away.id.clone() } else { game.team_home.id.clone() };
        crate::util::util_server_dialog::UtilServerDialog::show_dialog(game,
            ffb_model::dialog::dialog_parameter_factory::DialogParameter::PlayerChoice(
                ffb_model::dialog::dialog_player_choice_parameter::DialogPlayerChoiceParameter {
                    team_id: Some(team_id),
                    player_choice_mode: Some(ffb_model::model::player_choice_mode::PlayerChoiceMode::DIVING_TACKLE.get_name().to_string()),
                    player_ids: tacklers.clone(),
                    descriptions: Vec::new(),
                    max_selects: 1,
                    min_selects: 0,
                }), true);
        self.waiting_for_diving_tackle = true;
        LeapStatus::WaitingForReRoll(ffb_model::prompts::AgentPrompt::PlayerChoice {
            eligible_players: tacklers,
            reason: "DIVING_TACKLE".into(),
            descriptions: Vec::new(),
        })
    }

    fn handle_failure(&mut self, game: &mut Game) -> StepOutcome {
        // Java: actingPlayer.setJumping(false)
        game.acting_player.set_jumping(false);
        // Java: publishParameter(STEADY_FOOTING_CONTEXT, new SteadyFootingContext(new InjuryTypeDropJump(defender)))
        // Java passes `game.getDefender()` as the DropJump diving tackler (fallback Arm Bar source).
        let injury_name = match game.defender_id.as_deref() {
            Some(d) if !d.is_empty() => format!("InjuryTypeDropJump#dt:{d}"),
            _ => "InjuryTypeDropJump".to_string(),
        };
        let ctx = SteadyFootingContext::from_injury_type_name(injury_name);
        let label = self.goto_label_on_failure.clone();
        let coord_from = if self.roll > 1 {
            self.move_start
        } else {
            // Java: game.getFieldModel().updatePlayerAndBallPosition(actingPlayer.getPlayer(), moveStart)
            if let (Some(pid), Some(start)) = (
                game.acting_player.player_id.clone(),
                self.move_start,
            ) {
                let old_pos = game.field_model.player_coordinate(&pid);
                if !game.field_model.ball_moving {
                    if let (Some(old), Some(ball)) = (old_pos, game.field_model.ball_coordinate) {
                        if old == ball {
                            game.field_model.ball_coordinate = Some(start);
                        }
                    }
                }
                game.field_model.set_player_coordinate(&pid, start);
            }
            // Java: UtilServerPlayerMove.updateMoveSquares(getGameState(), actingPlayer.isJumping())
            UtilServerPlayerMove::update_move_squares(game, false);
            None
        };
        let mut out = StepOutcome::goto(&label)
            .publish(StepParameter::SteadyFootingContext(Box::new(ctx)));
        if let Some(c) = coord_from {
            out = out.publish(StepParameter::CoordinateFrom(Some(c)));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{Rules, TurnMode};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::enums::SkillId;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    fn make_game_with_leaper() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2025);

        let mut player = Player::default();
        player.id = "p1".into();
        player.agility = 3;
        player.starting_skills.push(SkillWithValue::new(SkillId::Leap));
        game.team_home.players.push(player);
        game.field_model.set_player_coordinate("p1", ffb_model::types::FieldCoordinate::new(5, 5));
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.jumping = true;
        game
    }

    #[test]
    fn not_jumping_returns_next_step() {
        let mut game = make_game();
        game.acting_player.jumping = false;
        let mut step = StepJump::new("fail".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn jumping_but_cannot_still_jump_returns_next_step() {
        let mut game = make_game();
        game.acting_player.jumping = true;
        let mut step = StepJump::new("fail".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn success_clears_jumping_flag() {
        let mut game = make_game_with_leaper();
        let mut step = StepJump::new("fail".into());
        step.roll = 4; // ag=3, min_roll=3, 4 >= 3 → success
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.acting_player.jumping);
    }

    #[test]
    fn success_publishes_jumped_true() {
        let mut game = make_game_with_leaper();
        let mut step = StepJump::new("fail".into());
        step.roll = 4;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::Jumped(true))));
    }

    #[test]
    fn failure_goes_to_failure_label() {
        let mut game = make_game_with_leaper();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        let mut step = StepJump::new("fail".into());
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    #[test]
    fn failure_roll_2_below_ag3_goes_to_label() {
        let mut game = make_game_with_leaper();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        let mut step = StepJump::new("fail".into());
        step.roll = 2;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn failure_roll_gt_1_publishes_coordinate_from() {
        let mut game = make_game_with_leaper();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        let start = FieldCoordinate::new(4, 4);
        let mut step = StepJump::new("fail".into());
        step.roll = 2;
        step.move_start = Some(start);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::CoordinateFrom(_))));
    }

    #[test]
    fn failure_roll_1_no_coordinate_from() {
        let mut game = make_game_with_leaper();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        let mut step = StepJump::new("fail".into());
        step.roll = 1;
        step.move_start = Some(FieldCoordinate::new(4, 4));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(!out.published.iter().any(|p| matches!(p, StepParameter::CoordinateFrom(_))));
    }

    #[test]
    fn failure_with_trr_offers_reroll_prompt() {
        let mut game = make_game_with_leaper();
        game.set_turn_mode(TurnMode::Regular);
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        let mut step = StepJump::new("fail".into());
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
        assert!(out.prompt.is_some());
    }

    #[test]
    fn decline_reroll_goes_to_failure_label() {
        let mut game = make_game_with_leaper();
        game.set_turn_mode(TurnMode::Regular);
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        let mut step = StepJump::new("fail".into());
        step.roll = 1;
        let _offer = step.start(&mut game, &mut GameRng::new(0));
        let out = step.handle_command(&Action::UseReRoll { use_reroll: false }, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn accept_reroll_with_success_returns_next_step() {
        let mut game = make_game_with_leaper();
        game.set_turn_mode(TurnMode::Regular);
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        let mut step = StepJump::new("fail".into());
        step.roll = 1; // first roll fails
        let _offer = step.start(&mut game, &mut GameRng::new(0));
        step.roll = 5; // success on re-roll (ag=3, min=3)
        let out = step.handle_command(&Action::UseReRoll { use_reroll: true }, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_move_start_accepted() {
        let mut step = StepJump::new("fail".into());
        let coord = FieldCoordinate::new(4, 4);
        assert!(step.set_parameter(&StepParameter::MoveStart(coord)));
        assert_eq!(step.move_start, Some(coord));
    }

    #[test]
    fn set_parameter_goto_label_on_failure_accepted() {
        let mut step = StepJump::new("old".into());
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("new".into())));
        assert_eq!(step.goto_label_on_failure, "new");
    }

    #[test]
    fn failure_publishes_steady_footing_context_drop_jump() {
        let mut game = make_game_with_leaper();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        let mut step = StepJump::new("fail".into());
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::SteadyFootingContext(_))));
    }

    #[test]
    fn high_agility_lowers_minimum_roll() {
        let mut game = make_game();
        let mut player = Player::default();
        player.id = "p2".into();
        player.agility = 2; // 2+ target
        player.starting_skills.push(SkillWithValue::new(SkillId::Leap));
        game.team_home.players.push(player);
        game.field_model.set_player_coordinate("p2", ffb_model::types::FieldCoordinate::new(5, 5));
        game.acting_player.player_id = Some("p2".into());
        game.acting_player.jumping = true;

        let mut step = StepJump::new("fail".into());
        step.roll = 2;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep, "AG2 player should succeed on roll 2");
    }

    #[test]
    fn tackle_zone_at_destination_increases_minimum_roll() {
        // AG3 player jumping TO a square adjacent to an opponent with TZ.
        // Without TZ: min_roll = 3. With 1 TZ: min_roll = 4.
        // Roll of 3 should fail when there's a TZ opponent at the landing square.
        use ffb_model::enums::PS_STANDING;
        use ffb_model::types::FieldCoordinate;
        let mut game = make_game();

        let mut leaper = Player::default();
        leaper.id = "leaper".into();
        leaper.agility = 3;
        leaper.starting_skills.push(SkillWithValue::new(SkillId::Leap));
        game.team_home.players.push(leaper);

        // Place leaper at (5,5) — this is the "to" (current) coordinate.
        game.field_model.set_player_coordinate("leaper", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("leaper", ffb_model::enums::PlayerState::new(PS_STANDING));
        game.acting_player.player_id = Some("leaper".into());
        game.acting_player.jumping = true;

        // Opponent adjacent to (5,5) at (5,6) — creates a tackle zone at the "to" square.
        let mut opp = Player::default();
        opp.id = "opp".into();
        opp.agility = 3;
        game.team_away.players.push(opp);
        game.field_model.set_player_coordinate("opp", FieldCoordinate::new(5, 6));
        game.field_model.set_player_state("opp", ffb_model::enums::PlayerState::new(PS_STANDING));

        // move_start = (3,3) — the "from" coordinate (no adjacent opponents there).
        let mut step = StepJump::new("fail".into());
        step.move_start = Some(FieldCoordinate::new(3, 3));
        step.roll = 3; // Exactly meets ag=3 if no modifiers; fails with +1 from 1 TZ.
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;

        let out = step.start(&mut game, &mut GameRng::new(0));
        // With 1 tackle zone at "to": min_roll = 3 + 1 = 4, so roll 3 fails.
        assert_eq!(out.action, StepAction::GotoLabel,
            "roll=3 should fail when 1 TZ at landing square (min_roll=4)");
    }

    #[test]
    fn no_tackle_zones_roll_equals_agility_succeeds() {
        // AG3 player with no adjacent opponents: min_roll = 3, roll=3 → success.
        use ffb_model::types::FieldCoordinate;
        let mut game = make_game_with_leaper(); // p1 at (5,5), ag=3, no opponents

        let mut step = StepJump::new("fail".into());
        step.move_start = Some(FieldCoordinate::new(3, 3));
        step.roll = 3;

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep,
            "roll=3 should succeed when no TZ (min_roll=3)");
    }

    #[test]
    fn success_adds_jump_roll_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game_with_leaper();
        let mut step = StepJump::new("fail".into());
        step.roll = 4; // ag=3, min_roll=3, success
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::JUMP_ROLL),
            "JUMP_ROLL report must be added on success");
    }

    /// Rowana-like leaper (Leap + Bounding Leap) at (5,5) with an opponent on (5,6), jumping from
    /// (3,3): one tackle zone on the landing square, so a 3 fails WITH modifiers.
    fn make_game_with_bounding_leaper() -> Game {
        use ffb_model::enums::PS_STANDING;
        let mut game = make_game_with_leaper();
        game.team_home.players[0].starting_skills.push(SkillWithValue::new(SkillId::BoundingLeap));
        game.field_model.set_player_state("p1", ffb_model::enums::PlayerState::new(PS_STANDING));
        let mut opp = Player::default();
        opp.id = "opp".into();
        opp.agility = 3;
        game.team_away.players.push(opp);
        game.field_model.set_player_coordinate("opp", FieldCoordinate::new(5, 6));
        game.field_model.set_player_state("opp", ffb_model::enums::PlayerState::new(PS_STANDING));
        game.set_turn_mode(TurnMode::Regular);
        game.home_playing = true;
        game
    }

    fn count_reports(game: &Game, id: ffb_model::report::report_id::ReportId) -> usize {
        game.report_list.get_reports().iter().filter(|r| r.get_id() == id).count()
    }

    /// Java StepJump: a jump declared WITH Bounding Leap (`jumpsWithoutModifiers`) reports
    /// PASS_JUMP_WITHOUT_MODIFIERS, rolls against the bare agility and spends the skill.
    #[test]
    fn bounding_leap_ignores_modifiers_and_is_spent() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game_with_bounding_leaper();
        game.acting_player.set_jumps_without_modifiers(true);
        assert!(game.acting_player.jumping, "Java setJumpsWithoutModifiers(true) sets jumping");
        let mut step = StepJump::new("fail".into());
        step.move_start = Some(FieldCoordinate::new(3, 3));
        step.roll = 3; // fails at 4+ with the tackle zone, passes at the bare 3+
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(game.report_list.has_report(ReportId::SKILL_USE));
        assert!(game.team_home.players[0].used_skills.contains(&SkillId::BoundingLeap));
        assert!(!game.acting_player.jumping && !game.acting_player.jumps_without_modifiers);
    }

    /// Java StepJump.leap(): with Bounding Leap in use, a failed roll is re-rolled at once by the
    /// skill's own re-roll source (`status = leap()`) -- two jump rolls, no dialog.
    #[test]
    fn bounding_leap_failure_rerolls_without_a_dialog() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game_with_bounding_leaper();
        game.turn_data_home.rerolls = 1;
        game.acting_player.set_jumps_without_modifiers(true);
        let mut step = StepJump::new("fail".into());
        step.move_start = Some(FieldCoordinate::new(3, 3));
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_ne!(out.action, StepAction::Continue, "no re-roll dialog");
        assert!(out.prompt.is_none());
        assert_eq!(count_reports(&game, ReportId::JUMP_ROLL), 2);
        assert_eq!(game.turn_data_home.rerolls, 1, "team re-roll untouched");
        assert!(game.team_home.players[0].used_skills.contains(&SkillId::BoundingLeap));
    }

    /// Java StepJump.leap(): an unused Bounding Leap NOT declared for this jump is never spent as
    /// the jump re-roll (`skill.getRerollSource(JUMP) == skillReRollSource`): the team re-roll is
    /// offered and the skill stays unused.
    #[test]
    fn undeclared_bounding_leap_is_not_a_reroll() {
        let mut game = make_game_with_bounding_leaper();
        game.turn_data_home.rerolls = 1;
        let mut step = StepJump::new("fail".into());
        step.move_start = Some(FieldCoordinate::new(3, 3));
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
        assert!(out.prompt.is_some());
        assert!(!game.team_home.players[0].used_skills.contains(&SkillId::BoundingLeap));
    }

    #[test]
    fn failure_adds_jump_roll_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game_with_leaper();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        let mut step = StepJump::new("fail".into());
        step.roll = 1; // failure
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::JUMP_ROLL),
            "JUMP_ROLL report must be added on failure");
    }

    /// Java checkDivingTackle: a leap that succeeds only without the Diving Tackle +2, away from an
    /// opposing Diving Tackler, asks the OPPOSING coach (DIVING_TACKLE player choice; no team
    /// re-roll to offer first). Accepting it re-reads the same roll with the +2 and fails the jump.
    #[test]
    fn diving_tackle_can_stop_a_leap() {
        let mut asked = false;
        for roll in 2..=6 {
            let mut game = make_game_with_leaper();
            game.home_playing = true;
            game.set_turn_mode(TurnMode::Regular);
            game.turn_data_home.rerolls = 0;
            // The leaper started next to an away Diving Tackler and landed two squares away.
            let mut tackler = Player::default();
            tackler.id = "dt".into();
            tackler.starting_skills.push(SkillWithValue::new(SkillId::DivingTackle));
            game.team_away.players.push(tackler);
            game.field_model.set_player_state("dt", ffb_model::enums::PlayerState::new(ffb_model::enums::PS_STANDING).change_active(true));
            game.field_model.set_player_coordinate("dt", ffb_model::types::FieldCoordinate::new(4, 5));
            game.field_model.set_player_coordinate("p1", ffb_model::types::FieldCoordinate::new(7, 5));
            let mut step = StepJump::new("fail".into());
            step.move_start = Some(ffb_model::types::FieldCoordinate::new(5, 5));
            step.roll = roll;
            let out = step.start(&mut game, &mut GameRng::new(0));
            let is_dt_prompt = matches!(&out.prompt, Some(ffb_model::prompts::AgentPrompt::PlayerChoice { reason, eligible_players, .. })
                if reason == "DIVING_TACKLE" && eligible_players == &vec!["dt".to_string()]);
            if !is_dt_prompt {
                continue;
            }
            asked = true;
            let out = step.handle_command(&Action::PlayerChoice { player_id: Some("dt".into()),
                player_ids: vec!["dt".into()], mode: "DIVING_TACKLE".into() }, &mut game, &mut GameRng::new(0));
            assert_eq!(out.action, StepAction::GotoLabel, "roll {roll}: the +2 fails the leap");
            assert!(out.published.iter().any(|p| matches!(p, StepParameter::UsingDivingTackle(true))));
            assert_eq!(game.defender_id.as_deref(), Some("dt"));
        }
        assert!(asked, "some roll passes the leap but not the Diving Tackle +2");
    }
}
