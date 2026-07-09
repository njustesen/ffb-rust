/// 1:1 translation of com.fumbbl.ffb.server.skillbehaviour.bb2016.BoneHeadBehaviour.
///
/// BB2016 differences from BB2025:
/// 1. Calls `recoverTacklezones()` on the player state before the roll.
/// 2. `cancelPlayerAction` uses the simplified BB2016 action list:
///    - KickTeamMate/KickTeamMateMove → blitz_used (same bucket as BLITZ)
///    - ThrowTeamMate/ThrowTeamMateMove → pass_used (same bucket as PASS)
///    - No KICK_EM_BLITZ split, no KtmUsed, no TtmUsed, no SecureTheBall, no Punt.
/// 3. Does NOT call `commitTargetSelection()` (BB2025 added that).
use crate::skill_behaviour::SkillBehaviour;
use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::model::step_modifier::StepModifierTrait;
use crate::step::framework::StepId;
use crate::step::action::common::step_bone_head::StepBoneHeadHookState;
use crate::skill_behaviour::registry::SkillRegistry;
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use ffb_model::enums::{PlayerAction, PS_PRONE, ReRollSource, SkillId};
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

impl SkillBehaviour for BoneHeadBehaviour {
    fn name(&self) -> &'static str { "BoneHeadBehaviour" }

    fn execute_step_hook(&self, game: &mut ffb_model::model::game::Game) -> bool {
        let has_skill = game.acting_player.player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill(SkillId::BoneHead))
            .unwrap_or(false);
        if !has_skill { return false; }
        false
    }
}

// ── BB2016 cancel helper ──────────────────────────────────────────────────────
//
// Java BB2016 BoneHeadBehaviour.cancelPlayerAction:
//   BLITZ/BLITZ_MOVE/KICK_TEAM_MATE/KICK_TEAM_MATE_MOVE → blitzUsed
//   PASS/PASS_MOVE/THROW_TEAM_MATE/THROW_TEAM_MATE_MOVE → passUsed
//   HAND_OVER/HAND_OVER_MOVE → handOverUsed
//   FOUL/FOUL_MOVE → foulUsed
//   isStandingUp → changeBase(PRONE).changeActive(false)
//   else         → changeConfused(true).changeActive(false)
fn cancel_bb2016_negatrait(game: &mut Game, player_id: &str) {
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
            state.change_base(PS_PRONE).change_active(false)
        } else {
            state.change_confused(true).change_active(false)
        };
        game.field_model.set_player_state(player_id, new_state);
    }

    game.pass_coordinate = None;
}

// ── BoneHeadStepModifier ──────────────────────────────────────────────────────

pub struct BoneHeadStepModifier;

impl StepModifierTrait for BoneHeadStepModifier {
    fn applies_to(&self, step_id: StepId) -> bool { step_id == StepId::BoneHead }

    fn priority(&self) -> i32 { 0 }

    /// Java: bb2016.BoneHeadBehaviour.handleExecuteStepHook(StepBoneHead step, StepState state)
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

        // BB2016: recover tacklezones before the check
        // Java: playerState = getPlayerState(actingPlayer).recoverTacklezones()
        //       setPlayerState(actingPlayer, playerState)
        if let Some(ps) = game.field_model.player_state(&player_id) {
            game.field_model.set_player_state(&player_id, ps.recover_tacklezones());
        }

        let has_bone_head = game.player(&player_id)
            .map(|p| p.has_skill(SkillId::BoneHead))
            .unwrap_or(false);
        if !has_bone_head {
            state.outcome = Some(StepOutcome::next());
            return false;
        }

        // Java: if (BONE_HEAD == reRolledAction && !useReRoll) doRoll = false
        let mut skip_roll = false;
        if state.re_rolled_action.as_deref() == Some("BONE_HEAD") {
            if let Some(ref source_name) = state.re_roll_source.clone() {
                let source = ReRollSource::new(source_name.as_str());
                if !use_reroll(game, &source, &player_id) {
                    skip_roll = true;
                }
            } else {
                skip_roll = true; // player declined
            }
        }

        if skip_roll {
            let confusion_event = GameEvent::ConfusionRoll { player_id: player_id.clone(), roll: 1, confused: true };
            cancel_bb2016_negatrait(game, &player_id);
            state.outcome = Some(
                StepOutcome::goto(&state.goto_label_on_failure)
                    .with_event(confusion_event)
                    .publish(StepParameter::EndPlayerAction(true))
            );
            return false;
        }

        // BB2016: no commitTargetSelection() call here (added in BB2025)
        let roll = rng.d6();
        let min_roll = minimum_roll_confusion(true);
        let successful = roll >= min_roll;

        // Java: actingPlayer.markSkillUsed(skill)
        let is_home = game.team_home.player(&player_id).is_some();
        if is_home {
            if let Some(p) = game.team_home.player_mut(&player_id) {
                p.used_skills.insert(SkillId::BoneHead);
            }
        } else if let Some(p) = game.team_away.player_mut(&player_id) {
            p.used_skills.insert(SkillId::BoneHead);
        }

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
            cancel_bb2016_negatrait(game, &player_id);
            state.outcome = Some(
                StepOutcome::goto(&state.goto_label_on_failure)
                    .with_event(confusion_event)
                    .publish(StepParameter::EndPlayerAction(true))
            );
        } else {
            cancel_bb2016_negatrait(game, &player_id);
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
        let behaviour = BoneHeadBehaviour::new();
        let mut game = test_game();
        assert!(!behaviour.execute_step_hook(&mut game));
    }

    #[test]
    fn execute_step_hook_returns_false() {
        let b = BoneHeadBehaviour::new();
        let mut game = test_game();
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = BoneHeadBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }

    #[test]
    fn name_is_not_empty() {
        assert!(!BoneHeadBehaviour::new().name().is_empty());
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
        use ffb_model::util::rng::GameRng;
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

    /// BB2016: KickTeamMate action maps to blitz_used (not ktm_used as in BB2025)
    #[test]
    fn cancel_bb2016_ktm_sets_blitz_used() {
        let mut game = test_game();
        game.acting_player.player_action = Some(PlayerAction::KickTeamMate);
        cancel_bb2016_negatrait(&mut game, "nobody");
        assert!(game.turn_data_mut().blitz_used);
        assert!(!game.turn_data_mut().ktm_used);
    }

    /// BB2016: ThrowTeamMate action maps to pass_used (not ttm_used as in BB2025)
    #[test]
    fn cancel_bb2016_ttm_sets_pass_used() {
        let mut game = test_game();
        game.acting_player.player_action = Some(PlayerAction::ThrowTeamMate);
        cancel_bb2016_negatrait(&mut game, "nobody");
        assert!(game.turn_data_mut().pass_used);
        assert!(!game.turn_data_mut().ttm_used);
    }

    /// BB2016: Foul action always sets foul_used (no allowsAdditionalFoul check in BB2016)
    #[test]
    fn cancel_bb2016_foul_sets_foul_used() {
        let mut game = test_game();
        game.acting_player.player_action = Some(PlayerAction::Foul);
        cancel_bb2016_negatrait(&mut game, "nobody");
        assert!(game.turn_data_mut().foul_used);
    }
}
