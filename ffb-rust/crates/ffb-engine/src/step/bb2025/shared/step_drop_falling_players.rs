use ffb_model::enums::{ApothecaryMode, PlayerState, Rules, PS_FALLING, PS_HIT_ON_GROUND};
use ffb_model::enums::{CardEffect, ReRollSource};
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::option::game_option_id::{
    PILING_ON_ARMOR_ONLY, PILING_ON_INJURY_ONLY, PILING_ON_TO_KO_ON_DOUBLE,
    PILING_ON_USES_A_TEAM_REROLL,
};
use ffb_model::option::util_game_option::is_option_enabled;
use ffb_model::prompts::agent_prompt::AgentPrompt;
use ffb_model::report::report_piling_on::ReportPilingOn;
use ffb_model::report::report_weeping_dagger_roll::ReportWeepingDaggerRoll;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_cards::UtilCards;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::drop_player_context::{DropPlayerContext, SteadyFootingContext};
use crate::injury::injuryType::injury_type_block::{BlockMode, InjuryTypeBlock};
use crate::injury::injuryType::injury_type_piling_on_armour::InjuryTypePilingOnArmour;
use crate::injury::injuryType::injury_type_piling_on_injury::InjuryTypePilingOnInjury;
use crate::injury::injuryType::injury_type_piling_on_knocked_out::InjuryTypePilingOnKnockedOut;
use crate::injury::InjuryResult;
use crate::step::framework::{Step, StepOutcome, StepParameter};
use crate::step::framework::StepId;
use crate::step::util_server_injury::{drop_player, handle_injury, handle_injury_by_name};
use crate::step::util_server_re_roll::use_reroll;
use crate::util::util_server_re_roll::UtilServerReRoll;

/// Java: `PilingOnBehaviour.rollWeepingDagger` (bb2016/bb2020 skillbehaviour hooks).
fn roll_weeping_dagger(game: &mut Game, rng: &mut GameRng, source_id: &str, target_id: &str) -> bool {
    let minimum_roll = DiceInterpreter::minimum_roll_weeping_dagger();
    let roll = rng.d6();
    let successful = DiceInterpreter::is_skill_roll_successful(roll, minimum_roll);
    if successful {
        game.field_model.add_card_effect(target_id, CardEffect::Poisoned);
    }
    game.report_list.add(ReportWeepingDaggerRoll::new(
        Some(source_id.to_string()), successful, roll, minimum_roll, false, vec![],
    ));
    successful
}

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.shared.StepDropFallingPlayers.
///
/// Drops players in FALLING state after a block, performing an injury roll for each:
///
/// Defender path:
///   - If `HIT_ON_GROUND` → upgrade to `FALLING`.
///   - If `FALLING && isRooted` → clear rooted.
///   - If `FALLING` → `handleInjury` → publish `STEADY_FOOTING_CONTEXT(DropPlayerContext)`.
///   - If saboteur triggered defender → publish `DROP_PLAYER_CONTEXT` directly (bypass Steady Footing).
///   - If defender is own team and not already prone → publish `END_TURN(true)`.
///
/// Attacker path:
///   - If `FALLING && isRooted` → clear rooted.
///   - If `FALLING` → publish `END_TURN(true)` + `handleInjury` →
///     publish `STEADY_FOOTING_CONTEXT(InjuryResult)` (InjuryResult variant, not DropPlayer variant).
///   - If `fell_from_rush` → injury type `InjuryTypeDropGFI`; else `InjuryTypeBlock`.
///   - If saboteur triggered attacker → publish `DROP_PLAYER_CONTEXT` directly.
///
/// Saboteur paths: not yet ported (always `false`).
/// `DeferredCommands` in attacker path: not yet ported (always empty).
pub struct StepDropFallingPlayers {
    /// Java: state.oldDefenderState
    pub old_defender_state: Option<PlayerState>,
    /// Java: state.injuryResultDefender — cached across sub-steps
    pub injury_result_defender: Option<Box<InjuryResult>>,
    /// Java: state.usingPilingOn (bb2016/bb2020 `PilingOnBehaviour` hook state) — None = not
    /// asked, Some = decided. Always None for BB2025 (no `PilingOn` skill/hook exists there).
    pub using_piling_on: Option<bool>,
    /// Java: local `success` flag from `PilingOnBehaviour.rollWeepingDagger` on the defender's
    /// initial (pre-PilingOn) injury — published as `StepParameter::DefenderPoisoned`.
    pub defender_poisoned: bool,
    /// Java: state.saboteurTriggeredAttacker
    pub saboteur_triggered_attacker: bool,
    /// Java: state.usingSaboteurAttacker (Boolean tristate)
    pub using_saboteur_attacker: Option<bool>,
    /// Java: state.saboteurTriggeredDefender
    pub saboteur_triggered_defender: bool,
    /// Java: state.usingSaboteurDefender (Boolean tristate)
    pub using_saboteur_defender: Option<bool>,
}

