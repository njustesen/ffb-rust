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

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.shared.StepBloodLust.
///
/// Handles the Vampire blood-lust mechanic (BB2025).
///
/// Identical to the BB2020 version: on failure for non-MOVE actions the server shows
/// a `DialogBloodlustActionParameter` dialog offering the player an action change.
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
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if self.status == BloodLustStatus::WaitForActionChange {
            return self.fail_blood_lust(game);
        }

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

    fn fail_blood_lust_for_action(&mut self, game: &mut Game, acting_id: &str) -> StepOutcome {
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
        let mut game = Game::new(home, away, Rules::Bb2025);
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
    }

    #[test]
    fn no_blood_lust_skill_returns_next() {
        let mut game = make_game(vec![], Some(PlayerAction::Block));
        let out = StepBloodLust::new("fail").start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn successful_roll_returns_next_no_suffering() {
        let seed = seed_for_d6(3);
        let mut game = make_game(vec![SkillId::BloodLust], Some(PlayerAction::Block));
        let out = StepBloodLust::new("fail").start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!game.acting_player.suffering_blood_lust);
    }

    #[test]
    fn failed_roll_block_action_shows_dialog() {
        let seed = seed_for_d6(1);
        let mut game = make_game(vec![SkillId::BloodLust], Some(PlayerAction::Block));
        let mut step = StepBloodLust::new("fail");
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::Continue);
        assert!(matches!(out.prompt, Some(AgentPrompt::BloodlustAction { .. })));
        assert!(!game.acting_player.suffering_blood_lust);
    }

    #[test]
    fn failed_roll_move_action_direct_failure() {
        let seed = seed_for_d6(1);
        let mut game = make_game(vec![SkillId::BloodLust], Some(PlayerAction::Move));
        let out = StepBloodLust::new("fail").start(&mut game, &mut GameRng::new(seed));
        assert_ne!(out.action, StepAction::Continue);
        assert!(game.acting_player.suffering_blood_lust);
    }

    #[test]
    fn bloodlust_dialog_yes_publishes_alternate_action() {
        let seed = seed_for_d6(1);
        let mut game = make_game(vec![SkillId::BloodLust], Some(PlayerAction::Pass));
        let mut step = StepBloodLust::new("fail");
        step.start(&mut game, &mut GameRng::new(seed));
        let out = step.handle_command(&Action::BloodlustAction { change: true }, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        let ba = out.published.iter().find_map(|p| {
            if let StepParameter::BloodLustAction(v) = p { *v } else { None }
        });
        assert_eq!(ba, Some(PlayerAction::PassMove));
    }

    #[test]
    fn bloodlust_dialog_no_goes_to_failure() {
        let seed = seed_for_d6(1);
        let mut game = make_game(vec![SkillId::BloodLust], Some(PlayerAction::Block));
        let mut step = StepBloodLust::new("fail");
        step.start(&mut game, &mut GameRng::new(seed));
        let out = step.handle_command(&Action::BloodlustAction { change: false }, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(game.acting_player.suffering_blood_lust);
        assert!(!out.published.iter().any(|p| matches!(p, StepParameter::BloodLustAction(Some(_)))));
    }

    #[test]
    fn failed_roll_with_trr_offers_reroll_before_dialog() {
        let seed = seed_for_d6(1);
        let mut game = make_game(vec![SkillId::BloodLust], Some(PlayerAction::Block));
        game.turn_data_home.rerolls = 1;
        let mut step = StepBloodLust::new("fail");
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::Continue);
        assert!(matches!(out.prompt, Some(AgentPrompt::ReRollOffer { .. })));
    }

    #[test]
    fn goto_label_on_failure_used() {
        let seed = seed_for_d6(1);
        let mut game = make_game(vec![SkillId::BloodLust], Some(PlayerAction::Move));
        let out = StepBloodLust::new("feed_label").start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("feed_label"));
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
    fn get_alternate_action_default_becomes_move() {
        assert_eq!(StepBloodLust::get_alternate_action(PlayerAction::Block), PlayerAction::Move);
    }

    #[test]
    fn step_id_is_blood_lust() {
        assert_eq!(StepBloodLust::new("").id(), StepId::BloodLust);
    }
}
