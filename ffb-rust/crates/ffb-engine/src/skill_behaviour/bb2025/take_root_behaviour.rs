/// 1:1 translation of com.fumbbl.ffb.server.skillbehaviour.bb2025.TakeRootBehaviour.
use crate::skill_behaviour::SkillBehaviour;
use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::model::step_modifier::StepModifierTrait;
use crate::step::framework::StepId;
use crate::step::bb2025::shared::step_take_root::StepTakeRootHookState;
use crate::skill_behaviour::registry::SkillRegistry;
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use ffb_model::enums::{PS_STANDING, ReRollSource, SkillId};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_mechanics::mechanics::minimum_roll_confusion;
use crate::step::framework::StepOutcome;

/// Take Root: roll 4+ or become rooted (cannot move) this turn.
pub struct TakeRootBehaviour;

impl TakeRootBehaviour {
    pub fn new() -> Self { Self }

    pub fn register_into(registry: &mut SkillRegistry) {
        let mut sb = SbContainer::new();
        sb.register_step_modifier(Box::new(TakeRootStepModifier));
        registry.register(SkillId::TakeRoot, sb);
    }
}

impl Default for TakeRootBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for TakeRootBehaviour {
    fn name(&self) -> &'static str { "TakeRootBehaviour" }

    fn execute_step_hook(&self, game: &mut ffb_model::model::game::Game) -> bool {
        let has_skill = game.acting_player.player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill(SkillId::TakeRoot))
            .unwrap_or(false);
        if !has_skill {
            return false;
        }
        false
    }
}

// ── TakeRootStepModifier ──────────────────────────────────────────────────────

pub struct TakeRootStepModifier;

impl StepModifierTrait for TakeRootStepModifier {
    fn applies_to(&self, step_id: StepId) -> bool { step_id == StepId::TakeRoot }

    fn priority(&self) -> i32 { 0 }

    /// Java: TakeRootBehaviour.handleExecuteStepHook(StepTakeRoot step, StepState state)
    fn handle_execute_step(
        &self,
        game: &mut Game,
        rng: &mut GameRng,
        step_state: &mut dyn std::any::Any,
    ) -> bool {
        let state = step_state
            .downcast_mut::<StepTakeRootHookState>()
            .expect("TakeRootStepModifier: step_state must be StepTakeRootHookState");

        let player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return false,
        };

        // Java: actingPlayer.getOldPlayerState().getBase() == PlayerState.STANDING
        let started_standing = state.old_player_state
            .map(|s| s.base() == PS_STANDING)
            .unwrap_or(true); // conservative default

        let is_rooted = game.field_model.player_state(&player_id)
            .map(|s| s.is_rooted())
            .unwrap_or(false);

        if !started_standing || is_rooted {
            return false;
        }

        // Java: if (TAKE_ROOT == reRolledAction) { if (source == null || !useReRoll) cancel }
        //       else { doRoll = hasUnusedSkill(actingPlayer, TakeRoot) }
        let do_roll;
        if state.re_rolled_action.as_deref() == Some("TAKE_ROOT") {
            if let Some(ref source_name) = state.re_roll_source.clone() {
                let source = ReRollSource::new(source_name.as_str());
                if use_reroll(game, &source, &player_id) {
                    do_roll = true;
                } else {
                    // player declined re-roll → cancel
                    cancel_take_root(game, &player_id);
                    return false;
                }
            } else {
                // no source → declined
                cancel_take_root(game, &player_id);
                return false;
            }
        } else {
            // Java: doRoll = UtilCards.hasUnusedSkill(actingPlayer, TakeRoot)
            do_roll = game.player(&player_id)
                .map(|p| p.has_skill(SkillId::TakeRoot) && !p.used_skills.contains(&SkillId::TakeRoot))
                .unwrap_or(false);
        }

        if !do_roll {
            return false;
        }

        // Java: minimumRollConfusion(true) = 2 (always good_conditions=true for TakeRoot)
        let roll = rng.d6();
        let minimum_roll = minimum_roll_confusion(true);
        let successful = roll >= minimum_roll;

