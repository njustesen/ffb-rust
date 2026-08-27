/// 1:1 translation of com.fumbbl.ffb.server.skillbehaviour.bb2025.BoneHeadBehaviour.
use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::model::step_modifier::StepModifierTrait;
use crate::step::framework::StepId;
use crate::step::action::common::step_bone_head::StepBoneHeadHookState;
use crate::skill_behaviour::registry::SkillRegistry;
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use crate::step::action::common::cancel_negatrait_player_action;
use ffb_model::enums::{ReRollSource, SkillId};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_mechanics::mechanics::minimum_roll_confusion;
use ffb_model::report::report_confusion_roll::ReportConfusionRoll;
use crate::step::framework::{StepOutcome, StepParameter};

pub struct BoneHeadBehaviour;

impl BoneHeadBehaviour {
    pub fn new() -> Self { Self }

    pub fn register_into(registry: &mut SkillRegistry) {
        let mut sb = SbContainer::new();
        sb.register_step_modifier(Box::new(BoneHeadStepModifier));
        registry.register(SkillId::BoneHead, sb);
    }
}

impl Default for BoneHeadBehaviour {
    fn default() -> Self { Self::new() }
}

// ── BoneHeadStepModifier ──────────────────────────────────────────────────────

pub struct BoneHeadStepModifier;

impl StepModifierTrait for BoneHeadStepModifier {
    fn applies_to(&self, step_id: StepId) -> bool { step_id == StepId::BoneHead }

    fn priority(&self) -> i32 { 0 }

    /// Java: BoneHeadBehaviour.handleExecuteStepHook(StepBoneHead step, StepState state)
    fn handle_execute_step(
        &self,
        game: &mut Game,
        rng: &mut GameRng,
        step_state: &mut dyn std::any::Any,
    ) -> bool {
        let state = step_state
            .downcast_mut::<StepBoneHeadHookState>()
            .expect("BoneHeadStepModifier: step_state must be StepBoneHeadHookState");

        // Java: if (!game.getTurnMode().checkNegatraits()) { setNextAction(NEXT_STEP); return false; }
        if !game.turn_mode.check_negatraits() {
            state.outcome = Some(StepOutcome::next());
            return false;
        }

        let player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => { state.outcome = Some(StepOutcome::next()); return false; }
        };

        let has_bone_head = game.player(&player_id)
            .map(|p| p.has_skill(SkillId::BoneHead))
            .unwrap_or(false);
        if !has_bone_head {
            state.outcome = Some(StepOutcome::next());
            return false;
        }

        // Java:
        //   boolean doRoll = true;
        //   if ((reRolledAction != null) && (reRolledAction == step.getReRolledAction())) {
        //       if (reRollSource == null || !useReRoll(...)) { doRoll = false; status = FAILURE; cancelPlayerAction(step); }
        //   } else {
        //       doRoll = UtilCards.hasUnusedSkill(actingPlayer, skill);
        //   }
        let mut do_roll = true;
        let mut cancel_as_failure = false;
        if state.re_rolled_action.as_deref() == Some("BONE_HEAD") {
            if let Some(ref source_name) = state.re_roll_source.clone() {
                let source = ReRollSource::new(source_name.as_str());
                if !use_reroll(game, &source, &player_id, rng) {
                    do_roll = false;
                    cancel_as_failure = true;
                }
            } else {
                do_roll = false;
                cancel_as_failure = true; // player declined
            }
        } else {
            // Java: doRoll = UtilCards.hasUnusedSkill(actingPlayer, skill) — guards against
            // re-firing the roll when StepBoneHead is re-entered later in the same player
            // action (e.g. Blitz's move phase and block phase both push StepId::BoneHead).
            // Java tracks Bone Head use on the ACTING PLAYER (`fUsedSkills`, cleared each
            // activation by setPlayerId), NOT the persistent Player — so a player who rolled
            // Bone Head on one turn's activation must roll fresh on its next activation. Using
            // Player.used_skills here left the skill marked forever, skipping every later roll
            // (ogre seed 1 i=21: away_06 rolled Bone Head in turn 1 and then silently skipped it
            // in turn 2, shifting every subsequent die).
            let already_used = game.acting_player.used_skills.contains(&SkillId::BoneHead);
            do_roll = !already_used;
        }

