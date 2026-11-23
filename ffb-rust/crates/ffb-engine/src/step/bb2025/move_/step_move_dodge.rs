use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::enums::ReRollSource;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::drop_player_context::SteadyFootingContext;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::abstract_step_with_re_roll::{ReRollState, find_skill_reroll_source};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, ask_for_reroll_if_available_for, use_reroll};
use ffb_mechanics::modifiers::dodge_modifier_factory::DodgeModifierFactory;
use ffb_mechanics::modifiers::dodge_context::DodgeContext;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.move.StepMoveDodge.
///
/// Resolves a dodge roll when leaving a tackle zone.  On failure → GoTo failure label
/// (Java: failDodge → GOTO_LABEL with STEADY_FOOTING_CONTEXT).  On success → NEXT_STEP
/// with RE_ROLL_USED + USING_BREAK_TACKLE published.
///
/// Init params: GOTO_LABEL_ON_FAILURE (mandatory).
/// Expects: COORDINATE_FROM, COORDINATE_TO, USING_BREAK_TACKLE, USING_DIVING_TACKLE,
///          RE_ROLL_USED, DODGE_ROLL set by preceding steps.
///
/// Re-roll order (mirroring Java AbstractStepWithReRoll):
///   1. Skill re-roll (Dodge — property canRerollDodge) — auto-used
///   2. Team Re-Roll token (TRR) — offered via ReRollOffer prompt
///
/// client-only: BreakTackle / canAddStrengthToDodge dialog — headless skips
/// client-only: canChooseToIgnoreDodgeModifierAfterRoll dialog — headless skips
/// client-only: ArmBar (DialogPlayerChoiceParameter) — headless skips
/// client-only: DivingTackle pre-roll check / dtRerollAsked — headless skips
/// STAND_FIRM_NO_DROP_ON_FAILED_DODGE game option → wired in execute_step (final fail path only).
/// isDodging guard, SteadyFootingContext publish on failDodge, re-roll infra, and DodgeModifierFactory are wired.
/// failDodge: SteadyFootingContext(InjuryTypeDropDodge) published — corrected from InjuryTypeFallDown
/// (matches the fix already applied in the BB2020 sibling step).
pub struct StepMoveDodge {
    /// Java: fGotoLabelOnFailure
    pub goto_label_on_failure: String,
    /// Java: fCoordinateFrom
    pub coordinate_from: Option<FieldCoordinate>,
    /// Java: fCoordinateTo
    pub coordinate_to: Option<FieldCoordinate>,
    /// Java: fDodgeRoll
    pub dodge_roll: i32,
    /// Java: fUsingDivingTackle (Boolean tristate)
    pub using_diving_tackle: Option<bool>,
    /// Java: fUsingBreakTackle
    pub using_break_tackle: bool,
    /// Java: fReRollUsed
    pub re_roll_used: bool,
    /// Java: usingModifyingSkill (Boolean tristate)
    pub using_modifying_skill: Option<bool>,
    /// Java: usingModifierIgnoringSkill (Boolean tristate)
    pub using_modifier_ignoring_skill: Option<bool>,
    /// Java: armBarPlayers (Player<?>[]) — stored as player IDs
    pub arm_bar_players: Vec<String>,
    /// Java: armBarPlayerId
    pub arm_bar_player_id: Option<String>,
    /// Java: armBarChoice
    pub arm_bar_choice: bool,
    /// Java: dtRerollAsked
    pub dt_reroll_asked: bool,
    /// Java: AbstractStepWithReRoll fields
    pub re_roll_state: ReRollState,
    /// A die rolled on THIS pass of `execute_step`, waiting to be published as
    /// `StepParameter::DodgeRoll`. Java `dodge(doRoll=true)` publishes `DODGE_ROLL` the moment it
    /// rolls, in every edition; Rust kept the die private, so `StepDivingTackle` (whose body is
    /// gated on `dodgeRoll > 0`) never ran and Diving Tackle was a dead mechanic (BACKLOG §H.60).
    dodge_roll_to_publish: Option<i32>,
    /// The minimum of the last `dodge()` pass — Java keeps `dodgeModifiers` as a field, so a
    /// re-entry's `getModifyingSkillInCaseItHelps` re-tests the STALE roll against it.
    last_minimum_roll: i32,
    /// Java `dodge()`: `publishParameter(USING_MODIFYING_SKILL, true)` once Incorporeal's
    /// strength is applied — published by the `execute_step` wrapper.
    publish_using_modifying_skill: bool,
}

impl StepMoveDodge {
    pub fn new(goto_label_on_failure: String) -> Self {
        Self {
            goto_label_on_failure,
            coordinate_from: None,
            coordinate_to: None,
            dodge_roll: 0,
            using_diving_tackle: None,
            using_break_tackle: false,
            re_roll_used: false,
            using_modifying_skill: None,
            using_modifier_ignoring_skill: None,
            arm_bar_players: Vec::new(),
            arm_bar_player_id: None,
            arm_bar_choice: false,
            dt_reroll_asked: false,
            re_roll_state: ReRollState::new(),
            dodge_roll_to_publish: None,
            last_minimum_roll: 0,
            publish_using_modifying_skill: false,
        }
    }
}

impl Step for StepMoveDodge {
    fn id(&self) -> StepId { StepId::MoveDodge }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::UseReRoll { use_reroll: false } => {
                self.re_roll_state.re_roll_source = None;
                self.execute_step(game, rng)
            }
            // Java (bb2020 + bb2025 handleCommand): CLIENT_USE_SKILL with a canAddStrengthToDodge
            // skill (Incorporeal) → `usingModifyingSkill = isSkillUsed()`; a DECLINE spends the
            // Dodge skill re-roll on the spot (`useSkillReRollSource`), then EXECUTE_STEP. Rust
            // auto-declined the dialog without ever raising it (star tier: bb2020 Gretchen
            // Wächter, chaos_pact vs khemri seed 1).
            Action::UseSkill { skill_id, use_skill }
                if skill_id.properties_for(game.rules).contains(
                    &ffb_model::model::property::named_properties::NamedProperties::CAN_ADD_STRENGTH_TO_DODGE) =>
            {
                self.using_modifying_skill = Some(*use_skill);
                if !*use_skill {
                    if let Some(source) = find_skill_reroll_source(game, "DODGE") {
                        self.re_roll_state.re_roll_source = Some(source.clone());
                        let pid = game.acting_player.player_id.clone().unwrap_or_default();
                        use_reroll(game, &source, &pid, rng);
                    }
                    // Java `doRoll = reRolledAction && getReRollSource() != null` → a fresh die.
                    // The source is the step's, so a SECOND decline -- the Dodge skill now spent,
                    // `findSkillReRollSource` null, nothing re-set -- still finds the first
                    // decline's source and rolls again (chaos_pact v necromantic bb2020 seed 2:
                    // Java 2-fail, decline, Dodge 2-fail, decline, fresh 1; Rust re-read the stale
                    // 2 and re-offered Incorporeal in a loop).
                    let rerolled_dodge = self.re_roll_state.re_rolled_action.as_ref()
                        .map(|a| a.name == "DODGE").unwrap_or(false);
                    if rerolled_dodge && self.re_roll_state.re_roll_source.is_some() {
                        self.dodge_roll = 0;
                    }
                }
                self.execute_step(game, rng)
            }
            // Java: CLIENT_USE_SKILL with a canChooseToIgnoreDodgeModifierAfterRoll skill (Swift As
            // The Breeze) → `usingModifierIgnoringSkill = isSkillUsed(); if (!used)
            // setReRollSource(findSkillReRollSource(DODGE))` — the source is only SET; the re-entry
            // gate spends it. EXECUTE_STEP (star tier: Jordell Freshbreeze, slann_fumbbl vs
            // wood_elf bb2020 seed 1).
            Action::UseSkill { skill_id, use_skill }
                if skill_id.properties().contains(
                    &ffb_model::model::property::named_properties::NamedProperties::CAN_CHOOSE_TO_IGNORE_DODGE_MODIFIER_AFTER_ROLL) =>
            {
                self.using_modifier_ignoring_skill = Some(*use_skill);
                if !*use_skill {
                    self.re_roll_state.re_roll_source = find_skill_reroll_source(game, "DODGE");
                }
                self.execute_step(game, rng)
            }
            // client-only: CLIENT_USE_SKILL → canRerollDodge — headless auto-declines.
            // Java: CLIENT_PLAYER_CHOICE (ARM_BAR mode) → armBarPlayerId = command.getPlayerId();
            // armBarChoice = true; EXECUTE_STEP. The agent's generic PlayerChoice decline sends
            // SelectPlayer with an empty id (no arm-bar player chosen).
            Action::SelectPlayer { player_id } => {
                self.arm_bar_player_id = Some(player_id.clone());
                self.arm_bar_choice = true;
                self.execute_step(game, rng)
            }
            Action::PlayerChoice { player_id, .. } => {
                self.arm_bar_player_id = Some(player_id.clone().unwrap_or_default());
                self.arm_bar_choice = true;
                self.execute_step(game, rng)
            }
            _ => self.execute_step(game, rng),
        }
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(v) => { self.goto_label_on_failure = v.clone(); true }
            StepParameter::CoordinateFrom(v) => { self.coordinate_from = *v; true }
            StepParameter::CoordinateTo(v) => { self.coordinate_to = Some(*v); true }
            StepParameter::DodgeRoll(v) => { self.dodge_roll = *v; true }
            StepParameter::UsingDivingTackle(v) => { self.using_diving_tackle = Some(*v); true }
            StepParameter::UsingBreakTackle(v) => { self.using_break_tackle = *v; true }
            StepParameter::ReRollUsed(v) => { self.re_roll_used = *v; true }
            StepParameter::UsingModifyingSkill(v) => { self.using_modifying_skill = Some(*v); true }
            StepParameter::ArmBarPlayerId(v) => { self.arm_bar_player_id = v.clone(); true }
            StepParameter::DtRerollAsked(v) => { self.dt_reroll_asked = *v; true }
            _ => false,
        }
    }
}

