/// 1:1 translation of com.fumbbl.ffb.server.skillbehaviour.bb2016.WildAnimalBehaviour.
///
/// BB2016 differences from BB2025 (UnchannelledFury):
/// 1. Uses WildAnimal skill (not UnchannelledFury).
/// 2. Calls `recoverTacklezones()` before the roll (same as BoneHead/ReallyStupid BB2016).
/// 3. `cancelPlayerAction` sets `changeBase(STANDING).changeActive(false)` when NOT standing up
///    (BB2025 uses `changeConfused(true).changeActive(false)`) — Wild Animal leaves player
///    standing but inactive, not confused.
/// 4. `cancelPlayerAction` uses simplified BB2016 action list (KTM → blitz_used, TTM → pass_used).
/// 5. `goodConditions`: action is BLITZ/BLITZ_MOVE/BLOCK/MULTIPLE_BLOCK/STAND_UP_BLITZ.
use crate::skill_behaviour::SkillBehaviour;
use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::model::step_modifier::StepModifierTrait;
use crate::step::framework::StepId;
use crate::step::bb2016::step_wild_animal::StepWildAnimalHookState;
use crate::skill_behaviour::registry::SkillRegistry;
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use ffb_model::enums::{PlayerAction, PS_PRONE, PS_STANDING, ReRollSource, SkillId};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_mechanics::mechanics::minimum_roll_confusion;
use ffb_model::report::report_confusion_roll::ReportConfusionRoll;
use crate::step::framework::{StepOutcome, StepParameter};

/// Wild Animal: player must pass confusion check (good conditions = blitz/block action type).
/// On failure: player set STANDING + inactive (not confused), action cancelled.
pub struct WildAnimalBehaviour;

impl WildAnimalBehaviour {
    pub fn new() -> Self { Self }

    pub fn register_into(registry: &mut SkillRegistry) {
        let mut sb = SbContainer::new();
        sb.register_step_modifier(Box::new(WildAnimalStepModifier));
        registry.register(SkillId::WildAnimal, sb);
    }
}

impl Default for WildAnimalBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for WildAnimalBehaviour {
    fn name(&self) -> &'static str { "WildAnimalBehaviour" }

    fn execute_step_hook(&self, game: &mut ffb_model::model::game::Game) -> bool {
        let has_skill = game.acting_player.player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill(SkillId::WildAnimal))
            .unwrap_or(false);
        if !has_skill { return false; }
        false
    }
}

// ── BB2016 WildAnimal cancel helper ──────────────────────────────────────────
//
// Java BB2016 WildAnimalBehaviour.cancelPlayerAction:
//   BLITZ/BLITZ_MOVE/KICK_TEAM_MATE/KICK_TEAM_MATE_MOVE → blitzUsed
//   PASS/PASS_MOVE/THROW_TEAM_MATE/THROW_TEAM_MATE_MOVE → passUsed
//   HAND_OVER/HAND_OVER_MOVE → handOverUsed
//   FOUL/FOUL_MOVE → foulUsed
//   isStandingUp → changeBase(PRONE).changeActive(false)
//   else         → changeBase(STANDING).changeActive(false)   ← DIFFERENT from BoneHead/ReallyStupid
fn cancel_wild_animal_bb2016(game: &mut Game, player_id: &str) {
    match game.acting_player.player_action {
        Some(PlayerAction::Blitz)
        | Some(PlayerAction::BlitzMove)
        | Some(PlayerAction::KickTeamMate)
        | Some(PlayerAction::KickTeamMateMove) => {
            game.turn_data_mut().blitz_used = true;
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
            game.turn_data_mut().foul_used = true;
        }
        _ => {}
    }

    if let Some(state) = game.field_model.player_state(player_id) {
        let new_state = if game.acting_player.standing_up {
            // Standing up → knocked back prone
            state.change_base(PS_PRONE).change_active(false)
        } else {
            // Wild Animal: stays STANDING but inactive (not confused like BoneHead)
            // Java: playerState.changeBase(PlayerState.STANDING).changeActive(false)
            state.change_base(PS_STANDING).change_active(false)
        };
        game.field_model.set_player_state(player_id, new_state);
    }

    game.pass_coordinate = None;
}

// ── WildAnimalStepModifier ──────────────────────────────────────────────────

pub struct WildAnimalStepModifier;

impl StepModifierTrait for WildAnimalStepModifier {
    fn applies_to(&self, step_id: StepId) -> bool { step_id == StepId::WildAnimal }

