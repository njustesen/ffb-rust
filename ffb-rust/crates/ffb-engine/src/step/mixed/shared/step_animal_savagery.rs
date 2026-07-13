/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.shared.StepAnimalSavagery`
/// and its hooks `com.fumbbl.ffb.server.skillbehaviour.bb2020.AnimalSavageryBehaviour` /
/// `com.fumbbl.ffb.server.skillbehaviour.bb2025.AnimalSavageryBehaviour`.
///
/// Handles the Animal Savagery skill (BB2020+/Mixed) — a player with this skill may
/// lash out at opponents instead of being controlled.  The step drives:
///   1. Init: records `goto_label_on_failure`, `catcher_id` (from TARGET_COORDINATE),
///      and `block_defender_id`.
///   2. On `CLIENT_PLAYER_CHOICE(ANIMAL_SAVAGERY)` → set `player_id` and execute.
///   3. On `CLIENT_USE_SKILL(canLashOutAgainstOpponents)` → set `attack_opponent` flag.
///   4. Entire execution delegated to `executeStepHooks` — inlined here directly
///      (single-modifier-per-step; follows the Dauntless/UnchannelledFury "direct-in-step"
///      precedent rather than the generic dispatch registry).
///
/// Java: `com.fumbbl.ffb.server.step.mixed.shared.StepAnimalSavagery`
///        extends `AbstractStepWithReRoll`.
///
/// BB2020 vs BB2025 differ only in: the injury `Mode` used for the lash-out (Do-Not-Use vs
/// Use-Modifiers-Against-Team-Mates for BB2020, Use-Armour-Modifiers-Only for BB2025), how the
/// "end turn" branch is realised (BB2020 mutates state inline; BB2025 defers via
/// `SteadyFootingContext`'s deferred-command list), the BB2025-only
/// `ANIMAL_SAVAGERY_LASH_OUT_ENDS_ACTIVATION` game option branch, and
/// `ThrowTeamMate`'s turn-resource flag (`pass_used` for BB2020, `ttm_used` for BB2025) inside
/// `fallback_action`. There is no BB2016 variant (confirmed: only bb2020/ and bb2025/ packages
/// exist under `server/skillbehaviour/`).
use std::collections::HashSet;
use std::sync::Arc;
use ffb_model::enums::{ApothecaryMode, PlayerAction, ReRollSource, Rules, SkillId, PS_PRONE, PS_STANDING};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::model::team::Team;
use ffb_model::option::game_option_id::ANIMAL_SAVAGERY_LASH_OUT_ENDS_ACTIVATION;
use ffb_model::option::util_game_option::is_option_enabled;
use ffb_model::prompts::AgentPrompt;
use ffb_model::report::mixed::report_animal_savagery::ReportAnimalSavagery;
use ffb_model::report::report_confusion_roll::ReportConfusionRoll;
use ffb_model::report::report_skill_use::ReportSkillUse;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use ffb_mechanics::mechanics::minimum_roll_confusion;
use crate::action::Action;
use crate::drop_player_context::{DropPlayerContext, SteadyFootingContext, VictimStateKey};
use crate::injury::InjuryResult;
use crate::injury::injuryType::injury_type_block::{BlockMode, InjuryTypeBlock};
use crate::model::drop_player_context_builder::DropPlayerContextBuilder;
use crate::step::bb2025::command::{AnimalSavageryCancelActionCommand, AnimalSavageryControlCommand, StandingUpCommand};
use crate::step::framework::{DeferredCommand, Step, StepOutcome, StepId, StepParameter};
use crate::step::util_server_injury::handle_injury;
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use crate::util::UtilServerPlayerMove;

/// Internal state — mirrors Java inner class `StepAnimalSavagery.StepState`.
#[derive(Debug, Default)]
pub struct AnimalSavageryState {
    /// Java: `goToLabelOnFailure` (mandatory init param)
    pub goto_label_on_failure: String,
    /// Java: `playerId` — player selected via ANIMAL_SAVAGERY dialog
    pub player_id: Option<String>,
    /// Java: `thrownPlayerId`
    pub thrown_player_id: Option<String>,
    /// Java: `playerIds` — set of active players (populated by hooks)
    pub player_ids: std::collections::HashSet<String>,
    /// Java: `endTurn`
    pub end_turn: bool,
    /// Java: `catcherId` — resolved from TARGET_COORDINATE init param
    pub catcher_id: Option<String>,
    /// Java: `attackOpponent` — true when the skill use dialog approved lashing out
    pub attack_opponent: Option<bool>,
    /// Java: `blockDefenderId`
    pub block_defender_id: Option<String>,
}

/// Java: `StepAnimalSavagery` (mixed/shared, BB2020 + BB2025).
pub struct StepAnimalSavagery {
    pub state: AnimalSavageryState,
    // AbstractStepWithReRoll stubs
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
    /// Pending coordinate from TARGET_COORDINATE init param; resolved to catcher_id in start().
    pending_target_coordinate: Option<FieldCoordinate>,
}

impl StepAnimalSavagery {
    pub fn new(goto_label_on_failure: impl Into<String>) -> Self {
        Self {
            state: AnimalSavageryState {
                goto_label_on_failure: goto_label_on_failure.into(),
                ..Default::default()
            },
            re_rolled_action: None,
            re_roll_source: None,
            pending_target_coordinate: None,
        }
    }

    /// Java: `AnimalSavageryBehaviour.handleExecuteStepHook` (bb2020/bb2025, inlined).
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: if (StringTool.isProvided(state.playerId)) { lashOut(...); return false; }
        if let Some(target_id) = self.state.player_id.clone() {
            return self.lash_out(game, rng, target_id);
        }

        // Java: if (!game.getTurnMode().checkNegatraits()) → NEXT_STEP
        if !game.turn_mode.check_negatraits() {
            return StepOutcome::next();
        }

        let is_bb2025 = game.rules == Rules::Bb2025;
        let acting_player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        let mut outcome = StepOutcome::next();
        let mut status_failure;

        let has_skill = game
            .player(&acting_player_id)
            .map(|p| p.has_skill(SkillId::AnimalSavagery))
            .unwrap_or(false);

        if has_skill && self.state.attack_opponent.is_none() {
            status_failure = false;
            let re_rolled = self.re_rolled_action.as_deref() == Some("ANIMAL_SAVAGERY");
            let mut do_roll = true;

            // Java: reRolledAction handling (ReRolledActionFactory not ported — matches the
            // Dauntless/UnchannelledFury convention of a plain string tag).
            if re_rolled {
                if let Some(source_name) = self.re_roll_source.clone() {
                    let source = ReRollSource::new(source_name.as_str());
                    if !use_reroll(game, &source, &acting_player_id) {
                        do_roll = false;
                        status_failure = true;
                    }
                } else {
                    do_roll = false;
                    status_failure = true;
                }
            } else {
                // Java: doRoll = UtilCards.hasUnusedSkill(actingPlayer, skill)
                do_roll = game
                    .player(&acting_player_id)
                    .map(|p| p.has_skill(SkillId::AnimalSavagery) && !p.used_skills.contains(&SkillId::AnimalSavagery))
                    .unwrap_or(false);
            }

            if do_roll {
                // Java: step.commitTargetSelection()
                if let Some(ts) = &mut game.field_model.target_selection_state {
                    ts.commit();
                }
                let roll = rng.d6();
                let player_action = game.acting_player.player_action;
                let good_conditions = good_conditions_for_savagery(player_action);
                let min_roll = minimum_roll_confusion(good_conditions);
                let successful = roll >= min_roll;
                mark_skill_used(game, &acting_player_id, SkillId::AnimalSavagery);

                let mut waiting_prompt = None;
                if !successful {
                    status_failure = true;
                    if !re_rolled {
                        if let Some(prompt) = ask_for_reroll_if_available(game, "ANIMAL_SAVAGERY", min_roll, false) {
                            self.re_rolled_action = Some("ANIMAL_SAVAGERY".into());
                            self.re_roll_source = Some("TRR".into());
                            waiting_prompt = Some(prompt);
                        }
                    }
                } else {
                    status_failure = false;
                }

                // Java: reRolled = (reRolledAction != null && reRolledAction == step.getReRolledAction()
                //                    && step.getReRollSource() != null)
                // do_roll stayed true only if !re_rolled, or re_rolled && useReRoll() succeeded
                // (which requires a source) — so within this block, `re_rolled` alone is
                // equivalent to Java's tri-part condition.
                let reported_rerolled = re_rolled;
                game.report_list.add(ReportConfusionRoll::new(
                    Some(acting_player_id.clone()),
                    successful,
                    roll,
                    min_roll,
                    reported_rerolled,
                    Some(SkillId::AnimalSavagery.class_name().to_string()),
                ));
                let event = GameEvent::AnimalSavagery {
                    player_id: acting_player_id.clone(),
                    roll,
                    success: successful,
                };

                if let Some(prompt) = waiting_prompt {
                    return StepOutcome::cont().with_event(event).with_prompt(prompt);
                }
                outcome = outcome.with_event(event);
            }
        } else if self.state.attack_opponent.is_some() {
            status_failure = true;
        } else {
            status_failure = false;
        }

        if !status_failure {
            return outcome;
        }

        // ── Failure branch ────────────────────────────────────────────────────
        // Java: Team team = game.getActingTeam(); Team opponentTeam = game.getOtherTeam(team);
        let home_active = game.home_playing;
        let player_coordinate = game
            .field_model
            .player_coordinate(&acting_player_id)
            .unwrap_or(FieldCoordinate::new(0, 0));

        if self.state.attack_opponent.is_none() {
            // Java: UtilCards.hasUnusedSkillWithProperty(actingPlayer, canLashOutAgainstOpponents)
            let lash_out_skill = game
                .player(&acting_player_id)
                .and_then(|p| p.skill_id_with_property(NamedProperties::CAN_LASH_OUT_AGAINST_OPPONENTS));
            if let Some(skill_id) = lash_out_skill {
                let has_unused = game
                    .player(&acting_player_id)
                    .map(|p| p.has_unused_skill_with_property(NamedProperties::CAN_LASH_OUT_AGAINST_OPPONENTS))
                    .unwrap_or(false);
                if has_unused {
                    let opponent_team: &Team = if home_active { &game.team_away } else { &game.team_home };
                    let adjacent = adjacent_targets(game, opponent_team, player_coordinate);
                    if !adjacent.is_empty() {
                        return StepOutcome::cont().with_prompt(AgentPrompt::SkillUse {
                            player_id: acting_player_id.clone(),
                            skill_id: skill_id as u16,
                            skill_name: skill_id.class_name().to_string(),
                        });
                    }
                }
            }
            self.state.attack_opponent = Some(false);
        }

        let team_ref: &Team = if self.state.attack_opponent == Some(true) {
            if home_active { &game.team_away } else { &game.team_home }
        } else if home_active {
            &game.team_home
        } else {
            &game.team_away
        };
        let mut players = adjacent_targets(game, team_ref, player_coordinate);

        if let Some(thrown_id) = self.state.thrown_player_id.clone() {
            if !thrown_id.is_empty() {
                players.insert(thrown_id);
            }
        }

        if players.is_empty() {
            // Java: cancelPlayerAction(step, false) [bb2020] / new AnimalSavageryCancelActionCommand()
            // .execute(step) + manual player-state change [bb2025]
            if is_bb2025 {
                AnimalSavageryCancelActionCommand.execute(game);
                if let Some(state) = game.field_model.player_state(&acting_player_id) {
                    let base = if game.acting_player.standing_up { PS_PRONE } else { PS_STANDING };
                    let new_state = state.change_base(base).change_active(false).change_confused(true);
                    game.field_model.set_player_state(&acting_player_id, new_state);
                }
                // Java: step.getResult().setSound(SoundId.ROAR) — client-only, no Rust equivalent hook.
            } else {
                cancel_player_action_bb2020(game, &acting_player_id, false);
            }
            if let Some(ts) = &mut game.field_model.target_selection_state {
                ts.failed();
            }
            game.report_list.add(ReportAnimalSavagery::new(Some(acting_player_id.clone()), None));
            return StepOutcome::goto(&self.state.goto_label_on_failure)
                .publish(StepParameter::EndPlayerAction(true));
        }

        if players.len() == 1 {
            let only = players.into_iter().next().unwrap();
            self.lash_out(game, rng, only)
        } else {
            self.state.player_ids = players.clone();
            StepOutcome::cont().with_prompt(AgentPrompt::PlayerChoice {
                eligible_players: players.into_iter().collect(),
                reason: "ANIMAL_SAVAGERY".into(),
                descriptions: vec![],
            })
        }
    }

    /// Java: `AnimalSavageryBehaviour.lashOut` (bb2020/bb2025).
    fn lash_out(&mut self, game: &mut Game, rng: &mut GameRng, target_id: String) -> StepOutcome {
        let is_bb2025 = game.rules == Rules::Bb2025;
        let mut outcome = StepOutcome::next();

        // Java: if (StringTool.isProvided(game.getDefenderId())) publish GAZE_VICTIM_ID
        let old_defender_id = game.defender_id.clone();
        if old_defender_id.as_deref().map(|s| !s.is_empty()).unwrap_or(false) {
            outcome = outcome.publish(StepParameter::GazeVictimId(old_defender_id.clone()));
        }

        let acting_player_id = game.acting_player.player_id.clone().unwrap_or_default();
        game.defender_id = Some(target_id.clone());
        game.report_list.add(ReportAnimalSavagery::new(
            Some(acting_player_id.clone()),
            Some(target_id.clone()),
        ));

        let player_coordinate = game
            .field_model
            .player_coordinate(&target_id)
            .unwrap_or(FieldCoordinate::new(0, 0));

        let acting_home = game.team_home.has_player(&acting_player_id);
        let target_home = game.team_home.has_player(&target_id);
        let lashed_out_against_opponent = acting_home != target_home;

        // Java: InjuryTypeBlock.Mode mode = ... (bb2020: standing-up-or-different-teams →
        // DO_NOT_USE_MODIFIERS, else USE_MODIFIERS_AGAINST_TEAM_MATES; bb2025: always
        // USE_ARMOUR_MODIFIERS_ONLY_AGAINST_TEAM_MATES).
        let mode = if is_bb2025 {
            BlockMode::UseArmourModifiersOnlyAgainstTeamMates
        } else if game.acting_player.standing_up || lashed_out_against_opponent {
            BlockMode::DoNotUseModifiers
        } else {
            BlockMode::UseModifiersAgainstTeamMates
        };

        let mut injury_type = InjuryTypeBlock::new(mode, true);
        let injury_result = handle_injury(
            game,
            rng,
            &mut injury_type,
            Some(acting_player_id.as_str()),
            &target_id,
            player_coordinate,
            None,
            None,
            ApothecaryMode::AnimalSavagery,
        );

        let end_turn = UtilPlayer::has_ball(game, &target_id) && !lashed_out_against_opponent;

        let mut victim_state_key: Option<VictimStateKey> = None;
        let mut label: Option<String> = None;
        let mut additional_keys: Vec<VictimStateKey> = Vec::new();
        let mut deferred_commands: Vec<Arc<dyn DeferredCommand>> = Vec::new();

        if end_turn {
            if is_bb2025 {
                deferred_commands = vec![
                    Arc::new(StandingUpCommand),
                    Arc::new(AnimalSavageryCancelActionCommand),
                    Arc::new(AnimalSavageryControlCommand),
                ];
            } else {
                game.acting_player.standing_up = false;
                cancel_player_action_bb2020(game, &acting_player_id, true);
                outcome = outcome
                    .publish(StepParameter::EndPlayerAction(true))
                    .publish(StepParameter::UseAlternateLabel(true))
                    .publish(StepParameter::ThrownPlayerCoordinate(None));
            }
        } else if is_bb2025 && is_option_enabled(game, ANIMAL_SAVAGERY_LASH_OUT_ENDS_ACTIVATION) {
            AnimalSavageryCancelActionCommand.execute(game);
            outcome = outcome
                .publish(StepParameter::EndPlayerAction(true))
                .publish(StepParameter::UseAlternateLabel(true));
            if let Some(state) = game.field_model.player_state(&acting_player_id) {
                let base = if game.acting_player.standing_up { PS_PRONE } else { PS_STANDING };
                let new_state = state.change_base(base).change_active(false);
                game.field_model.set_player_state(&acting_player_id, new_state);
            }
            if let Some(ts) = &mut game.field_model.target_selection_state {
                ts.failed();
            }
        } else {
            let hit_target_team_mate = Some(target_id.as_str()) == self.state.thrown_player_id.as_deref()
                || Some(target_id.as_str()) == self.state.catcher_id.as_deref();
            let (action, extra_params) = fallback_action(
                game,
                game.acting_player.player_action,
                &injury_result,
                self.state.block_defender_id.as_deref(),
                hit_target_team_mate,
                is_bb2025,
            );
            for p in extra_params {
                outcome = outcome.publish(p);
            }

            let block_and_not_opponent =
                action.map(|a| a.is_block_action()).unwrap_or(false) && !lashed_out_against_opponent;
            if block_and_not_opponent {
                label = Some(self.state.goto_label_on_failure.clone());
            } else if action.is_some() && hit_target_team_mate {
                outcome = outcome
                    .publish(StepParameter::ResetPlayerAction(action.unwrap()))
                    .publish(StepParameter::UseAlternateLabel(true))
                    .publish(StepParameter::ThrownPlayerCoordinate(None));
            } else if action.is_none() && Some(target_id.as_str()) == self.state.thrown_player_id.as_deref() {
                victim_state_key = Some(VictimStateKey::ThrownPlayerState);
                additional_keys = vec![VictimStateKey::OldDefenderState];
            } else if lashed_out_against_opponent
                && !(game
                    .acting_player
                    .player_action
                    .map(|a| a.is_block_action())
                    .unwrap_or(false)
                    && action.is_none())
            {
                UtilServerPlayerMove::update_move_squares(game, false);
                outcome = outcome
                    .publish(StepParameter::MoveStack(vec![]))
                    .publish(StepParameter::UseAlternateLabel(true));
            }
        }

        if is_bb2025 {
            let mut builder = DropPlayerContextBuilder::builder()
                .injury_result(injury_result)
                .end_turn(end_turn)
                .eligible_for_safe_pair_of_hands(true)
                .apothecary_mode(ApothecaryMode::AnimalSavagery)
                .player_id(target_id.clone());
            if let Some(l) = label {
                builder = builder.label(l);
            }
            if let Some(k) = victim_state_key {
                builder = builder.victim_state_key(k);
            }
            if !additional_keys.is_empty() {
                builder = builder.additional_victim_state_keys(additional_keys);
            }
            let dpc = builder.build();
            let ctx = SteadyFootingContext::from_drop_player_with_commands(dpc, deferred_commands);
            outcome.publish(StepParameter::SteadyFootingContext(Box::new(ctx)))
        } else {
            let mut builder = DropPlayerContextBuilder::builder()
                .injury_result(injury_result)
                .end_turn(end_turn)
                .eligible_for_safe_pair_of_hands(true)
                .apothecary_mode(ApothecaryMode::AnimalSavagery)
                .player_id(target_id.clone());
            if let Some(l) = label {
                builder = builder.label(l);
            }
            if let Some(k) = victim_state_key {
                builder = builder.victim_state_key(k);
            }
            if !additional_keys.is_empty() {
                builder = builder.additional_victim_state_keys(additional_keys);
            }
            let dpc: DropPlayerContext = builder.build();
            outcome.publish(StepParameter::DropPlayerContext(Box::new(dpc)))
        }
    }
}

impl Default for StepAnimalSavagery {
    fn default() -> Self { Self::new("") }
}

impl Step for StepAnimalSavagery {
    fn id(&self) -> StepId { StepId::AnimalSavagery }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: init() resolves TARGET_COORDINATE → catcherId via FieldModel.getPlayer(coord)
        if let Some(coord) = self.pending_target_coordinate.take() {
            self.state.catcher_id = game.field_model.player_at(coord).cloned();
        }
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java CLIENT_PLAYER_CHOICE (ANIMAL_SAVAGERY) → state.playerId = chosen; EXECUTE_STEP
        // Java CLIENT_USE_SKILL (canLashOutAgainstOpponents) → state.attackOpponent = used; mark skill; EXECUTE_STEP
        match action {
            Action::PlayerChoice { player_id, mode, .. } if mode == "ANIMAL_SAVAGERY" => {
                self.state.player_id = player_id.clone();
                self.execute_step(game, rng)
            }
            Action::UseSkill { skill_id, use_skill } if *skill_id == SkillId::PrimalSavagery => {
                self.state.attack_opponent = Some(*use_skill);
                if *use_skill {
                    // Java: game.getPlayerById(commandUseSkill.getPlayerId()).markUsed(skill, game)
                    if let Some(ref pid) = self.state.player_id {
                        let is_home = game.team_home.player(pid).is_some();
                        if is_home { game.team_home.player_mut(pid).map(|p| p.used_skills.insert(SkillId::PrimalSavagery)); }
                        else { game.team_away.player_mut(pid).map(|p| p.used_skills.insert(SkillId::PrimalSavagery)); }
                    }
                    // Java: addReport(new ReportSkillUse(playerId, skill, skillUsed, SkillUse.LASH_OUT_AGAINST_OPPONENT))
                    let player_id_opt = self.state.player_id.clone();
                    game.report_list.add(ReportSkillUse::new(
                        player_id_opt.clone(),
                        SkillId::PrimalSavagery,
                        true,
                        SkillUse::LASH_OUT_AGAINST_OPPONENT,
                    ));
                    let skill_event = player_id_opt.map(|pid| {
                        GameEvent::SkillUse {
                            player_id: pid,
                            skill_id: SkillId::PrimalSavagery as u16,
                            used: true,
                        }
                    });
                    let out = self.execute_step(game, rng);
                    if let Some(ev) = skill_event { return out.with_event(ev); }
                    return out;
                }
                self.execute_step(game, rng)
            }
            Action::UseReRoll { use_reroll: false } => {
                self.re_roll_source = None;
                self.execute_step(game, rng)
            }
            _ => StepOutcome::next(),
        }
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(v) => {
                self.state.goto_label_on_failure = v.clone();
                true
            }
            StepParameter::TargetCoordinate(coord) => {
                // Java: Player catcher = game.getFieldModel().getPlayer(coord); state.catcherId = catcher.getId()
                // set_parameter has no game reference; store and resolve in start().
                self.pending_target_coordinate = Some(*coord);
                true
            }
            StepParameter::BlockDefenderId(v) => {
                self.state.block_defender_id = Some(v.clone());
                false // Java: super.setParameter() → not consumed
            }
            StepParameter::ThrownPlayerId(v) => {
                self.state.thrown_player_id = v.clone();
                false
            }
            StepParameter::EndTurn(v) => {
                self.state.end_turn = *v;
                false
            }
            _ => false,
        }
    }
}

/// Java: `AnimalSavageryBehaviour.adjacentTargets` (bb2020/bb2025, identical).
fn adjacent_targets(game: &Game, team: &Team, player_coordinate: FieldCoordinate) -> HashSet<String> {
    let mut set: HashSet<String> = UtilPlayer::find_adjacent_blockable_players(game, team, player_coordinate)
        .into_iter()
        .cloned()
        .collect();
    if let Some(defender_id) = game.defender_id.clone() {
        if team.has_player(&defender_id) {
            if let Some(defender_coord) = game.field_model.player_coordinate(&defender_id) {
                if player_coordinate.is_adjacent(defender_coord) {
                    set.insert(defender_id);
                }
            }
        }
    }
    set
}

/// Java: `AnimalSavageryBehaviour.fallbackAction` (bb2020/bb2025; differ only in the
/// ThrowTeamMate turn-resource flag). Returns the fallback `PlayerAction` plus any extra
/// `StepParameter`s the Java method published as a side effect (`MULTIPLE_BLOCK` case).
fn fallback_action(
    game: &mut Game,
    player_action: Option<PlayerAction>,
    injury_result: &InjuryResult,
    old_defender_id: Option<&str>,
    hit_target_team_mate: bool,
    is_bb2025: bool,
) -> (Option<PlayerAction>, Vec<StepParameter>) {
    let mut params = Vec::new();
    let player_removed = injury_result.injury_context().is_casualty() || injury_result.injury_context().is_knocked_out();
    let defender_id = match game.defender_id.clone() {
        Some(d) => d,
        None => return (None, params),
    };

    let action = match player_action {
        Some(PlayerAction::KickTeamMate) | Some(PlayerAction::KickTeamMateMove) => {
            if hit_target_team_mate {
                game.turn_data_mut().ktm_used = true;
                game.pass_coordinate = None;
                game.field_model.range_ruler = None;
                Some(PlayerAction::KickTeamMateMove)
            } else {
                None
            }
        }
        Some(PlayerAction::HandOver) | Some(PlayerAction::HandOverMove) => {
            if hit_target_team_mate { Some(PlayerAction::HandOverMove) } else { None }
        }
        Some(PlayerAction::Pass) | Some(PlayerAction::PassMove) => {
            if hit_target_team_mate {
                game.pass_coordinate = None;
                game.acting_player.has_passed = false;
                game.field_model.range_ruler = None;
                Some(PlayerAction::Pass)
            } else {
                None
            }
        }
        Some(PlayerAction::ThrowTeamMate) | Some(PlayerAction::ThrowTeamMateMove) => {
            if player_removed && hit_target_team_mate {
                if is_bb2025 {
                    game.turn_data_mut().ttm_used = true;
                } else {
                    game.turn_data_mut().pass_used = true;
                }
                game.pass_coordinate = None;
                game.field_model.range_ruler = None;
                Some(PlayerAction::ThrowTeamMateMove)
            } else {
                None
            }
        }
        Some(PlayerAction::Blitz) | Some(PlayerAction::BlitzMove) => {
            if let Some(state) = game.field_model.player_state(&defender_id) {
                let new_state = state.remove_all_target_selections();
                game.field_model.set_player_state(&defender_id, new_state);
            }
            None
        }
        Some(a @ PlayerAction::Block) | Some(a @ PlayerAction::ViciousVines) => {
            if old_defender_id == Some(defender_id.as_str()) { Some(a) } else { None }
        }
        Some(PlayerAction::MultipleBlock) => {
            params.push(StepParameter::PlayerIdToRemove(defender_id.clone()));
            if let Some(state) = game.field_model.player_state(&defender_id) {
                let new_state = state.change_selected_block_target(false);
                game.field_model.set_player_state(&defender_id, new_state);
            }
            None
        }
        _ => None,
    };

    (action, params)
}

/// Java: `AnimalSavageryBehaviour.cancelPlayerAction` (bb2020 only — bb2025 delegates to the
/// already-ported `AnimalSavageryCancelActionCommand` for both the empty-players branch and the
/// end-turn deferred-command list).
fn cancel_player_action_bb2020(game: &mut Game, player_id: &str, lashed_out: bool) {
    match game.acting_player.player_action {
        Some(PlayerAction::Blitz) | Some(PlayerAction::BlitzMove) | Some(PlayerAction::KickEmBlitz) => {
            game.turn_data_mut().blitz_used = true;
        }
        Some(PlayerAction::KickTeamMate) | Some(PlayerAction::KickTeamMateMove) => {
            game.turn_data_mut().ktm_used = true;
        }
        Some(PlayerAction::Pass)
        | Some(PlayerAction::PassMove)
        | Some(PlayerAction::ThrowTeamMate)
        | Some(PlayerAction::ThrowTeamMateMove) => {
            game.turn_data_mut().pass_used = true;
        }
        Some(PlayerAction::HandOver) | Some(PlayerAction::HandOverMove) => {
            game.turn_data_mut().hand_over_used = true;
        }
        Some(PlayerAction::Foul) | Some(PlayerAction::FoulMove) => {
            let allows_additional_foul = game
                .player(player_id)
                .map(|p| p.has_skill_property(NamedProperties::ALLOWS_ADDITIONAL_FOUL))
                .unwrap_or(false);
            if !allows_additional_foul {
                game.turn_data_mut().foul_used = true;
            }
        }
        _ => {}
    }

    if !lashed_out {
        if let Some(state) = game.field_model.player_state(player_id) {
            let new_state = if game.acting_player.standing_up {
                state.change_base(PS_PRONE).change_active(false)
            } else {
                state.change_base(PS_STANDING).change_active(false).change_confused(true)
            };
            game.field_model.set_player_state(player_id, new_state);
        }
        // Java: step.getResult().setSound(SoundId.ROAR) — client-only, no Rust equivalent hook.
    }

    game.pass_coordinate = None;
}

/// Java: `goodConditions = BlitzMove | isKickingDowned | Blitz | isBlockAction | MultipleBlock | StandUpBlitz`
fn good_conditions_for_savagery(player_action: Option<PlayerAction>) -> bool {
    match player_action {
        Some(pa) => {
            pa == PlayerAction::BlitzMove
                || pa.is_kicking_downed()
                || pa == PlayerAction::Blitz
                || pa.is_block_action()
                || pa == PlayerAction::MultipleBlock
                || pa == PlayerAction::StandUpBlitz
        }
        None => false,
    }
}

/// Mark a skill as used for the given player.
fn mark_skill_used(game: &mut Game, player_id: &str, skill: SkillId) {
    let is_home = game.team_home.player(player_id).is_some();
    if is_home {
        if let Some(p) = game.team_home.player_mut(player_id) {
            p.used_skills.insert(skill);
        }
    } else if let Some(p) = game.team_away.player_mut(player_id) {
        p.used_skills.insert(skill);
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{PlayerGender, PlayerType, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_player(game: &mut Game, home: bool, id: &str, skills: Vec<SkillId>) {
        let player = Player {
            id: id.into(),
            name: id.into(),
            nr: 1,
            position_id: "pos".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6,
            strength: 3,
            agility: 3,
            passing: 4,
            armour: 8,
            starting_skills: skills.into_iter().map(|s| SkillWithValue { skill_id: s, value: None }).collect(),
            extra_skills: vec![],
            temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0,
            stat_injuries: vec![],
            current_spps: 0,
            career_spps: 0,
            race: None,
            is_big_guy: false,
            ..Default::default()
        };
        if home {
            game.team_home.players.push(player);
        } else {
            game.team_away.players.push(player);
        }
    }

    fn seed_for_d6(target: i32) -> u64 {
        for s in 0u64..10_000 {
            if GameRng::new(s).d6() == target {
                return s;
            }
        }
        panic!("no seed for d6={}", target);
    }

    fn seed_for_d6_pair(first: i32, second: i32) -> u64 {
        for s in 0u64..50_000 {
            let mut rng = GameRng::new(s);
            if rng.d6() == first && rng.d6() == second {
                return s;
            }
        }
        panic!("no seed for d6 pair=({}, {})", first, second);
    }

    #[test]
    fn id_is_animal_savagery() {
        assert_eq!(StepAnimalSavagery::new("fail").id(), StepId::AnimalSavagery);
    }

    #[test]
    fn goto_label_set_from_parameter() {
        let mut step = StepAnimalSavagery::default();
        step.set_parameter(&StepParameter::GotoLabelOnFailure("skip".into()));
        assert_eq!(step.state.goto_label_on_failure, "skip");
    }

    #[test]
    fn block_defender_id_stored() {
        let mut step = StepAnimalSavagery::default();
        step.set_parameter(&StepParameter::BlockDefenderId("def1".into()));
        assert_eq!(step.state.block_defender_id.as_deref(), Some("def1"));
    }

    #[test]
    fn thrown_player_id_stored() {
        let mut step = StepAnimalSavagery::default();
        step.set_parameter(&StepParameter::ThrownPlayerId(Some("thrown1".into())));
        assert_eq!(step.state.thrown_player_id.as_deref(), Some("thrown1"));
    }

    #[test]
    fn end_turn_stored() {
        let mut step = StepAnimalSavagery::default();
        step.set_parameter(&StepParameter::EndTurn(true));
        assert!(step.state.end_turn);
    }

    #[test]
    fn player_choice_animal_savagery_sets_player_id() {
        let mut step = StepAnimalSavagery::new("fail");
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        step.handle_command(
            &Action::PlayerChoice {
                player_id: Some("p1".into()),
                player_ids: vec![],
                mode: "ANIMAL_SAVAGERY".into(),
            },
            &mut game,
            &mut rng,
        );
        assert_eq!(step.state.player_id.as_deref(), Some("p1"));
    }

    #[test]
    fn target_coordinate_resolves_to_catcher_id_in_start() {
        let mut step = StepAnimalSavagery::new("fail");
        let mut game = make_game();
        let mut rng = GameRng::new(0);

        let player = Player { id: "catcher1".into(), ..Default::default() };
        game.team_home.players.push(player);
        let coord = FieldCoordinate::new(5, 5);
        game.field_model.set_player_coordinate("catcher1", coord);

        assert!(step.set_parameter(&StepParameter::TargetCoordinate(coord)));
        assert!(step.state.catcher_id.is_none());

        step.start(&mut game, &mut rng);
        assert_eq!(step.state.catcher_id.as_deref(), Some("catcher1"));
    }

    #[test]
    fn use_skill_lash_out_sets_attack_opponent() {
        let mut step = StepAnimalSavagery::new("fail");
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        step.handle_command(
            &Action::UseSkill { skill_id: SkillId::PrimalSavagery, use_skill: true },
            &mut game,
            &mut rng,
        );
        assert_eq!(step.state.attack_opponent, Some(true));
    }

    #[test]
    fn use_skill_lash_out_adds_skill_use_report() {
        use ffb_model::report::report_id::ReportId;
        let mut step = StepAnimalSavagery::new("fail");
        step.state.player_id = Some("p1".into());
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        step.handle_command(
            &Action::UseSkill { skill_id: SkillId::PrimalSavagery, use_skill: true },
            &mut game,
            &mut rng,
        );
        assert!(game.report_list.has_report(ReportId::SKILL_USE),
            "should add a SKILL_USE report when lashing out");
    }

    #[test]
    fn use_skill_not_used_does_not_add_report() {
        use ffb_model::report::report_id::ReportId;
        let mut step = StepAnimalSavagery::new("fail");
        step.state.player_id = Some("p1".into());
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        step.handle_command(
            &Action::UseSkill { skill_id: SkillId::PrimalSavagery, use_skill: false },
            &mut game,
            &mut rng,
        );
        assert!(!game.report_list.has_report(ReportId::SKILL_USE),
            "should not add a report when skill is not used");
    }

    // ── Negatrait gate ───────────────────────────────────────────────────────

    #[test]
    fn negatrait_gate_skips_when_turn_mode_disallows() {
        use ffb_model::enums::TurnMode;
        let mut game = make_game();
        game.turn_mode = TurnMode::KickoffReturn;
        game.acting_player.player_id = Some("p1".into());
        add_player(&mut game, true, "p1", vec![SkillId::AnimalSavagery]);
        let out = StepAnimalSavagery::new("fail").start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn no_acting_player_returns_next() {
        let mut game = make_game();
        game.acting_player.player_id = None;
        let out = StepAnimalSavagery::new("fail").start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn no_skill_and_no_attack_opponent_returns_next() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        add_player(&mut game, true, "p1", vec![]);
        let out = StepAnimalSavagery::new("fail").start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    // ── Roll pass/fail ───────────────────────────────────────────────────────

    #[test]
    fn successful_roll_returns_next_with_event() {
        let seed = seed_for_d6(6); // good conditions min=2, 6 succeeds
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Blitz);
        add_player(&mut game, true, "p1", vec![SkillId::AnimalSavagery]);
        let out = StepAnimalSavagery::new("fail").start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.events.iter().any(|e| matches!(e, GameEvent::AnimalSavagery { success: true, .. })));
        assert!(game.report_list.has_report(ffb_model::report::report_id::ReportId::CONFUSION_ROLL));
    }

    #[test]
    fn failed_roll_no_reroll_no_lash_skill_goes_to_failure_label() {
        let seed = seed_for_d6(1); // bad conditions min=4, roll=1 fails
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Move);
        add_player(&mut game, true, "p1", vec![SkillId::AnimalSavagery]);
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        let out = StepAnimalSavagery::new("FAIL_LABEL").start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(game.report_list.has_report(ffb_model::report::report_id::ReportId::ANIMAL_SAVAGERY));
    }

    #[test]
    fn skill_already_used_skips_roll_and_returns_next() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        add_player(&mut game, true, "p1", vec![SkillId::AnimalSavagery]);
        game.team_home.players[0].used_skills.insert(SkillId::AnimalSavagery);
        let out = StepAnimalSavagery::new("fail").start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep, "already-used skill this drive: no repeat check");
    }

    // ── Re-roll paths ────────────────────────────────────────────────────────

    #[test]
    fn failed_roll_with_trr_offers_reroll_prompt() {
        let seed = seed_for_d6(1);
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Move);
        add_player(&mut game, true, "p1", vec![SkillId::AnimalSavagery]);
        game.turn_data_home.rerolls = 1;
        let mut step = StepAnimalSavagery::new("fail");
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::Continue);
        assert!(out.prompt.is_some());
        assert_eq!(step.re_rolled_action.as_deref(), Some("ANIMAL_SAVAGERY"));
    }

    #[test]
    fn decline_reroll_then_goes_to_failure() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Move);
        add_player(&mut game, true, "p1", vec![SkillId::AnimalSavagery]);
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        let mut step = StepAnimalSavagery::new("FAIL");
        step.re_rolled_action = Some("ANIMAL_SAVAGERY".into());
        step.re_roll_source = Some("TRR".into());
        let out = step.handle_command(&Action::UseReRoll { use_reroll: false }, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn reroll_success_after_decline_of_previous_offer() {
        // Second roll (post-reroll) succeeds.
        let seed = seed_for_d6(6);
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Blitz);
        add_player(&mut game, true, "p1", vec![SkillId::AnimalSavagery]);
        game.turn_data_home.rerolls = 1;
        let mut step = StepAnimalSavagery::new("fail");
        step.re_rolled_action = Some("ANIMAL_SAVAGERY".into());
        step.re_roll_source = Some("TRR".into());
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(game.turn_data_home.reroll_used);
    }

    // ── Single-target auto lash-out ──────────────────────────────────────────

    #[test]
    fn single_adjacent_opponent_auto_lashes_out() {
        // Java: without an unused `canLashOutAgainstOpponents` skill (e.g. PrimalSavagery),
        // `state.attackOpponent` defaults to `false` and the lash-out target pool is the
        // acting player's OWN team (`team = game.getActingTeam()`), not the opponent's.
        let seed = seed_for_d6(1); // fail roll, no reroll available
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Move);
        add_player(&mut game, true, "p1", vec![SkillId::AnimalSavagery]);
        add_player(&mut game, true, "mate1", vec![]);
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_coordinate("mate1", FieldCoordinate::new(6, 5));
        game.field_model.set_player_state("mate1", ffb_model::enums::PlayerState::new(PS_STANDING));
        let out = StepAnimalSavagery::new("fail").start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::NextStep, "single target -> auto lash-out -> NEXT_STEP");
        assert_eq!(game.defender_id.as_deref(), Some("mate1"));
        assert!(game.report_list.has_report(ffb_model::report::report_id::ReportId::ANIMAL_SAVAGERY));
    }

    // ── Multi-target dialog ──────────────────────────────────────────────────

    #[test]
    fn multiple_adjacent_opponents_prompts_player_choice() {
        let seed = seed_for_d6(1);
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Move);
        add_player(&mut game, true, "p1", vec![SkillId::AnimalSavagery]);
        add_player(&mut game, true, "mate1", vec![]);
        add_player(&mut game, true, "mate2", vec![]);
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_coordinate("mate1", FieldCoordinate::new(6, 5));
        game.field_model.set_player_coordinate("mate2", FieldCoordinate::new(4, 5));
        game.field_model.set_player_state("mate1", ffb_model::enums::PlayerState::new(PS_STANDING));
        game.field_model.set_player_state("mate2", ffb_model::enums::PlayerState::new(PS_STANDING));
        let out = StepAnimalSavagery::new("fail").start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::Continue);
        match out.prompt {
            Some(AgentPrompt::PlayerChoice { eligible_players, .. }) => {
                assert_eq!(eligible_players.len(), 2);
            }
            _ => panic!("expected PlayerChoice prompt"),
        }
    }

    #[test]
    fn player_choice_response_lashes_out_at_chosen_target() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        add_player(&mut game, true, "p1", vec![SkillId::AnimalSavagery]);
        add_player(&mut game, false, "opp1", vec![]);
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_coordinate("opp1", FieldCoordinate::new(6, 5));
        let mut step = StepAnimalSavagery::new("fail");
        let out = step.handle_command(
            &Action::PlayerChoice { player_id: Some("opp1".into()), player_ids: vec![], mode: "ANIMAL_SAVAGERY".into() },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.defender_id.as_deref(), Some("opp1"));
    }

    // ── Empty targets: cancel + goto label ───────────────────────────────────

    #[test]
    fn no_adjacent_targets_cancels_action_bb2020() {
        let seed = seed_for_d6(1);
        let mut game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020);
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Move);
        add_player(&mut game, true, "p1", vec![SkillId::AnimalSavagery]);
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("p1", ffb_model::enums::PlayerState::new(PS_STANDING));
        let out = StepAnimalSavagery::new("FAIL_LABEL").start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(game.turn_data_home.blitz_used == false); // Move action: no turn flag expected
        let state = game.field_model.player_state("p1").unwrap();
        assert!(state.is_confused());
    }

    #[test]
    fn no_adjacent_targets_cancels_action_bb2025() {
        let seed = seed_for_d6(1);
        let mut game = make_game(); // BB2025
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Move);
        add_player(&mut game, true, "p1", vec![SkillId::AnimalSavagery]);
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("p1", ffb_model::enums::PlayerState::new(PS_STANDING));
        let out = StepAnimalSavagery::new("FAIL_LABEL").start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::GotoLabel);
        let state = game.field_model.player_state("p1").unwrap();
        assert!(state.is_confused());
    }

    // ── Injury application ───────────────────────────────────────────────────

    #[test]
    fn lash_out_publishes_drop_player_context_bb2020() {
        let mut game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020);
        game.acting_player.player_id = Some("p1".into());
        add_player(&mut game, true, "p1", vec![SkillId::AnimalSavagery]);
        add_player(&mut game, false, "opp1", vec![]);
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_coordinate("opp1", FieldCoordinate::new(6, 5));
        let mut step = StepAnimalSavagery::new("fail");
        let out = step.handle_command(
            &Action::PlayerChoice { player_id: Some("opp1".into()), player_ids: vec![], mode: "ANIMAL_SAVAGERY".into() },
            &mut game,
            &mut GameRng::new(1),
        );
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::DropPlayerContext(_))));
    }

    #[test]
    fn lash_out_publishes_steady_footing_context_bb2025() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        add_player(&mut game, true, "p1", vec![SkillId::AnimalSavagery]);
        add_player(&mut game, false, "opp1", vec![]);
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_coordinate("opp1", FieldCoordinate::new(6, 5));
        let mut step = StepAnimalSavagery::new("fail");
        let out = step.handle_command(
            &Action::PlayerChoice { player_id: Some("opp1".into()), player_ids: vec![], mode: "ANIMAL_SAVAGERY".into() },
            &mut game,
            &mut GameRng::new(1),
        );
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::SteadyFootingContext(_))));
    }

    #[test]
    fn lash_out_sets_defender_and_reports() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        add_player(&mut game, true, "p1", vec![SkillId::AnimalSavagery]);
        add_player(&mut game, false, "opp1", vec![]);
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_coordinate("opp1", FieldCoordinate::new(6, 5));
        let mut step = StepAnimalSavagery::new("fail");
        step.lash_out(&mut game, &mut GameRng::new(1), "opp1".into());
        assert_eq!(game.defender_id.as_deref(), Some("opp1"));
        assert!(game.report_list.has_report(ffb_model::report::report_id::ReportId::ANIMAL_SAVAGERY));
    }

    #[test]
    fn lash_out_publishes_old_defender_as_gaze_victim() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.defender_id = Some("old_def".into());
        add_player(&mut game, true, "p1", vec![SkillId::AnimalSavagery]);
        add_player(&mut game, false, "opp1", vec![]);
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_coordinate("opp1", FieldCoordinate::new(6, 5));
        let mut step = StepAnimalSavagery::new("fail");
        let out = step.lash_out(&mut game, &mut GameRng::new(1), "opp1".into());
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::GazeVictimId(Some(id)) if id == "old_def")));
    }

    // ── Fallback action ──────────────────────────────────────────────────────

    #[test]
    fn fallback_action_blitz_removes_target_selections() {
        let mut game = make_game();
        add_player(&mut game, false, "def1", vec![]);
        game.defender_id = Some("def1".into());
        game.field_model.set_player_state("def1", ffb_model::enums::PlayerState::new(PS_STANDING).add_selected_blitz_target());
        let ir = InjuryResult::new(ApothecaryMode::AnimalSavagery);
        let (action, params) = fallback_action(&mut game, Some(PlayerAction::Blitz), &ir, None, false, true);
        assert!(action.is_none());
        assert!(params.is_empty());
        assert!(!game.field_model.player_state("def1").unwrap().is_selected_blitz_target());
    }

    #[test]
    fn fallback_action_block_same_defender_keeps_action() {
        let mut game = make_game();
        game.defender_id = Some("def1".into());
        let ir = InjuryResult::new(ApothecaryMode::AnimalSavagery);
        let (action, _) = fallback_action(&mut game, Some(PlayerAction::Block), &ir, Some("def1"), false, true);
        assert_eq!(action, Some(PlayerAction::Block));
    }

    #[test]
    fn fallback_action_block_different_defender_clears_action() {
        let mut game = make_game();
        game.defender_id = Some("def1".into());
        let ir = InjuryResult::new(ApothecaryMode::AnimalSavagery);
        let (action, _) = fallback_action(&mut game, Some(PlayerAction::Block), &ir, Some("other"), false, true);
        assert!(action.is_none());
    }

    #[test]
    fn fallback_action_multiple_block_publishes_player_id_to_remove() {
        let mut game = make_game();
        add_player(&mut game, false, "def1", vec![]);
        game.defender_id = Some("def1".into());
        game.field_model.set_player_state("def1", ffb_model::enums::PlayerState::new(PS_STANDING));
        let ir = InjuryResult::new(ApothecaryMode::AnimalSavagery);
        let (action, params) = fallback_action(&mut game, Some(PlayerAction::MultipleBlock), &ir, None, false, true);
        assert!(action.is_none());
        assert!(params.iter().any(|p| matches!(p, StepParameter::PlayerIdToRemove(id) if id == "def1")));
    }

    #[test]
    fn fallback_action_throw_team_mate_sets_ttm_used_bb2025_and_pass_used_bb2020() {
        let mut game_2025 = make_game();
        game_2025.defender_id = Some("def1".into());
        let mut ir = InjuryResult::new(ApothecaryMode::AnimalSavagery);
        ir.injury_context_mut().injury = Some(ffb_model::enums::PlayerState::new(ffb_model::enums::PS_KNOCKED_OUT));
        fallback_action(&mut game_2025, Some(PlayerAction::ThrowTeamMate), &ir, None, true, true);
        assert!(game_2025.turn_data_home.ttm_used);
        assert!(!game_2025.turn_data_home.pass_used);

        let mut game_2020 = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020);
        game_2020.defender_id = Some("def1".into());
        fallback_action(&mut game_2020, Some(PlayerAction::ThrowTeamMate), &ir, None, true, false);
        assert!(game_2020.turn_data_home.pass_used);
        assert!(!game_2020.turn_data_home.ttm_used);
    }

    // ── cancel_player_action_bb2020 ──────────────────────────────────────────

    #[test]
    fn cancel_player_action_bb2020_sets_blitz_used() {
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::Blitz);
        add_player(&mut game, true, "p1", vec![]);
        game.field_model.set_player_state("p1", ffb_model::enums::PlayerState::new(PS_STANDING));
        cancel_player_action_bb2020(&mut game, "p1", false);
        assert!(game.turn_data_home.blitz_used);
    }

    #[test]
    fn cancel_player_action_bb2020_lashed_out_skips_state_change() {
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::Move);
        add_player(&mut game, true, "p1", vec![]);
        game.field_model.set_player_state("p1", ffb_model::enums::PlayerState::new(PS_STANDING));
        cancel_player_action_bb2020(&mut game, "p1", true);
        let state = game.field_model.player_state("p1").unwrap();
        assert!(!state.is_confused(), "lashed_out=true should skip the player-state change");
    }

    #[test]
    fn cancel_player_action_bb2020_not_lashed_out_confuses_standing_player() {
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::Move);
        game.acting_player.standing_up = false;
        add_player(&mut game, true, "p1", vec![]);
        game.field_model.set_player_state("p1", ffb_model::enums::PlayerState::new(PS_STANDING));
        cancel_player_action_bb2020(&mut game, "p1", false);
        let state = game.field_model.player_state("p1").unwrap();
        assert!(state.is_confused());
        assert_eq!(state.base(), PS_STANDING);
    }

    #[test]
    fn cancel_player_action_bb2020_standing_up_goes_prone() {
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::Move);
        game.acting_player.standing_up = true;
        add_player(&mut game, true, "p1", vec![]);
        game.field_model.set_player_state("p1", ffb_model::enums::PlayerState::new(PS_STANDING));
        cancel_player_action_bb2020(&mut game, "p1", false);
        let state = game.field_model.player_state("p1").unwrap();
        assert_eq!(state.base(), PS_PRONE);
    }

    #[test]
    fn cancel_player_action_bb2020_foul_with_allows_additional_does_not_set_flag() {
        // NamedProperties::ALLOWS_ADDITIONAL_FOUL — no roster skill grants it, so this exercises
        // the "false" arm; a true arm would require a skill with that property in test data.
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::Foul);
        add_player(&mut game, true, "p1", vec![]);
        game.field_model.set_player_state("p1", ffb_model::enums::PlayerState::new(PS_STANDING));
        cancel_player_action_bb2020(&mut game, "p1", false);
        assert!(game.turn_data_home.foul_used);
    }

    #[test]
    fn cancel_player_action_bb2020_clears_pass_coordinate() {
        let mut game = make_game();
        game.pass_coordinate = Some(FieldCoordinate::new(1, 1));
        add_player(&mut game, true, "p1", vec![]);
        cancel_player_action_bb2020(&mut game, "p1", false);
        assert!(game.pass_coordinate.is_none());
    }

    // ── adjacent_targets ──────────────────────────────────────────────────────

    #[test]
    fn adjacent_targets_includes_defender_on_team_when_adjacent() {
        let mut game = make_game();
        add_player(&mut game, true, "teammate", vec![]);
        game.defender_id = Some("teammate".into());
        game.field_model.set_player_coordinate("teammate", FieldCoordinate::new(6, 5));
        game.field_model.set_player_state("teammate", ffb_model::enums::PlayerState::new(PS_STANDING));
        let coord = FieldCoordinate::new(5, 5);
        let targets = adjacent_targets(&game, &game.team_home, coord);
        assert!(targets.contains("teammate"));
    }

    #[test]
    fn adjacent_targets_excludes_defender_not_adjacent() {
        let mut game = make_game();
        add_player(&mut game, true, "teammate", vec![]);
        game.defender_id = Some("teammate".into());
        game.field_model.set_player_coordinate("teammate", FieldCoordinate::new(20, 20));
        let coord = FieldCoordinate::new(5, 5);
        let targets = adjacent_targets(&game, &game.team_home, coord);
        assert!(!targets.contains("teammate"));
    }

    // ── good_conditions_for_savagery ─────────────────────────────────────────

    #[test]
    fn good_conditions_true_for_blitz_and_block() {
        assert!(good_conditions_for_savagery(Some(PlayerAction::Blitz)));
        assert!(good_conditions_for_savagery(Some(PlayerAction::BlitzMove)));
        assert!(good_conditions_for_savagery(Some(PlayerAction::Block)));
        assert!(good_conditions_for_savagery(Some(PlayerAction::MultipleBlock)));
        assert!(good_conditions_for_savagery(Some(PlayerAction::StandUpBlitz)));
    }

    #[test]
    fn good_conditions_false_for_move_and_none() {
        assert!(!good_conditions_for_savagery(Some(PlayerAction::Move)));
        assert!(!good_conditions_for_savagery(None));
    }
}