impl StepMoveDodge {
    /// Java `dodge(pDoRoll)`: `if (pDoRoll) publishParameter(new StepParameter(DODGE_ROLL,
    /// rollSkill()))`. The body below has many exits (prompts, gotos, a recursive re-entry after
    /// a skill re-roll), so the publish is attached here, once, to whatever outcome the pass
    /// produced. A re-entry that rolls again overwrites the pending die, which is what Java's
    /// second `publishParameter` does to the steps on the stack.
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let mut out = self.execute_step_inner(game, rng);
        if let Some(roll) = self.dodge_roll_to_publish.take() {
            out = out.publish(StepParameter::DodgeRoll(roll));
        }
        if std::mem::take(&mut self.publish_using_modifying_skill) {
            out = out.publish(StepParameter::UsingModifyingSkill(true));
        }
        out
    }

    fn execute_step_inner(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: if (!actingPlayer.isDodging()) { setNextAction(NEXT_STEP); return; }
        if !game.acting_player.dodging {
            return StepOutcome::next();
        }

        let player_id = game.acting_player.player_id.clone();

        // Java: if (DODGE == reRolledAction && !usingModifierIgnoringSkill && !(dtRerollAsked && source==null))
        //         if (source == null || !useReRoll(...)) { failDodge(); return; } else fReRollUsed = true;
        let already_rerolled = self.re_roll_state.re_rolled_action
            .as_ref().map(|a| a.name == "DODGE").unwrap_or(false);
        let using_modifier_ignoring = self.using_modifier_ignoring_skill == Some(true);

        // Java re-entry gate: `if (DODGE == reRolledAction && !usingModifierIgnoringSkill
        // && !(dtRerollAsked && getReRollSource() == null))` — DECLINING the pre-emptive
        // Diving-Tackle re-roll keeps the (successful) original roll and proceeds to the DT
        // decision; it must not consume anything or fail the dodge.
        // Java nests the consume under `if (usingModifyingSkill == null)`: once the Incorporeal
        // dialog is answered, the re-roll (if declined) was already spent in handleCommand.
        if already_rerolled && !using_modifier_ignoring && self.using_modifying_skill.is_none()
            && !(self.dt_reroll_asked && self.re_roll_state.re_roll_source.is_none()) {
            let pid = player_id.as_deref().unwrap_or("").to_owned();
            let source_opt = self.re_roll_state.re_roll_source.clone();
            if std::env::var_os("FFB_TRACE").is_some() {
                // `RMDRR`: the re-entry re-roll attempt -- source, bank, and the roll state.
                eprintln!("RMDRR pid={pid} source={source_opt:?} bank={} dodge_roll={} arm_bar_choice={}",
                    game.turn_data().rerolls, self.dodge_roll, self.arm_bar_choice);
            }
            let consumed = source_opt
                .as_ref()
                .map(|s| use_reroll(game, s, &pid, rng))
                .unwrap_or(false);
            if consumed {
                self.re_roll_used = true;
                // Java: `doRoll = reRolledAction && source != null` → `dodge(true)` rolls a FRESH
                // die on EVERY executeStep entry in this state — including the ARM_BAR PlayerChoice
                // re-entry, where the stale reRollSource re-triggers useReRoll (a SECOND team
                // re-roll + Loner) and the dodge is rolled again (chaos bb2025 seed 28 i=60: Java
                // dodge 2-fail → TRR+Loner → 2-fail → ARM_BAR answer → TRR+Loner AGAIN → fresh 5
                // SUCCEEDS and the move continues; Rust reused the stale 2 and fell). Resetting
                // here is Java's dodge(doRoll=true), not just the offer-time reset.
                self.dodge_roll = 0;
            } else {
                // Java: `if (usingModifyingSkill != null || !showUseModifyingSkillDialog(...) ||
                // !TRUE.equals(usingModifierIgnoringSkill)) failDodge();` — the dialog probe calls
                // `getModifyingSkillInCaseItHelps(mechanic, dodgeModifiers, addReport=true)`, which
                // writes ReportModifiedDodgeResultSuccessful(player.getSkillWithProperty(
                // canAddStrengthToDodge)) — null for most players — whenever the stale roll beats
                // the last minimum (slann bb2025 seed 15: accepted DT re-roll, Loner fails).
                // The third disjunct is always true here, so the dodge fails either way.
                if self.using_modifying_skill.is_none() && self.last_minimum_roll > 0
                    && DiceInterpreter::is_skill_roll_successful(self.dodge_roll, self.last_minimum_roll)
                {
                    let skill = game.player(&pid).and_then(|p|
                        p.skill_id_with_property_in(game.rules, ffb_model::model::property::named_properties::NamedProperties::CAN_ADD_STRENGTH_TO_DODGE));
                    game.report_list.add(
                        ffb_model::report::mixed::report_modified_dodge_result_successful::ReportModifiedDodgeResultSuccessful::new(skill));
                    // Java showUseModifyingSkillDialog: if (modifyingSkill != null) UtilServerDialog.showDialog(
                    //   getGameState(), new DialogSkillUseParameter(actingPlayerId, modifyingSkill, 0), false)
                    if let Some(skill) = skill {
                        crate::util::util_server_dialog::UtilServerDialog::show_dialog(
                            game,
                            ffb_model::dialog::dialog_parameter_factory::DialogParameter::SkillUse(
                                ffb_model::dialog::dialog_skill_use_parameter::DialogSkillUseParameter {
                                    player_id: Some(pid.clone()),
                                    skill: Some(skill),
                                    minimum_roll: 0,
                                    ..Default::default()
                                },
                            ),
                            false,
                        );
                    }
                }
                return self.fail_dodge(game);
            }
        }

        // Java: `doRoll = (reRolledAction || (fUsingDivingTackle == null && !dtRerollAsked))
        //   && !usingModifyingSkill && !usingModifierIgnoringSkill` — and `dodge(pDoRoll)` writes
        // its ReportDodgeRoll (and the Break-Tackle WOULD_NOT_HELP report) only when pDoRoll.
        // A re-entry after a DECLINED pre-emptive Diving-Tackle re-roll re-evaluates the same die
        // without reporting it again (dwarf_vs_human bb2025 seed 2: Rust reported it twice).
        let re_rolled_action_with_source = self.re_roll_state.re_rolled_action.as_ref()
            .map(|a| a.name == "DODGE").unwrap_or(false)
            && self.re_roll_state.re_roll_source.is_some();
        let do_roll = (re_rolled_action_with_source
                || (self.using_diving_tackle.is_none() && !self.dt_reroll_asked))
            && self.using_modifying_skill != Some(true)
            && self.using_modifier_ignoring_skill != Some(true);

        // Java: dodge(doRoll) → roll and check; a fresh die is published as DODGE_ROLL.
        if self.dodge_roll == 0 {
            self.dodge_roll = rng.d6();
            self.dodge_roll_to_publish = Some(self.dodge_roll);
        }

        // Java `dodge()`: `minimumRollWithoutModifiers = mechanic.minimumRollDodge(game, player,
        // emptySet()); successWithoutModifiers = isSkillRollSuccessful(fDodgeRoll, …)`. When the
        // bare roll passes, an UNUSED canChooseToIgnoreDodgeModifierAfterRoll skill (Swift As The
        // Breeze) may be chosen; once chosen it passes the dodge outright, once declined (with no
        // re-roll source left) the dodge fails.
        let ignore_modifier_skill: Option<ffb_model::enums::SkillId> = {
            let bare_min = player_id.as_deref().and_then(|pid| game.player(pid))
                .map(|p| DodgeModifierFactory::minimum_roll(p.agility as i32, &[]))
                .unwrap_or(2);
            if DiceInterpreter::is_skill_roll_successful(self.dodge_roll, bare_min) {
                player_id.as_deref().and_then(|pid| game.player(pid)).and_then(|p| {
                    p.all_skill_ids().find(|id| {
                        id.properties().contains(
                            &ffb_model::model::property::named_properties::NamedProperties::CAN_CHOOSE_TO_IGNORE_DODGE_MODIFIER_AFTER_ROLL)
                            && !p.used_skills.contains(id)
                            && !game.acting_player.used_skills.contains(id)
                    })
                })
            } else {
                None
            }
        };
        // `successWithoutModifiers` survives only when the skill is present.
        let success_without_modifiers = ignore_modifier_skill.is_some();
        if let Some(skill) = ignore_modifier_skill {
            if self.using_modifier_ignoring_skill == Some(true) {
                // Java: `actingPlayer.markSkillUsed(skill); addReport(ReportSkillUse(pid, skill,
                // true, PASS_DODGE_WITHOUT_MODIFIERS)); return SUCCESS` — no ReportDodgeRoll.
                if let Some(pid) = player_id.as_deref() {
                    if let Some(p) = game.team_home.player_mut(pid).or_else(|| game.team_away.player_mut(pid)) {
                        p.used_skills.insert(skill);
                    }
                    game.acting_player.used_skills.insert(skill);
                    game.report_list.add(ffb_model::report::report_skill_use::ReportSkillUse::new(
                        Some(pid.to_string()), skill, true,
                        ffb_model::model::skill_use::SkillUse::PASS_DODGE_WITHOUT_MODIFIERS,
                    ));
                }
                let re_rolled = self.re_roll_state.re_rolled_action.as_ref()
                    .map(|a| a.name == "DODGE").unwrap_or(false)
                    && self.re_roll_state.re_roll_source.is_some();
                return StepOutcome::next()
                    .publish(StepParameter::ReRollUsed(self.re_roll_used || re_rolled));
            }
            // Java: `if (FALSE.equals(usingModifierIgnoringSkill) && getReRollSource() == null &&
            // getReRolledAction() != null) return FAILURE;`
            if self.using_modifier_ignoring_skill == Some(false)
                && self.re_roll_state.re_roll_source.is_none()
                && self.re_roll_state.re_rolled_action.is_some()
            {
                if game.options.is_enabled("standFirmNoDropOnFailedDodge") {
                    return StepOutcome::next().publish(StepParameter::EndPlayerAction(true));
                }
                return self.fail_dodge(game);
            }
        }

        // Java `actingPlayer.statBasedModifier(canAddStrengthToDodge)`: the skill's
        // StatBasedRollModifier (Incorporeal → the player's ST) while the skill is UNUSED, else null.
        let unused_strength_skill = |game: &Game| -> Option<(ffb_model::enums::SkillId, i32)> {
            let pid = game.acting_player.player_id.as_deref()?;
            let p = game.player(pid)?;
            let skill = p.skill_id_with_property_in(game.rules,
                ffb_model::model::property::named_properties::NamedProperties::CAN_ADD_STRENGTH_TO_DODGE)?;
            if p.used_skills.contains(&skill) || game.acting_player.used_skills.contains(&skill) {
                return None;
            }
            Some((skill, p.strength_with_modifiers() as i32))
        };
        // Java `dodge()`: `if (TRUE.equals(usingModifyingSkill)) { statBasedRollModifier = …;
        // actingPlayer.markSkillUsed(canAddStrengthToDodge); addReport(ReportSkillUse(pid, skill,
        // true, ADD_STRENGTH_TO_ROLL)); publishParameter(USING_MODIFYING_SKILL, true); }`
        let stat_used: Option<(ffb_model::enums::SkillId, i32)> = if self.using_modifying_skill == Some(true) {
            unused_strength_skill(game)
        } else {
            None
        };
        if let Some((skill, _)) = stat_used {
            if let Some(pid) = player_id.as_deref() {
                if let Some(p) = game.team_home.player_mut(pid).or_else(|| game.team_away.player_mut(pid)) {
                    p.used_skills.insert(skill);
                }
                game.acting_player.used_skills.insert(skill);
                game.report_list.add(ffb_model::report::report_skill_use::ReportSkillUse::new(
                    Some(pid.to_string()), skill, true,
                    ffb_model::model::skill_use::SkillUse::ADD_STRENGTH_TO_ROLL,
                ));
            }
            self.publish_using_modifying_skill = true;
        }
        let stat_value = stat_used.map(|(_, st)| st).unwrap_or(0);

        let factory = DodgeModifierFactory::for_rules(game.rules);
        // `agility` and the UNCLAMPED modifier sums travel out of this block alongside the
        // clamped minima: Java re-enters `AgilityMechanic.minimumRollDodge` with the augmented
        // modifier SET whenever it wants a what-if minimum (Diving Tackle, Break Tackle), so the
        // `Math.max(2, ...)` clamp is applied exactly once, to `agility + everything`. Adding a
        // modifier to an already-clamped minimum is a different function whenever the clamp binds.
        let (minimum_roll, mod_names, has_bt, min_no_bt, names_no_bt, dodge_agility, total_all, total_no_bt):
            (i32, Vec<String>, bool, i32, Vec<String>, i32, i32, i32) = if let Some(pid) = player_id.as_deref() {
            let acting = game.acting_player.clone();
            let src = self.coordinate_from.unwrap_or(FieldCoordinate::new(0, 0));
            let tgt = self.coordinate_to.unwrap_or(FieldCoordinate::new(0, 0));
            // Java: `modifierFactory.findModifiers(new DodgeContext(game, actingPlayer,
            //   fCoordinateFrom, fCoordinateTo, fUsingBreakTackle))` — once Break Tackle is
            // committed (fUsingBreakTackle), it applies to EVERY remaining dodge of the move
            // via the context's isUseBreakTackle() disjunct, even though the skill is marked
            // used (dwarf bb2025 @1e6 seed 57 i=118: Java's second dodge keeps BT, min 2;
            // Rust's factory saw only used_skills and dropped it, min 5 → fall → turnover).
            let ctx = DodgeContext::new_with_break_tackle(game, &acting, src, tgt, self.using_break_tackle);
            let mods = factory.find_applicable(&ctx);
            let skill_mods = factory.find_skill_modifiers(&ctx);
            let mut all: Vec<&ffb_mechanics::modifiers::dodge_modifier::DodgeModifier> = mods.iter().copied().chain(skill_mods.iter()).collect();
            // Java: `if ((fUsingDivingTackle != null) && fUsingDivingTackle)
            //   dodgeModifiers.addAll(modifierFactory.forType(ModifierType.DIVING_TACKLE));` —
            // the RETRY_DODGE re-evaluation after an accepted Diving Tackle. Missing, so a tackled
            // dodger kept moving (dwarf bb2025 seed 3 i=3: Java drops home_11, Rust ran on).
            let dt_modifier = ffb_mechanics::modifiers::dodge_modifier::DodgeModifier::new(
                "Diving Tackle", 2, ffb_mechanics::modifiers::modifier_type::ModifierType::DIVING_TACKLE);
            if self.using_diving_tackle == Some(true) {
                all.push(&dt_modifier);
            }
            let agility = game.player(pid).map(|p| p.agility as i32).unwrap_or(3);
            let min = DodgeModifierFactory::minimum_roll(agility, &all);
            let names: Vec<String> = all.iter().map(|m| m.get_name().to_string()).collect();
            // Java: `Optional<DodgeModifier> btModifier = dodgeModifiers.stream()
            //   .filter(DodgeModifier::isUseStrength).findFirst()` — the Break Tackle axis.
            let without_bt: Vec<&ffb_mechanics::modifiers::dodge_modifier::DodgeModifier> =
                all.iter().copied().filter(|m| !m.use_strength).collect();
            let has_bt = without_bt.len() != all.len();
            let min_no_bt = DodgeModifierFactory::minimum_roll(agility, &without_bt);
            let names_no_bt: Vec<String> = without_bt.iter().map(|m| m.get_name().to_string()).collect();
            let total_all: i32 = all.iter().map(|m| m.get_modifier()).sum();
            let total_no_bt: i32 = without_bt.iter().map(|m| m.get_modifier()).sum();
            (min, names, has_bt, min_no_bt, names_no_bt, agility, total_all, total_no_bt)
        } else {
            (2, vec![], false, 2, vec![], 2, 0, 0)
        };
        // Java `mechanic.minimumRollDodge(game, player, mods, statBasedRollModifier)` =
        // max(2, agility + sum(mods) - stat).
        let (minimum_roll, min_no_bt) = if stat_value != 0 {
            ((dodge_agility + total_all - stat_value).max(2), (dodge_agility + total_no_bt - stat_value).max(2))
        } else {
            (minimum_roll, min_no_bt)
        };
        // Java `getModifyingSkillInCaseItHelps(mechanic, dodgeModifiers, false)` (failure branch,
        // BEFORE the Break Tackle removal): the unused canAddStrengthToDodge skill when adding its
        // stat would turn this roll into a success.
        let modifying_skill: Option<ffb_model::enums::SkillId> = unused_strength_skill(game).and_then(|(skill, st)| {
            let with_stat = (dodge_agility + total_all - st).max(2);
            let failed = !DiceInterpreter::is_skill_roll_successful(self.dodge_roll, minimum_roll);
            (failed && DiceInterpreter::is_skill_roll_successful(self.dodge_roll, with_stat)).then_some(skill)
        });
        // Java StepMoveDodge (the btModifier block after `successful` is computed):
        // - SUCCESS + BT present: recompute WITHOUT Break Tackle; if the roll still succeeds, BT
        //   was not needed — the reported minimum drops and BT is NOT consumed. Otherwise BT is
        //   what saved the dodge: `fUsingBreakTackle = true; actingPlayer.markSkillUsed(btSkill)`
        //   (bb2025:516-521) — ONE Break Tackle per activation. Rust never marked it, so the
        //   Deathroller's SECOND dodge of the activation got -3 again where Java rolled bare
        //   (dwarf bb2025 seed 43 k=31: J min=6 mods=1 Tacklezone, R min=3 with BT ST 5+).
        // - FAILURE + BT present: Java removes the modifier before reporting (WOULD_NOT_HELP);
        //   BT is not consumed.
        let bt_saved_it = has_bt
            && DiceInterpreter::is_skill_roll_successful(self.dodge_roll, minimum_roll)
            && !DiceInterpreter::is_skill_roll_successful(self.dodge_roll, min_no_bt);
        // Java (failure branch, no modifying skill): remove the BT modifier and, when Break
        // Tackle is not already committed this activation, `addReport(new ReportSkillUse(null,
        // btSkill, false, WOULD_NOT_HELP))` — BEFORE the ReportDodgeRoll (dwarf_vs_orc bb2020 s3).
        // With a modifying skill Java takes the other arm (`if (modifyingSkill != null)`): no
        // WOULD_NOT_HELP report and `minimumRoll` keeps the Break Tackle modifier.
        if do_roll && has_bt && !self.using_break_tackle && modifying_skill.is_none()
            && !DiceInterpreter::is_skill_roll_successful(self.dodge_roll, minimum_roll)
        {
            game.report_list.add(ffb_model::report::report_skill_use::ReportSkillUse::new(
                None, ffb_model::enums::SkillId::BreakTackle, false,
                ffb_model::model::skill_use::SkillUse::WOULD_NOT_HELP,
            ));
        }
        let (minimum_roll, mod_names, dodge_total) = if has_bt && !bt_saved_it && modifying_skill.is_none() {
            (min_no_bt, names_no_bt, total_no_bt)
        } else {
            (minimum_roll, mod_names, total_all)
        };
        if bt_saved_it {
            self.using_break_tackle = true;
            if let Some(pid) = player_id.as_deref() {
                // Rust convention: the factory gate reads Player.used_skills; acted() reads the
                // acting set — write both (the keg/Zzharg lesson).
                if let Some(p) = game.team_home.player_mut(pid).or_else(|| game.team_away.player_mut(pid)) {
                    p.used_skills.insert(ffb_model::enums::SkillId::BreakTackle);
                }
                game.acting_player.used_skills.insert(ffb_model::enums::SkillId::BreakTackle);
            }
        }
        let successful = DiceInterpreter::is_skill_roll_successful(self.dodge_roll, minimum_roll);
        self.last_minimum_roll = minimum_roll;
        if std::env::var_os("FFB_TRACE").is_some() {
            eprintln!("RDODGEMIN pid={} roll={} min={} from={:?} to={:?} mods={:?} ok={}",
                player_id.as_deref().unwrap_or("-"), self.dodge_roll, minimum_roll,
                self.coordinate_from, self.coordinate_to, mod_names, successful);
        }

        // Java line 333-335: addReport(new ReportDodgeRoll(...))
        let re_rolled = self.re_roll_state.re_rolled_action.as_ref()
            .map(|a| a.name == "DODGE").unwrap_or(false)
            && self.re_roll_state.re_roll_source.is_some();
        // Java bb2025 reports only `if (pDoRoll)`; Java bb2020 (`StepMoveDodge:334`) reports
        // every evaluation, with `(pDoRoll ? fDodgeRoll : 0)` — the Diving Tackle re-check shows
        // up as roll 0 (slann bb2020 seed 3 i=35).
        let is_bb2020 = game.rules == ffb_model::enums::Rules::Bb2020;
        if do_roll || is_bb2020 {
            use ffb_model::report::mixed::report_dodge_roll::ReportDodgeRoll;
            let mut report = ReportDodgeRoll::new(
                player_id.clone(),
                successful,
                if do_roll { self.dodge_roll } else { 0 },
                minimum_roll,
                re_rolled,
                mod_names,
                None,
            );
            if let Some((skill, st)) = stat_used {
                report = report.with_stat_value(skill.category_and_name_for(game.rules).1.to_string(), st);
            }
            game.report_list.add(report);
        }
        // Emit one GameEvent per resolved roll (monolith parity: initial roll and
        // re-rolled resolution each produce their own DodgeRoll event).
        let roll_event = ffb_model::events::GameEvent::DodgeRoll {
            player_id: player_id.clone().unwrap_or_default(),
            target: minimum_roll,
            roll: self.dodge_roll,
            success: successful,
            rerolled: re_rolled,
        };

        if successful {
            // Java bb2025 StepMoveDodge (the success branch, :439-475): when the dodge SUCCEEDS
            // but an eligible Diving Tackler could flip it (the DIVING_TACKLE modifier is a flat
            // +2, so minWithDt = minimumRoll + 2), Java first tries an unused Break Tackle rescue,
            // then PRE-EMPTIVELY OFFERS A RE-ROLL ("Diving Tackle can make this dodge fail.
            // Reroll the dodge now?") — the heuristic answers it through its reroll sampler and
            // an accept spends the TRR and rolls a fresh die (dwarf bb2025 seed 3 i=164: Java
            // dodge 4-success → DT threat → TRR accepted → fresh 6, r 2→1; Rust sailed on with
            // one die and the streams split). dtRerollAsked stops a second ask.
            //
            // BB2025 ONLY. `driver::make_step_for` has no `Rules::Bb2020` arm for `MoveDodge`, so
            // BB2020 games run THIS step — but the BB2020 Java StepMoveDodge's success branch is
            // bare `status = ActionStatus.SUCCESS`: it never calls `findEligibleDivingTacklers`
            // and never pre-emptively offers a re-roll (grep the bb2020 file — `DIVING_TACKLE`
            // appears only in the `fUsingDivingTackle` modifier path, never in a what-if). So for
            // BB2020 this block invented a re-roll offer Java does not make, the heuristic
            // accepted it, and the fresh die replaced a dodge that had already SUCCEEDED
            // (slann bb2020 @0 seed 3 i=18: a Blitzer's 5 beat min 4, Rust re-rolled it to a 1,
            // fell prone and handed over the turn). Edition-gate the ONE differing behaviour
            // inside the shared step rather than routing BB2020 to its staler twin.
            if game.rules == ffb_model::enums::Rules::Bb2025
                && self.using_diving_tackle.is_none() && !self.dt_reroll_asked {
                let from = self.coordinate_from.unwrap_or(FieldCoordinate::new(0, 0));
                let to = self.coordinate_to.unwrap_or(FieldCoordinate::new(0, 0));
                let leaving_tz_only = ffb_model::option::util_game_option::is_option_enabled(
                    game, ffb_model::option::game_option_id::DIVING_TACKLE_LEAVING_TZ_ONLY);
                let dt_tacklers = ffb_model::util::util_player::UtilPlayer::find_eligible_diving_tacklers(
                    game, from, to, leaving_tz_only);
                if !dt_tacklers.is_empty() {
                    // Java: `withDt = new HashSet<>(dodgeModifiers); withDt.addAll(forType(
                    // DIVING_TACKLE)); minimumWithDt = mechanic.minimumRollDodge(game, player,
                    // withDt, statBasedRollModifier)` — the DIVING_TACKLE DodgeModifier is +2 and
                    // the mechanic is `Math.max(2, agility + sum(modifiers))`. So the +2 goes in
                    // BEFORE the clamp, not after it. `minimum_roll + 2` is the same number only
                    // when the clamp does not bind; for an AG1 Kroxigor dodging out of a bare
                    // square Java gets max(2, 1+2) = 3 where the old code got max(2,1)+2 = 4, so a
                    // dodge roll of 3 read as "Diving Tackle can flip this" and Rust pre-emptively
                    // offered — and the heuristic accepted — a team re-roll Java never asked for
                    // (slann bb2025 @0 seed 43 i=85: two extra dice and r2,2 → r1,2).
                    let min_with_dt = (dodge_agility + dodge_total + 2).max(2);
                    let mut fails_with_dt =
                        !DiceInterpreter::is_skill_roll_successful(self.dodge_roll, min_with_dt);
                    if fails_with_dt && !self.using_break_tackle {
                        // Java: an UNUSED canAddStrengthToDodge that makes minWithDtBt succeed is
                        // consumed on the spot (fUsingBreakTackle + markSkillUsed + publish).
                        let bt_mod = player_id.as_deref().and_then(|pid| game.player(pid)).and_then(|p| {
                            use ffb_model::enums::SkillId;
                            if p.has_skill(SkillId::BreakTackle)
                                && !p.used_skills.contains(&SkillId::BreakTackle)
                            {
                                let st = p.strength_with_modifiers();
                                Some(if st >= 5 { -3 } else if st == 4 { -2 } else { -1 })
                            } else {
                                None
                            }
                        });
                        if let Some(bt) = bt_mod {
                            // Java: `minimumWithDtBt = mechanic.minimumRollDodge(game, player,
                            // withDtAndBt, btStat)` = max(2, agility + sum + 2 - btStat). Same
                            // clamp-once rule as `min_with_dt` above; `bt` is the already-negated
                            // strength modifier (-3/-2/-1).
                            let min_with_dt_bt = (dodge_agility + dodge_total + 2 + bt).max(2);
                            if DiceInterpreter::is_skill_roll_successful(self.dodge_roll, min_with_dt_bt) {
                                self.using_break_tackle = true;
                                if let Some(pid) = player_id.as_deref() {
                                    if let Some(p) = game.team_home.player_mut(pid)
                                        .or_else(|| game.team_away.player_mut(pid))
                                    {
                                        p.used_skills.insert(ffb_model::enums::SkillId::BreakTackle);
                                    }
                                    game.acting_player.used_skills.insert(ffb_model::enums::SkillId::BreakTackle);
                                }
                                fails_with_dt = false;
                            }
                        }
                    }
                    if fails_with_dt && !self.re_roll_used {
                        // Java's DT what-if calls the PLAYER overload with an EXPLICIT re-roll
                        // skill: `askForReRollIfAvailable(gameState, actingPlayer.getPlayer(),
                        // DODGE, minimumWithDt, false, null, rerollSkill, null, null, message)`
                        // where `rerollSkill = uncanceledDodgeRerollSource(game, actingPlayer)`.
                        // Rust called the ACTING-PLAYER overload, which looks a skill source up by
                        // action WITHOUT the Tackle cancellation the normal failure path below
                        // applies -- so on a dodge next to a player carrying BOTH Diving Tackle and
                        // Tackle (the OWA Dwarf Blitzer, and the dwarf Blitzer), Rust offered a
                        // Dodge-skill re-roll that Java had cancelled. Java showed NO dialog and
                        // spent NO draw; Rust spent one, and every decision after that diverged
                        // (old_world_alliance bb2025 @1e6 seeds 68 and 93).
                        // Java: `rerollSource = uncanceledDodgeRerollSource(game, actingPlayer)`
                        // and the dialog is shown when `teamReRollOption || reRollSkill != null`.
                        // So the SKILL term still counts -- but only when it is UNCANCELLED. Tackle
                        // registers CancelSkillProperty(canRerollDodge), so a dodge next to a player
                        // carrying both Diving Tackle and Tackle (the OWA Dwarf Blitzer) has NO skill
                        // source and falls back to the team-only question, which Java does not ask
                        // when the bank is unavailable. Using the acting-player overload
                        // unconditionally offered a cancelled Dodge re-roll (OWA seeds 68/93);
                        // dropping the skill term entirely broke slann bb2025 @1e6, where the source
                        // is uncancelled and Java DOES ask. Both terms are needed.
                        let uncancelled_dodge_source = find_skill_reroll_source(game, "DODGE")
                            .filter(|_| {
                                use ffb_model::util::util_player::UtilPlayer;
                                use ffb_model::model::property::NamedProperties;
                                let acting = game.acting_player.player_id.clone().unwrap_or_default();
                                let from_sq = self.coordinate_from
                                    .unwrap_or_else(|| FieldCoordinate::new(0, 0));
                                UtilPlayer::find_adjacent_opposing_players_with_property(
                                    game, &acting, from_sq,
                                    NamedProperties::CANCELS_CAN_REROLL_DODGE, false,
                                ).is_empty()
                            });
                        let dt_prompt = if uncancelled_dodge_source.is_some() {
                            ask_for_reroll_if_available(game, "DODGE", min_with_dt, false)
                        } else {
                            ask_for_reroll_if_available_for(
                                game, player_id.as_deref(), "DODGE", min_with_dt, false)
                        };
                        if let Some(prompt) = dt_prompt {
                            self.dt_reroll_asked = true;
                            use ffb_model::model::re_rolled_action::ReRolledAction;
                            self.re_roll_state.re_rolled_action = Some(ReRolledAction::new("DODGE"));
                            self.re_roll_state.re_roll_source = Some(ReRollSource::new("TRR"));
                            return StepOutcome::cont().with_prompt(prompt).with_event(roll_event);
                        }
                    }
                    // Reaching here: DT can't flip it, BT saved it, or no re-roll available —
                    // the DT usage decision itself follows in StepDivingTackle.
                }
            }
            StepOutcome::next()
                .with_event(roll_event)
                .publish(StepParameter::ReRollUsed(self.re_roll_used || re_rolled))
                .publish(StepParameter::UsingBreakTackle(self.using_break_tackle))
        } else {
            // Try re-roll on first failure. Java bb2025: `if (fUsingDivingTackle == null &&
            // !fReRollUsed && (getReRolledAction() != DODGE || …))` — the RETRY_DODGE
            // re-evaluation after an accepted Diving Tackle is final (dwarf bb2025 seed 3 i=3:
            // Rust spent a team re-roll Java never offers). Java bb2020 (`StepMoveDodge:343`) has
            // only `!fReRollUsed && …`, so there the re-evaluation can still be re-rolled.
            let dt_blocks_reroll = game.rules == ffb_model::enums::Rules::Bb2025
                && self.using_diving_tackle.is_some();
            // Java: `(getReRolledAction() != DODGE || (successWithoutModifiers &&
            // !TRUE.equals(usingModifierIgnoringSkill)))`.
            let reroll_branch_open = !already_rerolled
                || (success_without_modifiers && self.using_modifier_ignoring_skill != Some(true));
            if reroll_branch_open && !dt_blocks_reroll && !self.re_roll_used {
                use ffb_model::model::re_rolled_action::ReRolledAction;
                self.re_roll_state.re_rolled_action = Some(ReRolledAction::new("DODGE"));

                // Skill re-roll (Dodge property canRerollDodge) — auto-used
                let skill_source = find_skill_reroll_source(game, "DODGE");
                // Java StepMoveDodge (bb2020) lines 342-354: the Dodge skill re-roll source is
                // CANCELLED when an adjacent opposing player (with tacklezones) at the FROM square
                // cancels it — `UtilCards.cancelsSkill(opponent, dodgeSkill)`; Tackle registers
                // CancelSkillProperty(canRerollDodge) (→ the cancelsCanRerollDodge property). Without
                // this, a Dodge-skill player (e.g. a Catcher) dodging next to a Tackle Blitzer wrongly
                // re-rolled its failed dodge — one extra die that broke the fall armour Java leaves
                // unbroken (human seed 13 i=255: home_04 Stunned in Rust vs Prone in Java → turnover
                // desync). Nulling the skill source falls through to the TRR offer, exactly as Java.
                let skill_source = skill_source.filter(|_| {
                    use ffb_model::util::util_player::UtilPlayer;
                    use ffb_model::model::property::NamedProperties;
                    let acting = game.acting_player.player_id.clone().unwrap_or_default();
                    let from = self.coordinate_from.unwrap_or_else(|| FieldCoordinate::new(0, 0));
                    UtilPlayer::find_adjacent_opposing_players_with_property(
                        game, &acting, from, NamedProperties::CANCELS_CAN_REROLL_DODGE, false,
                    ).is_empty()
                });
                if let Some(source) = skill_source {
                    // Java: `if (modifyingSkill != null && usingModifyingSkill == null) {
                    //   addReport(ReportModifiedDodgeResultSuccessful(modifyingSkill));
                    //   showDialog(DialogSkillUseParameter(pid, modifyingSkill, 0)); WAITING }` —
                    // Incorporeal is offered BEFORE the Dodge skill re-roll is spent.
                    if let (Some(skill), None) = (modifying_skill, self.using_modifying_skill) {
                        return self.modifying_skill_prompt(game, skill, 0).with_event(roll_event);
                    }
                    // Java: `else if (ignoreModifierSkill.isPresent()) { addReport(
                    // ReportModifiedDodgeResultSuccessful(skill)); showDialog(DialogSkillUse
                    // Parameter(pid, skill, 0)); usingModifierIgnoringSkill = null; }` — Swift As
                    // The Breeze is offered BEFORE the Dodge skill re-roll is spent.
                    if let Some(skill) = ignore_modifier_skill {
                        self.using_modifier_ignoring_skill = None;
                        return self.modifying_skill_prompt(game, skill, 0).with_event(roll_event);
                    }
                    // Java `RollMechanic.askForReRollIfAvailable` only SETS reRolledAction +
                    // reRollSource for a skill source; the single `useReRoll` happens on the
                    // re-entry (`StepMoveDodge.executeStep:229`), which is the `already_rerolled`
                    // branch above. Rust consumed here AND on re-entry — harmless to the dice
                    // (a skill re-roll rolls nothing and marking it used twice is idempotent) but
                    // it wrote TWO `ReportReRoll(Dodge)` where Java writes one (H.50).
                    self.re_roll_state.re_roll_source = Some(source);
                    self.dodge_roll = 0;
                    // Failed initial roll resolved — event goes first, re-roll events follow.
                    let mut out = self.execute_step(game, rng);
                    out.events.insert(0, roll_event);
                    return out;
                }

                // TRR offer.
                //
                // Java reaches this branch with `skillRerollSource == null` — the Dodge skill
                // re-roll is used or CANCELLED — and calls the PLAYER overload with an explicit
                // `reRollSkill` of null:
                //
                // ```java
                // UtilServerReRoll.askForReRollIfAvailable(getGameState(), actingPlayer.getPlayer(),
                //     ReRolledActions.DODGE, minimumRoll, false, modifyingSkill, null)
                // ```
                //
                // `RollMechanic.askForReRollIfAvailable` then adds exactly ONE skill term of its
                // own — `getUnusedSkillWithProperty(player, canRerollSingleDieOncePerPeriod)` —
                // and never re-derives an action-keyed source. `ask_for_reroll_if_available` is
                // Rust's ACTING-PLAYER overload: it calls `find_skill_reroll_source(game, "DODGE")`
                // again, which resurrects the very Dodge source the Tackle filter above had just
                // nulled. The offer then went out as `ReRollOffer{source: Dodge}`, the agent took
                // it, and `use_reroll` charged an empty team-re-roll bank (human bb2025 @1e6
                // seed 3 i=32: home_10, a Halfling Hopeful with Dodge, dodging out of a Tackle
                // tacklezone with `r0` — Java takes the failure and the turnover, Rust re-rolled
                // and drove the bank to `r-1`).
                // The modifying skill rides on the same dialog (`askForReRollIfAvailable(…,
                // modifyingSkill, null)`): `dialogShown` counts `modificationSkill != null`.
                // bb2025's DialogReRollPropertiesParameter always reaches the coach; bb2020's
                // DialogReRollParameter is answered null (no draw) by the harness when the team
                // re-roll is not an option — the dodge then fails on re-entry.
                // Java: `if (modifyingSkill == null && ignoreModifierSkill.isPresent())
                // modifyingSkill = ignoreModifierSkill.get();` — it rides on the same dialog.
                let ignore_rides = modifying_skill.is_none() && ignore_modifier_skill.is_some();
                let modifying_skill = modifying_skill.or(ignore_modifier_skill);
                let mut prompt = ask_for_reroll_if_available_for(
                    game, player_id.as_deref(), "DODGE", minimum_roll, false);
                if prompt.is_none() && modifying_skill.is_some()
                    && game.rules == ffb_model::enums::Rules::Bb2025
                {
                    let team_id = if game.home_playing { game.team_home.id.clone() } else { game.team_away.id.clone() };
                    prompt = Some(ffb_model::prompts::AgentPrompt::ReRollOffer {
                        source: ReRollSource::new("TRR"),
                        action: "DODGE".to_owned(),
                        re_roll_skill: None, team_re_roll_option: true,
                        team_id,
                    });
                }
                if let Some(skill) = modifying_skill {
                    game.report_list.add(
                        ffb_model::report::mixed::report_modified_dodge_result_successful::ReportModifiedDodgeResultSuccessful::new(Some(skill)));
                }
                if let Some(prompt) = prompt {
                    if ignore_rides {
                        self.using_modifier_ignoring_skill = None;
                    }
                    self.re_roll_state.re_roll_source = Some(ReRollSource::new("TRR"));
                    self.dodge_roll = 0; // reset so the re-roll gets a fresh d6
                    return StepOutcome::cont().with_prompt(prompt).with_event(roll_event);
                }
                if modifying_skill.is_some() {
                    // bb2020: dialog shown, answered null → `failDodge()` on re-entry.
                    return self.fail_dodge(game).with_event(roll_event);
                }
            } else if let Some(skill) = modifying_skill {
                // Java: `else if (modifyingSkill != null) { addReport(ReportModifiedDodgeResult
                // Successful); showDialog(DialogSkillUseParameter(pid, modifyingSkill, minimumRoll)) }`
                // — e.g. after Incorporeal was DECLINED, the Dodge re-roll spent and the fresh die
                // failed again.
                return self.modifying_skill_prompt(game, skill, minimum_roll).with_event(roll_event);
            } else if let Some(skill) = ignore_modifier_skill {
                // Java: `else if (ignoreModifierSkill.isPresent()) { …report; showDialog(…);
                // usingModifierIgnoringSkill = null; }`
                self.using_modifier_ignoring_skill = None;
                return self.modifying_skill_prompt(game, skill, 0).with_event(roll_event);
            }

            // Java: if (UtilGameOption.isOptionEnabled(game, GameOptionId.STAND_FIRM_NO_DROP_ON_FAILED_DODGE))
            if game.options.is_enabled("standFirmNoDropOnFailedDodge") {
                return StepOutcome::next()
                    .with_event(roll_event)
                    .publish(StepParameter::EndPlayerAction(true));
            }
            self.fail_dodge(game).with_event(roll_event)
        }
    }

    /// Java: `addReport(new ReportModifiedDodgeResultSuccessful(modifyingSkill));
    /// UtilServerDialog.showDialog(DialogSkillUseParameter(pid, modifyingSkill, …))` — WAITING.
    fn modifying_skill_prompt(&mut self, game: &mut Game, skill: ffb_model::enums::SkillId, minimum_roll: i32) -> StepOutcome {
        game.report_list.add(
            ffb_model::report::mixed::report_modified_dodge_result_successful::ReportModifiedDodgeResultSuccessful::new(Some(skill)));
        let pid = game.acting_player.player_id.clone().unwrap_or_default();
        // Java: UtilServerDialog.showDialog(getGameState(),
        //   new DialogSkillUseParameter(actingPlayer.getPlayerId(), skill, minimumRoll), false)
        crate::util::util_server_dialog::UtilServerDialog::show_dialog(
            game,
            ffb_model::dialog::dialog_parameter_factory::DialogParameter::SkillUse(
                ffb_model::dialog::dialog_skill_use_parameter::DialogSkillUseParameter {
                    player_id: Some(pid.clone()),
                    skill: Some(skill),
                    minimum_roll,
                    ..Default::default()
                },
            ),
            false,
        );
        StepOutcome::cont().with_prompt(ffb_model::prompts::AgentPrompt::SkillUse {
            player_id: pid,
            skill_id: skill as u16,
            skill_name: format!("{:?}", skill),
        })
    }

    fn fail_dodge(&mut self, game: &mut Game) -> StepOutcome {
        use ffb_model::util::util_player::UtilPlayer;
        use ffb_model::model::property::NamedProperties;
        // BB2020 runs this shared step (its own twin is dead — driver.rs routes MoveDodge here for
        // both editions), and Java's bb2020 `StepMoveDodge.failDodge()` is two lines:
        //   publishParameter(INJURY_TYPE, new InjuryTypeDropDodge(game.getDefender()));
        //   GOTO fGotoLabelOnFailure
        // No Arm Bar dialog exists in bb2020 — `InjuryTypeDropDodge(divingTackler)` is built with
        // useArmBarModifiers=true and finds the adjacent Arm Bar players itself. Showing bb2025's
        // ARM_BAR PlayerChoice here did two wrong things: with two adjacent Arm Bar players it
        // asked, and the answer re-entered `execute_step` with the stale TRR source still set, so
        // the team re-roll was spent AGAIN (Loner + fresh dodge: old_world_alliance_ogre bb2020 @0
        // seed 29 i=24 — Rust dwarf blitzer stands on a second re-roll where Java is prone, two
        // dice ahead, one re-roll short). Reachable only once formations put two Arm Bar dwarfs
        // next to one dodge, which the heuristic setup does and the canonical never did.
        if game.rules == ffb_model::enums::Rules::Bb2020 {
            let name = match game.defender_id.as_deref() {
                Some(d) if !d.is_empty() => format!("InjuryTypeDropDodge#dt:{d}"),
                _ => "InjuryTypeDropDodge".to_string(),
            };
            let ctx = SteadyFootingContext::from_injury_type_name(name);
            let label = self.goto_label_on_failure.clone();
            return StepOutcome::goto(&label)
                .publish(StepParameter::SteadyFootingContext(Box::new(ctx)));
        }
        // Java: armBarPlayers = findAdjacentOpposingPlayersWithProperty(game, fCoordinateFrom,
        //   affectsEitherArmourOrInjuryOnDodge, true); filterThrower(...)  — computed once.
        if self.arm_bar_players.is_empty() {
            if let (Some(from), Some(acting)) = (self.coordinate_from, game.acting_player.player_id.clone()) {
                let found = UtilPlayer::find_adjacent_opposing_players_with_property(
                    game, &acting, from, NamedProperties::AFFECTS_EITHER_ARMOUR_OR_INJURY_ON_DODGE, true,
                );
                self.arm_bar_players = UtilPlayer::filter_thrower(game, found)
                    .into_iter().cloned().collect();
            }
        }

        if std::env::var_os("FFB_TRACE").is_some() {
            // `RARMBAR`: what the Arm Bar search saw -- the from-square and the candidates.
            eprintln!(
                "RARMBAR from={:?} acting={:?} found={:?} choice={} chosen_id={:?}",
                self.coordinate_from, game.acting_player.player_id,
                self.arm_bar_players, self.arm_bar_choice, self.arm_bar_player_id
            );
        }
        // Java: armBarPlayer resolution — explicit choice > single candidate > coach dialog.
        let mut arm_bar_player: Option<String> = None;
        if let Some(ref pid) = self.arm_bar_player_id {
            if !pid.is_empty() {
                arm_bar_player = Some(pid.clone());
            }
        } else if !self.arm_bar_choice && !self.arm_bar_players.is_empty() {
            if self.arm_bar_players.len() == 1 {
                arm_bar_player = Some(self.arm_bar_players[0].clone());
            } else {
                // Java: DialogPlayerChoiceParameter(otherTeam, ARM_BAR, armBarPlayers, null, 1) +
                // CONTINUE. ParityRunner's PLAYER_CHOICE handler declines with an EMPTY selection
                // (0 rng) — the Rust agent's generic PlayerChoice arm mirrors that, so a
                // multi-candidate Arm Bar never applies in parity games (chaos seed 29 i=3: two
                // adjacent Chosen → no Arm Bar; one adjacent → auto-applied, seed 3 i=15).
                // Java: String teamId = game.getOtherTeam(game.getActingTeam()).getId();
                // UtilServerDialog.showDialog(getGameState(), new DialogPlayerChoiceParameter(teamId,
                //   PlayerChoiceMode.ARM_BAR, armBarPlayers, null, 1), true)
                let team_id = if game.home_playing { game.team_away.id.clone() } else { game.team_home.id.clone() };
                crate::util::util_server_dialog::UtilServerDialog::show_dialog(
                    game,
                    ffb_model::dialog::dialog_parameter_factory::DialogParameter::PlayerChoice(
                        ffb_model::dialog::dialog_player_choice_parameter::DialogPlayerChoiceParameter {
                            team_id: Some(team_id),
                            player_choice_mode: Some(ffb_model::model::player_choice_mode::PlayerChoiceMode::ARM_BAR.get_name().to_string()),
                            player_ids: self.arm_bar_players.clone(),
                            descriptions: vec![],
                            max_selects: 1,
                            min_selects: 0,
                        },
                    ),
                    true,
                );
                return StepOutcome::cont().with_prompt(
                    ffb_model::prompts::AgentPrompt::PlayerChoice {
                        eligible_players: self.arm_bar_players.clone(),
                        reason: "ARM_BAR".into(),
                        descriptions: vec![],
                    },
                );
            }
        }

        // Java: injuryType = (armBarPlayer != null) ? new InjuryTypeDropDodgeForSpp(armBarPlayer)
        //                                            : new InjuryTypeDropDodge(false)
        // (constructor args travel as a '#' suffix — see make_injury_type).
        let injury_type_name = match arm_bar_player {
            Some(pid) => format!("InjuryTypeDropDodgeForSpp#{pid}"),
            None => "InjuryTypeDropDodge#noArmBar".to_string(),
        };
        let ctx = SteadyFootingContext::from_injury_type_name(injury_type_name);
        let label = self.goto_label_on_failure.clone();
        StepOutcome::goto(&label)
            .publish(StepParameter::SteadyFootingContext(Box::new(ctx)))
    }
}