    fn priority(&self) -> i32 { 0 }

    /// Java: bb2016.WildAnimalBehaviour.handleExecuteStepHook(StepWildAnimal step, StepState state)
    fn handle_execute_step(
        &self,
        game: &mut Game,
        rng: &mut GameRng,
        step_state: &mut dyn std::any::Any,
    ) -> bool {
        let state = step_state
            .downcast_mut::<StepWildAnimalHookState>()
            .expect("WildAnimalStepModifier: step_state must be StepWildAnimalHookState");

        // Java: if (!game.getTurnMode().checkNegatraits()) { setNextAction(NEXT_STEP); return false; }
        if !game.turn_mode.check_negatraits() {
            state.outcome = Some(StepOutcome::next());
            return false;
        }

        let player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => { state.outcome = Some(StepOutcome::next()); return false; }
        };

        // BB2016: recover tacklezones before the check
        if let Some(ps) = game.field_model.player_state(&player_id) {
            game.field_model.set_player_state(&player_id, ps.recover_tacklezones());
        }

        let has_wild_animal = game.player(&player_id)
            .map(|p| p.has_skill(SkillId::WildAnimal))
            .unwrap_or(false);
        if !has_wild_animal {
            state.outcome = Some(StepOutcome::next());
            return false;
        }

        // Java: if (WILD_ANIMAL == reRolledAction && !useReRoll) doRoll = false
        let mut skip_roll = false;
        if state.re_rolled_action.as_deref() == Some("WILD_ANIMAL") {
            if let Some(ref source_name) = state.re_roll_source.clone() {
                let source = ReRollSource::new(source_name.as_str());
                if !use_reroll(game, &source, &player_id) {
                    skip_roll = true;
                }
            } else {
                skip_roll = true; // player declined
            }
        }

        // goodConditions: BLITZ/BLITZ_MOVE/BLOCK/MULTIPLE_BLOCK/STAND_UP_BLITZ only
        // Java BB2016: (BLITZ_MOVE || BLITZ || BLOCK || MULTIPLE_BLOCK || STAND_UP_BLITZ)
        let good_conditions = matches!(game.acting_player.player_action,
            Some(PlayerAction::Blitz)
            | Some(PlayerAction::BlitzMove)
            | Some(PlayerAction::Block)
            | Some(PlayerAction::MultipleBlock)
            | Some(PlayerAction::StandUpBlitz)
        );

        let min_roll = minimum_roll_confusion(good_conditions);

        if skip_roll {
            let confusion_event = GameEvent::ConfusionRoll { player_id: player_id.clone(), roll: 1, confused: true };
            game.report_list.add(ReportConfusionRoll::new(
                Some(player_id.clone()),
                false,
                1,
                min_roll,
                true,
                Some(SkillId::WildAnimal.class_name().to_string()),
            ));
            cancel_wild_animal_bb2016(game, &player_id);
            state.outcome = Some(
                StepOutcome::goto(&state.goto_label_on_failure)
                    .with_event(confusion_event)
                    .publish(StepParameter::EndPlayerAction(true))
            );
            return false;
        }

        // Java: doRoll = UtilCards.hasUnusedSkill(actingPlayer, skill)
        let has_unused = game.player(&player_id)
            .map(|p| !p.used_skills.contains(&SkillId::WildAnimal))
            .unwrap_or(false);

        if state.re_rolled_action.is_none() && !has_unused {
            state.outcome = Some(StepOutcome::next());
            return false;
        }

        let roll = rng.d6();
        let successful = roll >= min_roll;

        // Java: actingPlayer.markSkillUsed(WildAnimal)
        let is_home = game.team_home.player(&player_id).is_some();
        if is_home {
            if let Some(p) = game.team_home.player_mut(&player_id) {
                p.used_skills.insert(SkillId::WildAnimal);
            }
        } else if let Some(p) = game.team_away.player_mut(&player_id) {
            p.used_skills.insert(SkillId::WildAnimal);
        }

        let re_rolled = state.re_rolled_action.as_deref() == Some("WILD_ANIMAL")
            && state.re_roll_source.is_some();
        game.report_list.add(ReportConfusionRoll::new(
            Some(player_id.clone()),
            successful,
            roll,
            min_roll,
            re_rolled,
            Some(SkillId::WildAnimal.class_name().to_string()),
        ));
        let confusion_event = GameEvent::ConfusionRoll { player_id: player_id.clone(), roll, confused: !successful };

