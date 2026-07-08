/// 1:1 translation of com.fumbbl.ffb.server.skillbehaviour.bb2025.BloodLustBehaviour.
use crate::skill_behaviour::SkillBehaviour;
use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::model::step_modifier::StepModifierTrait;
use crate::step::framework::StepId;
use crate::step::bb2025::shared::step_blood_lust::StepBloodLustHookState;
use crate::skill_behaviour::registry::SkillRegistry;
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use ffb_model::enums::{PlayerAction, ReRollSource, SkillId};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_mechanics::mechanics::minimum_roll_blood_lust;
use ffb_model::report::report_blood_lust_roll::ReportBloodLustRoll;
use ffb_model::prompts::agent_prompt::AgentPrompt;
use crate::step::framework::{StepOutcome, StepParameter};

/// Blood Lust: vampire must bite a thrall or suffer Blood Lust failure before acting.
pub struct BloodLustBehaviour;

impl BloodLustBehaviour {
    pub fn new() -> Self { Self }

    pub fn register_into(registry: &mut SkillRegistry) {
        let mut sb = SbContainer::new();
        sb.register_step_modifier(Box::new(BloodLustStepModifier));
        registry.register(SkillId::BloodLust, sb);
    }
}

impl Default for BloodLustBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for BloodLustBehaviour {
    fn name(&self) -> &'static str { "BloodLustBehaviour" }

    fn execute_step_hook(&self, game: &mut ffb_model::model::game::Game) -> bool {
        let has_skill = game.acting_player.player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill(SkillId::BloodLust))
            .unwrap_or(false);
        if !has_skill {
            return false;
        }
        false
    }
}

// ── BloodLustStepModifier ─────────────────────────────────────────────────────

pub struct BloodLustStepModifier;

impl StepModifierTrait for BloodLustStepModifier {
    fn applies_to(&self, step_id: StepId) -> bool { step_id == StepId::BloodLust }

    fn priority(&self) -> i32 { 0 }

    /// Java: BloodLustBehaviour.handleExecuteStepHook(StepBloodLust step, StepState state)
    fn handle_execute_step(
        &self,
        game: &mut Game,
        rng: &mut GameRng,
        step_state: &mut dyn std::any::Any,
    ) -> bool {
        let state = step_state
            .downcast_mut::<StepBloodLustHookState>()
            .expect("BloodLustStepModifier: step_state must be StepBloodLustHookState");

        // Java: Phase 2 — after dialog was shown (WAIT_FOR_ACTION_CHANGE)
        if state.wait_for_action_change {
            return handle_phase2(game, state);
        }

        // Java: if (!game.getTurnMode().checkNegatraits()) → NEXT_STEP
        if !game.turn_mode.check_negatraits() {
            state.outcome = Some(StepOutcome::next());
            return false;
        }

        let acting_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => { state.outcome = Some(StepOutcome::next()); return false; }
        };

        let re_rolled = state.re_rolled_action.as_deref() == Some("BLOOD_LUST");
        let do_roll;

        if re_rolled {
            if let Some(ref source_str) = state.re_roll_source.clone() {
                let source = ReRollSource::new(source_str.as_str());
                if use_reroll(game, &source, &acting_id) {
                    do_roll = true;
                } else {
                    // declined → go straight to failure
                    state.outcome = Some(fail_for_action(game, state));
                    return false;
                }
            } else {
                // no source → declined
                state.outcome = Some(fail_for_action(game, state));
                return false;
            }
        } else {
            do_roll = game.player(&acting_id)
                .map(|p| p.has_skill(SkillId::BloodLust) && !p.used_skills.contains(&SkillId::BloodLust))
                .unwrap_or(false);
        }

        if !do_roll {
            state.outcome = Some(StepOutcome::next());
            return false;
        }

        let roll = rng.d6();
        let min_roll = minimum_roll_blood_lust();
        let successful = roll >= min_roll;

        if let Some(player) = game.player_mut(&acting_id) {
            player.used_skills.insert(SkillId::BloodLust);
        }

        let re_rolled_flag = re_rolled && state.re_roll_source.is_some();
        game.report_list.add(ReportBloodLustRoll::new(
            Some(acting_id.clone()),
            successful,
            roll,
            min_roll,
            re_rolled_flag,
            vec![],
        ));
        let event = GameEvent::BloodLustRoll { player_id: acting_id.clone(), roll, success: successful };

        if !successful {
            if !re_rolled {
                if let Some(prompt) = ask_for_reroll_if_available(game, "BLOOD_LUST", min_roll, false) {
                    state.updated_re_rolled_action = Some("BLOOD_LUST".into());
                    state.updated_re_roll_source = Some("TRR".into());
                    state.outcome = Some(StepOutcome::cont().with_event(event).with_prompt(prompt));
                    return false;
                }
            }
            let fail_outcome = fail_for_action(game, state).with_event(event);
            state.outcome = Some(fail_outcome);
            return false;
        }