#[cfg(test)]
mod tests {
    // ── The Diving-Tackle what-if minimum is clamped ONCE (Java StepMoveDodge bb2025:415-441) ──

    /// AG1 dodger, bare destination square, an adjacent opposing Diving Tackler that is not
    /// adjacent to the destination, and a full re-roll bank.
    fn dt_threat_fixture(agility: i32) -> (Game, StepMoveDodge) {
        dt_threat_fixture_for(agility, ffb_model::enums::Rules::Bb2025)
    }

    fn dt_threat_fixture_for(agility: i32, rules: ffb_model::enums::Rules) -> (Game, StepMoveDodge) {
        use ffb_model::enums::{PlayerAction, PS_STANDING, PlayerState as PSt, SkillId};
        use ffb_model::model::skill_def::SkillWithValue;
        let mut game = Game::new(
            crate::step::framework::test_team("home", 0),
            crate::step::framework::test_team("away", 0),
            rules,
        );
        let dodger = ffb_model::model::player::Player {
            id: "dodger".into(), name: "d".into(), nr: 1, position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 5, agility, passing: 0, armour: 9,
            ..Default::default()
        };
        game.team_home.players.push(dodger);
        let mut tackler = ffb_model::model::player::Player {
            id: "tackler".into(), name: "t".into(), nr: 2, position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 7, strength: 3, agility: 3, passing: 4, armour: 8,
            ..Default::default()
        };
        tackler.starting_skills.push(SkillWithValue { skill_id: SkillId::DivingTackle, value: None });
        game.team_away.players.push(tackler);
        game.field_model.set_player_coordinate("dodger", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("dodger", PSt::new(PS_STANDING));
        // Adjacent to the SOURCE only: an eligible Diving Tackler that adds no tacklezone to the
        // destination, so the bare dodge minimum is `max(2, agility)`.
        game.field_model.set_player_coordinate("tackler", FieldCoordinate::new(4, 4));
        game.field_model.set_player_state("tackler", PSt::new(PS_STANDING));
        game.home_playing = true;
        game.turn_mode = TurnMode::Regular;
        game.turn_data_home.rerolls = 2;
        game.turn_data_away.rerolls = 2;
        game.acting_player.set_player("dodger".into(), PlayerAction::Move);
        game.acting_player.dodging = true;
        let mut step = StepMoveDodge::new(String::new());
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        step.coordinate_to = Some(FieldCoordinate::new(6, 5));
        (game, step)
    }

    /// Java `dodge()`: a roll that fails with modifiers but passes bare offers an unused Swift As
    /// The Breeze BEFORE the Dodge skill re-roll is spent; using it passes the dodge on the same die
    /// and marks the skill used.
    #[test]
    fn swift_as_the_breeze_is_offered_before_the_dodge_reroll_and_passes_the_dodge() {
        use ffb_model::enums::{PS_STANDING, PlayerState as PSt, SkillId};
        use ffb_model::model::skill_def::SkillWithValue;
        let (mut game, mut step) = dt_threat_fixture_for(2, ffb_model::enums::Rules::Bb2020);
        {
            let p = game.team_home.player_mut("dodger").unwrap();
            p.starting_skills.push(SkillWithValue { skill_id: SkillId::Dodge, value: None });
            p.starting_skills.push(SkillWithValue { skill_id: SkillId::SwiftAsTheBreeze, value: None });
        }
        // A marker on the destination: min = max(2, 2 + 1) = 3, while the bare minimum is 2.
        game.team_away.players.push(ffb_model::model::player::Player {
            id: "marker".into(), name: "m".into(), nr: 3, position_id: "pos".into(),
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8, ..Default::default()
        });
        game.field_model.set_player_coordinate("marker", FieldCoordinate::new(7, 5));
        game.field_model.set_player_state("marker", PSt::new(PS_STANDING));
        step.dodge_roll = 2;
        let mut rng = GameRng::new(1);
        let out = step.start(&mut game, &mut rng);
        match out.prompt {
            Some(ffb_model::prompts::AgentPrompt::SkillUse { skill_id, .. }) =>
                assert_eq!(skill_id, SkillId::SwiftAsTheBreeze as u16),
            other => panic!("expected the Swift As The Breeze dialog, got {other:?}"),
        }
        let out = step.handle_command(
            &Action::UseSkill { skill_id: SkillId::SwiftAsTheBreeze, use_skill: true }, &mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
        assert!(game.acting_player.used_skills.contains(&SkillId::SwiftAsTheBreeze));
        assert!(!game.team_home.player("dodger").unwrap().used_skills.contains(&SkillId::Dodge));
    }

    /// Java's bb2020 `failDodge()` has no Arm Bar dialog: with TWO adjacent Arm Bar players and
    /// no re-roll left, the step must fall straight to the failure label, where bb2025 asks the
    /// opposing coach which Arm Bar applies. The bb2025 dialog's re-entry is what spent a second
    /// team re-roll in bb2020 games (old_world_alliance_ogre bb2020 @0 seed 29).
    #[test]
    fn bb2020_failed_dodge_between_two_arm_bars_falls_without_a_dialog() {
        use ffb_model::enums::{PS_STANDING, PlayerState as PSt, Rules, SkillId};
        use ffb_model::model::skill_def::SkillWithValue;
        let build = |rules: Rules| {
            let (mut game, mut step) = dt_threat_fixture_for(3, rules);
            game.turn_data_home.rerolls = 0;
            game.turn_data_away.rerolls = 0;
            game.defender_id = None;
            // Replace the Diving Tackler's skill with Arm Bar and add a second Arm Bar dwarf, both
            // adjacent to the FROM square (5,5).
            let t = game.team_away.players.iter_mut().find(|p| p.id == "tackler").unwrap();
            t.starting_skills.clear();
            t.starting_skills.push(SkillWithValue { skill_id: SkillId::ArmBar, value: None });
            let mut second = ffb_model::model::player::Player {
                id: "armbar2".into(), name: "a".into(), nr: 3, position_id: "pos".into(),
                player_type: ffb_model::enums::PlayerType::Regular,
                gender: ffb_model::enums::PlayerGender::Male,
                movement: 4, strength: 3, agility: 3, passing: 4, armour: 9,
                ..Default::default()
            };
            second.starting_skills.push(SkillWithValue { skill_id: SkillId::ArmBar, value: None });
            game.team_away.players.push(second);
            game.field_model.set_player_coordinate("armbar2", FieldCoordinate::new(4, 6));
            // The adjacency search wants tackle zones: STANDING + active on both dwarfs.
            game.field_model.set_player_state("armbar2", PSt::new(PS_STANDING).change_active(true));
            game.field_model.set_player_state("tackler", PSt::new(PS_STANDING).change_active(true));
            step.dodge_roll = 1;
            (game, step)
        };
        let (mut game, mut step) = build(Rules::Bb2020);
        let out = step.execute_step(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, crate::step::framework::StepAction::GotoLabel, "bb2020 falls at once");
        assert!(out.prompt.is_none(), "bb2020 has no ARM_BAR dialog");
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::SteadyFootingContext(_))));