        if successful {
            state.outcome = Some(StepOutcome::next().with_event(confusion_event));
        } else if state.re_rolled_action.is_none() {
            if let Some(prompt) = ask_for_reroll_if_available(game, "WILD_ANIMAL", min_roll, false) {
                state.updated_re_rolled_action = Some("WILD_ANIMAL".into());
                state.updated_re_roll_source = Some("TRR".into());
                state.outcome = Some(StepOutcome::cont().with_event(confusion_event).with_prompt(prompt));
                return false;
            }
            cancel_wild_animal_bb2016(game, &player_id);
            state.outcome = Some(
                StepOutcome::goto(&state.goto_label_on_failure)
                    .with_event(confusion_event)
                    .publish(StepParameter::EndPlayerAction(true))
            );
        } else {
            cancel_wild_animal_bb2016(game, &player_id);
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
        Game::new(home, away, Rules::Bb2016)
    }

    #[test]
    fn hook_is_noop_returns_false() {
        let behaviour = WildAnimalBehaviour::new();
        let mut game = test_game();
        assert!(!behaviour.execute_step_hook(&mut game));
    }

    #[test]
    fn execute_step_hook_returns_false() {
        let b = WildAnimalBehaviour::new();
        let mut game = test_game();
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = WildAnimalBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }

    #[test]
    fn name_is_not_empty() {
        assert!(!WildAnimalBehaviour::new().name().is_empty());
    }

    #[test]
    fn register_into_adds_step_modifier() {
        let mut reg = SkillRegistry::empty();
        WildAnimalBehaviour::register_into(&mut reg);
        let sb = reg.get(SkillId::WildAnimal).expect("WildAnimal must be registered");
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }

    #[test]
    fn step_modifier_applies_to_correct_step() {
        let m = WildAnimalStepModifier;
        assert!(m.applies_to(StepId::WildAnimal));
    }

    #[test]
    fn step_modifier_does_not_apply_to_wrong_step() {
        let m = WildAnimalStepModifier;
        assert!(!m.applies_to(StepId::BlockRoll));
    }

    #[test]
    fn step_modifier_no_negatraits_gives_next() {
        use ffb_model::enums::TurnMode;
        use ffb_model::util::rng::GameRng;
        let m = WildAnimalStepModifier;
        let mut game = test_game();
        game.turn_mode = TurnMode::KickoffReturn;
        let mut hook = StepWildAnimalHookState {
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

    /// BB2016: Wild Animal on failure sets STANDING (not confused) when not standing up
    #[test]
    fn cancel_wild_animal_sets_standing_not_confused() {
        use ffb_model::enums::{PS_STANDING, PlayerState};
        let mut game = test_game();
        game.acting_player.standing_up = false;
        game.acting_player.player_action = Some(PlayerAction::Pass);
        // Place a player state
        game.field_model.set_player_state("p1", PlayerState::new(PS_STANDING).change_active(true));
        cancel_wild_animal_bb2016(&mut game, "p1");
        let ps = game.field_model.player_state("p1").unwrap();
        assert_eq!(ps.base(), PS_STANDING, "Wild Animal sets base to STANDING not PRONE");
        assert!(!ps.is_active(), "player must be inactive after cancel");
        assert!(!ps.is_confused(), "Wild Animal should NOT set confused flag");
    }

    /// BB2016: Wild Animal KickTeamMate → blitz_used
    #[test]
    fn cancel_wild_animal_ktm_sets_blitz_used() {
        let mut game = test_game();
        game.acting_player.player_action = Some(PlayerAction::KickTeamMate);
        cancel_wild_animal_bb2016(&mut game, "nobody");
        assert!(game.turn_data_mut().blitz_used);
        assert!(!game.turn_data_mut().ktm_used);
    }

    /// BB2016: Wild Animal standing-up player → PRONE on cancel
    #[test]
    fn cancel_wild_animal_standing_up_sets_prone() {
        use ffb_model::enums::{PS_PRONE, PS_STANDING, PlayerState};
        let mut game = test_game();
        game.acting_player.standing_up = true;
        game.acting_player.player_action = Some(PlayerAction::Pass);
        game.field_model.set_player_state("p1", PlayerState::new(PS_STANDING).change_active(true));
        cancel_wild_animal_bb2016(&mut game, "p1");
        let ps = game.field_model.player_state("p1").unwrap();
        assert_eq!(ps.base(), PS_PRONE, "standing-up player gets set to PRONE on cancel");
    }
}
