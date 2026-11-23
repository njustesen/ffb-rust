/// 1:1 translation of com.fumbbl.ffb.server.step.action.block.StepDumpOff (COMMON rules)
/// and its BB2016/BB2020/BB2025 hook com.fumbbl.ffb.server.skillbehaviour.*.DumpOffBehaviour.
///
/// DumpOff lets the defender make a pass when they are about to be blocked.
/// The defender must have the DumpOff skill, be holding the ball, and not be confused/hypnotized.
///
/// Expects DEFENDER_POSITION parameter from a preceding step.
use ffb_model::enums::{PlayerAction, Rules, SkillId, TurnMode};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::prompts::agent_prompt::AgentPrompt;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::report::report_skill_use::ReportSkillUse;
use ffb_model::report::report_id::ReportId;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

pub struct StepDumpOff {
    /// Java: state.usingDumpOff — None = not asked, Some = decided.
    pub using_dump_off: Option<bool>,
    /// Java: state.defenderPosition — set by DEFENDER_POSITION parameter.
    pub defender_position: Option<FieldCoordinate>,
    /// Java: state.oldTurnMode — saved when DumpOff pass is initiated, restored when done.
    pub old_turn_mode: Option<TurnMode>,
}

impl StepDumpOff {
    pub fn new() -> Self {
        Self {
            using_dump_off: None,
            defender_position: None,
            old_turn_mode: None,
        }
    }
}

impl Default for StepDumpOff {
    fn default() -> Self { Self::new() }
}

impl Step for StepDumpOff {
    fn id(&self) -> StepId { StepId::DumpOff }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if let Action::UseSkill { use_skill, .. } = action {
            self.using_dump_off = Some(*use_skill);
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::DefenderPosition(v) => { self.defender_position = Some(*v); true }
            _ => false,
        }
    }
}

impl StepDumpOff {
    /// Java: DumpOffBehaviour.handleExecuteStepHook (logic identical across editions).
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java bb2025 DumpOffBehaviour: UtilServerDialog.hideDialog(step.getGameState())
        crate::util::util_server_dialog::UtilServerDialog::hide_dialog(game);
        // Java: if (turnMode == DUMP_OFF) → restore old turn mode and clear thrower.
        if game.turn_mode == TurnMode::DumpOff {
            if let Some(old) = self.old_turn_mode {
                game.turn_mode = old;
            }
            game.thrower_id = None;
            return StepOutcome::next();
        }

        // Java: if usingDumpOff == null → check conditions.
        if self.using_dump_off.is_none() {
            let can_dump_off = self.can_use_dump_off(game);
            if can_dump_off {
                let defender_id = self.dump_off_defender(game).unwrap_or_default();
                // Java: showDialog(new DialogSkillUseParameter(defender.getId(), skill, 0), true)
                crate::util::util_server_dialog::UtilServerDialog::show_dialog(
                    game,
                    ffb_model::dialog::dialog_parameter_factory::DialogParameter::SkillUse(
                        ffb_model::dialog::dialog_skill_use_parameter::DialogSkillUseParameter {
                            player_id: Some(defender_id.clone()),
                            skill: Some(SkillId::DumpOff),
                            minimum_roll: 0,
                            ..Default::default()
                        },
                    ),
                    true,
                );
                return StepOutcome::cont().with_prompt(AgentPrompt::SkillUse {
                    player_id: defender_id,
                    skill_id: SkillId::DumpOff as u16,
                    skill_name: format!("{:?}", SkillId::DumpOff),
                });
            } else {
                // Java (all three DumpOffBehaviours): `state.usingDumpOff = false;
                // setNextAction(NEXT_STEP)` — straight out, NO report. Only a declined DIALOG
                // reaches the `else` branch below that writes ReportSkillUse(..., false, null);
                // falling through here wrote a spurious Dump-Off skillUse before every block on
                // a Dump-Off carrier that was not holding the ball (bb2016
                // amazon_vs_dark_elf_league_fumbbl seed 1).
                self.using_dump_off = Some(false);
                return StepOutcome::next();
            }
        }