impl StepDropFallingPlayers {
    pub fn new() -> Self {
        Self {
            old_defender_state: None,
            injury_result_defender: None,
            using_piling_on: None,
            defender_poisoned: false,
            saboteur_triggered_attacker: false,
            using_saboteur_attacker: None,
            saboteur_triggered_defender: false,
            using_saboteur_defender: None,
        }
    }
}

impl Default for StepDropFallingPlayers {
    fn default() -> Self { Self::new() }
}

impl Step for StepDropFallingPlayers {
    fn id(&self) -> StepId { StepId::DropFallingPlayers }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_USE_SKILL → handleSkillCommand → EXECUTE_STEP
        // (bb2016/bb2020 `PilingOnBehaviour.handleCommandHook`; never reached for BB2025 since
        // no `AgentPrompt::PilingOn` is ever emitted there.)
        if let Action::UseSkill { use_skill, .. } = action {
            self.using_piling_on = Some(*use_skill);
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::OldDefenderState(v) => {
                self.old_defender_state = Some(*v);
                true
            }
            _ => false,
        }
    }
}

impl StepDropFallingPlayers {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: if (getGameState().executeStepHooks(this, state)) return;
        // client-only: Saboteur step hook / dialog — headless skips Saboteur activation

        let attacker_id = game.acting_player.player_id.clone().unwrap_or_default();
        let defender_id = game.defender_id.clone().unwrap_or_default();

        // ── Defender fall ──────────────────────────────────────────────────────

        let mut defender_state = game.field_model.player_state(&defender_id);
        let defender_coord = game.field_model.player_coordinate(&defender_id)
            .unwrap_or(FieldCoordinate::new(0, 0));

        // Java: if (FALLING && isRooted) changeRooted(false)
        if let Some(s) = defender_state {
            if s.base() == PS_FALLING && s.is_rooted() {
                let new_s = s.change_rooted(false);
                game.field_model.set_player_state(&defender_id, new_s);
                defender_state = Some(new_s);
            }
        }
        // Java: if (HIT_ON_GROUND) changeBase(FALLING)
        if let Some(s) = defender_state {
            if s.base() == PS_HIT_ON_GROUND {
                let new_s = s.change_base(PS_FALLING);
                game.field_model.set_player_state(&defender_id, new_s);
                defender_state = Some(new_s);
            }
        }

        let defender_is_falling = defender_state.map(|s| s.base() == PS_FALLING).unwrap_or(false);

        // Java: `PilingOn` only exists for BB2016/BB2020 (`skill/bb2016/PilingOn.java`,
        // `skill/bb2020/PilingOn.java` — no BB2025 equivalent); its behaviour hooks are
        // registered on the COMMON `action.block.StepDropFallingPlayers`, which this shared
        // BB2025+ step also stands in for since this crate has one step-dispatch table.
        let piling_on_supported = matches!(game.rules, Rules::Bb2016 | Rules::Bb2020);

