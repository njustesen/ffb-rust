use ffb_model::enums::{PlayerAction, ReRollSource, SkillId, PS_PRONE, PS_STANDING};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.StepWildAnimal`.
///
/// Resolves the Wild Animal negatrait check.
/// Core logic from Java's `WildAnimalBehaviour.handleExecuteStepHook`, inlined here.
///
/// Init params: GOTO_LABEL_ON_FAILURE (mandatory).
pub struct StepWildAnimal {
    /// Java: state.goToLabelOnFailure
    pub goto_label_on_failure: String,
    /// Java: AbstractStepWithReRoll.reRolledAction
    pub re_rolled_action: Option<String>,
    /// Java: AbstractStepWithReRoll.reRollSource
    pub re_roll_source: Option<String>,
}

impl StepWildAnimal {
    pub fn new(goto_label_on_failure: String) -> Self {
        Self { goto_label_on_failure, re_rolled_action: None, re_roll_source: None }
    }
}

impl Default for StepWildAnimal {
    fn default() -> Self { Self::new(String::new()) }
}

impl Step for StepWildAnimal {
    fn id(&self) -> StepId { StepId::WildAnimal }

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

impl StepWildAnimal {
    /// Java: `WildAnimalBehaviour.handleExecuteStepHook(StepWildAnimal, StepState)`.
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: if (!game.getTurnMode().checkNegatraits()) → NEXT_STEP
        if !game.turn_mode.check_negatraits() {
            return StepOutcome::next();
        }

        let acting_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        // Java: recover tacklezones at start of wild animal check
        if let Some(state) = game.field_model.player_state(&acting_id) {
            let recovered = state.recover_tacklezones();
            game.field_model.set_player_state(&acting_id, recovered);
        }

        let has_wild_animal = game.player(&acting_id)
            .map(|p| p.has_skill(SkillId::WildAnimal))
            .unwrap_or(false);
        if !has_wild_animal {
            return StepOutcome::next();
        }

        let re_rolled = self.re_rolled_action.as_deref() == Some("WILD_ANIMAL");
        let do_roll;

        if re_rolled {
            if let Some(ref source_str) = self.re_roll_source.clone() {
                let source = ReRollSource::new(source_str.as_str());
                if use_reroll(game, &source, &acting_id) {
                    do_roll = true;
                } else {
                    return self.cancel_action(game);
                }
            } else {
                return self.cancel_action(game);
            }
        } else {
            // Java: doRoll = UtilCards.hasUnusedSkill(actingPlayer, WildAnimal)
            do_roll = game.player(&acting_id)
                .map(|p| !p.used_skills.contains(&SkillId::WildAnimal))
                .unwrap_or(false);
        }

        if !do_roll {
            return StepOutcome::next();
        }

        // Java: goodConditions = BLITZ/BLOCK type action
        let good_conditions = matches!(
            game.acting_player.player_action,
            Some(PlayerAction::BlitzMove)
            | Some(PlayerAction::Blitz)
            | Some(PlayerAction::Block)
            | Some(PlayerAction::MultipleBlock)
            | Some(PlayerAction::StandUpBlitz)
        );

        let roll = rng.d6();
        let min_roll = DiceInterpreter::minimum_roll_confusion(good_conditions);
        let successful = roll >= min_roll;

        // Java: actingPlayer.markSkillUsed(WildAnimal)
        if let Some(player) = game.player_mut(&acting_id) {
            player.used_skills.insert(SkillId::WildAnimal);
        }

        let event = GameEvent::ConfusionRoll {
            player_id: acting_id.clone(),
            roll,
            confused: !successful,
        };

        if !successful {
            if !re_rolled {
                if let Some(prompt) = ask_for_reroll_if_available(game, "WILD_ANIMAL", min_roll, false) {
                    self.re_rolled_action = Some("WILD_ANIMAL".into());
                    self.re_roll_source = Some("TRR".into());
                    return StepOutcome::cont().with_event(event).with_prompt(prompt);
                }
            }
            return self.cancel_action(game).with_event(event);
        }

        StepOutcome::next().with_event(event)
    }

    /// Java: `cancelPlayerAction(step)` — mark turn resources used, deactivate player.
    fn cancel_action(&mut self, game: &mut Game) -> StepOutcome {
        let player_action = game.acting_player.player_action;
        let td = game.turn_data_mut();
        match player_action {
            Some(PlayerAction::Blitz) | Some(PlayerAction::BlitzMove)
            | Some(PlayerAction::KickTeamMate) | Some(PlayerAction::KickTeamMateMove) => {
                td.blitz_used = true;
            }
            Some(PlayerAction::Pass) | Some(PlayerAction::PassMove)
            | Some(PlayerAction::ThrowTeamMate) | Some(PlayerAction::ThrowTeamMateMove) => {
                td.pass_used = true;
            }
            Some(PlayerAction::HandOver) | Some(PlayerAction::HandOverMove) => {
                td.hand_over_used = true;
            }
            Some(PlayerAction::Foul) | Some(PlayerAction::FoulMove) => {
                td.foul_used = true;
            }
            _ => {}
        }

        // Java: playerState.changeBase(PRONE if standing_up else STANDING).changeActive(false)
        if let Some(acting_id) = game.acting_player.player_id.clone() {
            if let Some(state) = game.field_model.player_state(&acting_id) {
                let new_base = if game.acting_player.standing_up { PS_PRONE } else { PS_STANDING };
                let new_state = state.change_base(new_base).change_active(false);
                game.field_model.set_player_state(&acting_id, new_state);
            }
        }

        // Java: game.setPassCoordinate(null)
        game.pass_coordinate = None;

        StepOutcome::goto(&self.goto_label_on_failure)
            .publish(StepParameter::EndPlayerAction(true))
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
            ..Default::default()
        });
    }

    fn make_game(skills: Vec<SkillId>, action: Option<PlayerAction>) -> Game {
        let mut home = test_team("home", 0);
        add_player(&mut home, "beast", skills);
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2016);
        game.home_playing = true;
        game.acting_player.player_id = Some("beast".into());
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
        let mut game = make_game(vec![SkillId::WildAnimal], Some(PlayerAction::Block));
        game.turn_mode = TurnMode::KickoffReturn;
        let out = StepWildAnimal::new("fail".into()).start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn no_wild_animal_skill_returns_next() {
        let mut game = make_game(vec![], Some(PlayerAction::Block));
        let out = StepWildAnimal::new("fail".into()).start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn good_conditions_block_min_roll_2() {
        // In good conditions (BLOCK action), min roll = 2, so roll 1 fails
        let seed = seed_for_d6(1);
        let mut game = make_game(vec![SkillId::WildAnimal], Some(PlayerAction::Block));
        let out = StepWildAnimal::new("fail".into()).start(&mut game, &mut GameRng::new(seed));
        // Roll 1 < 2 → failure
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    #[test]
    fn good_conditions_roll_2_succeeds() {
        let seed = seed_for_d6(2);
        let mut game = make_game(vec![SkillId::WildAnimal], Some(PlayerAction::Block));
        let out = StepWildAnimal::new("fail".into()).start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn bad_conditions_roll_3_fails() {
        // Bad conditions (non-BLITZ/BLOCK), min roll = 4
        let seed = seed_for_d6(3);
        let mut game = make_game(vec![SkillId::WildAnimal], Some(PlayerAction::Move));
        let out = StepWildAnimal::new("fail".into()).start(&mut game, &mut GameRng::new(seed));
        // Roll 3 < 4 → failure
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn bad_conditions_roll_4_succeeds() {
        let seed = seed_for_d6(4);
        let mut game = make_game(vec![SkillId::WildAnimal], Some(PlayerAction::Move));
        let out = StepWildAnimal::new("fail".into()).start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn cancel_action_sets_blitz_used_for_blitz_action() {
        let seed = seed_for_d6(1); // roll 1, min_roll=2 for BLITZ (good cond) → fail
        let mut game = make_game(vec![SkillId::WildAnimal], Some(PlayerAction::Blitz));
        game.turn_data_home.blitz_used = false;
        let out = StepWildAnimal::new("fail".into()).start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(game.turn_data().blitz_used);
    }

    #[test]
    fn cancel_action_sets_pass_used_for_pass_action() {
        let seed = seed_for_d6(3); // roll 3, min_roll=4 for PASS (bad cond) → fail
        let mut game = make_game(vec![SkillId::WildAnimal], Some(PlayerAction::Pass));
        let out = StepWildAnimal::new("fail".into()).start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(game.turn_data().pass_used);
    }

    #[test]
    fn failed_roll_with_trr_offers_reroll() {
        let seed = seed_for_d6(1); // definitely fails good_conditions min 2
        let mut game = make_game(vec![SkillId::WildAnimal], Some(PlayerAction::Block));
        game.turn_data_home.rerolls = 1;
        let mut step = StepWildAnimal::new("fail".into());
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::Continue);
        assert!(out.prompt.is_some());
    }

    #[test]
    fn marks_skill_used_on_roll() {
        let seed = seed_for_d6(5); // success in good or bad conditions
        let mut game = make_game(vec![SkillId::WildAnimal], Some(PlayerAction::Block));
        StepWildAnimal::new("fail".into()).start(&mut game, &mut GameRng::new(seed));
        assert!(game.player("beast").unwrap().used_skills.contains(&SkillId::WildAnimal));
    }

    #[test]
    fn set_parameter_goto_label_on_failure() {
        let mut step = StepWildAnimal::new("old".into());
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("new".into())));
        assert_eq!(step.goto_label_on_failure, "new");
    }
}
