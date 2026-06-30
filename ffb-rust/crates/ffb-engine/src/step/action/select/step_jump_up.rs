/// 1:1 translation of com.fumbbl.ffb.server.step.action.select.StepJumpUp (COMMON rules)
/// and its BB2020/BB2025 hook com.fumbbl.ffb.server.skillbehaviour.mixed.JumpUpBehaviour.
///
/// Resolves the Jump Up skill roll for a standing-up block action.
/// Needs GOTO_LABEL_ON_FAILURE init parameter.
///
/// If the player is standing up, hasn't moved, has unused JumpUp, and is performing
/// a block action: rolls agility. On success → NEXT_STEP (proceed). On failure →
/// player is set prone, END_PLAYER_ACTION published, GOTO_LABEL_ON_FAILURE.
/// For non-block actions or players not standing up: NEXT_STEP.
use ffb_model::enums::{PlayerAction, ReRollSource, SkillId, PS_PRONE};
use ffb_model::enums::PlayerState;
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_mechanics::mechanics::minimum_roll_jump_up;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

pub struct StepJumpUp {
    /// Java: state.goToLabelOnFailure — GOTO_LABEL_ON_FAILURE init parameter.
    pub goto_label_on_failure: String,
    // AbstractStepWithReRoll stubs (TODO: translate full re-roll infrastructure)
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
}

impl StepJumpUp {
    pub fn new() -> Self {
        Self {
            goto_label_on_failure: String::new(),
            re_rolled_action: None,
            re_roll_source: None,
        }
    }
}

impl Default for StepJumpUp {
    fn default() -> Self { Self::new() }
}

impl Step for StepJumpUp {
    fn id(&self) -> StepId { StepId::JumpUp }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if let Action::UseReRoll { use_reroll: false } = action {
            self.re_roll_source = None;
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(v) => { self.goto_label_on_failure = v.clone(); true }
            _ => false,
        }
    }
}

impl StepJumpUp {
    /// Java: JumpUpBehaviour(BB2020/BB2025).handleExecuteStepHook
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        let re_rolled = self.re_rolled_action.as_deref() == Some("JUMP_UP");

        // Java: (isStandingUp && !hasMoved && hasUnusedSkill(JumpUp)) || reRolledAction == JUMP_UP
        let has_unused_jump_up = game.player(&player_id)
            .map(|p| p.has_skill(SkillId::JumpUp) && !p.used_skills.contains(&SkillId::JumpUp))
            .unwrap_or(false);

        let enter_logic = (game.acting_player.standing_up
            && !game.acting_player.has_moved
            && has_unused_jump_up)
            || re_rolled;

        if !enter_logic {
            return StepOutcome::next();
        }

        // Java: game.setConcessionPossible(false) — stub: concession_possible not yet in model.

        let action = game.acting_player.player_action;

        // Java: if (playerAction.isBlockAction() || playerAction == MULTIPLE_BLOCK)
        let is_block = action.map(|a| a.is_block_action()).unwrap_or(false)
            || action == Some(PlayerAction::MultipleBlock);

        if !is_block {
            return StepOutcome::next();
        }

        // Java: if (JUMP_UP == reRolledAction) { if (source == null || !useReRoll) → fail }
        if re_rolled {
            if let Some(ref source_name) = self.re_roll_source.clone() {
                let source = ReRollSource::new(source_name.as_str());
                if !use_reroll(game, &source, &player_id) {
                    // Token exhausted — fail immediately
                    let ps = game.field_model.player_state(&player_id).unwrap_or_else(|| PlayerState::new(PS_PRONE));
                    game.field_model.set_player_state(&player_id, ps.change_base(PS_PRONE).change_active(false));
                    mark_used(game, &player_id);
                    return StepOutcome::goto(&self.goto_label_on_failure)
                        .publish(StepParameter::EndPlayerAction(true));
                }
                // Re-roll consumed — fall through to re-roll dice
            } else {
                // Player declined (handle_command cleared source) — fail immediately
                let ps = game.field_model.player_state(&player_id).unwrap_or_else(|| PlayerState::new(PS_PRONE));
                game.field_model.set_player_state(&player_id, ps.change_base(PS_PRONE).change_active(false));
                mark_used(game, &player_id);
                return StepOutcome::goto(&self.goto_label_on_failure)
                    .publish(StepParameter::EndPlayerAction(true));
            }
        }

        // No JumpUpModifier support yet → modifiers = empty.
        let agility = game.player(&player_id).map(|p| p.agility).unwrap_or(3);
        let min_roll = minimum_roll_jump_up(agility, &[]);
        let roll = rng.d6();
        let successful = roll >= min_roll;