        if defender_is_falling && self.injury_result_defender.is_none() {
            // Java: pick injuryType based on oldDefenderState
            // Ball and Chain + Violent Innovator grantsSpp check — stub false (skill props not yet wired)
            let grants_spp = false;
            let injury_type_name = match self.old_defender_state {
                Some(s) if s.is_stunned() => {
                    if grants_spp { "InjuryTypeBlockStunnedForSpp" } else { "InjuryTypeBlockStunned" }
                }
                Some(s) if s.is_prone_or_stunned() => {
                    if grants_spp { "InjuryTypeBlockProneForSpp" } else { "InjuryTypeBlockProne" }
                }
                // BB2016 PilingOnBehaviour: `new InjuryTypeBlock()` (Mode.REGULAR, chainsaw=true).
                _ if game.rules == Rules::Bb2016 => "InjuryTypeBlock",
                // BB2020 PilingOnBehaviour (attacker not also falling): `new InjuryTypeBlock(mode, false)`
                // with `mode = REGULAR` — the attacker-also-falling `DO_NOT_USE_MODIFIERS` sub-case is
                // handled below via a direct construction (no matching dispatch-by-name key exists).
                // BB2025 (non-PilingOn): `new InjuryTypeBlock(Mode.REGULAR, false)`, unconditionally.
                _ => "InjuryTypeBlockNoAttackerChainsaw",
            };
            // Saboteur overrides (not yet ported)
            let injury_type_name = if self.saboteur_triggered_defender { "InjuryTypeSaboteur" }
                else if self.saboteur_triggered_attacker { "InjuryTypeSabotaged" }
                else { injury_type_name };

            // BB2020 PilingOnBehaviour: when the attacker is ALSO falling, the regular-case mode
            // is `DO_NOT_USE_MODIFIERS` rather than `REGULAR` — no string key for that combination
            // exists in `make_injury_type`, so construct it directly for this one narrow case.
            let attacker_also_falling = game.field_model.player_state(&attacker_id)
                .map(|s| s.base() == PS_FALLING).unwrap_or(false);
            let result = if game.rules == Rules::Bb2020
                && injury_type_name == "InjuryTypeBlockNoAttackerChainsaw"
                && attacker_also_falling
                && !self.saboteur_triggered_defender && !self.saboteur_triggered_attacker
            {
                handle_injury(
                    game, rng, &mut InjuryTypeBlock::new_with_chainsaw(BlockMode::DoNotUseModifiers, true, false),
                    Some(&attacker_id.clone()), &defender_id,
                    defender_coord, None, None, ApothecaryMode::Defender,
                )
            } else {
                handle_injury_by_name(
                    game, rng, injury_type_name,
                    Some(&attacker_id.clone()), &defender_id,
                    defender_coord, None, None, ApothecaryMode::Defender,
                )
            };
            self.injury_result_defender = Some(Box::new(result));

            // Java: PilingOnBehaviour — Weeping Dagger poison on the defender's initial injury.
            if piling_on_supported
                && game.player(&attacker_id)
                    .map(|p| p.has_skill_property(NamedProperties::APPLIES_POISON_ON_BADLY_HURT))
                    .unwrap_or(false)
                && self.injury_result_defender.as_ref()
                    .map(|ir| ir.injury_context.is_badly_hurt())
                    .unwrap_or(false)
            {
                if roll_weeping_dagger(game, rng, &attacker_id, &defender_id) {
                    self.defender_poisoned = true;
                }
            }
        }

        let mut out = StepOutcome::next();

