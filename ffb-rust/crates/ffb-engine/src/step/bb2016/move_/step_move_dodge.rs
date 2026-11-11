use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::enums::ReRollSource;
use ffb_model::util::rng::GameRng;
use ffb_model::report::bb2016::report_dodge_roll::ReportDodgeRoll;
use ffb_model::report::report_id::ReportId;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::drop_player_context::SteadyFootingContext;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::abstract_step_with_re_roll::{ReRollState, find_skill_reroll_source};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available_for, ask_for_reroll_if_available, use_reroll};
use ffb_mechanics::modifiers::dodge_modifier_factory::DodgeModifierFactory;
use ffb_mechanics::modifiers::dodge_context::DodgeContext;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.move.StepMoveDodge.
///
/// Resolves a dodge roll when leaving a tackle zone in BB2016.
///
/// BB2016 differences from BB2025:
/// - `actingPlayer.isDodging()` guard: if !dodging → NEXT_STEP
/// - Uses `InjuryTypeDropDodge` on failure (not InjuryTypeFallDown)
/// - Has STAND_FIRM_NO_DROP_ON_FAILED_DODGE game option check
/// - Has fUsingBreakTackle / fUsingDivingTackle fields (same as BB2025)
/// - Has fReRollUsed field
///
/// Init params: GOTO_LABEL_ON_FAILURE (mandatory).
/// Expects: COORDINATE_FROM, COORDINATE_TO, DODGE_ROLL, USING_BREAK_TACKLE,
///          USING_DIVING_TACKLE, RE_ROLL_USED published by preceding steps.
///
/// standFirmNoDropOnFailedDodge game option → wired (NEXT_STEP + EndPlayerAction instead of failDodge).
/// client-only: Break-Tackle dialog (canAddStrengthToDodge) — headless auto-skips strength bonus offer.
/// client-only: Diving Tackle pre-roll dialog — headless auto-skips diving tackle activation.
/// client-only: Arm-Bar choice dialog — headless skips arm-bar activation.
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
    /// Java: AbstractStepWithReRoll fields
    pub re_roll_state: ReRollState,
    /// A die rolled on THIS pass, waiting to be published as `StepParameter::DodgeRoll` —
    /// Java `dodge(doRoll=true)` publishes `DODGE_ROLL` the moment it rolls (see the bb2025
    /// twin for why this matters: `StepDivingTackle` is gated on it).
    dodge_roll_to_publish: Option<i32>,
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
            re_roll_state: ReRollState::new(),
            dodge_roll_to_publish: None,
        }
    }
}

impl Default for StepMoveDodge {
    fn default() -> Self { Self::new(String::new()) }
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
            // client-only: canAddStrengthToDodge (BreakTackle) — skill use dialog skipped in headless
            // client-only: CLIENT_PLAYER_CHOICE ARM_BAR mode — defender uses arm bar via dialog; headless skips
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
            _ => false,
        }
    }
}