        if successful {
            // NEXT_STEP — hook returns, step continues normally
        } else {
            // Java: if (reRolledAction != current && askForReRollIfAvailable) → WAITING_FOR_RE_ROLL
            if state.re_rolled_action.is_none() {
                if let Some(prompt) = ask_for_reroll_if_available(game, "TAKE_ROOT", minimum_roll, false) {
                    state.updated_re_rolled_action = Some("TAKE_ROOT".into());
                    state.updated_re_roll_source = Some("TRR".into());
                    state.outcome = Some(StepOutcome::cont().with_prompt(prompt));
                    return false;
                }
            }
            cancel_take_root(game, &player_id);
        }
        false
    }
}

/// Java: StepTakeRoot.cancelPlayerAction() inlined here — set player to rooted state.
fn cancel_take_root(game: &mut Game, player_id: &str) {
    if let Some(ps) = game.field_model.player_state(player_id) {
        game.field_model.set_player_state(player_id, ps.change_rooted(true));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, TurnMode, PlayerAction, PS_STANDING, PS_PRONE, PlayerState};
    use ffb_model::model::skill_def::SkillWithValue;
    use crate::step::framework::{test_team, StepAction};

    fn test_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    fn make_game_with_take_root() -> (Game, String) {
        let pid = "root".to_string();
        let mut home = test_team("home", 0);
        home.players.push(ffb_model::model::player::Player {
            id: pid.clone(),
            name: pid.clone(),
            nr: 1,
            position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![SkillWithValue { skill_id: SkillId::TakeRoot, value: None }],
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
        });
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.acting_player.player_id = Some(pid.clone());
        game.acting_player.player_action = Some(PlayerAction::Move);
        game.field_model.set_player_state(&pid, PlayerState::new(PS_STANDING).change_active(true));
        (game, pid)
    }

    fn seed_for_d6(target: i32) -> u64 {
        for s in 0u64..10_000 {
            if ffb_model::util::rng::GameRng::new(s).d6() == target { return s; }
        }
        panic!("no seed for d6={}", target);
    }

    #[test]
    fn register_into_adds_step_modifier() {
        let mut reg = SkillRegistry::empty();
        TakeRootBehaviour::register_into(&mut reg);
        let sb = reg.get(SkillId::TakeRoot).expect("TakeRoot must be registered");
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }

    #[test]
    fn step_modifier_applies_to_correct_step() {
        let m = TakeRootStepModifier;
        assert!(m.applies_to(StepId::TakeRoot));
    }

    #[test]
    fn step_modifier_does_not_apply_to_wrong_step() {
        let m = TakeRootStepModifier;
        assert!(!m.applies_to(StepId::BlockRoll));
    }

    #[test]
    fn hook_is_noop_returns_false() {
        let behaviour = TakeRootBehaviour::new();
        let mut game = test_game();
        assert!(!behaviour.execute_step_hook(&mut game));
    }

    #[test]
    fn execute_step_hook_returns_bool() {
        let behaviour = TakeRootBehaviour::new();
        let mut game = test_game();
        let _result: bool = behaviour.execute_step_hook(&mut game);
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use crate::step::framework::test_team;
        let b = TakeRootBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2025,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = TakeRootBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }

    #[test]
    fn name_is_not_empty() {
        assert!(!TakeRootBehaviour::new().name().is_empty());
    }

    #[test]
    fn execute_step_hook_false_with_bb2025() {
        use crate::step::framework::test_team;
        let b = TakeRootBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2025,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn modifier_no_acting_player_returns_false() {
        let m = TakeRootStepModifier;
        let mut game = test_game();
        let mut hook = StepTakeRootHookState {
            re_rolled_action: None, re_roll_source: None,
            old_player_state: Some(PlayerState::new(PS_STANDING)),
            outcome: None,
            updated_re_rolled_action: None, updated_re_roll_source: None,
        };
        let result = m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hook);
        assert!(!result);
    }

    #[test]
    fn modifier_started_prone_skips_roll() {
        let m = TakeRootStepModifier;
        let (mut game, pid) = make_game_with_take_root();
        let mut hook = StepTakeRootHookState {
            re_rolled_action: None, re_roll_source: None,
            old_player_state: Some(PlayerState::new(PS_PRONE)),
            outcome: None,
            updated_re_rolled_action: None, updated_re_roll_source: None,
        };
        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hook);
        assert!(!game.field_model.player_state(&pid).unwrap().is_rooted());
    }

    #[test]
    fn modifier_already_rooted_skips_roll() {
        let m = TakeRootStepModifier;
        let (mut game, pid) = make_game_with_take_root();
        let state = game.field_model.player_state(&pid).unwrap().change_rooted(true);
        game.field_model.set_player_state(&pid, state);
        let mut hook = StepTakeRootHookState {
            re_rolled_action: None, re_roll_source: None,
            old_player_state: Some(PlayerState::new(PS_STANDING)),
            outcome: None,
            updated_re_rolled_action: None, updated_re_roll_source: None,
        };
        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hook);
        // still rooted, no crash
        assert!(game.field_model.player_state(&pid).unwrap().is_rooted());
    }

    #[test]
    fn modifier_failed_roll_roots_player() {
        let seed = seed_for_d6(1);
        let m = TakeRootStepModifier;
        let (mut game, pid) = make_game_with_take_root();
        let mut hook = StepTakeRootHookState {
            re_rolled_action: None, re_roll_source: None,
            old_player_state: Some(PlayerState::new(PS_STANDING)),
            outcome: None,
            updated_re_rolled_action: None, updated_re_roll_source: None,
        };
        m.handle_execute_step(&mut game, &mut GameRng::new(seed), &mut hook);
        assert!(game.field_model.player_state(&pid).unwrap().is_rooted());
    }

    #[test]
    fn modifier_successful_roll_not_rooted() {
        let seed = seed_for_d6(3);
        let m = TakeRootStepModifier;
        let (mut game, pid) = make_game_with_take_root();
        let mut hook = StepTakeRootHookState {
            re_rolled_action: None, re_roll_source: None,
            old_player_state: Some(PlayerState::new(PS_STANDING)),
            outcome: None,
            updated_re_rolled_action: None, updated_re_roll_source: None,
        };
        m.handle_execute_step(&mut game, &mut GameRng::new(seed), &mut hook);
        assert!(!game.field_model.player_state(&pid).unwrap().is_rooted());
    }

    #[test]
    fn modifier_failed_with_trr_sets_waiting() {
        let seed = seed_for_d6(1);
        let m = TakeRootStepModifier;
        let (mut game, _) = make_game_with_take_root();
        game.turn_data_home.rerolls = 1;
        let mut hook = StepTakeRootHookState {
            re_rolled_action: None, re_roll_source: None,
            old_player_state: Some(PlayerState::new(PS_STANDING)),
            outcome: None,
            updated_re_rolled_action: None, updated_re_roll_source: None,
        };
        m.handle_execute_step(&mut game, &mut GameRng::new(seed), &mut hook);
        assert!(hook.outcome.is_some());
        assert_eq!(
            hook.outcome.as_ref().unwrap().action,
            StepAction::Continue
        );
        assert_eq!(hook.updated_re_rolled_action.as_deref(), Some("TAKE_ROOT"));
    }

    #[test]
    fn modifier_declined_reroll_roots_player() {
        let m = TakeRootStepModifier;
        let (mut game, pid) = make_game_with_take_root();
        let mut hook = StepTakeRootHookState {
            re_rolled_action: Some("TAKE_ROOT".into()),
            re_roll_source: None, // declined
            old_player_state: Some(PlayerState::new(PS_STANDING)),
            outcome: None,
            updated_re_rolled_action: None, updated_re_roll_source: None,
        };
        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hook);
        assert!(game.field_model.player_state(&pid).unwrap().is_rooted());
    }
}