        if !do_roll {
            if cancel_as_failure {
                let confusion_event = GameEvent::ConfusionRoll { player_id: player_id.clone(), roll: 1, confused: true };
                cancel_negatrait_player_action(game, &player_id);
                crate::step::action::common::mark_target_selection_failed(game);
                state.outcome = Some(
                    StepOutcome::goto(&state.goto_label_on_failure)
                        .with_event(confusion_event)
                        .publish(StepParameter::EndPlayerAction(true))
                );
            } else {
                // Java: doRoll == false via hasUnusedSkill → status stays SUCCESS → NEXT_STEP,
                // no roll, no report, no cancellation.
                state.outcome = Some(StepOutcome::next());
            }
            return false;
        }

        // Java: step.commitTargetSelection()
        if let Some(ref mut ts) = game.field_model.target_selection_state {
            ts.commit();
        }
        let roll = rng.d6();
        let min_roll = minimum_roll_confusion(true);
        // Java `BoneHeadBehaviour` uses DiceInterpreter.isSkillRollSuccessful: a natural 6 always succeeds and a
        // natural 1 always fails, whatever the target. Only differs from a bare `>=` when the target
        // leaves 2..6, which is exactly when it matters.
        let successful = crate::dice_interpreter::DiceInterpreter::is_skill_roll_successful(roll, min_roll);

        // Java: actingPlayer.markSkillUsed(skill) — Bone Head's SkillUsageType is not
        // "track outside activation", so it is recorded ONLY on the acting player (reset each
        // activation), not on the persistent Player.
        game.acting_player.used_skills.insert(SkillId::BoneHead);

        let re_rolled = state.re_rolled_action.as_deref() == Some("BONE_HEAD")
            && state.re_roll_source.is_some();
        game.report_list.add(ReportConfusionRoll::new(
            Some(player_id.clone()),
            successful,
            roll,
            min_roll,
            re_rolled,
            Some(SkillId::BoneHead.class_name().to_string()),
        ));
        let confusion_event = GameEvent::ConfusionRoll { player_id: player_id.clone(), roll, confused: !successful };

        if successful {
            state.outcome = Some(StepOutcome::next().with_event(confusion_event));
        } else if state.re_rolled_action.is_none() {
            if let Some(prompt) = ask_for_reroll_if_available(game, "BONE_HEAD", min_roll, false) {
                state.updated_re_rolled_action = Some("BONE_HEAD".into());
                state.updated_re_roll_source = Some("TRR".into());
                state.outcome = Some(StepOutcome::cont().with_event(confusion_event).with_prompt(prompt));
                return false;
            }
            cancel_negatrait_player_action(game, &player_id);
            crate::step::action::common::mark_target_selection_failed(game);
            state.outcome = Some(
                StepOutcome::goto(&state.goto_label_on_failure)
                    .with_event(confusion_event)
                    .publish(StepParameter::EndPlayerAction(true))
            );
        } else {
            cancel_negatrait_player_action(game, &player_id);
            crate::step::action::common::mark_target_selection_failed(game);
            state.outcome = Some(
                StepOutcome::goto(&state.goto_label_on_failure)
                    .with_event(confusion_event)
                    .publish(StepParameter::EndPlayerAction(true))
            );
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use crate::step::framework::test_team;

    fn test_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn register_into_adds_step_modifier() {
        let mut reg = SkillRegistry::empty();
        BoneHeadBehaviour::register_into(&mut reg);
        let sb = reg.get(SkillId::BoneHead).expect("BoneHead must be registered");
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }

    #[test]
    fn step_modifier_applies_to_correct_step() {
        let m = BoneHeadStepModifier;
        assert!(m.applies_to(StepId::BoneHead));
    }

    #[test]
    fn step_modifier_does_not_apply_to_wrong_step() {
        let m = BoneHeadStepModifier;
        assert!(!m.applies_to(StepId::BlockRoll));
    }

    #[test]
    fn step_modifier_no_negatraits_gives_next() {
        use ffb_model::enums::TurnMode;
        let m = BoneHeadStepModifier;
        let mut game = test_game();
        game.turn_mode = TurnMode::KickoffReturn;
        let mut hook = StepBoneHeadHookState {
            goto_label_on_failure: "FAIL".into(),
            re_rolled_action: None,
            re_roll_source: None,
            outcome: None,
            updated_re_rolled_action: None,
            updated_re_roll_source: None,
        };
        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hook);
        assert!(hook.outcome.is_some());
    }