        let (mut game, mut step) = build(Rules::Bb2025);
        let out = step.execute_step(&mut game, &mut GameRng::new(0));
        assert!(
            matches!(out.prompt, Some(ffb_model::prompts::AgentPrompt::PlayerChoice { ref reason, .. }) if reason == "ARM_BAR"),
            "bb2025 asks which Arm Bar applies: {:?}", out.prompt
        );
    }

    /// Java computes the Diving-Tackle what-if as `mechanic.minimumRollDodge(game, player, withDt,
    /// stat)` where the mechanic is `Math.max(2, agility + sum(modifiers))` and the DIVING_TACKLE
    /// DodgeModifier is +2 — so the clamp is applied ONCE, after the +2. For an AG1 Kroxigor with
    /// no other modifiers that is `max(2, 1 + 2) = 3`, and a dodge roll of 3 still succeeds, so
    /// Java asks for nothing. Rust used to add +2 to the ALREADY-clamped minimum (`max(2,1) + 2 =
    /// 4`), read the 3 as flippable, and pre-emptively offered a team re-roll the heuristic then
    /// spent (slann bb2025 @0 seed 43 i=85: two extra dice, home re-rolls 2 → 1).
    #[test]
    fn diving_tackle_what_if_minimum_is_clamped_once_for_ag1() {
        let (mut game, mut step) = dt_threat_fixture(1);
        step.dodge_roll = 3;
        let mut rng = GameRng::new(0);
        let out = step.execute_step(&mut game, &mut rng);
        assert!(out.prompt.is_none(),
            "max(2, 1+2) = 3 <= roll 3: Diving Tackle cannot flip this dodge, so Java offers no \
             re-roll — got {:?}", out.prompt);
        assert_eq!(out.action, crate::step::framework::StepAction::NextStep);
    }

    /// The companion case that must NOT change: when the clamp does not bind, adding the Diving
    /// Tackle +2 still turns a bare success into a threat and Java DOES pre-emptively offer the
    /// re-roll. AG3, no modifiers → min 3; with DT → max(2, 3+2) = 5, and a roll of 3 fails it.
    #[test]
    fn diving_tackle_what_if_still_offers_the_reroll_when_the_clamp_does_not_bind() {
        let (mut game, mut step) = dt_threat_fixture(3);
        step.dodge_roll = 3;
        let mut rng = GameRng::new(0);
        let out = step.execute_step(&mut game, &mut rng);
        assert!(out.prompt.is_some(),
            "AG3: min 3 succeeds bare but max(2, 3+2) = 5 fails, so Java asks 'Diving Tackle can \
             make this dodge fail. Reroll the dodge now?'");
    }

    /// Java writes ReportDodgeRoll only when `pDoRoll`; DECLINING the pre-emptive Diving-Tackle
    /// re-roll re-enters with `doRoll == false`, so the dodge is reported exactly once
    /// (dwarf_vs_human bb2025 seed 2: Rust reported it a second time).
    #[test]
    fn declined_diving_tackle_reroll_does_not_report_the_dodge_twice() {
        use ffb_model::report::report_id::ReportId;
        let (mut game, mut step) = dt_threat_fixture(3);
        step.dodge_roll = 3;
        let mut rng = GameRng::new(0);
        let out = step.execute_step(&mut game, &mut rng);
        assert!(out.prompt.is_some(), "the DT re-roll is offered");
        step.handle_command(&Action::UseReRoll { use_reroll: false }, &mut game, &mut rng);
        let dodges = game.report_list.get_reports().iter()
            .filter(|r| r.get_id() == ReportId::DODGE_ROLL).count();
        assert_eq!(dodges, 1, "one dodge, one report");
    }

    /// The whole pre-emptive Diving-Tackle re-roll is a BB2025 addition. `make_step_for` has no
    /// `Rules::Bb2020` arm for `MoveDodge`, so BB2020 runs this same file — but the BB2020 Java
    /// StepMoveDodge's success branch is bare `status = ActionStatus.SUCCESS` and never calls
    /// `findEligibleDivingTacklers`. Offering here re-rolled dodges that had already succeeded
    /// (slann bb2020 @0 seed 3 i=18: Blitzer's 5 beat min 4, the fresh die was a 1 → prone →
    /// turnover).
    #[test]
    fn bb2020_never_pre_emptively_offers_the_diving_tackle_reroll() {
        let (mut game, mut step) = dt_threat_fixture_for(3, ffb_model::enums::Rules::Bb2020);
        step.dodge_roll = 3;
        let mut rng = GameRng::new(0);
        let out = step.execute_step(&mut game, &mut rng);
        assert!(out.prompt.is_none(),
            "BB2020 Java's success branch is bare SUCCESS — no Diving-Tackle what-if, no re-roll \
             offer — got {:?}", out.prompt);
        assert_eq!(out.action, crate::step::framework::StepAction::NextStep);
    }

    // ── Break Tackle consumption (Java StepMoveDodge bb2025:363-380 + 516-521) ────────────

    /// Java: when the dodge succeeds ONLY thanks to Break Tackle, `fUsingBreakTackle = true;
    /// actingPlayer.markSkillUsed(btSkill)` — ONE BT per activation. Rust never marked it, so the
    /// Deathroller's second dodge of the activation got the -3 again (dwarf bb2025 seed 43 k=31:
    /// Java min=6 mods="1 Tacklezone", Rust min=3 with "Break Tackle ST 5+").
    #[test]
    fn break_tackle_that_saved_the_dodge_is_consumed() {
        use ffb_model::enums::SkillId;
        let (mut game, mut step) = bt_dodge_fixture();
        // roll 3: base AG4 + 1 TZ = min 5 fails bare; with BT ST5+ (-3) min 2 succeeds.
        step.dodge_roll = 3;
        let mut rng = GameRng::new(0);
        let out = step.execute_step(&mut game, &mut rng);
        assert_eq!(out.action, crate::step::framework::StepAction::NextStep);
        assert!(step.using_break_tackle, "BT saved the dodge -> fUsingBreakTackle");
        assert!(game.player("dodger").unwrap().used_skills.contains(&SkillId::BreakTackle),
            "BT must be marked used (one per activation)");
        assert!(game.acting_player.used_skills.contains(&SkillId::BreakTackle));
    }

    /// Java: when the roll succeeds even WITHOUT Break Tackle, the modifier is dropped and BT is
    /// NOT consumed (the reported minimum is the bare one).
    #[test]
    fn break_tackle_not_needed_is_not_consumed() {
        use ffb_model::enums::SkillId;
        let (mut game, mut step) = bt_dodge_fixture();
        step.dodge_roll = 6; // succeeds bare
        let mut rng = GameRng::new(0);
        step.execute_step(&mut game, &mut rng);
        assert!(!step.using_break_tackle);
        assert!(!game.player("dodger").unwrap().used_skills.contains(&SkillId::BreakTackle),
            "BT unused when the bare roll already succeeds");
    }

    /// Java FAILURE branch: BT could not save the dodge → `ReportSkillUse(null, btSkill, false,
    /// WOULD_NOT_HELP)` before the dodge roll report (dwarf_vs_orc bb2020 seed 3).
    #[test]
    fn break_tackle_that_cannot_save_the_dodge_reports_would_not_help() {
        use ffb_model::report::report_id::ReportId;
        let (mut game, mut step) = bt_dodge_fixture();
        step.dodge_roll = 1; // fails even with BT
        let mut rng = GameRng::new(0);
        step.execute_step(&mut game, &mut rng);
        let ids: Vec<ReportId> = game.report_list.get_reports().iter().map(|r| r.get_id()).collect();
        let su = ids.iter().position(|i| *i == ReportId::SKILL_USE).expect("BT wouldNotHelp report");
        let dr = ids.iter().position(|i| *i == ReportId::DODGE_ROLL).expect("dodge roll report");
        assert!(su < dr, "skillUse precedes the dodge roll: {ids:?}");
        assert!(!step.using_break_tackle);
    }

    /// Java `StepMoveDodge.dodge` FAILURE branch: `uncanceledDodgeRerollSource` returns null when
    /// an adjacent opponent cancels the Dodge re-roll (Tackle registers
    /// `CancelSkillProperty(canRerollDodge)`), and the fall-back ask is the PLAYER overload with an
    /// explicit `reRollSkill` of null:
    ///
    /// ```java
    /// UtilServerReRoll.askForReRollIfAvailable(getGameState(), actingPlayer.getPlayer(),
    ///     ReRolledActions.DODGE, minimumRoll, false, modifyingSkill, null)
    /// ```
    ///
    /// With an empty team-re-roll bank `RollMechanic.askForReRollIfAvailable` shows NO dialog, so
    /// the failed dodge stands and the step goes to its failure label. Rust used the ACTING-PLAYER
    /// overload, which re-derived the Dodge source the Tackle filter had just nulled and offered it
    /// (human bb2025 @1e6 seed 3 i=32).
    #[test]
    fn tackle_cancelled_dodge_reroll_is_not_re_offered_with_an_empty_bank() {
        use ffb_model::enums::{PlayerAction, PS_STANDING, PlayerState as PSt, SkillId};
        use ffb_model::model::skill_def::SkillWithValue;
        let mut game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let mut dodger = Player {
            id: "dodger".into(), name: "d".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 5, strength: 2, agility: 3, passing: 4, armour: 7,
            ..Default::default()
        };
        dodger.starting_skills.push(SkillWithValue { skill_id: SkillId::Dodge, value: None });
        game.team_home.players.push(dodger);
        let mut tackler = Player {
            id: "tackler".into(), name: "t".into(), nr: 2, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            ..Default::default()
        };
        tackler.starting_skills.push(SkillWithValue { skill_id: SkillId::Tackle, value: None });
        game.team_away.players.push(tackler);
        game.field_model.set_player_coordinate("dodger", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("dodger", PSt::new(PS_STANDING));
        game.field_model.set_player_coordinate("tackler", FieldCoordinate::new(5, 6));
        game.field_model.set_player_state("tackler", PSt::new(PS_STANDING));
        game.home_playing = true;
        game.turn_mode = TurnMode::Regular;
        // Java `isTeamReRollAvailable`: an empty bank offers nothing.
        game.turn_data_home.rerolls = 0;
        game.acting_player.set_player("dodger".into(), PlayerAction::Move);
        game.acting_player.dodging = true;
        let mut step = StepMoveDodge::new("fail".into());
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        step.coordinate_to = Some(FieldCoordinate::new(6, 4));
        step.dodge_roll = 1; // a natural 1 always fails
        let out = step.execute_step(&mut game, &mut GameRng::new(0));
        assert!(out.prompt.is_none(),
            "Tackle cancels the Dodge re-roll and the bank is empty: Java shows no dialog, got {:?}",
            out.prompt);
        assert_eq!(out.action, StepAction::GotoLabel, "the failed dodge goes to the failure label");
    }

    fn bt_dodge_fixture() -> (Game, StepMoveDodge) {
        use ffb_model::enums::{PlayerAction, PS_STANDING, PlayerState as PSt, SkillId};
        use ffb_model::model::skill_def::SkillWithValue;
        let mut game = Game::new(
            crate::step::framework::test_team("home", 0),
            crate::step::framework::test_team("away", 0),
            ffb_model::enums::Rules::Bb2025,
        );
        let mut dodger = ffb_model::model::player::Player {
            id: "dodger".into(), name: "d".into(), nr: 1, position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 4, strength: 7, agility: 4, passing: 6, armour: 11,
            ..Default::default()
        };
        dodger.starting_skills.push(SkillWithValue { skill_id: SkillId::BreakTackle, value: None });
        game.team_home.players.push(dodger);
        let mut marker = ffb_model::model::player::Player {
            id: "marker".into(), name: "m".into(), nr: 2, position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            ..Default::default()
        };
        let _ = &mut marker;
        game.team_away.players.push(marker);
        game.field_model.set_player_coordinate("dodger", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("dodger", PSt::new(PS_STANDING));
        // marker adjacent to the DESTINATION so the dodge has 1 TZ
        game.field_model.set_player_coordinate("marker", FieldCoordinate::new(7, 5));
        game.field_model.set_player_state("marker", PSt::new(PS_STANDING));
        game.acting_player.set_player("dodger".into(), PlayerAction::Move);
        game.acting_player.dodging = true;
        let mut step = StepMoveDodge::new(String::new());
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        step.coordinate_to = Some(FieldCoordinate::new(6, 5));
        (game, step)
    }

    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{Rules, TurnMode};
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;
    use std::collections::HashSet;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    fn add_player(game: &mut Game, id: &str) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 4, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
        game.acting_player.player_id = Some(id.into());
        game.acting_player.dodging = true;
    }

    #[test]
    fn success_on_roll_two_or_above_returns_next_step() {
        let mut game = make_game();
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 4;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn failure_on_roll_one_goes_to_failure_label() {
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        add_player(&mut game, "p1");
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    /// Java `StepMoveDodge.executeStep` (bb2025): with reRolledAction=DODGE and a live
    /// reRollSource, EVERY entry re-runs useReRoll and rolls a FRESH dodge die — including the
    /// ARM_BAR PlayerChoice re-entry (chaos bb2025 seed 28 i=60: the answer burns a SECOND team
    /// re-roll and the fresh die succeeds; Rust reused the stale failed die and fell).
    #[test]
    fn arm_bar_answer_rerolls_the_dodge_with_a_fresh_die() {
        let mut game = make_game();
        add_player(&mut game, "p1");
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.dodging = true;
        game.turn_data_mut().rerolls = 2;
        let mut step = StepMoveDodge::new("fail".into());
        step.re_roll_state.re_rolled_action =
            Some(ffb_model::model::re_rolled_action::ReRolledAction::new("DODGE"));
        step.re_roll_state.re_roll_source = Some(ReRollSource::new("TRR"));
        step.dodge_roll = 2; // the stale failed die from before the ARM_BAR prompt
        // Seed whose first d6 is a 6 — the fresh roll must succeed where the stale 2 fails.
        let mut seed = 0u64;
        loop {
            if ffb_model::util::rng::GameRng::new(seed).d6() == 6 { break; }
            seed += 1;
        }
        let mut rng = ffb_model::util::rng::GameRng::new(seed);
        let before = rng.call_count;
        let out = step.handle_command(
            &Action::SelectPlayer { player_id: String::new() }, &mut game, &mut rng);
        assert!(rng.call_count > before, "the ARM_BAR re-entry must roll a fresh dodge die");
        assert_eq!(out.action, StepAction::NextStep,
            "a fresh 6 succeeds; the stale 2 must not be reused");
    }

    /// Regression (human seed 13): a Dodge-skill player's failed-dodge re-roll is CANCELLED by an
    /// adjacent opposing Tackle player at the FROM square (Java StepMoveDodge lines 342-354 —
    /// UtilCards.cancelsSkill via Tackle's CancelSkillProperty(canRerollDodge)). With the Tackle
    /// opponent the skill re-roll must NOT fire (no fresh die rolled → the preset failing roll
    /// stands → fall); without it, the Dodge skill re-roll fires (resets + rolls a fresh die).
    #[test]
    fn tackle_opponent_cancels_dodge_skill_reroll() {
        use ffb_model::enums::{SkillId, PlayerState, PS_STANDING};
        use ffb_model::model::skill_def::SkillWithValue;
        let dodger_with_tackle_neighbour = |tackle: bool| -> u64 {
            let mut game = make_game();
            game.home_playing = true;
            game.turn_mode = ffb_model::enums::TurnMode::Regular; // skill re-roll requires Regular
            game.turn_data_home.rerolls = 0; // no TRR — isolate the skill re-roll
            // Home dodger with the Dodge skill at (5,5).
            game.team_home.players.push(Player {
                id: "p1".into(), name: "p1".into(), nr: 1, position_id: "catcher".into(),
                player_type: PlayerType::Regular, gender: PlayerGender::Male,
                movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
                starting_skills: vec![SkillWithValue::new(SkillId::Dodge)],
                extra_skills: vec![], temporary_skills: vec![], used_skills: HashSet::new(),
                niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0,
                race: None, is_big_guy: false, ..Default::default()
            });
            game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
            game.field_model.set_player_state("p1", PlayerState::new(PS_STANDING));
            game.acting_player.player_id = Some("p1".into());
            game.acting_player.dodging = true;
            // Opposing player adjacent to the FROM square (5,5), with/without Tackle.
            game.team_away.players.push(Player {
                id: "opp".into(), name: "opp".into(), nr: 2, position_id: "blitzer".into(),
                player_type: PlayerType::Regular, gender: PlayerGender::Male,
                movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
                starting_skills: if tackle { vec![SkillWithValue::new(SkillId::Tackle)] } else { vec![] },
                extra_skills: vec![], temporary_skills: vec![], used_skills: HashSet::new(),
                niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0,
                race: None, is_big_guy: false, ..Default::default()
            });
            game.field_model.set_player_coordinate("opp", FieldCoordinate::new(5, 6));
            game.field_model.set_player_state("opp", PlayerState::new(PS_STANDING));

            let mut step = StepMoveDodge::new("fail".into());
            step.coordinate_from = Some(FieldCoordinate::new(5, 5));
            step.coordinate_to = Some(FieldCoordinate::new(6, 6));
            step.dodge_roll = 1; // preset failing roll — start() rolls no initial die
            let mut rng = GameRng::new(0);
            step.start(&mut game, &mut rng);
            rng.call_count
        };
        // Tackle adjacent → skill re-roll cancelled → no fresh die rolled.
        assert_eq!(dodger_with_tackle_neighbour(true), 0,
            "an adjacent Tackle opponent must cancel the Dodge skill re-roll (no re-roll die)");
        // No Tackle → Dodge skill re-roll fires → one fresh die rolled.
        assert_eq!(dodger_with_tackle_neighbour(false), 1,
            "without Tackle the Dodge skill re-roll must fire (one fresh die)");
    }

    /// Java bb2020/bb2025 `StepMoveDodge.dodge`: a failed dodge that adding the player's ST would
    /// save, with the Dodge skill re-roll available, first shows the Incorporeal SkillUse dialog;
    /// accepting it passes the dodge with NO further die and consumes the skill (star tier:
    /// bb2020 Gretchen Wächter, chaos_pact vs khemri seed 1).
    #[test]
    fn incorporeal_is_offered_before_the_dodge_reroll_and_passes_the_dodge() {
        use ffb_model::enums::{SkillId, PlayerState, PS_STANDING};
        use ffb_model::model::skill_def::SkillWithValue;
        let mut game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020);
        game.home_playing = true;
        game.turn_mode = ffb_model::enums::TurnMode::Regular;
        game.turn_data_home.rerolls = 0;
        game.team_home.players.push(Player {
            id: "g".into(), name: "g".into(), nr: 1, position_id: "star".into(),
            movement: 7, strength: 3, agility: 2, passing: 4, armour: 9,
            starting_skills: vec![SkillWithValue::new(SkillId::Dodge), SkillWithValue::new(SkillId::Incorporeal)],
            ..Default::default()
        });
        game.field_model.set_player_coordinate("g", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("g", PlayerState::new(PS_STANDING));
        game.acting_player.player_id = Some("g".into());
        game.acting_player.dodging = true;
        game.team_away.players.push(Player {
            id: "opp".into(), name: "opp".into(), nr: 2, position_id: "lineman".into(),
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            ..Default::default()
        });
        game.field_model.set_player_coordinate("opp", FieldCoordinate::new(7, 7));
        game.field_model.set_player_state("opp", PlayerState::new(PS_STANDING));

        let mut step = StepMoveDodge::new("fail".into());
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        step.coordinate_to = Some(FieldCoordinate::new(6, 6)); // 1 tacklezone → min 3
        step.dodge_roll = 2;
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert!(matches!(out.prompt, Some(ffb_model::prompts::AgentPrompt::SkillUse { .. })),
            "Incorporeal must be offered, not the Dodge re-roll: {:?}", out.prompt);
        assert_eq!(rng.call_count, 0);

        let out = step.handle_command(
            &Action::UseSkill { skill_id: SkillId::Incorporeal, use_skill: true }, &mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep, "2 + ST 3 beats min 3");
        assert_eq!(rng.call_count, 0, "no fresh die when the skill is used");
        assert!(game.player("g").unwrap().used_skills.contains(&SkillId::Incorporeal));
        assert!(!game.player("g").unwrap().used_skills.contains(&SkillId::Dodge));
    }

    #[test]
    fn fresh_dodge_die_is_published_as_dodge_roll() {
        // Java `dodge(pDoRoll=true)`: `publishParameter(new StepParameter(DODGE_ROLL,
        // rollSkill()))` — the die must reach `StepDivingTackle`, whose whole body is gated on
        // `dodgeRoll > 0`. Before the fix nothing in the engine published it (BACKLOG §H.60).
        let mut game = make_game();
        game.acting_player.dodging = true;
        let mut step = StepMoveDodge::new("fail".into());
        let mut rng = GameRng::new(7);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(rng.call_count, 1, "one fresh d6");
        let published: Vec<i32> = out.published.iter().filter_map(|p| match p {
            StepParameter::DodgeRoll(v) => Some(*v),
            _ => None,
        }).collect();
        assert_eq!(published, vec![step.dodge_roll], "the rolled die is published exactly once");
        assert!((1..=6).contains(&step.dodge_roll));
    }

    #[test]
    fn preset_dodge_roll_is_not_republished() {
        // Java `dodge(pDoRoll=false)` (a RETRY_DODGE re-entry with usingDivingTackle set)
        // publishes nothing — the step reuses the die the stack already carries.
        let mut game = make_game();
        game.acting_player.dodging = true;
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 6;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(!out.published.iter().any(|p| matches!(p, StepParameter::DodgeRoll(_))));
    }

    #[test]
    fn success_publishes_re_roll_used_false() {
        let mut game = make_game();
        game.acting_player.dodging = true;
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 3;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::ReRollUsed(false))));
    }

    #[test]
    fn success_publishes_using_break_tackle_state() {
        let mut game = make_game();
        game.acting_player.dodging = true;
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 5;
        step.using_break_tackle = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::UsingBreakTackle(true))));
    }

    #[test]
    fn not_dodging_returns_next_step_immediately() {
        let mut game = make_game();
        game.acting_player.dodging = false;
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.is_empty());
    }

    #[test]
    fn set_parameter_dodge_roll_accepted() {
        let mut step = StepMoveDodge::new("fail".into());
        assert!(step.set_parameter(&StepParameter::DodgeRoll(4)));
        assert_eq!(step.dodge_roll, 4);
    }

    #[test]
    fn failure_with_trr_offers_reroll_prompt() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        add_player(&mut game, "p1");
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
        assert!(out.prompt.is_some());
    }

    #[test]
    fn accept_reroll_then_success_returns_next_step() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        add_player(&mut game, "p1");
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 1;
        let _offer = step.start(&mut game, &mut GameRng::new(0));
        step.dodge_roll = 5; // success on re-roll
        let out = step.handle_command(&Action::UseReRoll { use_reroll: true }, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn decline_reroll_goes_to_failure_label() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        add_player(&mut game, "p1");
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 1;
        let _offer = step.start(&mut game, &mut GameRng::new(0));
        let out = step.handle_command(&Action::UseReRoll { use_reroll: false }, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn stand_firm_no_drop_option_returns_next_step_on_failure() {
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        game.options.set("standFirmNoDropOnFailedDodge", "true");
        add_player(&mut game, "p1");
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn stand_firm_no_drop_option_disabled_still_goes_to_failure_label() {
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        add_player(&mut game, "p1");
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn success_emits_dodge_roll_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        add_player(&mut game, "p1");
        game.acting_player.dodging = true;
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 6;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::DODGE_ROLL));
    }

    #[test]
    fn failure_emits_dodge_roll_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        add_player(&mut game, "p1");
        game.acting_player.dodging = true;
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 1;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::DODGE_ROLL));
    }

    #[test]
    fn failure_publishes_drop_dodge_injury_type_not_fall_down() {
        // Java: failDodge() publishes SteadyFootingContext(new InjuryTypeDropDodge(false))
        // when no arm-bar player is involved — never InjuryTypeFallDown (that's StepFallDown's
        // injury type, used for e.g. Really Stupid/Bonehead knockdowns, not failed dodges).
        use crate::drop_player_context::SteadyFootingContext as SFC;
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        add_player(&mut game, "p1");
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        let injury_type_name = out.published.iter().find_map(|p| match p {
            StepParameter::SteadyFootingContext(ctx) => {
                let ctx: &SFC = ctx;
                ctx.injury_type_name().map(|s| s.to_string())
            }
            _ => None,
        });
        // Java: new InjuryTypeDropDodge(false) — the useArmBarModifiers=false arg travels
        // as the '#noArmBar' suffix (see make_injury_type).
        assert_eq!(injury_type_name.as_deref(), Some("InjuryTypeDropDodge#noArmBar"));
    }

    /// Java failDodge: exactly ONE adjacent opposing Arm Bar player at the FROM square →
    /// auto-selected, InjuryTypeDropDodgeForSpp(#pid); two or more → PlayerChoice dialog
    /// (declined by the parity agents → no arm bar). chaos seeds 3 (single) / 29 (double).
    #[test]
    fn arm_bar_candidates_route_injury_type() {
        use crate::drop_player_context::SteadyFootingContext as SFC;
        use ffb_model::enums::{SkillId, PlayerState, PS_STANDING};
        use ffb_model::model::SkillWithValue;
        fn armbar_player(id: &str) -> Player {
            Player {
                id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
                player_type: PlayerType::Regular, gender: PlayerGender::Male,
                movement: 4, strength: 3, agility: 3, passing: 4, armour: 8,
                starting_skills: vec![SkillWithValue::new(SkillId::ArmBar)],
                ..Default::default()
            }
        }
        let injury_name = |out: &StepOutcome| out.published.iter().find_map(|p| match p {
            StepParameter::SteadyFootingContext(ctx) => {
                let ctx: &SFC = ctx;
                ctx.injury_type_name().map(|s| s.to_string())
            }
            _ => None,
        });

        // Single candidate → ForSpp with that player.
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        add_player(&mut game, "p1");
        game.team_away.players.push(armbar_player("ab1"));
        game.field_model.set_player_coordinate("ab1", FieldCoordinate::new(4, 5)); // adjacent to FROM (5,5)... from set below
        game.field_model.set_player_state("ab1", PlayerState::new(PS_STANDING).change_active(true));
        let mut step = StepMoveDodge::new("fail".into());
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        step.dodge_roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(injury_name(&out).as_deref(), Some("InjuryTypeDropDodgeForSpp#ab1"));

        // Two candidates → PlayerChoice prompt; agent-style empty decline → noArmBar.
        let mut game2 = make_game();
        game2.home_playing = true;
        game2.turn_data_home.rerolls = 0;
        add_player(&mut game2, "p1");
        for (id, coord) in [("ab1", FieldCoordinate::new(4, 5)), ("ab2", FieldCoordinate::new(4, 6))] {
            game2.team_away.players.push(armbar_player(id));
            game2.field_model.set_player_coordinate(id, coord);
            game2.field_model.set_player_state(id, PlayerState::new(PS_STANDING).change_active(true));
        }
        let mut step2 = StepMoveDodge::new("fail".into());
        step2.coordinate_from = Some(FieldCoordinate::new(5, 5));
        step2.dodge_roll = 1;
        let out2 = step2.start(&mut game2, &mut GameRng::new(0));
        assert!(matches!(out2.prompt, Some(ffb_model::prompts::AgentPrompt::PlayerChoice { ref reason, .. }) if reason == "ARM_BAR"),
            "two candidates must prompt the ARM_BAR player choice");
        let out3 = step2.handle_command(&Action::SelectPlayer { player_id: String::new() }, &mut game2, &mut GameRng::new(0));
        assert_eq!(injury_name(&out3).as_deref(), Some("InjuryTypeDropDodge#noArmBar"));
    }
}