        let jump_up_event = GameEvent::JumpUpRoll {
            player_id: player_id.clone(),
            target: min_roll,
            roll,
            success: successful,
        };

        mark_used(game, &player_id);

        if successful {
            game.acting_player.has_moved = true;
            game.acting_player.standing_up = false;
            StepOutcome::next().with_event(jump_up_event)
        } else {
            // Java: if (reRolledAction != JUMP_UP && askForReRollIfAvailable(...)) → CONTINUE
            if !re_rolled {
                if let Some(prompt) = ask_for_reroll_if_available(game, "JUMP_UP", min_roll, false) {
                    self.re_rolled_action = Some("JUMP_UP".into());
                    self.re_roll_source = Some("TRR".into());
                    return StepOutcome::cont().with_event(jump_up_event).with_prompt(prompt);
                }
            }
            let ps = game.field_model.player_state(&player_id).unwrap_or_else(|| PlayerState::new(PS_PRONE));
            game.field_model.set_player_state(&player_id, ps.change_base(PS_PRONE).change_active(false));
            StepOutcome::goto(&self.goto_label_on_failure)
                .with_event(jump_up_event)
                .publish(StepParameter::EndPlayerAction(true))
        }
    }
}

fn mark_used(game: &mut Game, player_id: &str) {
    let is_home = game.team_home.player(player_id).is_some();
    if is_home {
        if let Some(p) = game.team_home.player_mut(player_id) {
            p.used_skills.insert(SkillId::JumpUp);
        }
    } else if let Some(p) = game.team_away.player_mut(player_id) {
        p.used_skills.insert(SkillId::JumpUp);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::Action;
    use crate::step::framework::{StepAction, StepParameter};
    use crate::step::framework::test_team;
    use ffb_model::enums::{Rules, TurnMode, PS_STANDING};
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::types::FieldCoordinate;

    fn add_player(team: &mut ffb_model::model::team::Team, id: &str, nr: i32, skills: Vec<SkillId>, agility: i32) {
        team.players.push(ffb_model::model::player::Player {
            id: id.into(),
            name: id.into(),
            nr,
            position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility, passing: 4, armour: 8,
            starting_skills: skills.into_iter().map(|s| SkillWithValue { skill_id: s, value: None }).collect(),
            extra_skills: vec![],
            temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0,
            stat_injuries: vec![],
            current_spps: 0,
            career_spps: 0,
            race: None,
        });
    }

    fn make_game_block_standing_up(skills: Vec<SkillId>, agility: i32) -> (Game, String) {
        let pid = "p1".to_string();
        let mut home = test_team("home", 0);
        add_player(&mut home, &pid, 1, skills, agility);
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.acting_player.player_id = Some(pid.clone());
        game.acting_player.player_action = Some(PlayerAction::Block);
        game.acting_player.standing_up = true;
        game.acting_player.has_moved = false;
        game.field_model.set_player_state(&pid, PlayerState::new(PS_STANDING).change_active(true));
        game.field_model.set_player_coordinate(&pid, FieldCoordinate::new(5, 7));
        (game, pid)
    }

    fn seed_for_d6(target: i32) -> u64 {
        for s in 0u64..10_000 {
            if GameRng::new(s).d6() == target { return s; }
        }
        panic!("no seed for d6={}", target);
    }

    #[test]
    fn no_acting_player_skips() {
        let (mut game, _) = make_game_block_standing_up(vec![SkillId::JumpUp], 3);
        game.acting_player.player_id = None;
        let outcome = StepJumpUp::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
    }

    #[test]
    fn not_standing_up_skips() {
        let (mut game, _) = make_game_block_standing_up(vec![SkillId::JumpUp], 3);
        game.acting_player.standing_up = false;
        let outcome = StepJumpUp::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.events.is_empty());
    }

    #[test]
    fn already_moved_skips() {
        let (mut game, _) = make_game_block_standing_up(vec![SkillId::JumpUp], 3);
        game.acting_player.has_moved = true;
        let outcome = StepJumpUp::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.events.is_empty());
    }

    #[test]
    fn no_jump_up_skill_skips() {
        let (mut game, _) = make_game_block_standing_up(vec![], 3);
        let outcome = StepJumpUp::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.events.is_empty());
    }

    #[test]
    fn skill_already_used_skips() {
        let (mut game, pid) = make_game_block_standing_up(vec![SkillId::JumpUp], 3);
        game.team_home.player_mut(&pid).unwrap().used_skills.insert(SkillId::JumpUp);
        let outcome = StepJumpUp::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
    }

    #[test]
    fn non_block_action_skips() {
        let (mut game, _) = make_game_block_standing_up(vec![SkillId::JumpUp], 3);
        game.acting_player.player_action = Some(PlayerAction::Move);
        let outcome = StepJumpUp::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.events.is_empty());
    }

    #[test]
    fn multiple_block_triggers_roll() {
        let seed = seed_for_d6(3); // ag=3 → min_roll=3 → success
        let (mut game, _) = make_game_block_standing_up(vec![SkillId::JumpUp], 3);
        game.acting_player.player_action = Some(PlayerAction::MultipleBlock);
        let mut step = StepJumpUp::new();
        step.goto_label_on_failure = "FAIL".into();
        let outcome = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(outcome.action, StepAction::NextStep);
    }

    #[test]
    fn successful_roll_clears_standing_up() {
        let seed = seed_for_d6(3); // ag=3 → min_roll=3 → success
        let (mut game, _) = make_game_block_standing_up(vec![SkillId::JumpUp], 3);
        let mut step = StepJumpUp::new();
        step.goto_label_on_failure = "FAIL".into();
        step.start(&mut game, &mut GameRng::new(seed));
        assert!(!game.acting_player.standing_up, "standing_up cleared on success");
        assert!(game.acting_player.has_moved, "has_moved set on success");
    }

    #[test]
    fn successful_roll_emits_event() {
        let seed = seed_for_d6(4);
        let (mut game, _) = make_game_block_standing_up(vec![SkillId::JumpUp], 3);
        let mut step = StepJumpUp::new();
        step.goto_label_on_failure = "FAIL".into();
        let outcome = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.events.iter().any(|e| matches!(
            e, GameEvent::JumpUpRoll { success: true, .. }
        )));
    }

    #[test]
    fn failed_roll_goes_to_label_and_makes_prone() {
        let seed = seed_for_d6(1); // 1 < 3 → failure for ag=3
        let (mut game, pid) = make_game_block_standing_up(vec![SkillId::JumpUp], 3);
        let mut step = StepJumpUp::new();
        step.goto_label_on_failure = "JUMP_FAIL".into();
        let outcome = step.start(&mut game, &mut GameRng::new(seed));

        assert_eq!(outcome.action, StepAction::GotoLabel);
        assert_eq!(outcome.goto_label.as_deref(), Some("JUMP_FAIL"));
        assert!(matches!(outcome.published.first(), Some(StepParameter::EndPlayerAction(true))));

        let state = game.field_model.player_state(&pid).unwrap();
        assert_eq!(state.base(), PS_PRONE);
        assert!(!state.is_active());
    }

    #[test]
    fn failed_roll_emits_event() {
        let seed = seed_for_d6(1);
        let (mut game, _) = make_game_block_standing_up(vec![SkillId::JumpUp], 3);
        let mut step = StepJumpUp::new();
        step.goto_label_on_failure = "FAIL".into();
        let outcome = step.start(&mut game, &mut GameRng::new(seed));
        assert!(outcome.events.iter().any(|e| matches!(
            e, GameEvent::JumpUpRoll { success: false, .. }
        )));
    }

    #[test]
    fn jump_up_marked_used_after_roll() {
        let seed = seed_for_d6(4);
        let (mut game, pid) = make_game_block_standing_up(vec![SkillId::JumpUp], 3);
        StepJumpUp::new().start(&mut game, &mut GameRng::new(seed));
        assert!(game.team_home.player(&pid).unwrap().used_skills.contains(&SkillId::JumpUp));
    }

    #[test]
    fn set_parameter_stores_goto_label() {
        let mut step = StepJumpUp::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("X".into())));
        assert_eq!(step.goto_label_on_failure, "X");
    }

    #[test]
    fn failed_roll_with_trr_offers_reroll_prompt() {
        let seed = seed_for_d6(1); // 1 < 3 → failure
        let (mut game, _) = make_game_block_standing_up(vec![SkillId::JumpUp], 3);
        game.turn_data_home.rerolls = 1;
        let mut step = StepJumpUp::new();
        step.goto_label_on_failure = "FAIL".into();
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::Continue, "TRR available → should offer re-roll");
        assert!(out.prompt.is_some());
        assert_eq!(step.re_rolled_action.as_deref(), Some("JUMP_UP"));
    }

    #[test]
    fn decline_reroll_clears_source_and_fails() {
        let (mut game, pid) = make_game_block_standing_up(vec![SkillId::JumpUp], 3);
        let mut step = StepJumpUp::new();
        step.goto_label_on_failure = "FAIL".into();
        step.re_rolled_action = Some("JUMP_UP".into());
        step.re_roll_source = Some("TRR".into());
        let out = step.handle_command(
            &Action::UseReRoll { use_reroll: false },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("FAIL"));
        let state = game.field_model.player_state(&pid).unwrap();
        assert_eq!(state.base(), PS_PRONE);
    }
}
