/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.multiblock.StepFoulAppearanceMultiple`.
///
/// Rolls Foul Appearance for all current block targets (BB2020 + BB2025).
/// Logic inlined from `AbstractStepModifierMultipleBlock` / `FoulAppearanceBehaviour`.
///
/// For each target with `forceRollBeforeBeingBlocked`:
///   - Roll D6; success (>=2) → attacker may block them (removed from blockTargets)
///   - Failure → `PLAYER_ID_TO_REMOVE` published (target removed from block list)
///   - If all targets failed and goToLabelOnFailure is set → GOTO failure label
///
/// Client command `CLIENT_USE_RE_ROLL_FOR_TARGET(FOUL_APPEARANCE)` triggers a re-roll;
/// `CLIENT_PLAYER_CHOICE(LORD_OF_CHAOS)` chooses the single-use re-roll player.
use ffb_mechanics::mechanics::minimum_roll_resisting_foul_appearance;
use ffb_model::enums::ReRollSource;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_cards::UtilCards;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::mixed::multiblock::abstract_step_multiple::{AbstractStepMultiple, SingleReRollUseState};
use crate::step::util_server_re_roll::use_reroll;

/// Java: `StepFoulAppearanceMultiple` (mixed/multiblock, BB2020 + BB2025).
///
/// Logic inlined from `AbstractStepModifierMultipleBlock<StepFoulAppearanceMultiple>` in
/// `FoulAppearanceBehaviour`.
pub struct StepFoulAppearanceMultiple {
    /// Java: `state.goToLabelOnFailure` (mandatory init param GOTO_LABEL_ON_FAILURE)
    pub goto_label_on_failure: String,
    /// Java: `state.blockTargets` — mutable during rolling (successes removed)
    pub block_targets: Vec<String>,
    /// Java: `state.initialCount` — count before rolling (after requiresRoll filter)
    initial_count: usize,
    /// Java: `state.firstRun`
    first_run: bool,
    /// Java: `state.reRollTarget`
    pub re_roll_target: Option<String>,
    /// Java: `state.reRollSource`
    re_roll_source: Option<ReRollSource>,
    /// Java: `state.reRollAvailableAgainst`
    re_roll_available_against: Vec<String>,
    /// Java: base `AbstractStepMultiple` / `SingleReRollUseState`
    base: AbstractStepMultiple,
}

impl StepFoulAppearanceMultiple {
    pub fn new(goto_label_on_failure: impl Into<String>) -> Self {
        Self {
            goto_label_on_failure: goto_label_on_failure.into(),
            block_targets: Vec::new(),
            initial_count: 0,
            first_run: true,
            re_roll_target: None,
            re_roll_source: None,
            re_roll_available_against: Vec::new(),
            base: AbstractStepMultiple::new(),
        }
    }

    fn state(&mut self) -> &mut SingleReRollUseState {
        &mut self.base.state
    }

    /// Java: `FoulAppearanceBehaviour.canBeSkipped` — attacker has a skill cancelling FA.
    fn can_be_skipped(attacker: &Player) -> bool {
        UtilCards::has_skill_to_cancel_property(attacker, NamedProperties::FORCE_ROLL_BEFORE_BEING_BLOCKED)
    }

    /// Java: `FoulAppearanceBehaviour.requiresRoll` — opponent has FA property.
    fn requires_roll(opponent: &Player) -> bool {
        UtilCards::has_skill_with_property(opponent, NamedProperties::FORCE_ROLL_BEFORE_BEING_BLOCKED)
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

        let min_roll = minimum_roll_resisting_foul_appearance();

        if self.first_run {
            // Java: state.firstRun = false; state.initialCount = state.blockTargets.size()
            self.first_run = false;
            self.initial_count = self.block_targets.len();

            // Java: filter blockTargets to only those requiring a roll
            let needs_roll: Vec<String> = self.block_targets.iter()
                .filter(|target_id| {
                    game.player(target_id)
                        .map(|opp| Self::requires_roll(opp))
                        .unwrap_or(false)
                })
                .cloned()
                .collect();

            self.block_targets = needs_roll;

            // Java: if (!state.blockTargets.isEmpty()) { actingPlayer.setHasBlocked(true); }
            if !self.block_targets.is_empty() {
                game.acting_player.has_blocked = true;
            }

            // Java: for (String targetId : ...) { roll(step, actingPlayer, targetId, false, ...) }
            let targets_to_roll: Vec<String> = self.block_targets.clone();
            for target_id in targets_to_roll {
                let roll = rng.d6();
                // Java: isSkillRollSuccessful(roll, minimumRoll) = roll==6 || (roll!=1 && roll>=min)
                let successful = DiceInterpreter::is_skill_roll_successful(roll, min_roll);
                if successful {
                    // Java: state.blockTargets.remove(currentTargetId); successFulRollCallback (no-op for FA)
                    self.block_targets.retain(|t| t != &target_id);
                }
                // client-only: SoundId.EW sound effect — no audio in headless engine
            }

            // Java: state.reRollAvailableAgainst.addAll(state.blockTargets)
            self.re_roll_available_against = self.block_targets.clone();

            // Headless: skip dialog, go straight to nextStep logic
            self.handle_remaining(game)
        } else {
            // Java: re-roll path — called after UseReRollForTarget command
            if let (Some(target), Some(source)) = (self.re_roll_target.clone(), self.re_roll_source.clone()) {
                if use_reroll(game, &source, &attacker_id) {
                    let roll = rng.d6();
                    let successful = DiceInterpreter::is_skill_roll_successful(roll, min_roll);
                    if successful {
                        self.block_targets.retain(|t| t != &target);
                    }
                }
                self.re_roll_available_against.retain(|t| t != &target);
            }
            self.handle_remaining(game)
        }
    }

    /// Java: `nextStep()` — resolves outcome after all rolls are done.
    ///
    /// If goToLabelOnFailure set AND all targets still remain (all failed) → GOTO failure.
    /// Otherwise: `unhandledTargetsCallback` (publish PLAYER_ID_TO_REMOVE) + NEXT_STEP.
    fn handle_remaining(&mut self, game: &mut Game) -> StepOutcome {
        if self.block_targets.is_empty() {
            return StepOutcome::next();
        }

        if !self.goto_label_on_failure.is_empty() && self.block_targets.len() == self.initial_count {
            // All required-roll targets failed — attacker is fully blocked
            StepOutcome::goto(&self.goto_label_on_failure.clone())
        } else {
            // Java: unhandledTargetsCallback — deselect remaining targets, publish PLAYER_ID_TO_REMOVE
            let mut outcome = StepOutcome::next();
            let remaining: Vec<String> = self.block_targets.drain(..).collect();
            for target_id in remaining {
                // Deselect block/stab target state bits in the field model
                if let Some(state) = game.field_model.player_state(&target_id) {
                    let new_state = state
                        .change_selected_stab_target(false)
                        .change_selected_block_target(false);
                    game.field_model.set_player_state(&target_id, new_state);
                }
                outcome = outcome.publish(StepParameter::PlayerIdToRemove(target_id));
            }
            outcome
        }
    }
}

impl Default for StepFoulAppearanceMultiple {
    fn default() -> Self { Self::new("") }
}

impl Step for StepFoulAppearanceMultiple {
    fn id(&self) -> StepId { StepId::FoulAppearanceMultiple }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::UseReRollForTarget { re_rolled_action, re_roll_source, target_id }
                if re_rolled_action.as_deref() == Some("FOUL_APPEARANCE") =>
            {
                self.re_roll_target = target_id.clone();
                // Java: AbstractStepMultiple.reRollSourceSuccessfully(command.getReRollSource())
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
            StepParameter::GotoLabelOnFailure(v) => {
                self.goto_label_on_failure = v.clone();
                true
            }
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

    fn add_player(game: &mut Game, home: bool, id: &str, coord: FieldCoordinate, skills: Vec<SkillId>) {
        let player = Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
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
    fn id_is_foul_appearance_multiple() {
        assert_eq!(StepFoulAppearanceMultiple::new("fail").id(), StepId::FoulAppearanceMultiple);
    }

    #[test]
    fn no_attacker_returns_next_step() {
        let mut step = StepFoulAppearanceMultiple::new("fail");
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn goto_label_set_from_parameter() {
        let mut step = StepFoulAppearanceMultiple::default();
        step.set_parameter(&StepParameter::GotoLabelOnFailure("skip_block".into()));
        assert_eq!(step.goto_label_on_failure, "skip_block");
    }

    #[test]
    fn block_targets_added_via_parameter() {
        let mut step = StepFoulAppearanceMultiple::default();
        step.set_parameter(&StepParameter::BlockTargets(vec!["t1".into(), "t2".into()]));
        assert_eq!(step.block_targets, vec!["t1", "t2"]);
    }

    #[test]
    fn player_id_to_remove_shrinks_targets() {
        let mut step = StepFoulAppearanceMultiple::new("fail");
        step.set_parameter(&StepParameter::BlockTargets(vec!["p1".into(), "p2".into()]));
        step.set_parameter(&StepParameter::PlayerIdToRemove("p1".into()));
        assert_eq!(step.block_targets, vec!["p2"]);
    }

    #[test]
    fn no_fa_targets_returns_next_step() {
        // Attacker vs defender with no FA — requiresRoll false for all → NEXT_STEP
        let mut game = make_game();
        game.home_playing = true;
        add_player(&mut game, true, "attacker", FieldCoordinate::new(5, 5), vec![]);
        add_player(&mut game, false, "defender", FieldCoordinate::new(5, 6), vec![]);
        game.acting_player.player_id = Some("attacker".into());

        let mut step = StepFoulAppearanceMultiple::new("fail");
        step.set_parameter(&StepParameter::BlockTargets(vec!["defender".into()]));

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn fa_target_success_removes_from_block_targets() {
        // Defender has FoulAppearance. Roll of 6 → success → removed from blockTargets.
        let mut game = make_game();
        game.home_playing = true;
        add_player(&mut game, true, "attacker", FieldCoordinate::new(5, 5), vec![]);
        add_player(&mut game, false, "defender", FieldCoordinate::new(5, 6), vec![SkillId::FoulAppearance]);
        game.acting_player.player_id = Some("attacker".into());

        let mut step = StepFoulAppearanceMultiple::new("fail");
        step.set_parameter(&StepParameter::BlockTargets(vec!["defender".into()]));

        // Seed that produces a high roll (success >=2)
        let out = step.start(&mut game, &mut GameRng::new(1));
        // Success: block_targets now empty → NEXT_STEP
        assert_eq!(out.action, StepAction::NextStep);
        assert!(step.block_targets.is_empty());
    }

    #[test]
    fn all_fa_targets_fail_with_goto_label_returns_goto() {
        // 1 defender with FA, 1 total initial target. Roll of 1 → fail → all failed → GOTO.
        // Use seed 0 which consistently produces 1 on first roll.
        let mut game = make_game();
        game.home_playing = true;
        add_player(&mut game, true, "attacker", FieldCoordinate::new(5, 5), vec![]);
        add_player(&mut game, false, "def1", FieldCoordinate::new(5, 6), vec![SkillId::FoulAppearance]);
        game.acting_player.player_id = Some("attacker".into());

        // Pre-set initial_count to match the single FA defender
        let mut step = StepFoulAppearanceMultiple::new("goto_fail");
        step.block_targets = vec!["def1".into()];
        step.first_run = false; // skip first-run filtering
        step.initial_count = 1;
        step.re_roll_available_against = vec!["def1".into()];
        // Simulate: still 1 target remaining (failed roll)
        // handle_remaining directly
        let out = step.handle_remaining(&mut game);
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn unhandled_targets_publishes_player_id_to_remove() {
        // 2 initial targets, 1 has FA and fails, 1 doesn't have FA
        // After filtering: 1 in block_targets. After fail: 1 remains.
        // initial_count was 2 (before filtering), block_targets.len() = 1 → not equal → unhandled callback
        let mut game = make_game();
        game.home_playing = true;
        add_player(&mut game, true, "attacker", FieldCoordinate::new(5, 5), vec![]);
        add_player(&mut game, false, "def_no_fa", FieldCoordinate::new(5, 6), vec![]);
        add_player(&mut game, false, "def_fa", FieldCoordinate::new(6, 6), vec![SkillId::FoulAppearance]);
        game.acting_player.player_id = Some("attacker".into());

        let mut step = StepFoulAppearanceMultiple::new("goto_fail");
        // initial_count = 2 (def_no_fa + def_fa), after filtering block_targets = [def_fa]
        // After FA roll fails: block_targets = [def_fa], initial_count = 2
        // block_targets.len() (1) != initial_count (2) → unhandledTargetsCallback
        step.block_targets = vec!["def_fa".into()];
        step.first_run = false;
        step.initial_count = 2; // 2 total originally
        let out = step.handle_remaining(&mut game);
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::PlayerIdToRemove(id) if id == "def_fa")));
    }

    #[test]
    fn attacker_with_fa_cancel_returns_next_step() {
        // Attacker has a skill cancelling forceRollBeforeBeingBlocked → canBeSkipped → NEXT_STEP
        let mut game = make_game();
        game.home_playing = true;
        // Juggernaut cancels forceRollBeforeBeingBlocked (via cancelsForceRollBeforeBeingBlocked)
        add_player(&mut game, true, "attacker", FieldCoordinate::new(5, 5), vec![SkillId::Juggernaut]);
        add_player(&mut game, false, "def_fa", FieldCoordinate::new(5, 6), vec![SkillId::FoulAppearance]);
        game.acting_player.player_id = Some("attacker".into());

        let mut step = StepFoulAppearanceMultiple::new("fail");
        step.set_parameter(&StepParameter::BlockTargets(vec!["def_fa".into()]));

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn empty_block_targets_returns_next_step() {
        let mut game = make_game();
        game.home_playing = true;
        add_player(&mut game, true, "attacker", FieldCoordinate::new(5, 5), vec![]);
        game.acting_player.player_id = Some("attacker".into());

        let mut step = StepFoulAppearanceMultiple::new("fail");
        // No block targets set
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn handle_command_unrecognised_returns_next() {
        let mut step = StepFoulAppearanceMultiple::new("fail");
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }
}