        // Java: PilingOnBehaviour.handleExecuteStepHook — BB2016/BB2020 only.
        if piling_on_supported {
            if let Some(using) = self.using_piling_on {
                // Phase 2: PilingOn dialog answered.
                let re_roll_injury = self.injury_result_defender.as_ref()
                    .map(|ir| ir.injury_context.is_armor_broken())
                    .unwrap_or(false);
                game.report_list.add(ReportPilingOn::new(attacker_id.clone(), using, re_roll_injury));

                let uses_a_team_reroll = is_option_enabled(game, PILING_ON_USES_A_TEAM_REROLL);
                let reroll_spent = !uses_a_team_reroll
                    || use_reroll(game, &ReRollSource::new("TRR"), &attacker_id);

                if using && reroll_spent {
                    game.mark_skill_used(&attacker_id, ffb_model::enums::SkillId::PilingOn);
                    for p in drop_player(game, &attacker_id, false) {
                        out = out.publish(p);
                    }

                    let attacker_coord_for_ko = game.field_model.player_coordinate(&attacker_id)
                        .unwrap_or(FieldCoordinate::new(0, 0));
                    let (new_ir, rolled_double) = if re_roll_injury {
                        let old = self.injury_result_defender.as_deref();
                        let ir = handle_injury(
                            game, rng, &mut InjuryTypePilingOnInjury::new(),
                            Some(&attacker_id), &defender_id, defender_coord, None, old, ApothecaryMode::Defender,
                        );
                        let d = ir.injury_context.get_injury_roll()
                            .map(|r| DiceInterpreter::is_double(&r)).unwrap_or(false);
                        (ir, d)
                    } else {
                        let ir = handle_injury(
                            game, rng, &mut InjuryTypePilingOnArmour::new(),
                            Some(&attacker_id), &defender_id, defender_coord, None, None, ApothecaryMode::Defender,
                        );
                        let d = ir.injury_context.get_armor_roll()
                            .map(|r| DiceInterpreter::is_double(&r)).unwrap_or(false);
                        (ir, d)
                    };
                    self.injury_result_defender = Some(Box::new(new_ir));

                    if rolled_double && is_option_enabled(game, PILING_ON_TO_KO_ON_DOUBLE) {
                        let ko = handle_injury(
                            game, rng, &mut InjuryTypePilingOnKnockedOut::new(),
                            None, &attacker_id, attacker_coord_for_ko, None, None, ApothecaryMode::Attacker,
                        );
                        out = out.publish(StepParameter::InjuryResult(Box::new(ko)));
                    }
                }
            } else if defender_is_falling {
                // Phase 1: check PilingOn eligibility (mirrors the just-computed initial drop).
                let uses_a_team_reroll = is_option_enabled(game, PILING_ON_USES_A_TEAM_REROLL);
                let armor_broken = self.injury_result_defender.as_ref()
                    .map(|ir| ir.injury_context.is_armor_broken()).unwrap_or(false);
                let is_casualty = self.injury_result_defender.as_ref()
                    .map(|ir| ir.injury_context.is_casualty()).unwrap_or(false);
                let attacker_rooted = game.field_model.player_state(&attacker_id)
                    .map(|s| s.is_rooted()).unwrap_or(false);
                let attacker_also_falling = game.field_model.player_state(&attacker_id)
                    .map(|s| s.base() == PS_FALLING).unwrap_or(false);
                let piling_on_eligible = !attacker_also_falling
                    && game.player(&attacker_id)
                        .map(|p| p.has_unused_skill_with_property(NamedProperties::CAN_PILE_ON_OPPONENT))
                        .unwrap_or(false)
                    && (!uses_a_team_reroll
                        || game.player(&attacker_id)
                            .map(|p| UtilServerReRoll::is_team_re_roll_available(game, p))
                            .unwrap_or(false))
                    && game.field_model.player_coordinate(&attacker_id).zip(Some(defender_coord))
                        .map(|(a, d)| a.is_adjacent(d)).unwrap_or(false)
                    && !is_casualty
                    && !attacker_rooted
                    && (!is_option_enabled(game, PILING_ON_INJURY_ONLY) || armor_broken)
                    && (!is_option_enabled(game, PILING_ON_ARMOR_ONLY) || !armor_broken)
                    && (!game.player(&defender_id)
                        .map(|p| p.has_skill_property(NamedProperties::PREVENT_ARMOUR_MODIFICATIONS))
                        .unwrap_or(false)
                        || armor_broken)
                    && !game.player(&attacker_id)
                        .map(|p| UtilCards::has_skill_to_cancel_property(p, NamedProperties::CAN_PILE_ON_OPPONENT))
                        .unwrap_or(false)
                    && !game.player(&defender_id)
                        .map(|p| p.has_skill_property(NamedProperties::PREVENT_DAMAGING_INJURY_MODIFICATIONS))
                        .unwrap_or(false);

                if piling_on_eligible {
                    if let Some(ir) = self.injury_result_defender.as_deref_mut() {
                        ir.report(game);
                    }
                    return StepOutcome::cont().with_prompt(AgentPrompt::PilingOn {
                        player_id: attacker_id.clone(),
                        target_id: defender_id.clone(),
                    });
                }
            }
        }

