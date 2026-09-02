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
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
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
            // client-only: CLIENT_USE_SKILL → canAddStrengthToDodge / canChooseToIgnoreDodgeModifierAfterRoll /
            //       canRerollDodge — headless auto-declines skill dialogs.
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
            StepParameter::CoordinateFrom(v) => { self.coordinate_from = Some(*v); true }
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
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
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
        if already_rerolled && !using_modifier_ignoring
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
                return self.fail_dodge(game);
            }
        }

        // Java: dodge(doRoll) → roll and check
        if self.dodge_roll == 0 {
            self.dodge_roll = rng.d6();
        }

        let factory = DodgeModifierFactory::for_rules(game.rules);
        let (minimum_roll, mod_names, has_bt, min_no_bt, names_no_bt): (i32, Vec<String>, bool, i32, Vec<String>) = if let Some(pid) = player_id.as_deref() {
            let acting = game.acting_player.clone();
            let src = self.coordinate_from.unwrap_or(FieldCoordinate::new(0, 0));
            let tgt = self.coordinate_to.unwrap_or(FieldCoordinate::new(0, 0));
            let ctx = DodgeContext::new(game, &acting, src, tgt);
            let mods = factory.find_applicable(&ctx);
            let skill_mods = factory.find_skill_modifiers(&ctx);
            let all: Vec<&ffb_mechanics::modifiers::dodge_modifier::DodgeModifier> = mods.iter().copied().chain(skill_mods.iter()).collect();
            let agility = game.player(pid).map(|p| p.agility as i32).unwrap_or(3);
            let min = DodgeModifierFactory::minimum_roll(agility, &all);
            let names: Vec<String> = all.iter().map(|m| m.get_report_string().to_string()).collect();
            // Java: `Optional<DodgeModifier> btModifier = dodgeModifiers.stream()
            //   .filter(DodgeModifier::isUseStrength).findFirst()` — the Break Tackle axis.
            let without_bt: Vec<&ffb_mechanics::modifiers::dodge_modifier::DodgeModifier> =
                all.iter().copied().filter(|m| !m.use_strength).collect();
            let has_bt = without_bt.len() != all.len();
            let min_no_bt = DodgeModifierFactory::minimum_roll(agility, &without_bt);
            let names_no_bt: Vec<String> = without_bt.iter().map(|m| m.get_report_string().to_string()).collect();
            (min, names, has_bt, min_no_bt, names_no_bt)
        } else {
            (2, vec![], false, 2, vec![])
        };
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
        let (minimum_roll, mod_names) = if has_bt && !bt_saved_it {
            (min_no_bt, names_no_bt)
        } else {
            (minimum_roll, mod_names)
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
        if std::env::var_os("FFB_TRACE").is_some() {
            eprintln!("RDODGEMIN pid={} roll={} min={} from={:?} to={:?} mods={:?} ok={}",
                player_id.as_deref().unwrap_or("-"), self.dodge_roll, minimum_roll,
                self.coordinate_from, self.coordinate_to, mod_names, successful);
        }

        // Java line 333-335: addReport(new ReportDodgeRoll(...))
        let re_rolled = self.re_roll_state.re_rolled_action.as_ref()
            .map(|a| a.name == "DODGE").unwrap_or(false)
            && self.re_roll_state.re_roll_source.is_some();
        {
            use ffb_model::report::mixed::report_dodge_roll::ReportDodgeRoll;
            game.report_list.add(ReportDodgeRoll::new(
                player_id.clone(),
                successful,
                self.dodge_roll,
                minimum_roll,
                re_rolled,
                mod_names,
                None, // stat_based_roll_modifier: headless never applies modifier-ignoring skill
            ));
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
            if self.using_diving_tackle.is_none() && !self.dt_reroll_asked {
                let from = self.coordinate_from.unwrap_or(FieldCoordinate::new(0, 0));
                let to = self.coordinate_to.unwrap_or(FieldCoordinate::new(0, 0));
                let leaving_tz_only = ffb_model::option::util_game_option::is_option_enabled(
                    game, ffb_model::option::game_option_id::DIVING_TACKLE_LEAVING_TZ_ONLY);
                let dt_tacklers = ffb_model::util::util_player::UtilPlayer::find_eligible_diving_tacklers(
                    game, from, to, leaving_tz_only);
                if !dt_tacklers.is_empty() {
                    let min_with_dt = minimum_roll + 2;
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
                            if DiceInterpreter::is_skill_roll_successful(self.dodge_roll, min_with_dt + bt) {
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
                        if let Some(prompt) = ask_for_reroll_if_available(game, "DODGE", min_with_dt, false) {
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
            // Try re-roll on first failure
            if !already_rerolled {
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
                    let pid = player_id.as_deref().unwrap_or("").to_owned();
                    use_reroll(game, &source, &pid, rng);
                    self.re_roll_state.re_roll_source = Some(source);
                    self.dodge_roll = 0;
                    // Failed initial roll resolved — event goes first, re-roll events follow.
                    let mut out = self.execute_step(game, rng);
                    out.events.insert(0, roll_event);
                    return out;
                }

                // TRR offer
                if let Some(prompt) = ask_for_reroll_if_available(game, "DODGE", minimum_roll, false) {
                    self.re_roll_state.re_roll_source = Some(ReRollSource::new("TRR"));
                    self.dodge_roll = 0; // reset so the re-roll gets a fresh d6
                    return StepOutcome::cont().with_prompt(prompt).with_event(roll_event);
                }
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

    fn fail_dodge(&mut self, game: &Game) -> StepOutcome {
        use ffb_model::util::util_player::UtilPlayer;
        use ffb_model::model::property::NamedProperties;
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