impl StepMoveDodge {
    /// Java `dodge(pDoRoll)`: `if (pDoRoll) publishParameter(new StepParameter(DODGE_ROLL,
    /// rollSkill()))` — attached once to whatever outcome the pass produced (see the bb2025 twin).
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let mut out = self.execute_step_inner(game, rng);
        if let Some(roll) = self.dodge_roll_to_publish.take() {
            out = out.publish(StepParameter::DodgeRoll(roll));
        }
        out
    }

    fn execute_step_inner(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: if (!actingPlayer.isDodging()) { setNextAction(NEXT_STEP); return; }
        if !game.acting_player.dodging {
            return StepOutcome::next();
        }

        let player_id = game.acting_player.player_id.clone();

        let already_rerolled = self.re_roll_state.re_rolled_action
            .as_ref().map(|a| a.name == "DODGE").unwrap_or(false);

        if already_rerolled {
            let pid = player_id.as_deref().unwrap_or("").to_owned();
            let source_opt = self.re_roll_state.re_roll_source.clone();
            let consumed = source_opt
                .as_ref()
                .map(|s| use_reroll(game, s, &pid, rng))
                .unwrap_or(false);
            if consumed {
                self.re_roll_used = true;
                // Java: `doRoll = reRolledAction || fUsingDivingTackle == null` where
                // `reRolledAction = DODGE == getReRolledAction() && getReRollSource() != null` —
                // a consumed re-roll ALWAYS rolls a fresh die, whatever `fDodgeRoll` holds. Rust
                // signals "roll fresh" with `dodge_roll == 0`, and the reset at the TRR offer is
                // undone before re-entry: `publishParameter` delivers DODGE_ROLL to the publishing
                // step itself (driver `apply_effects`), so the failed die was written back and the
                // re-roll re-used it (slann bb2016 @1.0 seed 10 i=7: Java re-rolled 2 -> 4 and ran
                // on, Rust "re-rolled" 2 -> 2, fell, and was KO'd — one die ahead, nine seeds red).
                // The bb2025 twin already resets here; this is its missing bb2016 half.
                self.dodge_roll = 0;
            } else {
                return self.fail_dodge();
            }
        }

        // Java: dodge(doRoll) rolls and publishes DODGE_ROLL.
        if self.dodge_roll == 0 {
            self.dodge_roll = rng.d6();
            self.dodge_roll_to_publish = Some(self.dodge_roll);
        }

        let (minimum_roll, mod_names, has_bt, min_no_bt, names_no_bt) =
            self.dodge_modifiers_and_minimum_roll(game, player_id.as_deref());
        // Java bb2016 StepMoveDodge (the btModifier block): consume Break Tackle only when it
        // saved the dodge; drop it (reported minimum and mods) otherwise — ONE per activation
        // (dwarf bb2016 @1e6 seeds 56/70/87/90: every red resolved at a Deathroller Move).
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
                if let Some(p) = game.team_home.player_mut(pid).or_else(|| game.team_away.player_mut(pid)) {
                    p.used_skills.insert(ffb_model::enums::SkillId::BreakTackle);
                }
                game.acting_player.used_skills.insert(ffb_model::enums::SkillId::BreakTackle);
            }
        }
        let successful = DiceInterpreter::is_skill_roll_successful(self.dodge_roll, minimum_roll);

        // Java: getResult().addReport(new ReportDodgeRoll(actingPlayer.getPlayerId(), successful,
        //         (pDoRoll ? fDodgeRoll : 0), minimumRoll, reRolled, dodgeModifiers.toArray(...)))
        let re_rolled = self.re_roll_state.re_rolled_action.as_ref()
            .map(|a| a.name == "DODGE").unwrap_or(false)
            && self.re_roll_state.re_roll_source.is_some();
        game.report_list.add(ReportDodgeRoll::new(
            player_id.clone(),
            successful,
            self.dodge_roll,
            minimum_roll,
            re_rolled,
            mod_names,
        ));

        // Coverage instrument: the BB2025 twin emits one DodgeRoll GameEvent per resolved roll,
        // and this BB2016 override - a LIVE edition override, not a dead twin - never did. The
        // parity coverage checklist aggregates the Rust engine's GameEvents, so BB2016 reported
        // `dodge success`/`dodge failure` as ZERO for every run while dodges obviously happened.
        // Events are not compared between engines and are not in the state hash, so this is
        // observability only.
        let dodge_event = ffb_model::events::GameEvent::DodgeRoll {
            player_id: player_id.clone().unwrap_or_default(),
            target: minimum_roll,
            roll: self.dodge_roll,
            success: successful,
            rerolled: re_rolled,
        };

        if successful {
            let re_rolled = self.re_roll_state.re_rolled_action.as_ref()
                .map(|a| a.name == "DODGE").unwrap_or(false)
                && self.re_roll_state.re_roll_source.is_some();
            return StepOutcome::next()
                .with_event(dodge_event)
                .publish(StepParameter::ReRollUsed(self.re_roll_used || re_rolled))
                .publish(StepParameter::UsingBreakTackle(self.using_break_tackle));
        }

        // First failure: try re-roll
        if !already_rerolled {
            use ffb_model::model::re_rolled_action::ReRolledAction;
            self.re_roll_state.re_rolled_action = Some(ReRolledAction::new("DODGE"));

            // Skill re-roll (Dodge property canRerollDodge) — auto-used
            let skill_source = find_skill_reroll_source(game, "DODGE");
            // Java bb2016 StepMoveDodge.dodge(): the skill source is CANCELLED when an adjacent
            // opposing player with tackle zones at the FROM square cancels it --
            //   opponents = UtilPlayer.findAdjacentPlayersWithTacklezones(game, otherTeam, fCoordinateFrom, false);
            //   for (opponent : opponents) if (UtilCards.cancelsSkill(opponent, skillRerollSource.getSkill(game))) skillRerollSource = null;
            // Tackle registers CancelSkillProperty(canRerollDodge). The bb2025 twin has carried this
            // filter since human seed 13; this twin never did, and a mirror cell can never show it
            // (Tackle and Dodge sit on opposite teams): dwarf vs halfling bb2016 seed 1 i=21, the
            // Halfling dodging out of a Blocker's zone re-rolled 2 -> 3 and ran on in Rust while
            // Java fell at the first square (BACKLOG §I.5).
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
                return self.execute_step(game, rng);
            }

            // TRR offer. Java bb2016: `askForReRollIfAvailable(getGameState(), actingPlayer.getPlayer(),
            // ReRolledActions.DODGE, minimumRoll, false)` -- the PLAYER overload, which passes NO
            // re-roll skill. Rust's acting-player overload re-derives the action-keyed source and
            // resurrected the very Dodge skill the Tackle filter above had just nulled: the offer
            // went out as `ReRollOffer{source: Dodge}`, the heuristic scored it (two sampler draws
            // Java never spends) and declined, and every draw after it was one off (dwarf vs
            // halfling bb2016 seed 1 i=22, after §I.5's filter had moved the red from i=21).
            if let Some(prompt) = ask_for_reroll_if_available_for(
                game, player_id.as_deref(), "DODGE", minimum_roll, false)
            {
                self.re_roll_state.re_roll_source = Some(ReRollSource::new("TRR"));
                self.dodge_roll = 0;
                // Emit the FAILED initial roll here, exactly as the BB2025 twin does when it
                // offers the re-roll prompt. This is where a failed dodge is actually counted:
                // the later declined-re-roll exit returns before a roll is resolved, so without
                // this `dodge failure` stayed at zero while `dodge success` was correct.
                return StepOutcome::cont().with_prompt(prompt).with_event(dodge_event);
            }
        }

        // Java: if (UtilGameOption.isOptionEnabled(game, GameOptionId.STAND_FIRM_NO_DROP_ON_FAILED_DODGE))
        if game.options.is_enabled("standFirmNoDropOnFailedDodge") {
            return StepOutcome::next()
                .with_event(dodge_event)
                .publish(StepParameter::EndPlayerAction(true));
        }
        self.fail_dodge().with_event(dodge_event)
    }

    /// Computes the minimum dodge roll and the reporting-string names of the applicable
    /// modifiers (regular + skill-based). Java: DodgeModifierFactory.findModifiers(...) combined
    /// with the acting player's skill-registered modifiers, fed into
    /// `dodgeModifiers.toArray(new DodgeModifier[0])` for the ReportDodgeRoll constructor.
    fn dodge_modifiers_and_minimum_roll(&self, game: &Game, player_id: Option<&str>) -> (i32, Vec<String>, bool, i32, Vec<String>) {
        let factory = DodgeModifierFactory::for_rules(game.rules);
        if let Some(pid) = player_id {
            let acting = game.acting_player.clone();
            let src = self.coordinate_from.unwrap_or(FieldCoordinate::new(0, 0));
            let tgt = self.coordinate_to.unwrap_or(FieldCoordinate::new(0, 0));
            // Java passes fUsingBreakTackle — a committed BT covers the rest of the move.
            let ctx = DodgeContext::new_with_break_tackle(game, &acting, src, tgt, self.using_break_tackle);
            let mods = factory.find_applicable(&ctx);
            let skill_mods = factory.find_skill_modifiers(&ctx);
            let all: Vec<&ffb_mechanics::modifiers::dodge_modifier::DodgeModifier> = mods.iter().copied().chain(skill_mods.iter()).collect();
            let agility = game.player(pid).map(|p| p.agility as i32).unwrap_or(3);
            // BB2016 uses the OLD AG scale for the dodge minimum ((7-stat)-1+total), NOT the shared
            // bb2025 (agility+total) — see DodgeModifierFactory::minimum_roll_edition. The bb2025
            // formula let a bb2016 Ogre (AG2) dodge succeed on a 2 (min 2) where Java's bb2016 min is
            // 4 (human seed4 i=185: fall+turnover in Java, standing in Rust).
            let strength = game.player(pid).map(|p| p.strength as i32).unwrap_or(3);
            let min = DodgeModifierFactory::minimum_roll_edition(strength, agility, &all, game.rules);
            let names: Vec<String> = all.iter().map(|m| m.get_report_string().to_string()).collect();
            // Java bb2016 StepMoveDodge: the same Break Tackle axis as the later editions —
            // the use-strength modifier (ST-for-AG substitution) is dropped when the roll
            // succeeds without it or fails regardless, and consumed (markSkillUsed) only when
            // it saved the dodge (bb2016:207-227 + 266-270).
            let without_bt: Vec<&ffb_mechanics::modifiers::dodge_modifier::DodgeModifier> =
                all.iter().copied().filter(|m| !m.use_strength).collect();
            let has_bt = without_bt.len() != all.len();
            let min_no_bt = DodgeModifierFactory::minimum_roll_edition(strength, agility, &without_bt, game.rules);
            let names_no_bt: Vec<String> = without_bt.iter().map(|m| m.get_report_string().to_string()).collect();
            (min, names, has_bt, min_no_bt, names_no_bt)
        } else {
            (2, vec![], false, 2, vec![])
        }
    }

    fn fail_dodge(&self) -> StepOutcome {
        // Java: BB2016 uses InjuryTypeDropDodge (not InjuryTypeFallDown)
        // client-only: Arm-Bar dialog can override injuryType to InjuryTypeArmBar — headless uses default injury type
        let ctx = SteadyFootingContext::from_injury_type_name("InjuryTypeDropDodge".into());
        let label = self.goto_label_on_failure.clone();
        StepOutcome::goto(&label)
            .publish(StepParameter::SteadyFootingContext(Box::new(ctx)))
    }
}

