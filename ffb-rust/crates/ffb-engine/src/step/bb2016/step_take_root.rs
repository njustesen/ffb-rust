use ffb_model::enums::{ReRollSource, SkillId};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use crate::util::UtilServerPlayerMove;

/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.StepTakeRoot`.
///
/// Resolves the Take Root negatrait check.
/// Core logic from Java's `TakeRootBehaviour.handleExecuteStepHook`, inlined here.
///
/// If the player is already rooted, skip the roll.
/// Roll d6 vs. minimumRollConfusion(true) = 2.
/// On failure: cancel player action and set rooted.
pub struct StepTakeRoot {
    /// Java: AbstractStepWithReRoll.reRolledAction
    pub re_rolled_action: Option<String>,
    /// Java: AbstractStepWithReRoll.reRollSource
    pub re_roll_source: Option<String>,
}

impl StepTakeRoot {
    pub fn new() -> Self { Self { re_rolled_action: None, re_roll_source: None } }
}

impl Default for StepTakeRoot {
    fn default() -> Self { Self::new() }
}

impl Step for StepTakeRoot {
    fn id(&self) -> StepId { StepId::TakeRoot }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if let Action::UseReRoll { use_reroll: false } = action {
            self.re_roll_source = None;
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

impl StepTakeRoot {
    /// Java: `TakeRootBehaviour.handleExecuteStepHook(StepTakeRoot, StepState)`.
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if !game.turn_mode.check_negatraits() {
            return StepOutcome::next();
        }

        let acting_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        // Java: recoverTacklezones at start of check
        if let Some(state) = game.field_model.player_state(&acting_id) {
            game.field_model.set_player_state(&acting_id, state.recover_tacklezones());
        }

        // Java: if (playerState.isRooted()) { skip — already rooted }
        if game.field_model.player_state(&acting_id).map(|s| s.is_rooted()).unwrap_or(false) {
            return StepOutcome::next();
        }

        let re_rolled = self.re_rolled_action.as_deref() == Some("TAKE_ROOT");
        let do_roll;

        if re_rolled {
            if let Some(ref source_str) = self.re_roll_source.clone() {
                let source = ReRollSource::new(source_str.as_str());
                if use_reroll(game, &source, &acting_id) {
                    do_roll = true;
                } else {
                    self.cancel_player_action(game);
                    return StepOutcome::next();
                }
            } else {
                self.cancel_player_action(game);
                return StepOutcome::next();
            }
        } else {
            // Java: doRoll = UtilCards.hasUnusedSkill(actingPlayer, TakeRoot)
            do_roll = game.player(&acting_id)
                .map(|p| p.has_skill(SkillId::TakeRoot) && !p.used_skills.contains(&SkillId::TakeRoot))
                .unwrap_or(false);
        }

        if !do_roll {
            return StepOutcome::next();
        }

        let roll = rng.d6();
        let min_roll = DiceInterpreter::minimum_roll_confusion(true);
        let successful = roll >= min_roll;

        if let Some(player) = game.player_mut(&acting_id) {
            player.used_skills.insert(SkillId::TakeRoot);
        }

        let event = GameEvent::ConfusionRoll {
            player_id: acting_id.clone(),
            roll,
            confused: !successful,
        };

        if !successful {
            if !re_rolled {
                if let Some(prompt) = ask_for_reroll_if_available(game, "TAKE_ROOT", min_roll, false) {
                    self.re_rolled_action = Some("TAKE_ROOT".into());
                    self.re_roll_source = Some("TRR".into());
                    return StepOutcome::cont().with_event(event).with_prompt(prompt);
                }
            }
            self.cancel_player_action(game);
            return StepOutcome::next().with_event(event);
        }

        StepOutcome::next().with_event(event)
    }

    /// Java: `StepTakeRoot.cancelPlayerAction()`.
    fn cancel_player_action(&self, game: &mut Game) {
        let player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return,
        };

        game.acting_player.goes_for_it = false;

        if let Some(action) = game.acting_player.player_action {
            let base_action = match action {
                PlayerAction::BlitzMove => Some(PlayerAction::Blitz),
                PlayerAction::PassMove => Some(PlayerAction::Pass),
                PlayerAction::ThrowTeamMateMove => Some(PlayerAction::ThrowTeamMate),
                PlayerAction::KickTeamMateMove => Some(PlayerAction::KickTeamMate),
                PlayerAction::HandOverMove => Some(PlayerAction::HandOver),
                PlayerAction::FoulMove => Some(PlayerAction::Foul),
                PlayerAction::Move => {
                    UtilServerPlayerMove::update_move_squares(game, false);
                    None
                }
                _ => None,
            };
            if let Some(new_action) = base_action {
                game.acting_player.player_action = Some(new_action);
            }
        }

        let state = game.field_model.player_state(&player_id)
            .unwrap_or_else(|| ffb_model::enums::PlayerState::new(ffb_model::enums::PS_STANDING));
        game.field_model.set_player_state(&player_id, state.change_rooted(true));
    }
}

use ffb_model::enums::PlayerAction;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{test_team, StepAction};
    use ffb_model::enums::{Rules, TurnMode};
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::types::FieldCoordinate;

    fn make_game(skills: Vec<SkillId>) -> Game {
        let mut home = test_team("home", 0);
        home.players.push(ffb_model::model::player::Player {
            id: "p1".into(), name: "p1".into(), nr: 1, position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: skills.into_iter()
                .map(|s| SkillWithValue { skill_id: s, value: None })
                .collect(),
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        });
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2016);
        game.home_playing = true;
        game.acting_player.player_id = Some("p1".into());
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("p1", ffb_model::enums::PlayerState::new(ffb_model::enums::PS_STANDING));
        game.turn_mode = TurnMode::Regular;
        game
    }

    fn seed_for_d6(target: i32) -> u64 {
        for s in 0u64..10_000 {
            if GameRng::new(s).d6() == target { return s; }
        }
        panic!("no seed for d6={}", target);
    }

    #[test]
    fn negatraits_disabled_skips_roll() {
        let mut game = make_game(vec![SkillId::TakeRoot]);
        game.turn_mode = TurnMode::KickoffReturn;
        let out = StepTakeRoot::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn already_rooted_skips_roll() {
        let mut game = make_game(vec![SkillId::TakeRoot]);
        if let Some(s) = game.field_model.player_state("p1") {
            game.field_model.set_player_state("p1", s.change_rooted(true));
        }
        let out = StepTakeRoot::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        // Player should still be rooted, not changed
        assert!(game.field_model.player_state("p1").map(|s| s.is_rooted()).unwrap_or(false));
    }

    #[test]
    fn no_take_root_skill_returns_next() {
        let mut game = make_game(vec![]);
        let out = StepTakeRoot::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn successful_roll_returns_next_not_rooted() {
        // minimum_roll_confusion(true) == 2, so roll >= 2 succeeds
        let seed = seed_for_d6(4);
        let mut game = make_game(vec![SkillId::TakeRoot]);
        let out = StepTakeRoot::new().start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!game.field_model.player_state("p1").map(|s| s.is_rooted()).unwrap_or(false));
    }

    #[test]
    fn failed_roll_sets_rooted() {
        let seed = seed_for_d6(1); // roll 1 < 2 → fail
        let mut game = make_game(vec![SkillId::TakeRoot]);
        let out = StepTakeRoot::new().start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(game.field_model.player_state("p1").map(|s| s.is_rooted()).unwrap_or(false));
    }

    #[test]
    fn failed_roll_with_trr_offers_reroll() {
        let seed = seed_for_d6(1);
        let mut game = make_game(vec![SkillId::TakeRoot]);
        game.turn_data_home.rerolls = 1;
        let mut step = StepTakeRoot::new();
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::Continue);
        assert!(out.prompt.is_some());
    }

    #[test]
    fn cancel_action_reverts_blitz_move_to_blitz() {
        let seed = seed_for_d6(1);
        let mut game = make_game(vec![SkillId::TakeRoot]);
        game.acting_player.player_action = Some(PlayerAction::BlitzMove);
        StepTakeRoot::new().start(&mut game, &mut GameRng::new(seed));
        assert_eq!(game.acting_player.player_action, Some(PlayerAction::Blitz));
    }

    #[test]
    fn marks_skill_used_on_roll() {
        let seed = seed_for_d6(4);
        let mut game = make_game(vec![SkillId::TakeRoot]);
        StepTakeRoot::new().start(&mut game, &mut GameRng::new(seed));
        assert!(game.player("p1").unwrap().used_skills.contains(&SkillId::TakeRoot));
    }
}