        if piling_on_supported {
            if self.defender_poisoned {
                out = out.publish(StepParameter::DefenderPoisoned(true));
            }
            if let Some(v) = self.using_piling_on {
                out = out.publish(StepParameter::UsingPilingOn(v));
            }
        }

        // Java: droppedOwnTeam = FALLING && defender.team == attacker.team
        //                         && oldDefenderState != null && !oldDefenderState.isProneOrStunned()
        let attacker_team_id = game.player_team_id(&attacker_id).map(|s| s.to_owned());
        let defender_team_id = game.player_team_id(&defender_id).map(|s| s.to_owned());
        let dropped_own_team = defender_is_falling
            && attacker_team_id.is_some()
            && attacker_team_id == defender_team_id
            && self.old_defender_state.is_some()
            && !self.old_defender_state.map(|s| s.is_prone_or_stunned()).unwrap_or(false);

        if let Some(ref injury_result) = self.injury_result_defender {
            let dpc = DropPlayerContext {
                injury_result: Some(injury_result.clone()),
                end_turn: dropped_own_team,
                eligible_for_safe_pair_of_hands: true,
                label: None,
                player_id: if defender_id.is_empty() { None } else { Some(defender_id.clone()) },
                apothecary_mode: Some(ApothecaryMode::Defender),
                requires_armour_break: false,
                ..DropPlayerContext::new()
            };

            if self.saboteur_triggered_defender {
                // Bypass Steady Footing — publish DROP_PLAYER_CONTEXT + INJURY_RESULT directly
                out = out.publish(StepParameter::DropPlayerContext(Box::new(dpc)));
                out = out.publish(StepParameter::InjuryResult(injury_result.clone()));
            } else {
                let ctx = SteadyFootingContext::from_drop_player(dpc);
                out = out.publish(StepParameter::SteadyFootingContext(Box::new(ctx)));
            }
        } else if dropped_own_team && !self.saboteur_triggered_defender && !self.saboteur_triggered_attacker {
            out = out.publish(StepParameter::EndTurn(true));
        }

        // ── Attacker fall ──────────────────────────────────────────────────────

        let mut attacker_state = game.field_model.player_state(&attacker_id);
        let attacker_coord = game.field_model.player_coordinate(&attacker_id)
            .unwrap_or(FieldCoordinate::new(0, 0));

        // Java: if (FALLING && isRooted) changeRooted(false)
        if let Some(s) = attacker_state {
            if s.base() == PS_FALLING && s.is_rooted() {
                let new_s = s.change_rooted(false);
                game.field_model.set_player_state(&attacker_id, new_s);
                attacker_state = Some(new_s);
            }
        }

        let attacker_is_falling = attacker_state.map(|s| s.base() == PS_FALLING).unwrap_or(false);