    /// Java: `doRoll = UtilCards.hasUnusedSkill(actingPlayer, skill)` — when StepBoneHead is
    /// re-entered later in the same player action (e.g. Blitz's move phase already rolled and
    /// marked the skill used), the block phase's re-entry must NOT roll again.
    #[test]
    fn already_used_this_action_skips_second_roll_silently() {
        use crate::step::framework::StepAction;
        use ffb_model::model::player::Player;
        use ffb_model::model::skill_def::SkillWithValue;
        use ffb_model::report::report_id::ReportId;

        let mut game = test_game();
        let mut p = Player {
            id: "p1".into(), name: "p1".into(), nr: 1, position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![SkillWithValue { skill_id: SkillId::BoneHead, value: None }],
            ..Default::default()
        };
        game.team_home.players.push(p);
        game.acting_player.player_id = Some("p1".into());
        // Bone Head already used THIS activation (acting-player tracking, per Java's fUsedSkills).
        game.acting_player.used_skills.insert(SkillId::BoneHead);

        let m = BoneHeadStepModifier;
        let mut hook = StepBoneHeadHookState {
            goto_label_on_failure: "FAIL".into(),
            re_rolled_action: None,
            re_roll_source: None,
            outcome: None,
            updated_re_rolled_action: None,
            updated_re_roll_source: None,
        };
        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hook);

        let outcome = hook.outcome.expect("outcome must be set");
        assert_eq!(outcome.action, StepAction::NextStep, "already-used skill must silently pass through");
        assert!(!game.report_list.has_report(ReportId::CONFUSION_ROLL),
            "no roll (and thus no report) should occur when the skill was already used this action");
    }

    /// §12: a failed Bone Head must mark the pending TARGET SELECTION as FAILED, not just cancel
    /// the player action. Java's `cancelPlayerAction` runs inside the blitz-target chain, whose
    /// SelectBlitzTargetEnd then branches on the selection status; leaving the state UNSET made
    /// the chain take the "selected" arm after a failed negatrait and block with a cancelled
    /// action. Same fix is applied at all three cancel sites here and in ReallyStupid (bb2025)
    /// and their bb2020 twins.
    #[test]
    fn declined_reroll_marks_the_target_selection_failed() {
        use ffb_model::model::player::Player;
        use ffb_model::model::skill_def::SkillWithValue;
        use ffb_model::model::target_selection_state::{TargetSelectionState, TargetSelectionStatus};

        let mut game = test_game();
        game.team_home.players.push(Player {
            id: "p1".into(), name: "p1".into(), nr: 1, position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![SkillWithValue { skill_id: SkillId::BoneHead, value: None }],
            ..Default::default()
        });
        game.acting_player.player_id = Some("p1".into());
        game.field_model.target_selection_state = Some(TargetSelectionState::new("def"));

        let m = BoneHeadStepModifier;
        let mut hook = StepBoneHeadHookState {
            goto_label_on_failure: "FAIL".into(),
            // the re-roll was offered for BONE_HEAD and DECLINED (no source) -> cancel as failure
            re_rolled_action: Some("BONE_HEAD".into()),
            re_roll_source: None,
            outcome: None,
            updated_re_rolled_action: None,
            updated_re_roll_source: None,
        };
        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hook);

        assert_eq!(
            game.field_model.target_selection_state.as_ref().map(|ts| ts.status),
            Some(TargetSelectionStatus::FAILED),
            "a cancelled negatrait action must fail the pending target selection"
        );
    }
}
