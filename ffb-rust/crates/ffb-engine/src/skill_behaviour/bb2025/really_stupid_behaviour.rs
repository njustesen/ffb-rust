/// 1:1 translation of com.fumbbl.ffb.server.skillbehaviour.bb2025.ReallyStupidBehaviour.
use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::model::step_modifier::StepModifierTrait;
use crate::step::framework::StepId;
use crate::step::action::common::step_really_stupid::{StepReallyStupidHookState, has_non_really_stupid_adjacent_teammate};
use crate::skill_behaviour::registry::SkillRegistry;
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use crate::step::action::common::cancel_negatrait_player_action;
use ffb_model::enums::{PlayerAction, ReRollSource, SkillId};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_mechanics::mechanics::minimum_roll_confusion;
use ffb_model::report::report_confusion_roll::ReportConfusionRoll;
use crate::step::framework::{StepOutcome, StepParameter};

/// Really Stupid: player must roll 2+ (good conditions) or 4+ (bad conditions) each activation.
pub struct ReallyStupidBehaviour;

impl ReallyStupidBehaviour {
    pub fn new() -> Self { Self }

    pub fn register_into(registry: &mut SkillRegistry) {
        let mut sb = SbContainer::new();
        sb.register_step_modifier(Box::new(ReallyStupidStepModifier));
        registry.register(SkillId::ReallyStupid, sb);
    }
}

impl Default for ReallyStupidBehaviour {
    fn default() -> Self { Self::new() }
}

// ── ReallyStupidStepModifier ──────────────────────────────────────────────────

pub struct ReallyStupidStepModifier;

impl StepModifierTrait for ReallyStupidStepModifier {
    fn applies_to(&self, step_id: StepId) -> bool { step_id == StepId::ReallyStupid }

    fn priority(&self) -> i32 { 0 }

    /// Java: ReallyStupidBehaviour.handleExecuteStepHook(StepReallyStupid step, StepState state)
    fn handle_execute_step(
        &self,
        game: &mut Game,
        rng: &mut GameRng,
        step_state: &mut dyn std::any::Any,
    ) -> bool {
        let state = step_state
            .downcast_mut::<StepReallyStupidHookState>()
            .expect("ReallyStupidStepModifier: step_state must be StepReallyStupidHookState");

        // Java: if (!game.getTurnMode().checkNegatraits()) { setNextAction(NEXT_STEP); return false; }
        if !game.turn_mode.check_negatraits() {
            state.outcome = Some(StepOutcome::next());
            return false;
        }

        let player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => { state.outcome = Some(StepOutcome::next()); return false; }
        };

        let has_really_stupid = game.player(&player_id)
            .map(|p| p.has_skill(SkillId::ReallyStupid))
            .unwrap_or(false);
        if !has_really_stupid {
            state.outcome = Some(StepOutcome::next());
            return false;
        }

        // Java: if (REALLY_STUPID == reRolledAction && !useReRoll) doRoll = false
        let mut skip_roll = false;
        if state.re_rolled_action.as_deref() == Some("REALLY_STUPID") {
            if let Some(ref source_name) = state.re_roll_source.clone() {
                let source = ReRollSource::new(source_name.as_str());
                if !use_reroll(game, &source, &player_id) {
                    skip_roll = true;
                }
            } else {
                skip_roll = true; // player declined
            }
        }

        // goodConditions: TTM/KTM actions override, or adjacent non-RS teammate present
        let good_conditions = matches!(game.acting_player.player_action,
            Some(PlayerAction::ThrowTeamMate) | Some(PlayerAction::ThrowTeamMateMove)
            | Some(PlayerAction::KickTeamMate) | Some(PlayerAction::KickTeamMateMove)
        ) || has_non_really_stupid_adjacent_teammate(game, &player_id);

        let min_roll = minimum_roll_confusion(good_conditions);

        if skip_roll {
            let confusion_event = GameEvent::ConfusionRoll { player_id: player_id.clone(), roll: 1, confused: true };
            game.report_list.add(ReportConfusionRoll::new(
                Some(player_id.clone()),
                false,
                1,
                min_roll,
                true,
                Some(SkillId::ReallyStupid.class_name().to_string()),
            ));
            cancel_negatrait_player_action(game, &player_id);
            state.outcome = Some(
                StepOutcome::goto(&state.goto_label_on_failure)
                    .with_event(confusion_event)
                    .publish(StepParameter::EndPlayerAction(true))
            );
            return false;
        }

        let roll = rng.d6();
        let successful = roll >= min_roll;

        // Java: actingPlayer.markSkillUsed(skill)
        let is_home = game.team_home.player(&player_id).is_some();
        if is_home {
            if let Some(p) = game.team_home.player_mut(&player_id) {
                p.used_skills.insert(SkillId::ReallyStupid);
            }
        } else if let Some(p) = game.team_away.player_mut(&player_id) {
            p.used_skills.insert(SkillId::ReallyStupid);
        }

        let re_rolled = state.re_rolled_action.as_deref() == Some("REALLY_STUPID")
            && state.re_roll_source.is_some();
        game.report_list.add(ReportConfusionRoll::new(
            Some(player_id.clone()),
            successful,
            roll,
            min_roll,
            re_rolled,
            Some(SkillId::ReallyStupid.class_name().to_string()),
        ));
        let confusion_event = GameEvent::ConfusionRoll { player_id: player_id.clone(), roll, confused: !successful };

        if successful {
            state.outcome = Some(StepOutcome::next().with_event(confusion_event));
        } else if state.re_rolled_action.is_none() {
            if let Some(prompt) = ask_for_reroll_if_available(game, "REALLY_STUPID", min_roll, false) {
                state.updated_re_rolled_action = Some("REALLY_STUPID".into());
                state.updated_re_roll_source = Some("TRR".into());
                state.outcome = Some(StepOutcome::cont().with_event(confusion_event).with_prompt(prompt));
                return false;
            }
            cancel_negatrait_player_action(game, &player_id);
            state.outcome = Some(
                StepOutcome::goto(&state.goto_label_on_failure)
                    .with_event(confusion_event)
                    .publish(StepParameter::EndPlayerAction(true))
            );
        } else {
            cancel_negatrait_player_action(game, &player_id);
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
        ReallyStupidBehaviour::register_into(&mut reg);
        let sb = reg.get(SkillId::ReallyStupid).expect("ReallyStupid must be registered");
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }

    #[test]
    fn step_modifier_applies_to_correct_step() {
        let m = ReallyStupidStepModifier;
        assert!(m.applies_to(StepId::ReallyStupid));
    }

    #[test]
    fn step_modifier_does_not_apply_to_wrong_step() {
        let m = ReallyStupidStepModifier;
        assert!(!m.applies_to(StepId::BlockRoll));
    }

    #[test]
    fn step_modifier_no_negatraits_gives_next() {
        use ffb_model::enums::TurnMode;
        let m = ReallyStupidStepModifier;
        let mut game = test_game();
        game.turn_mode = TurnMode::KickoffReturn;
        let mut hook = StepReallyStupidHookState {
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

}