        if attacker_is_falling {
            if !(self.saboteur_triggered_defender || self.saboteur_triggered_attacker) {
                out = out.publish(StepParameter::EndTurn(true));
            }

            let fell_from_rush = game.acting_player.fell_from_rush;

            // Java: `PilingOnBehaviour` (bb2016/bb2020) drops the attacker directly before
            // rolling their own injury, in the non-fell-from-rush branch; BB2025's native class
            // uses a not-yet-ported `DeferredCommand` instead (left as a documented gap).
            if piling_on_supported && !fell_from_rush
                && !self.saboteur_triggered_attacker && !self.saboteur_triggered_defender
            {
                // BB2020: `dropPlayer(..., true)`; BB2016: `dropPlayer(...)` (no SPOH arg, i.e. false).
                let eligible_for_safe_pair_of_hands = game.rules == Rules::Bb2020;
                for p in drop_player(game, &attacker_id, eligible_for_safe_pair_of_hands) {
                    out = out.publish(p);
                }
            }

            let injury_result_attacker = if piling_on_supported && game.rules == Rules::Bb2020
                && !fell_from_rush && !self.saboteur_triggered_attacker && !self.saboteur_triggered_defender
            {
                // Java: `new InjuryTypeBlock(Mode.DO_NOT_USE_MODIFIERS)` (allowAttackerChainsaw=true).
                handle_injury(
                    game, rng, &mut InjuryTypeBlock::new_with_chainsaw(BlockMode::DoNotUseModifiers, true, true),
                    Some(&defender_id), &attacker_id, attacker_coord, None, None, ApothecaryMode::Attacker,
                )
            } else {
                let injury_type_attacker = if fell_from_rush {
                    "InjuryTypeDropGFI"
                } else if self.saboteur_triggered_attacker {
                    "InjuryTypeSaboteur"
                } else if self.saboteur_triggered_defender {
                    "InjuryTypeSabotaged"
                } else {
                    "InjuryTypeBlock"
                };
                // Java: defender passed as attacker of the injury (punching back)
                handle_injury_by_name(
                    game, rng, injury_type_attacker,
                    Some(&defender_id),
                    &attacker_id,
                    attacker_coord, None, None, ApothecaryMode::Attacker,
                )
            };

            // Java: `PilingOnBehaviour` — Weeping Dagger poison on the attacker's own fall
            // injury, only in the non-fell-from-rush branch.
            if piling_on_supported && !fell_from_rush
                && game.player(&defender_id)
                    .map(|p| p.has_skill_property(NamedProperties::APPLIES_POISON_ON_BADLY_HURT))
                    .unwrap_or(false)
                && injury_result_attacker.injury_context.is_badly_hurt()
                && roll_weeping_dagger(game, rng, &defender_id, &attacker_id)
            {
                out = out.publish(StepParameter::AttackerPoisoned(true));
            }

            if self.saboteur_triggered_attacker {
                let dpc = DropPlayerContext {
                    injury_result: Some(Box::new(injury_result_attacker.clone())),
                    end_turn: false,
                    eligible_for_safe_pair_of_hands: false,
                    label: None,
                    player_id: if attacker_id.is_empty() { None } else { Some(attacker_id.clone()) },
                    apothecary_mode: Some(ApothecaryMode::Attacker),
                    requires_armour_break: false,
                    ..DropPlayerContext::new()
                };
                out = out.publish(StepParameter::DropPlayerContext(Box::new(dpc)));
                out = out.publish(StepParameter::InjuryResult(Box::new(injury_result_attacker)));
            } else {
                // Java: new SteadyFootingContext(injuryResultAttacker, deferredCommands)
                // deferredCommands not yet ported — always empty; InjuryResult variant
                let ctx = SteadyFootingContext::from_injury_result(injury_result_attacker);
                out = out.publish(StepParameter::SteadyFootingContext(Box::new(ctx)));
            }
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{PlayerState, PS_FALLING, PS_HIT_ON_GROUND, PS_STANDING, PS_STUNNED, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    fn make_game_with_rules(rules: Rules) -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, rules)
    }

    fn add_player(game: &mut Game, team: &str, id: &str, coord: FieldCoordinate, state_base: u32) {
        let p = Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        if team == "home" { game.team_home.players.push(p); }
        else { game.team_away.players.push(p); }
        game.field_model.set_player_coordinate(id, coord);
        game.field_model.set_player_state(id, PlayerState::new(state_base));
    }

