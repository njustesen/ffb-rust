use ffb_mechanics::mechanics::minimum_roll_blood_lust;
use ffb_model::enums::{PlayerAction, ReRollSource, SkillId};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::prompts::agent_prompt::AgentPrompt;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.shared.StepBloodLust.
///
/// Handles the Vampire blood-lust mechanic (BB2020).
///
/// BB2020 difference from BB2016: on failure, if the acting player's current action is not
/// already a MOVE-type action, the server shows a `DialogBloodlustActionParameter` dialog
/// asking the player whether to change to the alternate (move-phase) action to feed.
/// This is tracked via `status = WAIT_FOR_ACTION_CHANGE`.
///
/// Java state fields: goToLabelOnFailure, bloodlustAction, status (ActionStatus).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BloodLustStatus {
    None,
    WaitingForReRoll,
    WaitForActionChange,
    Success,
    Failure,
}

pub struct StepBloodLust {
    /// Java: state.goToLabelOnFailure (init param GOTO_LABEL_ON_FAILURE)
    pub goto_label_on_failure: Option<String>,
    /// Java: state.bloodlustAction — alternate action chosen by the client
    pub bloodlust_action: Option<PlayerAction>,
    /// Java: state.status (ActionStatus)
    pub status: BloodLustStatus,
    /// Java: AbstractStepWithReRoll.reRolledAction
    pub re_rolled_action: Option<String>,
    /// Java: AbstractStepWithReRoll.reRollSource
    pub re_roll_source: Option<String>,
}

impl StepBloodLust {
    pub fn new(goto_label_on_failure: impl Into<String>) -> Self {
        let label = goto_label_on_failure.into();
        Self {
            goto_label_on_failure: if label.is_empty() { None } else { Some(label) },
            bloodlust_action: None,
            status: BloodLustStatus::None,
            re_rolled_action: None,
            re_roll_source: None,
        }
    }
}

impl Default for StepBloodLust {
    fn default() -> Self { Self::new("") }
}

impl Step for StepBloodLust {
    fn id(&self) -> StepId { StepId::BloodLust }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if let Action::UseReRoll { use_reroll: false } = action {
            self.re_roll_source = None;
        }
        // Java: CLIENT_BLOODLUST_ACTION → if (change) state.bloodlustAction = getAlternateAction(currentAction)
        if let Action::BloodlustAction { change } = action {
            if *change {
                if let Some(current) = game.acting_player.player_action {
                    self.bloodlust_action = Some(Self::get_alternate_action(current));
                }
            }
            // status stays WAIT_FOR_ACTION_CHANGE; execute_step handles it
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(v) => {
                self.goto_label_on_failure = Some(v.clone());
                true
            }
            StepParameter::BloodLustAction(v) => {
                self.bloodlust_action = *v;
                true
            }
            _ => false,
        }
    }
}

impl StepBloodLust {
    /// Java: BloodLustBehaviour.handleExecuteStepHook (BB2020 version).
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: if (state.status == WAIT_FOR_ACTION_CHANGE) → publish bloodlustAction + fail
        if self.status == BloodLustStatus::WaitForActionChange {
            return self.fail_blood_lust(game);
        }

        // Java: if (!game.getTurnMode().checkNegatraits()) → NEXT_STEP
        if !game.turn_mode.check_negatraits() {
            return StepOutcome::next();
        }

        let acting_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        let re_rolled = self.re_rolled_action.as_deref() == Some("BLOOD_LUST");
        let do_roll;

        if re_rolled {
            if let Some(ref source_str) = self.re_roll_source.clone() {
                let source = ReRollSource::new(source_str.as_str());
                if use_reroll(game, &source, &acting_id) {
                    do_roll = true;
                } else {
                    return self.fail_blood_lust_for_action(game, &acting_id);
                }
            } else {
                return self.fail_blood_lust_for_action(game, &acting_id);
            }
        } else {
            // Java: doRoll = UtilCards.hasUnusedSkill(actingPlayer, BloodLust)
            do_roll = game.player(&acting_id)
                .map(|p| p.has_skill(SkillId::BloodLust) && !p.used_skills.contains(&SkillId::BloodLust))
                .unwrap_or(false);
        }

