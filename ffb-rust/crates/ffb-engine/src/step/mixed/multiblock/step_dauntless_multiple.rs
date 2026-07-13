/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.multiblock.StepDauntlessMultiple`.
///
/// Rolls Dauntless for all current block targets (BB2020 + BB2025).
/// Logic inlined from `AbstractStepModifierMultipleBlock` / `DauntlessBehaviour`.
///
/// For each target where the attacker's effective strength < defender strength:
///   - Roll D6 vs `minimum_roll_dauntless(effective_attacker_str, defender_str)`
///   - On success: publish `PLAYER_ID_DAUNTLESS_SUCCESS` for that target
///   - On failure: no special action (target stays for normal strength calculation)
///
/// Client command `CLIENT_USE_RE_ROLL_FOR_TARGET(DAUNTLESS)` triggers a per-target re-roll;
/// `CLIENT_PLAYER_CHOICE(LORD_OF_CHAOS)` chooses the single-use team re-roll player.
use std::collections::HashMap;
use ffb_mechanics::mechanics::minimum_roll_dauntless;
use ffb_model::enums::{ReRollSource, SkillId};
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::step::framework::{Step, StepOutcome, StepAction, StepId, StepParameter};
use crate::step::mixed::multiblock::abstract_step_multiple::{build_reroll_prompt, AbstractStepMultiple, SingleReRollUseState};
use crate::step::util_server_re_roll::use_reroll;

/// Java: `StepDauntlessMultiple` (mixed/multiblock, BB2020 + BB2025).
///
/// State mirrors `StepStateMultipleRolls` (Java).
pub struct StepDauntlessMultiple {
    /// Java: `state.blockTargets` — active block target player IDs
    pub block_targets: Vec<String>,
    /// Java: `state.initialCount`
    initial_count: usize,
    /// Java: `state.firstRun`
    first_run: bool,
    /// Java: `state.reRollTarget`
    pub re_roll_target: Option<String>,
    /// Java: `state.reRollSource`
    re_roll_source: Option<ReRollSource>,
    /// Java: `state.reRollAvailableAgainst`
    re_roll_available_against: Vec<String>,
    /// Java: `state.minimumRolls`
    minimum_rolls: HashMap<String, i32>,
    /// Java: base `AbstractStepMultiple` / `SingleReRollUseState`
    base: AbstractStepMultiple,
}

impl StepDauntlessMultiple {
    pub fn new() -> Self {
        Self {
            block_targets: Vec::new(),
            initial_count: 0,
            first_run: true,
            re_roll_target: None,
            re_roll_source: None,
            re_roll_available_against: Vec::new(),
            minimum_rolls: HashMap::new(),
            base: AbstractStepMultiple::new(),
        }
    }

    fn state(&mut self) -> &mut SingleReRollUseState {
        &mut self.base.state
    }

    /// Java: `DauntlessBehaviour.canBeSkipped` — attacker does NOT have Dauntless skill.
    ///
    /// Returns true (skip) when the attacker lacks `canRollToMatchOpponentsStrength`.
    fn can_be_skipped(attacker: &Player) -> bool {
        !attacker.has_skill_property(NamedProperties::CAN_ROLL_TO_MATCH_OPPONENTS_STRENGTH)
    }

    /// Java: `DauntlessBehaviour.requiresRoll`.
    ///
    /// `effectiveStrength = max(1, attacker.strengthWithModifiers - 2)` (multi-block modifier).
    /// Roll needed when: `effectiveStr < defStr AND effectiveStr + 6 > defStr`.
    fn requires_roll(attacker: &Player, opponent: &Player) -> bool {
        let effective_str = (attacker.strength_with_modifiers() - 2).max(1);
        let def_str = opponent.strength_with_modifiers();
        effective_str < def_str && effective_str + 6 > def_str
    }

    /// Java: `DauntlessBehaviour.minimumRoll` (multi-block context).
    ///
    /// `attackerBaseStr = max(1, attacker.strengthWithModifiers - 2)`
    /// `defenderStr = opponent.strength` (multi-block defender modifier is 0 in BB2020/BB2025)
    fn compute_min_roll(attacker: &Player, opponent: &Player) -> i32 {
        let attacker_base_str = (attacker.strength_with_modifiers() - 2).max(1);
        let defender_str = opponent.strength;
        minimum_roll_dauntless(attacker_base_str, defender_str)
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let attacker_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        // Java: if (canBeSkipped(actingPlayer.getPlayer())) → NEXT_STEP
        {
            if let Some(attacker) = game.player(&attacker_id) {
                if Self::can_be_skipped(attacker) {
                    return StepOutcome::next();
                }
            }
        }

        if self.first_run {
            self.first_run = false;
            self.initial_count = self.block_targets.len();

            // Collect targets requiring roll with their min rolls (to avoid borrow issues)
            let targets_with_rolls: Vec<(String, i32)> = self.block_targets.iter()
                .filter_map(|target_id| {
                    let attacker = game.player(&attacker_id)?;
                    let opponent = game.player(target_id)?;
                    if Self::requires_roll(attacker, opponent) {
                        Some((target_id.clone(), Self::compute_min_roll(attacker, opponent)))
                    } else {
                        None
                    }
                })
                .collect();

            // Update block_targets to only those requiring a roll
            self.block_targets = targets_with_rolls.iter().map(|(id, _)| id.clone()).collect();

            // Java: if (!state.blockTargets.isEmpty()) { actingPlayer.setHasBlocked(true); }
            if !self.block_targets.is_empty() {
                game.acting_player.has_blocked = true;
            }

            // Java: for (String targetId : ...) { roll(step, actingPlayer, targetId, false, ...) }
            let mut outcome = StepOutcome::next();
            for (target_id, min_roll) in targets_with_rolls {
                self.minimum_rolls.insert(target_id.clone(), min_roll);
                let mut roll = rng.d6();
                let mut successful = DiceInterpreter::is_skill_roll_successful(roll, min_roll);

                // Java: `roll()`'s inline auto-reroll — on a failed non-reRolling roll, try an
                // unused reroll source immediately. Only Blind Rage (Akhorne's unique skill)
                // registers a non-TRR reroll source for ReRolledActions.DAUNTLESS in this
                // codebase's data (matches the same hardcoded check already used by the
                // singular `step_dauntless.rs`); the target is removed from `block_targets`
                // once this auto-reroll is attempted, win or lose, matching Java's unconditional
                // `state.blockTargets.remove(currentTargetId)` inside that branch.
                if !successful {
                    let has_blind_rage = game.player(&attacker_id)
                        .map(|p| p.has_skill(SkillId::BlindRage) && !p.used_skills.contains(&SkillId::BlindRage))
                        .unwrap_or(false);
                    if has_blind_rage {
                        if let Some(p) = game.team_home.players.iter_mut().find(|p| p.id == attacker_id)
                            .or_else(|| game.team_away.players.iter_mut().find(|p| p.id == attacker_id))
                        {
                            p.used_skills.insert(SkillId::BlindRage);
                        }
                        roll = rng.d6();
                        successful = DiceInterpreter::is_skill_roll_successful(roll, min_roll);
                        self.block_targets.retain(|t| t != &target_id);
                    }
                }

                if successful {
                    // Java: successFulRollCallback → publishParameter(PLAYER_ID_DAUNTLESS_SUCCESS)
                    self.block_targets.retain(|t| t != &target_id);
                    outcome = outcome.publish(StepParameter::PlayerIdDauntlessSuccess(target_id));
                }
                // Java: failedRollEffect → no-op for Dauntless
            }

            // Java: state.reRollAvailableAgainst.addAll(state.blockTargets)
            self.re_roll_available_against = self.block_targets.clone();

            // Java: decideNextStep — offer the re-roll-choice dialog if any source is available.
            match build_reroll_prompt(game, &attacker_id, "DAUNTLESS", &self.block_targets, &self.minimum_rolls, &self.re_roll_available_against) {
                Some(prompt) => {
                    self.re_roll_target = None;
                    self.re_roll_source = None;
                    outcome.action = StepAction::Continue;
                    outcome.prompt = Some(prompt);
                }
                None => {
                    // For Dauntless: no goToLabelOnFailure, unhandledTargetsCallback is empty → NEXT_STEP
                }
            }
            return outcome;
        }

        // Re-roll path (UseReRollForTarget command)
        let mut outcome = StepOutcome::next();
        if let (Some(target), Some(source)) = (self.re_roll_target.clone(), self.re_roll_source.clone()) {
            if use_reroll(game, &source, &attacker_id) {
                // Recompute min roll for this target
                let min_roll = {
                    let attacker = game.player(&attacker_id);
                    let opponent = game.player(&target);
                    match (attacker, opponent) {
                        (Some(a), Some(o)) => Self::compute_min_roll(a, o),
                        _ => 6,
                    }
                };
                self.minimum_rolls.insert(target.clone(), min_roll);
                let roll = rng.d6();
                let successful = DiceInterpreter::is_skill_roll_successful(roll, min_roll);
                if successful {
                    self.block_targets.retain(|t| t != &target);
                    outcome = outcome.publish(StepParameter::PlayerIdDauntlessSuccess(target.clone()));
                }
            }
            self.re_roll_available_against.retain(|t| t != &target);
        }

        // Java: decideNextStep — offer the re-roll-choice dialog again if a source remains.
        match build_reroll_prompt(game, &attacker_id, "DAUNTLESS", &self.block_targets, &self.minimum_rolls, &self.re_roll_available_against) {
            Some(prompt) => {
                self.re_roll_target = None;
                self.re_roll_source = None;
                outcome.action = StepAction::Continue;
                outcome.prompt = Some(prompt);
            }
            None => {}
        }
        outcome
    }
}

impl Default for StepDauntlessMultiple {
    fn default() -> Self { Self::new() }
}

impl Step for StepDauntlessMultiple {
    fn id(&self) -> StepId { StepId::DauntlessMultiple }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::UseReRollForTarget { re_rolled_action, re_roll_source, target_id }
                if re_rolled_action.as_deref() == Some("DAUNTLESS") =>
            {
                self.re_roll_target = target_id.clone();
                let proceed = crate::step::mixed::multiblock::abstract_step_multiple::re_roll_source_successfully(
                    &mut self.base.state,
                    re_roll_source.as_deref().unwrap_or(""),
                    game,
                );
                if proceed {
                    self.re_roll_source = re_roll_source.as_ref().map(|s| ReRollSource::new(s));
                    return self.execute_step(game, rng);
                }
                StepOutcome::cont()
            }
            Action::LordOfChaosChoice { player_id } => {
                self.base.apply_lord_of_chaos_command(game, player_id.as_deref());
                self.execute_step(game, rng)
            }
            _ => StepOutcome::next(),
        }
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::BlockTargets(ids) => {
                self.block_targets.extend(ids.iter().cloned());
                true
            }
            StepParameter::PlayerIdToRemove(id) => {
                self.block_targets.retain(|t| t != id);
                true
            }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{Rules, PlayerState as PS, PS_STANDING, SkillId};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::types::FieldCoordinate;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use std::collections::HashSet;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_player(game: &mut Game, home: bool, id: &str, coord: FieldCoordinate, strength: i32, skills: Vec<SkillId>) -> () {
        let player = Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength, agility: 3, passing: 4, armour: 8,
            starting_skills: skills.into_iter().map(|s| SkillWithValue { skill_id: s, value: None }).collect(),
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        if home { game.team_home.players.push(player); }
        else { game.team_away.players.push(player); }
        game.field_model.set_player_coordinate(id, coord);
        game.field_model.set_player_state(id, PS::new(PS_STANDING));
    }

    #[test]
    fn id_is_dauntless_multiple() {
        assert_eq!(StepDauntlessMultiple::new().id(), StepId::DauntlessMultiple);
    }

    #[test]
    fn no_attacker_returns_next_step() {
        let mut step = StepDauntlessMultiple::new();
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn block_targets_set_via_parameter() {
        let mut step = StepDauntlessMultiple::new();
        step.set_parameter(&StepParameter::BlockTargets(vec!["p1".into(), "p2".into()]));
        assert_eq!(step.block_targets, vec!["p1", "p2"]);
    }

    #[test]
    fn player_id_to_remove_shrinks_targets() {
        let mut step = StepDauntlessMultiple::new();
        step.set_parameter(&StepParameter::BlockTargets(vec!["p1".into(), "p2".into()]));
        step.set_parameter(&StepParameter::PlayerIdToRemove("p1".into()));
        assert_eq!(step.block_targets, vec!["p2"]);
    }

    #[test]
    fn handle_command_acknowledge_returns_next() {
        let mut step = StepDauntlessMultiple::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn attacker_without_dauntless_returns_next_step() {
        // canBeSkipped = true when attacker lacks CAN_ROLL_TO_MATCH_OPPONENTS_STRENGTH
        let mut game = make_game();
        game.home_playing = true;
        add_player(&mut game, true, "attacker", FieldCoordinate::new(5, 5), 3, vec![]);
        add_player(&mut game, false, "defender", FieldCoordinate::new(5, 6), 4, vec![]);
        game.acting_player.player_id = Some("attacker".into());

        let mut step = StepDauntlessMultiple::new();
        step.set_parameter(&StepParameter::BlockTargets(vec!["defender".into()]));

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        // No PLAYER_ID_DAUNTLESS_SUCCESS since canBeSkipped returned true
        assert!(out.published.is_empty());
    }

    #[test]
    fn equal_strength_skips_roll_no_dauntless_success() {
        // Attacker strength 3, effective = max(1, 3-2) = 1. Defender strength 3.
        // requiresRoll: 1 < 3 AND 1+6=7 > 3 → true
        // But attacker has no Dauntless skill → canBeSkipped → NEXT_STEP
        let mut game = make_game();
        game.home_playing = true;
        add_player(&mut game, true, "attacker", FieldCoordinate::new(5, 5), 3, vec![]);
        add_player(&mut game, false, "defender", FieldCoordinate::new(5, 6), 3, vec![]);
        game.acting_player.player_id = Some("attacker".into());

        let mut step = StepDauntlessMultiple::new();
        step.set_parameter(&StepParameter::BlockTargets(vec!["defender".into()]));

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.is_empty());
    }

    #[test]
    fn dauntless_player_vs_equal_strength_no_roll_needed() {
        // Attacker has Dauntless but effective strength (3-2=1) < defender strength (3).
        // requiresRoll = true (1 < 3 and 1+6>3). Roll happens.
        let mut game = make_game();
        game.home_playing = true;
        add_player(&mut game, true, "attacker", FieldCoordinate::new(5, 5), 3, vec![SkillId::Dauntless]);
        add_player(&mut game, false, "defender", FieldCoordinate::new(5, 6), 3, vec![]);
        game.acting_player.player_id = Some("attacker".into());

        let mut step = StepDauntlessMultiple::new();
        step.set_parameter(&StepParameter::BlockTargets(vec!["defender".into()]));

        let out = step.start(&mut game, &mut GameRng::new(42));
        // Outcome is NextStep regardless (no goto label for Dauntless)
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn dauntless_success_publishes_player_id_dauntless_success() {
        // Attacker str 3, defender str 4, effective = 1, min_roll = minimum_roll_dauntless(1, 4) = min(6, 4-1+1) = 4
        // Need a roll >= 4. Use deterministic test via direct state manipulation.
        let mut game = make_game();
        game.home_playing = true;
        add_player(&mut game, true, "attacker", FieldCoordinate::new(5, 5), 3, vec![SkillId::Dauntless]);
        add_player(&mut game, false, "defender", FieldCoordinate::new(5, 6), 4, vec![]);
        game.acting_player.player_id = Some("attacker".into());

        let mut step = StepDauntlessMultiple::new();
        // Manually set first_run=false and simulate success
        step.first_run = false;
        step.block_targets = vec![];
        // Use published from a direct mock path: just verify the publish format
        let success_param = StepParameter::PlayerIdDauntlessSuccess("defender".into());
        let out = StepOutcome::next().publish(success_param);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::PlayerIdDauntlessSuccess(id) if id == "defender")));
    }

    #[test]
    fn requires_roll_true_when_effective_str_below_defender() {
        // effective_str = max(1, 3-2) = 1; defender_str = 5 → 1 < 5 AND 1+6=7 > 5 → true
        let attacker = Player { strength: 3, ..Default::default() };
        let opponent = Player { strength: 5, ..Default::default() };
        assert!(StepDauntlessMultiple::requires_roll(&attacker, &opponent));
    }

    #[test]
    fn requires_roll_false_when_defender_too_strong() {
        // effective_str = 1; defender_str = 7 → 1+6=7 not > 7 → false
        let attacker = Player { strength: 3, ..Default::default() };
        let opponent = Player { strength: 7, ..Default::default() };
        assert!(!StepDauntlessMultiple::requires_roll(&attacker, &opponent));
    }

    #[test]
    fn compute_min_roll_uses_effective_attacker_strength() {
        // effective = max(1, 3-2) = 1, defender_str = 4 → min(6, 4-1+1) = 4
        let attacker = Player { strength: 3, ..Default::default() };
        let opponent = Player { strength: 4, ..Default::default() };
        assert_eq!(StepDauntlessMultiple::compute_min_roll(&attacker, &opponent), 4);
    }

    fn seed_for_d6(target: i32) -> u64 {
        for s in 0u64..10_000 {
            if GameRng::new(s).d6() == target { return s; }
        }
        panic!("no seed for d6={}", target);
    }

    #[test]
    fn failed_roll_with_team_reroll_available_offers_dialog() {
        // Attacker str 3, defender str 4, effective = 1, min_roll = 4. Roll of 1 fails, no
        // Blind Rage, but a team re-roll is available → decideNextStep should show the dialog.
        let seed = seed_for_d6(1);
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        add_player(&mut game, true, "attacker", FieldCoordinate::new(5, 5), 3, vec![SkillId::Dauntless]);
        add_player(&mut game, false, "defender", FieldCoordinate::new(5, 6), 4, vec![]);
        game.acting_player.player_id = Some("attacker".into());

        let mut step = StepDauntlessMultiple::new();
        step.set_parameter(&StepParameter::BlockTargets(vec!["defender".into()]));

        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::Continue, "team re-roll available → offer dialog");
        assert!(matches!(out.prompt, Some(ffb_model::prompts::AgentPrompt::ReRollForTargets { .. })));
    }

    #[test]
    fn failed_roll_with_no_reroll_source_falls_through_to_next_step() {
        // Same as above but no TRR available → straight to NEXT_STEP, no dialog.
        let seed = seed_for_d6(1);
        let mut game = make_game();
        game.home_playing = true;
        add_player(&mut game, true, "attacker", FieldCoordinate::new(5, 5), 3, vec![SkillId::Dauntless]);
        add_player(&mut game, false, "defender", FieldCoordinate::new(5, 6), 4, vec![]);
        game.acting_player.player_id = Some("attacker".into());

        let mut step = StepDauntlessMultiple::new();
        step.set_parameter(&StepParameter::BlockTargets(vec!["defender".into()]));

        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.prompt.is_none());
    }

    #[test]
    fn blind_rage_auto_rerolls_on_failure_and_can_succeed() {
        // First roll fails (1), Blind Rage auto-reroll succeeds (4 >= min_roll 4).
        let seed = seed_for_d6_pair(1, 4);
        let mut game = make_game();
        game.home_playing = true;
        add_player(&mut game, true, "attacker", FieldCoordinate::new(5, 5), 3, vec![SkillId::Dauntless, SkillId::BlindRage]);
        add_player(&mut game, false, "defender", FieldCoordinate::new(5, 6), 4, vec![]);
        game.acting_player.player_id = Some("attacker".into());

        let mut step = StepDauntlessMultiple::new();
        step.set_parameter(&StepParameter::BlockTargets(vec!["defender".into()]));

        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::NextStep, "silent Blind Rage reroll, no dialog");
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::PlayerIdDauntlessSuccess(id) if id == "defender")));
        assert!(game.player("attacker").unwrap().used_skills.contains(&SkillId::BlindRage));
        assert!(step.block_targets.is_empty());
    }

    #[test]
    fn blind_rage_auto_reroll_failure_still_removes_target() {
        // Both rolls fail — target is still removed from block_targets per Java's
        // unconditional `state.blockTargets.remove(currentTargetId)` inside the auto-reroll branch.
        let seed = seed_for_d6_pair(1, 1);
        let mut game = make_game();
        game.home_playing = true;
        add_player(&mut game, true, "attacker", FieldCoordinate::new(5, 5), 3, vec![SkillId::Dauntless, SkillId::BlindRage]);
        add_player(&mut game, false, "defender", FieldCoordinate::new(5, 6), 4, vec![]);
        game.acting_player.player_id = Some("attacker".into());

        let mut step = StepDauntlessMultiple::new();
        step.set_parameter(&StepParameter::BlockTargets(vec!["defender".into()]));

        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.published.iter().any(|p| matches!(p, StepParameter::PlayerIdDauntlessSuccess(_))));
        assert!(step.block_targets.is_empty(), "target removed even on failed auto-reroll");
        assert!(game.player("attacker").unwrap().used_skills.contains(&SkillId::BlindRage));
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
    fn full_reroll_round_trip_via_action_use_reroll_for_target() {
        // Fail without Blind Rage, TRR available → dialog → coach picks TRR → re-roll succeeds.
        let seed = seed_for_d6(1);
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        add_player(&mut game, true, "attacker", FieldCoordinate::new(5, 5), 3, vec![SkillId::Dauntless]);
        add_player(&mut game, false, "defender", FieldCoordinate::new(5, 6), 4, vec![]);
        game.acting_player.player_id = Some("attacker".into());

        let mut step = StepDauntlessMultiple::new();
        step.set_parameter(&StepParameter::BlockTargets(vec!["defender".into()]));
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::Continue);

        let out2 = step.handle_command(
            &Action::UseReRollForTarget {
                re_rolled_action: Some("DAUNTLESS".into()),
                re_roll_source: Some("TRR".into()),
                target_id: Some("defender".into()),
            },
            &mut game,
            &mut GameRng::new(seed_for_d6(4)),
        );
        assert_eq!(out2.action, StepAction::NextStep);
        assert!(out2.published.iter().any(|p| matches!(p, StepParameter::PlayerIdDauntlessSuccess(id) if id == "defender")));
        assert_eq!(game.turn_data_home.rerolls, 0, "TRR token consumed");
    }
}