    #[test]
    fn start_no_players_returns_next() {
        let mut game = make_game();
        let mut step = StepDropFallingPlayers::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn old_defender_state_parameter_accepted() {
        let mut step = StepDropFallingPlayers::default();
        let state = PlayerState::new(PS_STUNNED);
        assert!(step.set_parameter(&StepParameter::OldDefenderState(state)));
        assert_eq!(step.old_defender_state, Some(state));
    }

    #[test]
    fn unknown_parameter_rejected() {
        let mut step = StepDropFallingPlayers::default();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn defender_falling_publishes_steady_footing_context() {
        let mut game = make_game();
        let coord = FieldCoordinate::new(5, 5);
        add_player(&mut game, "home", "atk", coord, PS_STANDING);
        add_player(&mut game, "away", "def", coord, PS_FALLING);
        game.acting_player.player_id = Some("atk".into());
        game.defender_id = Some("def".into());

        let mut step = StepDropFallingPlayers::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::SteadyFootingContext(_))),
            "expected SteadyFootingContext for falling defender");
    }

    #[test]
    fn defender_hit_on_ground_upgraded_to_falling() {
        let mut game = make_game();
        let coord = FieldCoordinate::new(5, 5);
        add_player(&mut game, "home", "atk", coord, PS_STANDING);
        add_player(&mut game, "away", "def", coord, PS_HIT_ON_GROUND);
        game.acting_player.player_id = Some("atk".into());
        game.defender_id = Some("def".into());

        let mut step = StepDropFallingPlayers::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        // HIT_ON_GROUND is upgraded → injury is performed → STEADY_FOOTING_CONTEXT published
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::SteadyFootingContext(_))));
    }

    #[test]
    fn attacker_falling_publishes_steady_footing_context_and_end_turn() {
        let mut game = make_game();
        let coord = FieldCoordinate::new(5, 5);
        add_player(&mut game, "home", "atk", coord, PS_FALLING);
        add_player(&mut game, "away", "def", coord, PS_STANDING);
        game.acting_player.player_id = Some("atk".into());
        game.defender_id = Some("def".into());

        let mut step = StepDropFallingPlayers::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::SteadyFootingContext(_))),
            "expected SteadyFootingContext for falling attacker");
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))),
            "expected END_TURN for falling attacker");
    }

    #[test]
    fn no_players_falling_no_parameters_published() {
        let mut game = make_game();
        let coord = FieldCoordinate::new(5, 5);
        add_player(&mut game, "home", "atk", coord, PS_STANDING);
        add_player(&mut game, "away", "def", coord, PS_STANDING);
        game.acting_player.player_id = Some("atk".into());
        game.defender_id = Some("def".into());

        let mut step = StepDropFallingPlayers::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.is_empty(), "expected no parameters published");
    }

    #[test]
    fn saboteur_defender_bypasses_steady_footing() {
        let mut game = make_game();
        let coord = FieldCoordinate::new(5, 5);
        add_player(&mut game, "home", "atk", coord, PS_STANDING);
        add_player(&mut game, "away", "def", coord, PS_FALLING);
        game.acting_player.player_id = Some("atk".into());
        game.defender_id = Some("def".into());

        let mut step = StepDropFallingPlayers::new();
        step.saboteur_triggered_defender = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Should publish DROP_PLAYER_CONTEXT + INJURY_RESULT, NOT STEADY_FOOTING_CONTEXT
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::DropPlayerContext(_))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::InjuryResult(_))));
        assert!(!out.published.iter().any(|p| matches!(p, StepParameter::SteadyFootingContext(_))));
    }

    // ── PilingOn (BB2016/BB2020 only) — Phase ABI ────────────────────────────────────

    fn add_piling_on_skill(game: &mut Game, player_id: &str) {
        use ffb_model::model::skill_def::SkillWithValue;
        use ffb_model::enums::SkillId;
        if let Some(p) = game.team_home.player_mut(player_id) {
            p.extra_skills.push(SkillWithValue::new(SkillId::PilingOn));
        }
    }

    #[test]
    fn bb2020_defender_falling_piling_on_eligible_returns_continue() {
        let mut game = make_game_with_rules(Rules::Bb2020);
        add_player(&mut game, "home", "atk", FieldCoordinate::new(5, 5), PS_STANDING);
        add_player(&mut game, "away", "def", FieldCoordinate::new(6, 5), PS_FALLING);
        add_piling_on_skill(&mut game, "atk");
        game.acting_player.player_id = Some("atk".into());
        game.defender_id = Some("def".into());

        let mut step = StepDropFallingPlayers::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
        assert!(!out.published.iter().any(|p| matches!(p, StepParameter::SteadyFootingContext(_))),
            "should wait for the PilingOn dialog response, not publish yet");
    }

    #[test]
    fn bb2016_no_piling_on_skill_publishes_immediately() {
        let mut game = make_game_with_rules(Rules::Bb2016);
        let coord = FieldCoordinate::new(5, 5);
        add_player(&mut game, "home", "atk", coord, PS_STANDING);
        add_player(&mut game, "away", "def", coord, PS_FALLING);
        game.acting_player.player_id = Some("atk".into());
        game.defender_id = Some("def".into());

        let mut step = StepDropFallingPlayers::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::SteadyFootingContext(_))));
    }

    #[test]
    fn bb2020_piling_on_declined_publishes_using_piling_on_false() {
        let mut game = make_game_with_rules(Rules::Bb2020);
        let coord = FieldCoordinate::new(5, 5);
        add_player(&mut game, "home", "atk", coord, PS_STANDING);
        add_player(&mut game, "away", "def", coord, PS_FALLING);
        add_piling_on_skill(&mut game, "atk");
        game.acting_player.player_id = Some("atk".into());
        game.defender_id = Some("def".into());

        let mut step = StepDropFallingPlayers::new();
        step.start(&mut game, &mut GameRng::new(0));
        let out = step.handle_command(
            &Action::UseSkill { skill_id: ffb_model::enums::SkillId::PilingOn, use_skill: false },
            &mut game, &mut GameRng::new(0),
        );
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::UsingPilingOn(false))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::SteadyFootingContext(_))));
    }

    #[test]
    fn bb2020_piling_on_accepted_marks_skill_used_and_drops_attacker() {
        let mut game = make_game_with_rules(Rules::Bb2020);
        let coord = FieldCoordinate::new(5, 5);
        add_player(&mut game, "home", "atk", coord, PS_STANDING);
        add_player(&mut game, "away", "def", coord, PS_FALLING);
        add_piling_on_skill(&mut game, "atk");
        game.acting_player.player_id = Some("atk".into());
        game.defender_id = Some("def".into());

        let mut step = StepDropFallingPlayers::new();
        step.start(&mut game, &mut GameRng::new(0));
        step.handle_command(
            &Action::UseSkill { skill_id: ffb_model::enums::SkillId::PilingOn, use_skill: true },
            &mut game, &mut GameRng::new(0),
        );
        assert!(
            game.player("atk").unwrap().used_skills.contains(&ffb_model::enums::SkillId::PilingOn),
            "PilingOn should be marked used on the attacker"
        );
        assert_eq!(game.field_model.player_state("atk").unwrap().base(), ffb_model::enums::PS_PRONE);
    }

    #[test]
    fn bb2020_piling_on_accepted_publishes_fresh_using_piling_on_and_steady_footing() {
        let mut game = make_game_with_rules(Rules::Bb2020);
        let coord = FieldCoordinate::new(5, 5);
        add_player(&mut game, "home", "atk", coord, PS_STANDING);
        add_player(&mut game, "away", "def", coord, PS_FALLING);
        add_piling_on_skill(&mut game, "atk");
        game.acting_player.player_id = Some("atk".into());
        game.defender_id = Some("def".into());

        let mut step = StepDropFallingPlayers::new();
        step.start(&mut game, &mut GameRng::new(0));
        let out = step.handle_command(
            &Action::UseSkill { skill_id: ffb_model::enums::SkillId::PilingOn, use_skill: true },
            &mut game, &mut GameRng::new(0),
        );
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::UsingPilingOn(true))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::SteadyFootingContext(_))),
            "re-rolled defender InjuryResult should still be published via SteadyFootingContext");
    }

    #[test]
    fn bb2025_piling_on_never_offered_even_with_skill_present() {
        // BB2025 has no PilingOn model in Java; the gate must hold even if a player
        // somehow carries the skill (defense in depth beyond data-driven exclusion).
        let mut game = make_game_with_rules(Rules::Bb2025);
        add_player(&mut game, "home", "atk", FieldCoordinate::new(5, 5), PS_STANDING);
        add_player(&mut game, "away", "def", FieldCoordinate::new(6, 5), PS_FALLING);
        add_piling_on_skill(&mut game, "atk");
        game.acting_player.player_id = Some("atk".into());
        game.defender_id = Some("def".into());

        let mut step = StepDropFallingPlayers::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep, "BB2025 must never offer the PilingOn dialog");
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::SteadyFootingContext(_))));
    }
}
