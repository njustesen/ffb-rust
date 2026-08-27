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

    // Java: `if (actingPlayer.isStandingUp())` → PRONE, else STANDING. In stock Java bb2016 the
    // WildAnimal step runs in the SELECT sequence, BEFORE StepStandUp, so isStandingUp is still the
    // activation-start value: true iff the player was PRONE when activated (changeActingPlayer sets
    // standingUp = oldState.isProne(); StepStandUp later clears it). Rust's bb2016 activation loop
    // pushes the shared bb2020 Select (which has NO WildAnimal), so WildAnimal only fires in the
    // bb2016 MOVE sequence — AFTER StepStandUp has already cleared acting_player.standing_up. Reading
    // the live standing_up flag there is stale (false), which wrongly cancelled a prone Wild-Animal
    // player to STANDING instead of Java's PRONE (norse bb2016 seed2 i=14, the Snow Troll). Use the
    // sticky pre-activation state (`old_player_state`, set once per activation by changeActingPlayer
    // — same signal TakeRootBehaviour uses for `started_standing`), which equals Java's isStandingUp
    // at the select-sequence WildAnimal point regardless of the intervening stand-up.
    let was_standing_up = game.acting_player.old_player_state
        .map(|s| s.base() == PS_PRONE)
        .unwrap_or(false);
    if let Some(state) = game.field_model.player_state(player_id) {
        let new_state = if was_standing_up {
            // Standing up (was prone at activation) → knocked back prone
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
                if !use_reroll(game, &source, &player_id, rng) {
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

        // Java: doRoll = UtilCards.hasUnusedSkill(actingPlayer, skill) — Wild Animal is rolled
        // ONCE per activation. Java tracks it on the ACTING PLAYER (fUsedSkills, cleared each
        // activation by setPlayerId), so a re-entry of StepWildAnimal within the same activation
        // must NOT re-roll. This bb2016 behaviour previously read/marked the PERSISTENT
        // Player.used_skills (mirror of the old Bone Head / Really Stupid bug, FIX26/FIX29):
        // the persistent set is never cleared per-activation, so the Yhetee re-rolled Wild Animal
        // an extra time within a single activation → a 1-die desync (norse bb2016 seed2 i=14).
        // Gate on the acting player's per-activation used_skills instead.
        let has_unused = !game.acting_player.used_skills.contains(&SkillId::WildAnimal);

        if state.re_rolled_action.is_none() && !has_unused {
            state.outcome = Some(StepOutcome::next());
            return false;
        }

        let roll = rng.d6();
        // Java `WildAnimalBehaviour` uses DiceInterpreter.isSkillRollSuccessful: a natural 6 always succeeds and a
        // natural 1 always fails, whatever the target. Only differs from a bare `>=` when the target
        // leaves 2..6, which is exactly when it matters.
        let successful = crate::dice_interpreter::DiceInterpreter::is_skill_roll_successful(roll, min_roll);

        // Java: actingPlayer.markSkillUsed(WildAnimal) — recorded on the ACTING PLAYER (reset each
        // activation by setPlayerId), NOT the persistent Player, so the hasUnusedSkill guard above
        // fires per-activation.
        game.acting_player.used_skills.insert(SkillId::WildAnimal);

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

    /// BB2016: Wild Animal is rolled once per activation — a re-entry of StepWildAnimal
    /// within the same activation (acting_player.used_skills already contains WildAnimal)
    /// must NOT re-roll: it takes the NEXT_STEP no-roll path. Regression for the persistent-
    /// vs-acting used_skills desync (norse bb2016 seed2 i=14; mirrors FIX26/FIX29).
    #[test]
    fn wild_animal_not_re_rolled_within_same_activation() {
        use ffb_model::enums::{PS_STANDING, PlayerState, TurnMode};
        use ffb_model::model::skill_def::SkillWithValue;
        use ffb_model::types::FieldCoordinate;
        let mut game = test_game();
        let mut p = ffb_model::model::player::Player::default();
        p.id = "yhetee".into();
        p.starting_skills.push(SkillWithValue { skill_id: SkillId::WildAnimal, value: None });
        game.team_home.players.push(p);
        game.field_model.set_player_coordinate("yhetee", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("yhetee", PlayerState::new(PS_STANDING));
        game.home_playing = true;
        game.turn_mode = TurnMode::Regular;
        game.acting_player.player_id = Some("yhetee".into());
        game.acting_player.player_action = Some(PlayerAction::Move);
        // Simulate Wild Animal already rolled earlier THIS activation.
        game.acting_player.used_skills.insert(SkillId::WildAnimal);

        let m = WildAnimalStepModifier;
        let mut hook = StepWildAnimalHookState {
            goto_label_on_failure: "FAIL".into(),
            re_rolled_action: None, re_roll_source: None, outcome: None,
            updated_re_rolled_action: None, updated_re_roll_source: None,
        };
        let mut rng = GameRng::new(0);
        let before = rng.call_count;
        m.handle_execute_step(&mut game, &mut rng, &mut hook);
        assert_eq!(rng.call_count, before, "Wild Animal already used this activation must NOT roll again");
        assert_eq!(hook.outcome.as_ref().map(|o| o.action), Some(crate::step::framework::StepAction::NextStep));
    }

    /// BB2016: Wild Animal standing-up player → PRONE on cancel
    #[test]
    fn cancel_wild_animal_standing_up_sets_prone() {
        use ffb_model::enums::{PS_PRONE, PS_STANDING, PlayerState};
        let mut game = test_game();
        // "Standing up" = the player was PRONE at activation (Java isStandingUp). Mirrors the sticky
        // old_player_state signal the cancel now reads (robust to StepStandUp having already cleared
        // the live standing_up flag before the bb2016 MOVE-sequence WildAnimal fires).
        game.acting_player.old_player_state = Some(PlayerState::new(PS_PRONE));
        game.acting_player.player_action = Some(PlayerAction::Pass);
        game.field_model.set_player_state("p1", PlayerState::new(PS_STANDING).change_active(true));
        cancel_wild_animal_bb2016(&mut game, "p1");
        let ps = game.field_model.player_state("p1").unwrap();
        assert_eq!(ps.base(), PS_PRONE, "player prone at activation gets set back to PRONE on cancel");
    }
}
