use ffb_mechanics::mechanics::minimum_roll_blood_lust;
use ffb_model::enums::{ReRollSource, SkillId};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.StepBloodLust`.
///
/// Step in block sequence to handle blood lust.
///
/// Needs to be initialized with stepParameter GOTO_LABEL_ON_FAILURE.
/// Sets stepParameter MOVE_STACK for all steps on the stack.
///
/// Core logic lives in Java's `BloodLustBehaviour.handleExecuteStepHook`, inlined here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BloodLustStatus {
    None,
    Success,
    Failure,
    WaitingForReRoll,
}

pub struct StepBloodLust {
    /// Java: state.status (ActionStatus — hook output)
    pub status: BloodLustStatus,
    /// Java: state.goToLabelOnFailure
    pub goto_label_on_failure: Option<String>,
    /// Java: AbstractStepWithReRoll.reRolledAction
    pub re_rolled_action: Option<String>,
    /// Java: AbstractStepWithReRoll.reRollSource
    pub re_roll_source: Option<String>,
}

impl StepBloodLust {
    pub fn new() -> Self {
        Self {
            status: BloodLustStatus::None,
            goto_label_on_failure: None,
            re_rolled_action: None,
            re_roll_source: None,
        }
    }
}

impl Default for StepBloodLust {
    fn default() -> Self { Self::new() }
}

impl Step for StepBloodLust {
    fn id(&self) -> StepId { StepId::BloodLust }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: commandStatus = super.handleCommand(pReceivedCommand)
        // AbstractStepWithReRoll handles UseReRoll by setting re_roll_source = None on decline
        if let Action::UseReRoll { use_reroll: false } = action {
            self.re_roll_source = None;
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(label) => {
                self.goto_label_on_failure = Some(label.clone());
                true
            }
            _ => false,
        }
    }
}

impl StepBloodLust {
    /// Java: `BloodLustBehaviour.handleExecuteStepHook(StepBloodLust, StepState)`.
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
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
            // Java: if ((getReRollSource() == null) || !UtilServerReRoll.useReRoll(...))
            //       → doRoll = false, status = FAILURE, setSufferingBloodLust(true)
            if let Some(ref source_str) = self.re_roll_source.clone() {
                let source = ReRollSource::new(source_str.as_str());
                if use_reroll(game, &source, &acting_id) {
                    do_roll = true;
                } else {
                    return self.fail_blood_lust(game);
                }
            } else {
                // re_roll_source == None → player declined
                return self.fail_blood_lust(game);
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
        let rerolled = re_rolled && self.re_roll_source.is_some();

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
            // No re-roll available or already re-rolled — failure
            let _ = rerolled; // Java uses this for reporting; event covers it
            return self.fail_blood_lust(game).with_event(event);
        }

        // Success
        self.status = BloodLustStatus::Success;
        StepOutcome::next().with_event(event)
    }

    /// Java: `status = FAILURE; setSufferingBloodLust(true); publishParameter(MOVE_STACK, null);
    ///        if (goToLabelOnFailure) GOTO_LABEL else NEXT_STEP`.
    fn fail_blood_lust(&mut self, game: &mut Game) -> StepOutcome {
        self.status = BloodLustStatus::Failure;
        game.acting_player.suffering_blood_lust = true;
        let base = StepOutcome::next().publish(StepParameter::MoveStack(vec![]));
        match self.goto_label_on_failure {
            Some(ref label) => StepOutcome::goto(label).publish(StepParameter::MoveStack(vec![])),
            None => base,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{test_team, StepAction};
    use ffb_model::enums::{Rules, TurnMode};
    use ffb_model::model::skill_def::SkillWithValue;

    fn add_player(
        team: &mut ffb_model::model::team::Team,
        id: &str,
        skills: Vec<SkillId>,
    ) {
        team.players.push(ffb_model::model::player::Player {
            id: id.into(),
            name: id.into(),
            nr: 1,
            position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6,
            strength: 3,
            agility: 3,
            passing: 4,
            armour: 8,
            starting_skills: skills
                .into_iter()
                .map(|s| SkillWithValue { skill_id: s, value: None })
                .collect(),
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
    }

    fn make_game(skills: Vec<SkillId>) -> Game {
        let mut home = test_team("home", 0);
        add_player(&mut home, "vamp", skills);
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2016);
        game.home_playing = true;
        game.acting_player.player_id = Some("vamp".into());
        game.turn_mode = TurnMode::Regular;
        game
    }

    fn seed_for_d6(target: i32) -> u64 {
        for s in 0u64..10_000 {
            if GameRng::new(s).d6() == target {
                return s;
            }
        }
        panic!("no seed for d6={}", target);
    }

    #[test]
    fn negatraits_disabled_skips_roll() {
        let mut game = make_game(vec![SkillId::BloodLust]);
        // KickoffReturn is one of the turn modes where check_negatraits() returns false
        game.turn_mode = TurnMode::KickoffReturn;
        let mut step = StepBloodLust::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!game.acting_player.suffering_blood_lust);
    }

    #[test]
    fn no_acting_player_returns_next() {
        let mut game = make_game(vec![SkillId::BloodLust]);
        game.acting_player.player_id = None;
        let out = StepBloodLust::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn no_blood_lust_skill_returns_next() {
        let mut game = make_game(vec![]);
        let out = StepBloodLust::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn already_used_blood_lust_returns_next() {
        let mut game = make_game(vec![SkillId::BloodLust]);
        game.player_mut("vamp").unwrap().used_skills.insert(SkillId::BloodLust);
        let out = StepBloodLust::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn successful_roll_returns_next_no_suffering() {
        // minimum_roll_blood_lust() == 2, so any roll >= 2 succeeds
        let seed = seed_for_d6(3);
        let mut game = make_game(vec![SkillId::BloodLust]);
        let out = StepBloodLust::new().start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!game.acting_player.suffering_blood_lust);
        assert!(out.events.iter().any(|e| matches!(e, GameEvent::BloodLustRoll { success: true, .. })));
    }

    #[test]
    fn successful_roll_marks_skill_used() {
        let seed = seed_for_d6(4);
        let mut game = make_game(vec![SkillId::BloodLust]);
        StepBloodLust::new().start(&mut game, &mut GameRng::new(seed));
        assert!(game.player("vamp").unwrap().used_skills.contains(&SkillId::BloodLust));
    }

    #[test]
    fn failed_roll_without_trr_sets_suffering() {
        // Roll 1 is always a failure (1 < 2)
        let seed = seed_for_d6(1);
        let mut game = make_game(vec![SkillId::BloodLust]);
        // No TRR available
        let out = StepBloodLust::new().start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(game.acting_player.suffering_blood_lust);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::MoveStack(_))));
    }

    #[test]
    fn failed_roll_with_trr_offers_reroll() {
        let seed = seed_for_d6(1);
        let mut game = make_game(vec![SkillId::BloodLust]);
        game.turn_data_home.rerolls = 1;
        let mut step = StepBloodLust::new();
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::Continue);
        assert!(out.prompt.is_some());
        assert_eq!(step.re_rolled_action.as_deref(), Some("BLOOD_LUST"));
        assert!(!game.acting_player.suffering_blood_lust);
    }

    #[test]
    fn failed_roll_with_trr_then_decline_sets_suffering() {
        let seed = seed_for_d6(1);
        let mut game = make_game(vec![SkillId::BloodLust]);
        game.turn_data_home.rerolls = 1;
        let mut step = StepBloodLust::new();
        step.start(&mut game, &mut GameRng::new(seed));
        let out = step.handle_command(&Action::UseReRoll { use_reroll: false }, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(game.acting_player.suffering_blood_lust);
    }

    #[test]
    fn goto_label_on_failure_is_used() {
        let seed = seed_for_d6(1);
        let mut game = make_game(vec![SkillId::BloodLust]);
        let mut step = StepBloodLust::new();
        step.goto_label_on_failure = Some("feed_label".into());
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("feed_label"));
    }

    #[test]
    fn set_parameter_goto_label_on_failure() {
        let mut step = StepBloodLust::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("x".to_string())));
        assert_eq!(step.goto_label_on_failure.as_deref(), Some("x"));
    }

    #[test]
    fn step_id_is_blood_lust() {
        assert_eq!(StepBloodLust::new().id(), StepId::BloodLust);
    }
}
