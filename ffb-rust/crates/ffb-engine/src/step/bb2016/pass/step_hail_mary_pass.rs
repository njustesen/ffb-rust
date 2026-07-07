/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.pass.StepHailMaryPass`.
///
/// Step in the pass sequence to handle skill HAIL_MARY_PASS.
/// The roll logic is inlined from `PassBehaviour.handleExecuteStepHook` (BB2016).
///
/// Always results in FUMBLE (roll=1) or INACCURATE (roll>1).
/// On FUMBLE:
///   - Sets ball/bomb to thrower position.
///   - Offers Pass skill re-roll (if available and not used), then TRR.
///   - On no re-roll or declined re-roll: publishes PASS_FUMBLE=true, ScatterBall, goto failure.
/// On INACCURATE:
///   - Sets ball to pass_coordinate (or bomb to thrower for HAIL_MARY_BOMB).
///   - Publishes PASS_FUMBLE=false, NEXT_STEP.
///
/// Init parameter: GOTO_LABEL_ON_FAILURE (mandatory).
/// Sets stepParameter PASS_FUMBLE and CATCH_SCATTER_THROW_IN_MODE for all steps on the stack.
use ffb_model::enums::{PassResult, PlayerAction, ReRollSource, SkillId};
use ffb_model::events::GameEvent;
use ffb_model::enums::PassingDistance;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::prompts::AgentPrompt;
use crate::action::Action;
use crate::step::framework::{CatchScatterThrowInMode, Step, StepOutcome, StepId, StepParameter};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

/// Java: `StepHailMaryPass` (bb2016/pass).
pub struct StepHailMaryPass {
    /// Java: `state.goToLabelOnFailure` — mandatory init param.
    pub goto_label_on_failure: String,
    /// Java: `state.result` — FUMBLE or INACCURATE, persisted across re-roll wait.
    pub result: Option<PassResult>,
    /// Java: `state.passSkillUsed` — true after Pass skill dialog was shown.
    pub pass_skill_used: bool,
    /// Java: AbstractStepWithReRoll.reRolledAction — "PASS" when re-roll is pending.
    pub re_rolled_action: Option<String>,
    /// Java: AbstractStepWithReRoll.reRollSource — "Pass" (skill) or "TRR" or None (declined).
    pub re_roll_source: Option<String>,
}

impl StepHailMaryPass {
    pub fn new() -> Self {
        Self {
            goto_label_on_failure: String::new(),
            result: None,
            pass_skill_used: false,
            re_rolled_action: None,
            re_roll_source: None,
        }
    }
}

impl Default for StepHailMaryPass {
    fn default() -> Self { Self::new() }
}

impl Step for StepHailMaryPass {
    fn id(&self) -> StepId { StepId::HailMaryPass }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: handleCommandHook sets reRolledAction=PASS and reRollSource=PASS or null.
        // Java: AbstractStepWithReRoll.handleCommand sets reRollSource=null on "decline".
        match action {
            Action::UseSkill { use_skill, .. } => {
                // Pass skill use dialog response
                if !use_skill {
                    self.re_roll_source = None; // declined Pass skill
                }
                // If use_skill=true, re_roll_source stays as "Pass" (pre-set)
            }
            Action::UseReRoll { use_reroll: false } => {
                self.re_roll_source = None; // declined TRR
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(s) => { self.goto_label_on_failure = s.clone(); true }
            StepParameter::PassResultParam(r)    => { self.result = Some(*r); true }
            _ => false,
        }
    }
}

impl StepHailMaryPass {
    /// Java: `PassBehaviour.handleExecuteStepHook` for `StepHailMaryPass`.
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let (thrower_id, thrower_action) = match (game.thrower_id.clone(), game.thrower_action) {
            (Some(id), Some(a)) => (id, a),
            _ => return StepOutcome::next(),
        };

        let is_bomb = thrower_action == PlayerAction::HailMaryBomb;
        if is_bomb {
            game.field_model.bomb_moving = true;
        } else {
            game.field_model.ball_moving = true;
        }

        let re_rolled = self.re_rolled_action.as_deref() == Some("PASS");
        let do_roll;
        let mut do_next_step = false;

        if re_rolled {
            // Java: if (source == null || !useReRoll(...)) → doRoll=false, doNextStep=true
            if let Some(ref source_str) = self.re_roll_source.clone() {
                let source = ReRollSource::new(source_str.as_str());
                if use_reroll(game, &source, &thrower_id) {
                    do_roll = true;
                } else {
                    do_roll = false;
                    do_next_step = true;
                }
            } else {
                do_roll = false;
                do_next_step = true;
            }
        } else {
            do_roll = true;
        }

        if do_roll {
            let roll = rng.d6();
            let result = if roll == 1 { PassResult::Fumble } else { PassResult::Inaccurate };
            self.result = Some(result);
            let rerolled = re_rolled && self.re_roll_source.is_some();

            let event = GameEvent::PassRoll {
                player_id: thrower_id.clone(),
                target: 2,
                distance: PassingDistance::LongBomb,
                roll,
                result,
                rerolled,
            };

            do_next_step = true;

            if result == PassResult::Fumble && !re_rolled {
                // Java: if (reRolledAction != PASS) → offer re-roll
                self.re_rolled_action = Some("PASS".into());

                // Try Pass skill dialog first
                let has_pass_unused = game.player(&thrower_id)
                    .map(|p| p.has_skill(SkillId::Pass) && !p.used_skills.contains(&SkillId::Pass))
                    .unwrap_or(false);

                if has_pass_unused && !self.pass_skill_used {
                    self.pass_skill_used = true;
                    self.re_roll_source = Some("Pass".into()); // pre-set, cleared on decline
                    let prompt = AgentPrompt::SkillUse {
                        player_id: thrower_id.clone(),
                        skill_id: SkillId::Pass as u16,
                        skill_name: "Pass".into(),
                    };
                    return StepOutcome::cont().with_event(event).with_prompt(prompt);
                }

                // Try TRR if no Pass skill
                if let Some(prompt) = ask_for_reroll_if_available(game, "PASS", 2, false) {
                    self.re_roll_source = Some("TRR".into());
                    return StepOutcome::cont().with_event(event).with_prompt(prompt);
                }
            }

            if do_next_step {
                return self.proceed(game, &thrower_id, is_bomb).with_event(event);
            }
        }

        if do_next_step {
            self.proceed(game, &thrower_id, is_bomb)
        } else {
            StepOutcome::next()
        }
    }

    /// Publish final outcome after roll is settled (FUMBLE or INACCURATE).
    fn proceed(&self, game: &mut Game, thrower_id: &str, is_bomb: bool) -> StepOutcome {
        let result = self.result.unwrap_or(PassResult::Inaccurate);
        let thrower_coord = game.field_model.player_coordinate(thrower_id);

        if result == PassResult::Fumble {
            if is_bomb {
                game.field_model.bomb_coordinate = thrower_coord;
            } else {
                game.field_model.ball_coordinate = thrower_coord;
            }
            let mut out = StepOutcome::goto(&self.goto_label_on_failure)
                .publish(StepParameter::PassFumble(true));
            if !is_bomb {
                out = out.publish(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ScatterBall));
            }
            out
        } else {
            // INACCURATE
            if is_bomb {
                game.field_model.bomb_coordinate = thrower_coord;
                game.field_model.bomb_moving = false;
            } else {
                game.field_model.ball_coordinate = game.pass_coordinate;
            }
            StepOutcome::next().publish(StepParameter::PassFumble(false))
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{PassResult, PlayerType, PlayerGender, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::types::FieldCoordinate;
    use std::collections::HashSet;

    fn make_thrower_game(thrower_action: PlayerAction, skills: Vec<SkillId>) -> (Game, String) {
        let tid = "t1".to_string();
        let mut home = test_team("home", 0);
        home.players.push(Player {
            id: tid.clone(), name: "T".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 4, passing: 3, armour: 8,
            starting_skills: skills.into_iter().map(|s| SkillWithValue { skill_id: s, value: None }).collect(),
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
                    ..Default::default()
});
        let mut game = Game::new(home, test_team("away", 0), Rules::Bb2016);
        game.home_playing = true;
        game.thrower_id = Some(tid.clone());
        game.thrower_action = Some(thrower_action);
        game.field_model.set_player_coordinate(&tid, FieldCoordinate::new(5, 5));
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));
        (game, tid)
    }

    fn seed_for_d6(target: i32) -> u64 {
        for s in 0u64..10_000 {
            if GameRng::new(s).d6() == target { return s; }
        }
        panic!("no seed for d6={}", target);
    }

    #[test]
    fn no_thrower_returns_next() {
        let mut game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016);
        let out = StepHailMaryPass::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn id_is_hail_mary_pass() {
        assert_eq!(StepHailMaryPass::new().id(), StepId::HailMaryPass);
    }

    #[test]
    fn set_parameter_goto_label_on_failure() {
        let mut step = StepHailMaryPass::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("fail".into())));
        assert_eq!(step.goto_label_on_failure, "fail");
    }

    #[test]
    fn set_parameter_pass_result() {
        let mut step = StepHailMaryPass::new();
        assert!(step.set_parameter(&StepParameter::PassResultParam(PassResult::Fumble)));
        assert_eq!(step.result, Some(PassResult::Fumble));
    }

    #[test]
    fn roll_2_to_6_is_inaccurate_next_step() {
        let seed = seed_for_d6(3);
        let (mut game, _) = make_thrower_game(PlayerAction::HailMaryPass, vec![]);
        let mut step = StepHailMaryPass::new();
        step.goto_label_on_failure = "fail".into();
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(step.result, Some(PassResult::Inaccurate));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::PassFumble(false))));
    }

    #[test]
    fn inaccurate_sets_ball_to_pass_coordinate() {
        let seed = seed_for_d6(3);
        let (mut game, _) = make_thrower_game(PlayerAction::HailMaryPass, vec![]);
        let pass_coord = FieldCoordinate::new(10, 5);
        game.pass_coordinate = Some(pass_coord);
        let mut step = StepHailMaryPass::new();
        step.goto_label_on_failure = "fail".into();
        step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(game.field_model.ball_coordinate, Some(pass_coord));
    }

    #[test]
    fn inaccurate_sets_ball_moving() {
        let seed = seed_for_d6(3);
        let (mut game, _) = make_thrower_game(PlayerAction::HailMaryPass, vec![]);
        let mut step = StepHailMaryPass::new();
        step.goto_label_on_failure = "fail".into();
        step.start(&mut game, &mut GameRng::new(seed));
        assert!(game.field_model.ball_moving);
    }

    #[test]
    fn roll_1_is_fumble_goto_failure() {
        let seed = seed_for_d6(1);
        let (mut game, _) = make_thrower_game(PlayerAction::HailMaryPass, vec![]);
        let mut step = StepHailMaryPass::new();
        step.goto_label_on_failure = "fumble_label".into();
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fumble_label"));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::PassFumble(true))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ScatterBall))));
    }

    #[test]
    fn fumble_sets_ball_to_thrower_coord() {
        let seed = seed_for_d6(1);
        let (mut game, _) = make_thrower_game(PlayerAction::HailMaryPass, vec![]);
        let thrower_coord = FieldCoordinate::new(5, 5);
        let mut step = StepHailMaryPass::new();
        step.goto_label_on_failure = "fail".into();
        step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(game.field_model.ball_coordinate, Some(thrower_coord));
    }

    #[test]
    fn fumble_without_pass_skill_without_trr_goto_immediately() {
        let seed = seed_for_d6(1);
        let (mut game, _) = make_thrower_game(PlayerAction::HailMaryPass, vec![]);
        let mut step = StepHailMaryPass::new();
        step.goto_label_on_failure = "fail".into();
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn fumble_with_pass_skill_offers_skill_dialog() {
        let seed = seed_for_d6(1);
        let (mut game, _) = make_thrower_game(PlayerAction::HailMaryPass, vec![SkillId::Pass]);
        let mut step = StepHailMaryPass::new();
        step.goto_label_on_failure = "fail".into();
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::Continue);
        assert!(matches!(out.prompt, Some(AgentPrompt::SkillUse { .. })));
        assert_eq!(step.re_rolled_action.as_deref(), Some("PASS"));
    }

    #[test]
    fn fumble_decline_pass_skill_goto_failure() {
        let seed = seed_for_d6(1);
        let (mut game, _) = make_thrower_game(PlayerAction::HailMaryPass, vec![SkillId::Pass]);
        let mut step = StepHailMaryPass::new();
        step.goto_label_on_failure = "fail".into();
        step.start(&mut game, &mut GameRng::new(seed)); // triggers skill dialog
        let out = step.handle_command(
            &Action::UseSkill { skill_id: SkillId::Pass, use_skill: false },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    #[test]
    fn fumble_accept_pass_skill_re_rolls() {
        let seed = seed_for_d6(1);
        let reroll_seed = seed_for_d6(4); // success on re-roll
        let (mut game, _) = make_thrower_game(PlayerAction::HailMaryPass, vec![SkillId::Pass]);
        let mut step = StepHailMaryPass::new();
        step.goto_label_on_failure = "fail".into();
        step.start(&mut game, &mut GameRng::new(seed)); // triggers skill dialog
        let out = step.handle_command(
            &Action::UseSkill { skill_id: SkillId::Pass, use_skill: true },
            &mut game,
            &mut GameRng::new(reroll_seed),
        );
        // Re-roll succeeds → INACCURATE → NEXT_STEP
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(step.result, Some(PassResult::Inaccurate));
    }

    #[test]
    fn fumble_with_trr_offers_reroll_prompt() {
        let seed = seed_for_d6(1);
        let (mut game, _) = make_thrower_game(PlayerAction::HailMaryPass, vec![]);
        game.turn_data_home.rerolls = 1;
        let mut step = StepHailMaryPass::new();
        step.goto_label_on_failure = "fail".into();
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::Continue);
        assert!(out.prompt.is_some());
        assert_eq!(step.re_rolled_action.as_deref(), Some("PASS"));
        assert_eq!(step.re_roll_source.as_deref(), Some("TRR"));
    }

    #[test]
    fn fumble_decline_trr_goto_failure() {
        let seed = seed_for_d6(1);
        let (mut game, _) = make_thrower_game(PlayerAction::HailMaryPass, vec![]);
        game.turn_data_home.rerolls = 1;
        let mut step = StepHailMaryPass::new();
        step.goto_label_on_failure = "fail".into();
        step.start(&mut game, &mut GameRng::new(seed)); // offers TRR
        let out = step.handle_command(
            &Action::UseReRoll { use_reroll: false },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    #[test]
    fn hail_mary_bomb_does_not_set_ball_moving() {
        let seed = seed_for_d6(3); // inaccurate
        let (mut game, _) = make_thrower_game(PlayerAction::HailMaryBomb, vec![]);
        let mut step = StepHailMaryPass::new();
        step.goto_label_on_failure = "fail".into();
        step.start(&mut game, &mut GameRng::new(seed));
        assert!(!game.field_model.ball_moving);
    }

    #[test]
    fn hail_mary_bomb_inaccurate_sets_bomb_coord_and_clears_moving() {
        let seed = seed_for_d6(3); // inaccurate
        let (mut game, tid) = make_thrower_game(PlayerAction::HailMaryBomb, vec![]);
        let thrower_coord = FieldCoordinate::new(5, 5);
        let mut step = StepHailMaryPass::new();
        step.goto_label_on_failure = "fail".into();
        step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(game.field_model.bomb_coordinate, Some(thrower_coord));
        assert!(!game.field_model.bomb_moving);
        let _ = tid;
    }

    #[test]
    fn hail_mary_bomb_fumble_no_scatter_ball_published() {
        let seed = seed_for_d6(1);
        let (mut game, _) = make_thrower_game(PlayerAction::HailMaryBomb, vec![]);
        let mut step = StepHailMaryPass::new();
        step.goto_label_on_failure = "fail".into();
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::GotoLabel);
        // For bomb, no ScatterBall published
        assert!(!out.published.iter().any(|p| matches!(p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ScatterBall))));
    }

    #[test]
    fn pass_skill_not_offered_when_already_used() {
        let seed = seed_for_d6(1);
        let (mut game, _) = make_thrower_game(PlayerAction::HailMaryPass, vec![SkillId::Pass]);
        let mut step = StepHailMaryPass::new();
        step.goto_label_on_failure = "fail".into();
        step.pass_skill_used = true; // already used
        let out = step.start(&mut game, &mut GameRng::new(seed));
        // No Pass skill dialog, should go straight to failure (no TRR either)
        assert_eq!(out.action, StepAction::GotoLabel);
    }
}