        state.outcome = Some(StepOutcome::next().with_event(event));
        false
    }
}

/// Java: failBloodLustForAction — show dialog if action allows it, else direct failure.
fn fail_for_action(game: &mut Game, state: &mut StepBloodLustHookState) -> StepOutcome {
    let current_action = game.acting_player.player_action;
    let needs_dialog = current_action
        .map(|a| a != PlayerAction::Move && get_alternate_action(a) != a)
        .unwrap_or(false);

    if needs_dialog {
        state.wait_for_action_change = true;
        let player_id = game.acting_player.player_id.clone().unwrap_or_default();
        return StepOutcome::cont()
            .with_prompt(AgentPrompt::BloodlustAction { player_id });
    }

    fail_blood_lust(game, state)
}

/// Java: failBloodLust — set sufferingBloodLust, publish MOVE_STACK + optional goto.
fn fail_blood_lust(game: &mut Game, state: &StepBloodLustHookState) -> StepOutcome {
    game.acting_player.suffering_blood_lust = true;

    let base = match &state.goto_label_on_failure {
        Some(l) if !l.is_empty() => StepOutcome::goto(l),
        _ => StepOutcome::next(),
    };
    let out = base.publish(StepParameter::MoveStack(vec![]));
    match state.bloodlust_action {
        Some(action) => out.publish(StepParameter::BloodLustAction(Some(action))),
        None => out,
    }
}

/// Java: Phase 2 handler — after dialog was answered.
fn handle_phase2(game: &mut Game, state: &mut StepBloodLustHookState) -> bool {
    // The dialog answer was stored in state.bloodlust_action by handle_command
    let result = fail_blood_lust(game, state);
    state.outcome = Some(result);
    false
}

/// Java: getAlternateAction — maps action to its move variant.
fn get_alternate_action(current: PlayerAction) -> PlayerAction {
    match current {
        PlayerAction::Pass => PlayerAction::PassMove,
        PlayerAction::HandOver => PlayerAction::HandOverMove,
        PlayerAction::Foul => PlayerAction::FoulMove,
        PlayerAction::StandUpBlitz => PlayerAction::BlitzSelect,
        PlayerAction::ThrowTeamMate => PlayerAction::ThrowTeamMateMove,
        PlayerAction::KickTeamMate => PlayerAction::KickTeamMateMove,
        _ => PlayerAction::Move,
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
    fn hook_is_noop_returns_false() {
        let behaviour = BloodLustBehaviour::new();
        let mut game = test_game();
        assert!(!behaviour.execute_step_hook(&mut game));
    }

    #[test]
    fn execute_step_hook_returns_bool() {
        let behaviour = BloodLustBehaviour::new();
        let mut game = test_game();
        let _result: bool = behaviour.execute_step_hook(&mut game);
    }

    #[test]
    fn execute_step_hook_returns_false() {
        let b = BloodLustBehaviour::new();
        let mut game = test_game();
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = BloodLustBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }

    #[test]
    fn name_is_not_empty() {
        assert!(!BloodLustBehaviour::new().name().is_empty());
    }

    #[test]
    fn execute_step_hook_false_with_bb2025() {
        let b = BloodLustBehaviour::new();
        let mut game = test_game();
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn register_into_adds_step_modifier() {
        let mut reg = SkillRegistry::empty();
        BloodLustBehaviour::register_into(&mut reg);
        let sb = reg.get(SkillId::BloodLust).expect("BloodLust must be registered");
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }

    #[test]
    fn step_modifier_applies_to_correct_step() {
        let m = BloodLustStepModifier;
        assert!(m.applies_to(StepId::BloodLust));
    }

    #[test]
    fn step_modifier_does_not_apply_to_wrong_step() {
        let m = BloodLustStepModifier;
        assert!(!m.applies_to(StepId::BlockRoll));
    }

    #[test]
    fn modifier_no_negatraits_gives_next() {
        use ffb_model::enums::TurnMode;
        let m = BloodLustStepModifier;
        let mut game = test_game();
        game.turn_mode = TurnMode::KickoffReturn;
        let mut hook = StepBloodLustHookState {
            goto_label_on_failure: None,
            re_rolled_action: None,
            re_roll_source: None,
            bloodlust_action: None,
            wait_for_action_change: false,
            outcome: None,
            updated_re_rolled_action: None,
            updated_re_roll_source: None,
        };
        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hook);
        assert!(hook.outcome.is_some());
    }
}