        let using = self.using_dump_off.unwrap_or(false);
        let skill_num = SkillId::DumpOff as u16;
        // Java bb2020/bb2025: `step.commitTargetSelection()` before either answer is acted on,
        // and the thrower / reported player is the target selection's selected player when one
        // exists. bb2016 has no target selection and reads `game.getDefenderId()`.
        if game.rules != Rules::Bb2016 {
            if let Some(ts) = game.field_model.target_selection_state.as_mut() {
                ts.commit();
            }
        }
        let defender_id = if game.rules == Rules::Bb2016 {
            game.defender_id.clone()
        } else {
            game.field_model
                .target_selection_state
                .as_ref()
                .map(|ts| ts.selected_player_id.clone())
                .unwrap_or_else(|| game.defender_id.clone())
        };

        if using {
            // Java: save old turn mode, switch to DumpOff, set thrower, show the
            // DEFENDER_ACTION dialog, pushCurrentStepOnStack(), push the edition's Pass sequence.
            self.old_turn_mode = Some(game.turn_mode);
            game.turn_mode = TurnMode::DumpOff;
            game.thrower_id = defender_id.clone();
            // Java: game.setThrowerAction(PlayerAction.DUMP_OFF) — NOT actingPlayer's action.
            // StepDispatchPassing routes on game.thrower_action, so leaving this unset would
            // make the pushed Pass sequence immediately goto its "end" label instead of
            // resolving the dump-off pass.
            game.thrower_action = Some(PlayerAction::DumpOff);
            game.defender_action = Some(PlayerAction::DumpOff);
            // Java: showDialog(new DialogDefenderActionParameter(), true)
            crate::util::util_server_dialog::UtilServerDialog::show_dialog(
                game,
                ffb_model::dialog::dialog_parameter_factory::DialogParameter::DefenderAction(Default::default()),
                true,
            );
            let event = GameEvent::SkillUse {
                player_id: defender_id.unwrap_or_default(),
                skill_id: skill_num,
                used: true,
            };
            // The per-ruleset SequenceGeneratorFactory Pass sequence (bb2016's own; bb2020 and
            // bb2025 share the bb2025 generator, parameterised by rules). The old stub
            // (PickUp/Intercept/Pass/EndPlayerAction) had no InitPassing, no EndPassing and
            // ended the ATTACKER's action. `push_self` = Java's pushCurrentStepOnStack(): this
            // step re-runs after the pass, sees TurnMode.DUMP_OFF and restores the old mode.
            let seq = if game.rules == Rules::Bb2016 {
                crate::step::generator::bb2016::pass::Pass::build_sequence(
                    &crate::step::generator::bb2016::pass::PassParams { target_coordinate: None },
                )
            } else {
                crate::step::generator::bb2025::pass::Pass::build_sequence(
                    &crate::step::generator::bb2025::pass::PassParams {
                        target_coordinate: None,
                        rules: game.rules,
                    },
                )
            };
            StepOutcome::next().with_event(event).push_self().push_seq(seq)
        } else {
            // Java (every edition): addReport(new ReportSkillUse(defenderId, skill, false, null)).
            // Only a declined DIALOG reaches this branch, so the defender has the skill.
            game.report_list.add(ReportSkillUse::new_without_skill_use(
                defender_id.clone(),
                SkillId::DumpOff,
                false,
            ));
            let event = GameEvent::SkillUse {
                player_id: defender_id.unwrap_or_default(),
                skill_id: skill_num,
                used: false,
            };
            StepOutcome::next().with_event(event)
        }
    }

    /// The player the dialog is shown for. Java bb2020: the target selection's selected player
    /// when a TargetSelectionState exists, else `game.getDefender()`; bb2016/bb2025:
    /// `game.getDefender()`.
    fn dump_off_defender(&self, game: &Game) -> Option<String> {
        if game.rules == Rules::Bb2020 {
            if let Some(ts) = game.field_model.target_selection_state.as_ref() {
                return ts.selected_player_id.clone();
            }
        }
        game.defender_id.clone()
    }

    /// The defender's position for the ball check. Java bb2016: `state.defenderPosition` only;
    /// bb2020: the selected player's FIELD coordinate when a TargetSelectionState exists, else
    /// `state.defenderPosition`; bb2025: `state.defenderPosition`, falling back to the
    /// defender's field coordinate when the parameter never arrived.
    fn dump_off_position(&self, game: &Game, defender_id: &str) -> Option<FieldCoordinate> {
        match game.rules {
            Rules::Bb2016 => self.defender_position,
            Rules::Bb2020 => {
                if game.field_model.target_selection_state.is_some() {
                    game.field_model.player_coordinate(defender_id)
                } else {
                    self.defender_position
                }
            }
            _ => self.defender_position.or_else(|| game.field_model.player_coordinate(defender_id)),
        }
    }

    /// Java: condition check inside the (usingDumpOff == null) branch.
    fn can_use_dump_off(&self, game: &Game) -> bool {
        let defender_id = match self.dump_off_defender(game) {
            Some(id) => id,
            None => return false,
        };
        let defender_id = &defender_id;
        let has_skill = game.player(defender_id)
            .map(|p| p.has_skill(SkillId::DumpOff))
            .unwrap_or(false);
        if !has_skill { return false; }
        let position = match self.dump_off_position(game, defender_id) {
            Some(p) => p,
            None => return false,
        };
        // Java: defenderPosition.equals(game.getFieldModel().getBallCoordinate())
        let ball_at_defender = game.field_model.ball_coordinate == Some(position);
        if !ball_at_defender { return false; }
        // Java: !game.getFieldModel().isBallMoving()
        if game.field_model.ball_moving { return false; }
        // Java: !defenderState.isConfused() && !defenderState.isHypnotized()
        let state = game.field_model.player_state(defender_id);
        if state.map(|s| s.is_confused() || s.is_hypnotized()).unwrap_or(false) {
            return false;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{Rules, PS_STANDING};
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::types::FieldCoordinate;
    use ffb_model::enums::PlayerState;

    fn add_player(
        team: &mut ffb_model::model::team::Team,
        id: &str,
        nr: i32,
        skills: Vec<SkillId>,
    ) {
        team.players.push(ffb_model::model::player::Player {
            id: id.into(),
            name: id.into(),
            nr,
            position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
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

    fn make_game(defender_skills: Vec<SkillId>) -> Game {
        let mut home = test_team("home", 0);
        let mut away = test_team("away", 0);
        add_player(&mut home, "att", 1, vec![]);
        add_player(&mut away, "def", 2, defender_skills);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.home_playing = true;
        game.acting_player.player_id = Some("att".into());
        game.defender_id = Some("def".into());
        game.field_model.set_player_coordinate("att", FieldCoordinate::new(5, 5));
        game.field_model.set_player_coordinate("def", FieldCoordinate::new(6, 5));
        game.field_model.set_player_state("att", PlayerState::new(PS_STANDING).change_active(true));
        game.field_model.set_player_state("def", PlayerState::new(PS_STANDING));
        game
    }

    #[test]
    fn no_dump_off_skill_returns_next_no_event() {
        let mut game = make_game(vec![]);
        let outcome = StepDumpOff::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.events.is_empty());
    }

    #[test]
    fn defender_has_skill_but_no_ball_returns_next_with_declined_event() {
        let mut game = make_game(vec![SkillId::DumpOff]);
        // Ball not at defender's position
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(3, 3));
        let mut step = StepDumpOff::new();
        step.defender_position = Some(FieldCoordinate::new(6, 5)); // defender is at (6,5), ball at (3,3)
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        // Java: conditions not met → `usingDumpOff = false; NEXT_STEP` with NO report/event.
        assert!(outcome.events.is_empty());
        assert!(!game.report_list.has_report(ReportId::SKILL_USE));
    }

    #[test]
    fn ball_at_defender_with_skill_prompts_skill_use() {
        // Java: showDialog → CONTINUE when the defender can use DumpOff.
        let mut game = make_game(vec![SkillId::DumpOff]);
        let def_pos = FieldCoordinate::new(6, 5);
        game.field_model.ball_coordinate = Some(def_pos);
        let mut step = StepDumpOff::new();
        step.defender_position = Some(def_pos);
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::Continue);
        assert!(matches!(outcome.prompt, Some(AgentPrompt::SkillUse { .. })));
    }

    #[test]
    fn using_dump_off_true_sets_turn_mode_and_thrower() {
        let mut game = make_game(vec![SkillId::DumpOff]);
        let mut step = StepDumpOff::new();
        step.using_dump_off = Some(true);
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_mode, TurnMode::DumpOff);
        assert_eq!(game.thrower_id.as_deref(), Some("def"));
    }

    #[test]
    fn using_dump_off_true_sets_thrower_action_not_acting_player_action() {
        // Java: game.setThrowerAction(PlayerAction.DUMP_OFF) — game.thrower_action, not
        // actingPlayer's own player_action. StepDispatchPassing (the step that follows in the
        // pushed Pass sequence) routes purely on game.thrower_action; leaving it unset would
        // make the Pass sequence immediately bail out via its "end" goto label.
        let mut game = make_game(vec![SkillId::DumpOff]);
        let mut step = StepDumpOff::new();
        step.using_dump_off = Some(true);
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.thrower_action, Some(PlayerAction::DumpOff));
        assert_eq!(game.defender_action, Some(PlayerAction::DumpOff));
    }

    #[test]
    fn using_dump_off_true_emits_skill_used_event() {
        let mut game = make_game(vec![SkillId::DumpOff]);
        let mut step = StepDumpOff::new();
        step.using_dump_off = Some(true);
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert!(outcome.events.iter().any(|e| matches!(e, GameEvent::SkillUse { used: true, .. })));
    }

    #[test]
    fn dump_off_turn_mode_restores_old_mode() {
        let mut game = make_game(vec![]);
        game.turn_mode = TurnMode::DumpOff;
        let mut step = StepDumpOff::new();
        step.old_turn_mode = Some(TurnMode::Regular);
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert_eq!(game.turn_mode, TurnMode::Regular);
        assert!(game.thrower_id.is_none());
    }

    #[test]
    fn set_parameter_stores_defender_position() {
        let mut step = StepDumpOff::new();
        let pos = FieldCoordinate::new(7, 4);
        assert!(step.set_parameter(&StepParameter::DefenderPosition(pos)));
        assert_eq!(step.defender_position, Some(pos));
        assert!(!step.set_parameter(&StepParameter::EndPlayerAction(true)));
    }

    #[test]
    fn defender_has_skill_but_declines_adds_skill_use_report() {
        // Java: a DECLINED dialog (`usingDumpOff == false` from the command) writes
        // ReportSkillUse(defenderId, skill, false, null).
        let mut game = make_game(vec![SkillId::DumpOff]);
        let mut step = StepDumpOff::new();
        step.using_dump_off = Some(false);
        step.start(&mut game, &mut GameRng::new(0));
        assert!(
            game.report_list.has_report(ReportId::SKILL_USE),
            "SKILL_USE report must be added when defender has DumpOff but does not use it"
        );
    }

    /// Java: pushCurrentStepOnStack() + the per-ruleset Pass sequence (InitPassing first, so the
    /// DUMP_OFF park can be answered), and the step re-runs afterwards to restore the turn mode.
    #[test]
    fn using_dump_off_pushes_itself_and_the_real_pass_sequence() {
        let mut game = make_game(vec![SkillId::DumpOff]);
        let mut step = StepDumpOff::new();
        step.using_dump_off = Some(true);
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert!(outcome.push_self, "Java pushCurrentStepOnStack()");
        let first = outcome.pushes.first().and_then(|s| s.first()).map(|s| s.step_id);
        assert_eq!(first, Some(StepId::InitPassing));
    }

    /// Java bb2025: a missing DEFENDER_POSITION parameter falls back to the defender's square.
    #[test]
    fn bb2025_missing_defender_position_falls_back_to_the_field() {
        let mut game = make_game(vec![SkillId::DumpOff]);
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(6, 5));
        let outcome = StepDumpOff::new().start(&mut game, &mut GameRng::new(0));
        assert!(matches!(outcome.prompt, Some(AgentPrompt::SkillUse { .. })));
    }

    #[test]
    fn bb2016_missing_defender_position_never_offers() {
        let mut game = make_game(vec![SkillId::DumpOff]);
        game.rules = Rules::Bb2016;
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(6, 5));
        let outcome = StepDumpOff::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
    }

    #[test]
    fn defender_without_skill_does_not_add_skill_use_report() {
        let mut game = make_game(vec![]);
        let outcome = StepDumpOff::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, crate::step::framework::StepAction::NextStep);
        assert!(
            !game.report_list.has_report(ReportId::SKILL_USE),
            "SKILL_USE report must NOT be added when defender has no DumpOff skill"
        );
    }
}