        if !do_roll {
            return StepOutcome::next();
        }

        let roll = rng.d6();
        let min_roll = minimum_roll_blood_lust();
        let successful = roll >= min_roll;

        // Java: actingPlayer.markSkillUsed(skill)
        if let Some(player) = game.player_mut(&acting_id) {
            player.used_skills.insert(SkillId::BloodLust);
        }

        let event = GameEvent::BloodLustRoll { player_id: acting_id.clone(), roll, success: successful };

        if !successful {
            if !re_rolled {
                if let Some(prompt) = ask_for_reroll_if_available(game, "BLOOD_LUST", min_roll, false) {
                    self.re_rolled_action = Some("BLOOD_LUST".into());
                    self.re_roll_source = Some("TRR".into());
                    self.status = BloodLustStatus::WaitingForReRoll;
                    return StepOutcome::cont().with_event(event).with_prompt(prompt);
                }
            }
            return self.fail_blood_lust_for_action(game, &acting_id).with_event(event);
        }

        self.status = BloodLustStatus::Success;
        StepOutcome::next().with_event(event)
    }

    /// Decide whether to show the action-change dialog or go directly to failure.
    fn fail_blood_lust_for_action(&mut self, game: &mut Game, acting_id: &str) -> StepOutcome {
        // BB2020: if current action is non-MOVE type, show dialog asking if player wants to
        // change to the alternate (move-phase) action so the vampire can go feed.
        let current_action = game.acting_player.player_action;
        let needs_dialog = current_action
            .map(|a| a != PlayerAction::Move && Self::get_alternate_action(a) != a)
            .unwrap_or(false);

        if needs_dialog {
            self.status = BloodLustStatus::WaitForActionChange;
            let player_id = acting_id.to_string();
            return StepOutcome::cont()
                .with_prompt(AgentPrompt::BloodlustAction { player_id });
        }

        self.fail_blood_lust(game)
    }

    /// Java: status = FAILURE; setSufferingBloodLust(true); publish MOVE_STACK(null);
    ///        optionally publish BLOOD_LUST_ACTION; goto failure label or NEXT_STEP.
    fn fail_blood_lust(&mut self, game: &mut Game) -> StepOutcome {
        self.status = BloodLustStatus::Failure;
        game.acting_player.suffering_blood_lust = true;

        let label = self.goto_label_on_failure.clone();
        let bloodlust_param = self.bloodlust_action;

        let base = match label {
            Some(ref l) if !l.is_empty() => StepOutcome::goto(l),
            _ => StepOutcome::next(),
        };
        let out = base.publish(StepParameter::MoveStack(vec![]));
        match bloodlust_param {
            Some(action) => out.publish(StepParameter::BloodLustAction(Some(action))),
            None => out,
        }
    }

    /// Java: private PlayerAction getAlternateAction(PlayerAction currentAction)
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{test_team, StepAction};
    use ffb_model::enums::{Rules, TurnMode};
    use ffb_model::model::skill_def::SkillWithValue;

    fn add_player(team: &mut ffb_model::model::team::Team, id: &str, skills: Vec<SkillId>) {
        team.players.push(ffb_model::model::player::Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: skills.into_iter()
                .map(|s| SkillWithValue { skill_id: s, value: None }).collect(),
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        });
    }

    fn make_game(skills: Vec<SkillId>, action: Option<PlayerAction>) -> Game {
        let mut home = test_team("home", 0);
        add_player(&mut home, "vamp", skills);
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2020);
        game.home_playing = true;
        game.acting_player.player_id = Some("vamp".into());
        game.acting_player.player_action = action;
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
        let mut game = make_game(vec![SkillId::BloodLust], Some(PlayerAction::Block));
        game.turn_mode = TurnMode::KickoffReturn;
        let out = StepBloodLust::new("fail").start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!game.acting_player.suffering_blood_lust);
    }

    #[test]
    fn no_acting_player_returns_next() {
        let mut game = make_game(vec![SkillId::BloodLust], None);
        game.acting_player.player_id = None;
        let out = StepBloodLust::new("fail").start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn no_blood_lust_skill_returns_next() {
        let mut game = make_game(vec![], Some(PlayerAction::Block));
        let out = StepBloodLust::new("fail").start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn already_used_blood_lust_returns_next() {
        let mut game = make_game(vec![SkillId::BloodLust], Some(PlayerAction::Block));
        game.player_mut("vamp").unwrap().used_skills.insert(SkillId::BloodLust);
        let out = StepBloodLust::new("fail").start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn successful_roll_returns_next_no_suffering() {
        // minimum_roll_blood_lust() == 2, so roll >= 2 succeeds
        let seed = seed_for_d6(3);
        let mut game = make_game(vec![SkillId::BloodLust], Some(PlayerAction::Block));
        let out = StepBloodLust::new("fail").start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!game.acting_player.suffering_blood_lust);
        assert!(out.events.iter().any(|e| matches!(e, GameEvent::BloodLustRoll { success: true, .. })));
    }

    #[test]
    fn successful_roll_marks_skill_used() {
        let seed = seed_for_d6(4);
        let mut game = make_game(vec![SkillId::BloodLust], Some(PlayerAction::Block));
        StepBloodLust::new("fail").start(&mut game, &mut GameRng::new(seed));
        assert!(game.player("vamp").unwrap().used_skills.contains(&SkillId::BloodLust));
    }

    #[test]
    fn failed_roll_block_action_shows_dialog() {
        // BLOCK is a non-MOVE action → should show BloodlustAction dialog
        let seed = seed_for_d6(1); // roll 1 < 2 → fail
        let mut game = make_game(vec![SkillId::BloodLust], Some(PlayerAction::Block));
        let mut step = StepBloodLust::new("fail");
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::Continue);
        assert!(matches!(out.prompt, Some(AgentPrompt::BloodlustAction { .. })));
        assert!(!game.acting_player.suffering_blood_lust);
    }

    #[test]
    fn failed_roll_move_action_direct_failure() {
        // MOVE action → no dialog, direct failure
        let seed = seed_for_d6(1);
        let mut game = make_game(vec![SkillId::BloodLust], Some(PlayerAction::Move));
        let out = StepBloodLust::new("fail").start(&mut game, &mut GameRng::new(seed));
        // Direct failure — should go to label or next, not Continue
        assert_ne!(out.action, StepAction::Continue);
        assert!(game.acting_player.suffering_blood_lust);
    }

    #[test]
    fn bloodlust_dialog_yes_publishes_alternate_action() {
        let seed = seed_for_d6(1);
        let mut game = make_game(vec![SkillId::BloodLust], Some(PlayerAction::Pass));
        let mut step = StepBloodLust::new("fail");
        // First call: fail roll, show dialog
        step.start(&mut game, &mut GameRng::new(seed));
        // Player says "yes, change action"
        let out = step.handle_command(&Action::BloodlustAction { change: true }, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::BloodLustAction(Some(_)))));
        // Should have published PASS_MOVE as the alternate
        let ba = out.published.iter().find_map(|p| {
            if let StepParameter::BloodLustAction(v) = p { *v } else { None }
        });
        assert_eq!(ba, Some(PlayerAction::PassMove));
    }

    #[test]
    fn bloodlust_dialog_no_goes_to_failure_no_alternate() {
        let seed = seed_for_d6(1);
        let mut game = make_game(vec![SkillId::BloodLust], Some(PlayerAction::Block));
        let mut step = StepBloodLust::new("fail");
        step.start(&mut game, &mut GameRng::new(seed));
        // Player says "no, don't change action"
        let out = step.handle_command(&Action::BloodlustAction { change: false }, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(game.acting_player.suffering_blood_lust);
        // No BloodLustAction param published (player declined)
        assert!(!out.published.iter().any(|p| matches!(p, StepParameter::BloodLustAction(Some(_)))));
    }

    #[test]
    fn failed_roll_with_trr_offers_reroll_before_dialog() {
        let seed = seed_for_d6(1);
        let mut game = make_game(vec![SkillId::BloodLust], Some(PlayerAction::Block));
        game.turn_data_home.rerolls = 1;
        let mut step = StepBloodLust::new("fail");
        let out = step.start(&mut game, &mut GameRng::new(seed));
        // TRR takes priority over the bloodlust dialog
        assert_eq!(out.action, StepAction::Continue);
        assert!(matches!(out.prompt, Some(AgentPrompt::ReRollOffer { .. })));
        assert_eq!(step.status, BloodLustStatus::WaitingForReRoll);
    }

    #[test]
    fn failed_roll_then_decline_trr_shows_dialog_for_block() {
        let seed = seed_for_d6(1);
        let mut game = make_game(vec![SkillId::BloodLust], Some(PlayerAction::Block));
        game.turn_data_home.rerolls = 1;
        let mut step = StepBloodLust::new("fail");
        step.start(&mut game, &mut GameRng::new(seed));
        // Decline TRR
        let out = step.handle_command(&Action::UseReRoll { use_reroll: false }, &mut game, &mut GameRng::new(0));
        // Now should show bloodlust dialog (since BLOCK needs alternate action)
        assert_eq!(out.action, StepAction::Continue);
        assert!(matches!(out.prompt, Some(AgentPrompt::BloodlustAction { .. })));
    }

    #[test]
    fn goto_label_on_failure_is_used() {
        let seed = seed_for_d6(1);
        let mut game = make_game(vec![SkillId::BloodLust], Some(PlayerAction::Move));
        let out = StepBloodLust::new("feed_label").start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("feed_label"));
    }

    #[test]
    fn move_stack_published_on_failure() {
        let seed = seed_for_d6(1);
        let mut game = make_game(vec![SkillId::BloodLust], Some(PlayerAction::Move));
        let out = StepBloodLust::new("fail").start(&mut game, &mut GameRng::new(seed));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::MoveStack(_))));
    }

    #[test]
    fn set_parameter_goto_label_on_failure() {
        let mut step = StepBloodLust::default();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("x".to_string())));
        assert_eq!(step.goto_label_on_failure.as_deref(), Some("x"));
    }

    #[test]
    fn set_parameter_blood_lust_action() {
        let mut step = StepBloodLust::default();
        assert!(step.set_parameter(&StepParameter::BloodLustAction(Some(PlayerAction::Move))));
        assert_eq!(step.bloodlust_action, Some(PlayerAction::Move));
    }

    #[test]
    fn get_alternate_action_pass_becomes_pass_move() {
        assert_eq!(StepBloodLust::get_alternate_action(PlayerAction::Pass), PlayerAction::PassMove);
    }

    #[test]
    fn get_alternate_action_foul_becomes_foul_move() {
        assert_eq!(StepBloodLust::get_alternate_action(PlayerAction::Foul), PlayerAction::FoulMove);
    }

    #[test]
    fn get_alternate_action_block_becomes_move() {
        assert_eq!(StepBloodLust::get_alternate_action(PlayerAction::Block), PlayerAction::Move);
    }

    #[test]
    fn get_alternate_action_move_stays_move() {
        assert_eq!(StepBloodLust::get_alternate_action(PlayerAction::Move), PlayerAction::Move);
    }

    #[test]
    fn step_id_is_blood_lust() {
        assert_eq!(StepBloodLust::new("").id(), StepId::BloodLust);
    }
}
