/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.pass.StepPass`.
///
/// Step in the pass sequence to handle passing the ball (BB2016).
/// - Rolls pass using PassMechanic; evaluates result (ACCURATE/INACCURATE/FUMBLE/SAVED_FUMBLE).
/// - On ACCURATE: set ball/bomb coordinate, publish PASS_ACCURATE or CATCH_SCATTER mode.
/// - On FUMBLE: scatter ball.
/// - On SAVED_FUMBLE: ball stays with thrower (Safe Throw handled it).
/// - On INACCURATE/WILDLY_INACCURATE: goto missed-pass label.
/// - Re-roll (PASS) and skill auto-reroll supported.
///
/// Init parameters: GOTO_LABEL_ON_END (mandatory), GOTO_LABEL_ON_MISSED_PASS (mandatory).
/// Receives: CATCHER_ID.
/// Publishes: CATCHER_ID, PASS_ACCURATE, PASS_FUMBLE, DONT_DROP_FUMBLE,
///            CATCH_SCATTER_THROW_IN_MODE, PASS_DEVIATES.
///
/// client-only: DialogSkillUseParameter for passing skill re-roll — dialog is client-only.
/// NOTE(Pass-modifiers): PassModifierFactory wired; individual modifier reporting deferred to reporting layer.
use ffb_model::enums::{PassOutcome as ModelPassResult, PlayerAction, ReRollSource};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::report::mixed::report_pass_roll::ReportPassRoll;
use ffb_model::report::report_id::ReportId;
use ffb_mechanics::bb2016::pass_mechanic::PassMechanic;
use ffb_mechanics::modifiers::modifier_type::ModifierType;
use ffb_mechanics::modifiers::pass_context::PassContext;
use ffb_mechanics::modifiers::pass_modifier::PassModifier;
use ffb_mechanics::modifiers::pass_modifier_factory::PassModifierFactory;
use ffb_mechanics::pass_mechanic::PassMechanic as PassMechanicTrait;
use ffb_mechanics::pass_result::PassResult;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter, CatchScatterThrowInMode};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

/// Java: `StepPass` (bb2016/pass).
pub struct StepPass {
    /// Java: `state.goToLabelOnEnd`
    goto_label_on_end: String,
    /// Java: `state.goToLabelOnMissedPass`
    goto_label_on_missed_pass: String,
    /// Java: `state.CatcherId`
    catcher_id: Option<String>,
    /// Java: `state.passSkillUsed`
    pass_skill_used: bool,
    /// Java: `state.result` — mechanics PassResult from evaluatePass
    mech_result: Option<PassResult>,
    /// Model-level PassOutcome set via StepParameter (for test/replay).
    result: Option<ModelPassResult>,
    /// Java: fReRolledAction — set when a re-roll is in progress.
    re_rolled_action: Option<String>,
    /// Java: fReRollSource — the re-roll source name.
    re_roll_source: Option<String>,
    /// Java: minimumRoll — stored for re-roll prompt.
    minimum_roll: i32,
}

impl StepPass {
    pub fn new() -> Self {
        Self {
            goto_label_on_end: String::new(),
            goto_label_on_missed_pass: String::new(),
            catcher_id: None,
            pass_skill_used: false,
            mech_result: None,
            result: None,
            re_rolled_action: None,
            re_roll_source: None,
            minimum_roll: 0,
        }
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let (thrower_id, thrower_action) = match (game.thrower_id.clone(), game.thrower_action) {
            (Some(id), Some(action)) => (id, action),
            _ => return StepOutcome::next(),
        };
        let is_bomb = thrower_action == PlayerAction::ThrowBomb;
        if is_bomb {
            game.field_model.bomb_moving = true;
            if game.original_bombardier.is_none() {
                game.original_bombardier = game.thrower_id.clone();
            }
        } else {
            game.field_model.ball_moving = true;
        }

        let thrower = match game.player(&thrower_id) {
            Some(p) => p.clone(),
            None => return StepOutcome::next(),
        };

        // Java: if (ReRolledActions.PASS == getReRolledAction()) → useReRoll or handleFailedPass
        // Note: Java does NOT republish DONT_DROP_FUMBLE here — that publish only happens
        // once, immediately after a fresh evaluatePass() roll (see below).
        if self.re_rolled_action.as_deref() == Some("PASS") {
            if let Some(ref source_name) = self.re_roll_source.clone() {
                let source = ReRollSource::new(source_name.as_str());
                if !use_reroll(game, &source, &thrower_id) {
                    let thrower_coord = game.field_model.player_coordinate(&thrower_id);
                    let pass_coord = game.pass_coordinate;
                    return self.handle_failed_pass(game, is_bomb, thrower_coord, pass_coord, false);
                }
            } else {
                let thrower_coord = game.field_model.player_coordinate(&thrower_id);
                let pass_coord = game.pass_coordinate;
                return self.handle_failed_pass(game, is_bomb, thrower_coord, pass_coord, false);
            }
        }

        let mechanic = PassMechanic::new();
        let thrower_coord = game.field_model.player_coordinate(&thrower_id);
        let pass_coord = game.pass_coordinate;
        let passing_distance = match mechanic.find_passing_distance(game, thrower_coord, pass_coord, false) {
            Some(d) => d,
            None => return StepOutcome::next(),
        };

        let factory = PassModifierFactory::for_rules(game.rules);
        let ctx = PassContext::new(game, &thrower, passing_distance, false);
        let collection_total: i32 = factory.find_modifiers(&ctx).iter().map(|m| m.get_modifier()).sum();
        let skill_total: i32 = factory.find_skill_modifiers(&ctx).iter().map(|m| m.get_modifier()).sum();
        let card_total: i32 = factory.find_card_modifiers(&ctx).iter().map(|m| m.get_modifier()).sum();
        let modifier_total: i32 = collection_total + skill_total + card_total;
        let modifiers_vec: Vec<PassModifier> = if modifier_total != 0 {
            vec![PassModifier::new("pass_mods", modifier_total, ModifierType::REGULAR)]
        } else {
            vec![]
        };
        self.minimum_roll = mechanic.minimum_roll_simple(&thrower, passing_distance, &modifiers_vec).unwrap_or(0);
        let roll = rng.d6();
        let result = mechanic.evaluate_pass_simple(&thrower, roll, passing_distance, &modifiers_vec, is_bomb);
        self.mech_result = Some(result);

        // Java: getResult().addReport(new ReportPassRoll(...))
        let successful = result == PassResult::ACCURATE || result == PassResult::SAVED_FUMBLE;
        let re_rolled = self.re_rolled_action.is_some();
        let modifier_names: Vec<String> = modifiers_vec.iter().map(|m| m.get_name().to_string()).collect();
        game.report_list.add(ReportPassRoll::new(
            Some(thrower_id.clone()),
            successful,
            roll,
            self.minimum_roll,
            re_rolled,
            modifier_names,
            None,
            is_bomb,
            Some(format!("{:?}", result)),
            false,
            None,
        ));

        if result == PassResult::ACCURATE {
            game.field_model.range_ruler = None;
            let pass_coordinate = match pass_coord {
                Some(c) => c,
                None => return StepOutcome::next(),
            };
            let catcher_has_tacklezones = self.catcher_id.as_deref()
                .and_then(|id| game.field_model.player_state(id))
                .map(|s| s.has_tacklezones())
                .unwrap_or(false);
            if is_bomb {
                game.field_model.bomb_coordinate = Some(pass_coordinate);
                let mode = if catcher_has_tacklezones {
                    CatchScatterThrowInMode::CatchAccurateBomb
                } else if self.catcher_id.is_some() {
                    CatchScatterThrowInMode::CatchBomb
                } else {
                    CatchScatterThrowInMode::CatchAccurateBombEmptySquare
                };
                StepOutcome::next()
                    .publish(StepParameter::PassFumble(false))
                    .publish(StepParameter::CatchScatterThrowInMode(mode))
            } else {
                game.field_model.ball_coordinate = Some(pass_coordinate);
                let mode = if catcher_has_tacklezones {
                    CatchScatterThrowInMode::CatchAccuratePass
                } else if self.catcher_id.is_some() {
                    CatchScatterThrowInMode::CatchMissedPass
                } else {
                    CatchScatterThrowInMode::CatchAccuratePassEmptySquare
                };
                let mut out = StepOutcome::next()
                    .publish(StepParameter::PassFumble(false))
                    .publish(StepParameter::CatchScatterThrowInMode(mode));
                if catcher_has_tacklezones {
                    out = out.publish(StepParameter::PassAccurate(true));
                }
                out
            }
        } else {
            let dont_drop = result == PassResult::SAVED_FUMBLE;
            let is_fumble = result == PassResult::FUMBLE;
            // Java: publishParameter(DONT_DROP_FUMBLE, ...) happens immediately after
            // evaluatePass() — only for FUMBLE (false) / SAVED_FUMBLE (true) — and
            // unconditionally, i.e. even if a re-roll dialog is about to be shown below.
            // It must NOT be published for INACCURATE/WILDLY_INACCURATE.
            let dont_drop_fumble_param = if is_fumble {
                Some(StepParameter::DontDropFumble(false))
            } else if dont_drop {
                Some(StepParameter::DontDropFumble(true))
            } else {
                None
            };
            if !dont_drop && self.re_rolled_action.is_none() {
                // Java: mechanic.eligibleToReRoll → askForReRollIfAvailable
                // client-only: DialogSkillUseParameter for pass skill re-roll — headless uses auto re-roll logic
                if let Some(prompt) = ask_for_reroll_if_available(game, "PASS", self.minimum_roll, is_fumble) {
                    self.re_rolled_action = Some("PASS".into());
                    self.re_roll_source = Some("TRR".into());
                    let mut out = StepOutcome::cont().with_prompt(prompt);
                    if let Some(p) = dont_drop_fumble_param {
                        out = out.publish(p);
                    }
                    return out;
                }
            }
            let mut out = self.handle_failed_pass(game, is_bomb, thrower_coord, pass_coord, dont_drop);
            if let Some(p) = dont_drop_fumble_param {
                out = out.publish(p);
            }
            out
        }
    }

    fn handle_failed_pass(
        &mut self,
        game: &mut Game,
        is_bomb: bool,
        thrower_coord: Option<ffb_model::types::FieldCoordinate>,
        pass_coord: Option<ffb_model::types::FieldCoordinate>,
        saved_fumble: bool,
    ) -> StepOutcome {
        game.field_model.range_ruler = None;
        let result = self.mech_result.unwrap_or(PassResult::FUMBLE);
        let is_fumble = result == PassResult::FUMBLE;
        let is_wildly_inaccurate = result == PassResult::WILDLY_INACCURATE;
        if saved_fumble {
            if is_bomb {
                game.field_model.bomb_coordinate = None;
                game.field_model.bomb_moving = false;
                return StepOutcome::goto(&self.goto_label_on_end)
                    .publish(StepParameter::CatcherId(None))
                    .publish(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::CatchAccurateBomb));
            } else {
                game.field_model.ball_coordinate = thrower_coord;
                game.field_model.ball_moving = false;
                return StepOutcome::goto(&self.goto_label_on_end);
            }
        }
        if is_fumble {
            if is_bomb {
                game.field_model.bomb_coordinate = thrower_coord;
            } else {
                game.field_model.ball_coordinate = thrower_coord;
            }
            return StepOutcome::next()
                .publish(StepParameter::PassFumble(true))
                .publish(StepParameter::CatcherId(None))
                .publish(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ScatterBall));
        }
        // INACCURATE or WILDLY_INACCURATE
        if is_bomb {
            game.field_model.bomb_coordinate = pass_coord;
        } else {
            game.field_model.ball_coordinate = pass_coord;
        }
        StepOutcome::goto(&self.goto_label_on_missed_pass)
            .publish(StepParameter::CatcherId(None))
            .publish(StepParameter::PassDeviates(is_wildly_inaccurate))
    }
}

impl Default for StepPass {
    fn default() -> Self { Self::new() }
}

impl Step for StepPass {
    fn id(&self) -> StepId { StepId::Pass }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::UseReRoll { use_reroll: false } => {
                self.re_rolled_action = None;
                self.re_roll_source = None;
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(s)       => { self.goto_label_on_end = s.clone(); true }
            StepParameter::GotoLabelOnMissedPass(s) => { self.goto_label_on_missed_pass = s.clone(); true }
            StepParameter::CatcherId(v)            => { self.catcher_id = v.clone(); true }
            StepParameter::PassResultParam(r)      => { self.result = Some(*r); true } // model-level, for test/replay
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{PassOutcome, Rules};

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    #[test]
    fn id_is_pass() {
        assert_eq!(StepPass::new().id(), StepId::Pass);
    }

    #[test]
    fn set_parameter_goto_label_on_end() {
        let mut step = StepPass::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnEnd("end".into())));
        assert_eq!(step.goto_label_on_end, "end");
    }

    #[test]
    fn set_parameter_goto_label_on_missed_pass() {
        let mut step = StepPass::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnMissedPass("miss".into())));
        assert_eq!(step.goto_label_on_missed_pass, "miss");
    }

    #[test]
    fn set_parameter_catcher_id() {
        let mut step = StepPass::new();
        assert!(step.set_parameter(&StepParameter::CatcherId(Some("p2".into()))));
        assert_eq!(step.catcher_id, Some("p2".into()));
    }

    #[test]
    fn set_parameter_pass_result() {
        let mut step = StepPass::new();
        assert!(step.set_parameter(&StepParameter::PassResultParam(PassOutcome::Fumble)));
        assert_eq!(step.result, Some(PassOutcome::Fumble));
    }

    #[test]
    fn start_returns_next_step_when_no_thrower() {
        let mut game = make_game();
        let mut step = StepPass::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
    }

    fn make_thrower_game() -> (Game, ffb_model::types::FieldCoordinate) {
        use std::collections::HashSet;
        use ffb_model::enums::{PlayerType, PlayerGender, PlayerAction, Rules};
        use ffb_model::model::player::Player;
        use ffb_model::types::FieldCoordinate;
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(Player {
            id: "t1".into(), name: "T".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
                    ..Default::default()
});
        let mut game = Game::new(home, crate::step::framework::test_team("away", 0), Rules::Bb2016);
        game.thrower_id = Some("t1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        let thrower_coord = ffb_model::types::FieldCoordinate::new(13, 7);
        game.field_model.set_player_coordinate("t1", thrower_coord);
        game.pass_coordinate = Some(ffb_model::types::FieldCoordinate::new(14, 7));
        (game, thrower_coord)
    }

    #[test]
    fn with_thrower_sets_ball_moving() {
        let (mut game, _) = make_thrower_game();
        let mut step = StepPass::new();
        step.goto_label_on_end = "end".into();
        step.goto_label_on_missed_pass = "miss".into();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.field_model.ball_moving);
    }

    #[test]
    fn with_thrower_publishes_parameters() {
        let (mut game, _) = make_thrower_game();
        let mut step = StepPass::new();
        step.goto_label_on_end = "end".into();
        step.goto_label_on_missed_pass = "miss".into();
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Must publish at least one parameter regardless of roll result.
        assert!(!out.published.is_empty());
    }

    #[test]
    fn mech_result_is_set_after_roll() {
        let (mut game, _) = make_thrower_game();
        let mut step = StepPass::new();
        step.goto_label_on_end = "end".into();
        step.goto_label_on_missed_pass = "miss".into();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(step.mech_result.is_some());
    }

    #[test]
    fn pass_roll_report_added_after_roll() {
        let (mut game, _) = make_thrower_game();
        let mut step = StepPass::new();
        step.goto_label_on_end = "end".into();
        step.goto_label_on_missed_pass = "miss".into();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::PASS_ROLL),
            "should have PASS_ROLL report after rolling");
    }

    #[test]
    fn pass_roll_report_present_for_bomb_throw() {
        use std::collections::HashSet;
        use ffb_model::enums::{PlayerType, PlayerGender, PlayerAction, Rules};
        use ffb_model::model::player::Player;
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(Player {
            id: "b2".into(), name: "B".into(), nr: 3, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
                    ..Default::default()
        });
        let mut game = Game::new(home, crate::step::framework::test_team("away", 0), Rules::Bb2016);
        game.thrower_id = Some("b2".into());
        game.thrower_action = Some(PlayerAction::ThrowBomb);
        game.field_model.set_player_coordinate("b2", ffb_model::types::FieldCoordinate::new(13, 7));
        game.pass_coordinate = Some(ffb_model::types::FieldCoordinate::new(14, 7));
        let mut step = StepPass::new();
        step.goto_label_on_end = "end".into();
        step.goto_label_on_missed_pass = "miss".into();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::PASS_ROLL),
            "should have PASS_ROLL report for bomb throw");
    }

    #[test]
    fn dont_drop_fumble_not_published_on_inaccurate_result() {
        // Java `StepPass.executeStep()` only publishes DONT_DROP_FUMBLE when
        // state.result is FUMBLE (false) or SAVED_FUMBLE (true) — never for
        // INACCURATE/WILDLY_INACCURATE. Find a seed whose first pass roll (with
        // this thrower's AG 3, adjacent QuickPass, minimum_roll 3) lands on
        // INACCURATE (roll 2, since roll 1 is always FUMBLE, roll 3 meets the
        // minimum_roll of 3 and is ACCURATE, and roll 6 is always ACCURATE).
        let (mut game, _) = make_thrower_game();
        let mut seed = None;
        for s in 0..200u64 {
            let mut probe = GameRng::new(s);
            let roll = probe.d6();
            if roll == 2 {
                seed = Some(s);
                break;
            }
        }
        let seed = seed.expect("should find a seed producing an inaccurate first roll");
        let mut step = StepPass::new();
        step.goto_label_on_end = "end".into();
        step.goto_label_on_missed_pass = "miss".into();
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(step.mech_result, Some(PassResult::INACCURATE));
        assert!(
            !out.published.iter().any(|p| matches!(p, StepParameter::DontDropFumble(_))),
            "DONT_DROP_FUMBLE must not be published for an INACCURATE pass result"
        );
    }

    #[test]
    fn throw_bomb_sets_bomb_moving() {
        use std::collections::HashSet;
        use ffb_model::enums::{PlayerType, PlayerGender, PlayerAction, Rules};
        use ffb_model::model::player::Player;
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(Player {
            id: "b1".into(), name: "B".into(), nr: 2, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
                    ..Default::default()
});
        let mut game = Game::new(home, crate::step::framework::test_team("away", 0), Rules::Bb2016);
        game.thrower_id = Some("b1".into());
        game.thrower_action = Some(PlayerAction::ThrowBomb);
        game.field_model.set_player_coordinate("b1", ffb_model::types::FieldCoordinate::new(13, 7));
        game.pass_coordinate = Some(ffb_model::types::FieldCoordinate::new(14, 7));
        let mut step = StepPass::new();
        step.goto_label_on_end = "end".into();
        step.goto_label_on_missed_pass = "miss".into();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.field_model.bomb_moving);
    }
}