#[cfg(test)]
mod tests {
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
        Game::new(home, away, Rules::Bb2016)
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
    fn success_on_roll_returns_next_step() {
        let mut game = make_game();
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 4;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn failure_goes_to_failure_label() {
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

    #[test]
    fn failure_publishes_steady_footing_context() {
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        add_player(&mut game, "p1");
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::SteadyFootingContext(_))));
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
    fn consumed_team_reroll_rolls_fresh_even_after_self_delivery_of_dodge_roll() {
        // The driver delivers every published parameter to the publishing step too (Java
        // AbstractStep.publishParameter: setParameter(self) then the stack). After the TRR offer
        // the step has reset `dodge_roll` to 0 and published the FAILED die; self-delivery writes
        // it back. Java still rolls fresh on the re-entry (`doRoll = reRolledAction`), so the
        // re-entry must not re-use the delivered die.
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        add_player(&mut game, "p1");
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue, "TRR offered");
        assert_eq!(step.dodge_roll, 0, "the offer resets the die so the re-entry rolls fresh");
        // What the driver does with the DODGE_ROLL the step published when it rolled the 1
        // (the preset above stands in for that roll): deliver it back to the publishing step.
        step.set_parameter(&StepParameter::DodgeRoll(1));
        assert_eq!(step.dodge_roll, 1, "self-delivery wrote the failed die back");
        let mut rng = GameRng::new(3);
        let out = step.handle_command(&Action::UseReRoll { use_reroll: true }, &mut game, &mut rng);
        assert_eq!(rng.call_count, 1, "the consumed re-roll must roll ONE fresh die");
        assert_ne!(out.action, StepAction::Continue);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::DodgeRoll(v) if *v == step.dodge_roll)),
            "the fresh die is published");
    }

    #[test]
    fn tackle_opponent_cancels_dodge_skill_reroll_bb2016() {
        use ffb_model::enums::{SkillId, PlayerState, PS_STANDING};
        use ffb_model::model::skill_def::SkillWithValue;
        let dodger_with_tackle_neighbour = |tackle: bool| -> u64 {
            let mut game = make_game();
            game.home_playing = true;
            game.turn_mode = TurnMode::Regular;
            game.turn_data_home.rerolls = 0; // no TRR -- isolate the skill re-roll
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
            game.team_away.players.push(Player {
                id: "opp".into(), name: "opp".into(), nr: 2, position_id: "blocker".into(),
                player_type: PlayerType::Regular, gender: PlayerGender::Male,
                movement: 4, strength: 3, agility: 2, passing: 4, armour: 9,
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
            step.dodge_roll = 1; // preset failing roll -- start() rolls no initial die
            let mut rng = GameRng::new(0);
            let out = step.start(&mut game, &mut rng);
            if tackle {
                // Java's PLAYER overload offers nothing here (no TRR, no re-roll skill): the dodge
                // simply fails. The acting-player overload re-found the cancelled Dodge source and
                // raised a ReRollOffer the Java side never shows.
                assert!(out.prompt.is_none(), "no re-roll offer once Tackle cancelled Dodge: {:?}", out.prompt);
            }
            rng.call_count
        };
        assert_eq!(dodger_with_tackle_neighbour(true), 0,
            "an adjacent Tackle opponent must cancel the Dodge skill re-roll (no re-roll die)");
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
    fn success_publishes_using_break_tackle() {
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
        step.dodge_roll = 1; // would fail if guard weren't hit
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.is_empty());
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
    fn accept_reroll_with_success_returns_next_step() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        add_player(&mut game, "p1");
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 1;
        let _offer = step.start(&mut game, &mut GameRng::new(0));
        step.dodge_roll = 5;
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
    fn set_parameter_goto_label_on_failure_accepted() {
        let mut step = StepMoveDodge::new("old".into());
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("new".into())));
        assert_eq!(step.goto_label_on_failure, "new");
    }

    #[test]
    fn set_parameter_dodge_roll_accepted() {
        let mut step = StepMoveDodge::new("fail".into());
        assert!(step.set_parameter(&StepParameter::DodgeRoll(4)));
        assert_eq!(step.dodge_roll, 4);
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepMoveDodge::new("fail".into());
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn stand_firm_no_drop_option_returns_next_step_on_failure() {
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        game.options.set("standFirmNoDropOnFailedDodge", "true");
        add_player(&mut game, "p1");
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 1; // guaranteed fail
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))));
    }

    #[test]
    fn stand_firm_no_drop_option_disabled_still_goes_to_failure_label() {
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        game.options.set("standFirmNoDropOnFailedDodge", "false");
        add_player(&mut game, "p1");
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn success_adds_dodge_roll_report() {
        // Java: getResult().addReport(new ReportDodgeRoll(...)) on success
        let mut game = make_game();
        add_player(&mut game, "p1");
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 5; // high roll → success
        step.start(&mut game, &mut GameRng::new(0));
        assert!(
            game.report_list.has_report(ReportId::DODGE_ROLL),
            "DODGE_ROLL report should be added on a successful dodge"
        );
    }

    #[test]
    fn failure_adds_dodge_roll_report() {
        // Java: addReport is always called in the dodge() method
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        add_player(&mut game, "p1");
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 1; // failure
        step.start(&mut game, &mut GameRng::new(0));
        assert!(
            game.report_list.has_report(ReportId::DODGE_ROLL),
            "DODGE_ROLL report should be added on a failed dodge"
        );
    }

    #[test]
    fn dodge_modifier_names_include_actual_skill_modifiers() {
        // Java: dodge() passes dodgeModifiers.toArray(...) into ReportDodgeRoll — the report must
        // carry the real modifier names (e.g. skill-based ones like "Two Heads"), not an empty
        // list. Previously the Rust translation hardcoded `vec![]` in the ReportDodgeRoll call
        // regardless of the modifiers actually found by dodge_modifiers_and_minimum_roll.
        use ffb_model::model::SkillWithValue;
        use ffb_model::enums::SkillId;
        let mut game = make_game();
        add_player(&mut game, "p1");
        game.team_home.players[0].starting_skills.push(SkillWithValue::new(SkillId::TwoHeads));
        let step = StepMoveDodge::new("fail".into());
        let (_min, names, _has_bt, _min_no_bt, _names_no_bt) = step.dodge_modifiers_and_minimum_roll(&game, Some("p1"));
        assert!(
            names.iter().any(|n| n == "Two Heads"),
            "expected 'Two Heads' in roll modifiers, got: {:?}", names
        );
    }
}
