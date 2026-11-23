/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.StepSelectBlitzTarget (BB2025).
///
/// Prompts the coach to select a blitz target during FuriousOutburst / AnimalSavagery.
/// Dispatches to skill-specific sequence generators (BlackInk, BalefulHex, etc.) based on
/// the selected skill.
///
/// Init params: GOTO_LABEL_ON_END.
/// Runtime params: END_TURN, END_PLAYER_ACTION, CHECK_FORGO.
///
/// Stub: all generator dispatches (BlackInk, BalefulHex, CatchOfTheDay, LookIntoMyEyes,
/// Treacherous, ThenIStartedBlastin, RaidingParty) not translated → NEXT_STEP immediately.
use ffb_model::enums::SkillId;
use ffb_model::model::game::Game;
use ffb_model::model::property::NamedProperties;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::report::mixed::report_select_blitz_target::ReportSelectBlitzTarget;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_skill_use::ReportSkillUse;
use ffb_model::enums::TurnMode;
use ffb_model::model::target_selection_state::TargetSelectionState;
use ffb_model::prompts::AgentPrompt;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::bb2025::EndPlayerAction;
use crate::step::generator::bb2025::end_player_action::EndPlayerActionParams;

pub struct StepSelectBlitzTarget {
    /// Java: gotoLabelOnEnd — GOTO_LABEL_ON_END init parameter.
    pub goto_label_on_end: String,
    /// Java: selectedPlayerId — set from CLIENT_TARGET_SELECTED command.
    pub selected_player_id: Option<String>,
    /// Java: confirmed — set when selection is confirmed.
    pub confirmed: bool,
    /// Java: endPlayerAction — set by END_PLAYER_ACTION parameter.
    pub end_player_action: bool,
    /// Java: endTurn — set by END_TURN parameter.
    pub end_turn: bool,
    /// Java: checkForgo — set by CHECK_FORGO parameter.
    pub check_forgo: bool,
    /// Java: usedSkill — set when CLIENT_USE_SKILL received.
    pub used_skill: Option<SkillId>,
}

impl StepSelectBlitzTarget {
    pub fn new() -> Self {
        Self {
            goto_label_on_end: String::new(),
            selected_player_id: None,
            confirmed: false,
            end_player_action: false,
            end_turn: false,
            check_forgo: false,
            used_skill: None,
        }
    }
}

impl Default for StepSelectBlitzTarget {
    fn default() -> Self { Self::new() }
}

impl Step for StepSelectBlitzTarget {
    fn id(&self) -> StepId { StepId::SelectBlitzTarget }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::SelectPlayer { player_id } => {
                // Java: CLIENT_TARGET_SELECTED -> selectedPlayerId; the reports are written by
                // executeStep's opponent branch.
                self.selected_player_id = Some(player_id.clone());
            }
            Action::EndTurn => {
                self.end_turn = true;
                self.check_forgo = true;
            }
            // Java: CLIENT_USE_SKILL → store usedSkill for ReportSkillUse emission
            Action::UseSkill { skill_id, use_skill: true } => {
                self.used_skill = Some(*skill_id);
            }
            _ => {}
        }
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::EndTurn(v)           => { self.end_turn = *v; true }
            StepParameter::EndPlayerAction(v)   => { self.end_player_action = *v; true }
            StepParameter::CheckForgo(v)        => { self.check_forgo = *v; true }
            StepParameter::GotoLabelOnEnd(v)    => { self.goto_label_on_end = v.clone(); true }
            _ => false,
        }
    }

    // Java: setParameter consume()s these keys (END_TURN is set WITHOUT consuming).
    fn consumes_parameter(&self, param: &StepParameter) -> bool {
        matches!(param, StepParameter::EndPlayerAction(_))
    }
}

impl StepSelectBlitzTarget {
    /// Java `StepSelectBlitzTarget.executeStep`. The whole body used to be
    /// `StepOutcome::next()` under a "stub" note, which is why Rust never dispatched this step at
    /// all: the blitz target was folded into the declaration instead and the chain was skipped.
    /// Java runs it on EVERY blitz (~750 selections per 100 games).
    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        // Java: getResult().setNextAction(StepAction.CONTINUE) up front; each branch below
        // overrides it.
        if self.end_player_action || self.end_turn {
            // Java: turnMode = lastTurnMode; stepStack.clear(); push EndPlayerAction sequence.
            if let Some(ltm) = game.last_turn_mode { game.turn_mode = ltm; }
            let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
                feeding_allowed: false,
                end_player_action: true,
                end_turn: self.end_turn,
                check_forgo: self.check_forgo,
                rules: game.rules,
            });
            return StepOutcome::next().with_clear_stack().push_seq(seq);
        }

        let acting_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => {
                return StepOutcome::next();
            }
        };

        match self.selected_player_id.clone() {
            // Java: selectedPlayerId == null -> either ask, or skip when nobody can be blitzed.
            None => {
                // Java gates the dialog on `hasStandingOpponents`, which is NOT adjacency-based:
                // it is "ANY opponent anywhere in bounds whose state canBeBlocked". The harness's
                // pickBlockTarget candidate list IS adjacency-based, and the two are deliberately
                // different predicates. Collapsing them into one (the earlier port used the
                // adjacency list for both) made Rust take the skip() path in positions where Java
                // shows the dialog and the harness answers it with EndTurn - lineman bb2025
                // seed 14, where Java ends the turn on a no-target blitz and Rust played on.
                let targets = Self::standing_opponents(game, &acting_id);
                if !Self::has_standing_opponents(game) {
                    // Java: setTargetSelectionState(new TargetSelectionState().skip()); NEXT_STEP
                    let mut ts = TargetSelectionState::default();
                    ts.skip();
                    game.field_model.target_selection_state = Some(ts);
                    return StepOutcome::next();
                }
                // Java: game.setTurnMode(TurnMode.SELECT_BLITZ_TARGET);
                //       showDialog(new DialogSelectBlitzTargetParameter()); CONTINUE
                // A bare cont() here would be the driver's stall shape, so carry the wait as a
                // real prompt. Candidates and order match ParityRunner.pickBlockTarget exactly:
                // adjacent opponents whose base is STANDING or MOVING (NOT hasTackleZones, no
                // confused check), coordinate-sorted, answered with ONE actionRng pick.
                if game.turn_mode != TurnMode::SelectBlitzTarget {
                    game.last_turn_mode = Some(game.turn_mode);
                    game.turn_mode = TurnMode::SelectBlitzTarget;
                }
                // Java: UtilServerDialog.showDialog(getGameState(), new DialogSelectBlitzTargetParameter(), false)
                crate::util::util_server_dialog::UtilServerDialog::show_dialog(
                    game,
                    ffb_model::dialog::dialog_parameter_factory::DialogParameter::SelectBlitzTarget(Default::default()),
                    false,
                );
                let gain_skill = if self.used_skill.is_none() {
                    Self::unused_blitz_gain_skill(game, &acting_id)
                } else {
                    None
                };
                StepOutcome::cont().with_prompt(AgentPrompt::BlitzTarget {
                    attacker_id: acting_id,
                    eligible_players: targets,
                    gain_skill,
                })
            }
            Some(selected) if selected == acting_id => {
                // Java: selecting the acting player himself cancels the selection.
                if let Some(ltm) = game.last_turn_mode { game.turn_mode = ltm; }
                let mut ts = TargetSelectionState::default();
                ts.cancel();
                game.field_model.target_selection_state = Some(ts);
                StepOutcome::goto(&self.goto_label_on_end)
            }
            Some(selected) => {
                // Java: only an OPPOSING player is a legal target.
                if game.inactive_team().player(&selected).is_none() {
                    return StepOutcome::cont();
                }
                if let Some(ltm) = game.last_turn_mode { game.turn_mode = ltm; }
                // Java: playerState.addSelectedBlitzTarget()
                if let Some(st) = game.field_model.player_state(&selected) {
                    game.field_model.set_player_state(&selected, st.add_selected_blitz_target());
                }
                // Java: new TargetSelectionState(selectedPlayerId); commit() when the acting
                // player hasActed(); then .select().
                let mut ts = TargetSelectionState::new(selected.clone());
                if game.acting_player.acted() {
                    ts.commit();
                }
                ts.select();
                // Java: addReport(new ReportSelectBlitzTarget(actingPlayerId, selectedPlayerId))
                game.report_list.add(ReportSelectBlitzTarget::new(
                    Some(acting_id.clone()),
                    Some(selected.clone()),
                ));
                if let Some(skill) = self.used_skill {
                    // Java: targetSelectionState.addUsedSkill(usedSkill)
                    ts.add_used_skill(skill);
                    // Java: if (usedSkill.getEnhancements() != null) {
                    //   fieldModel.addSkillEnhancements(actingPlayer, usedSkill); <reports> }
                    if let Some(granted) = skill.enhancement_skills() {
                        let rules = game.rules;
                        if let Some(p) = game.player_mut(&acting_id) {
                            p.add_skill_enhancement(skill.category_and_name_for(rules).1, granted);
                        }
                        let props = skill.properties_for(rules);
                        let frenzy = props.contains(&NamedProperties::CAN_GAIN_FRENZY_FOR_BLITZ);
                        let claws = props.contains(&NamedProperties::CAN_GAIN_CLAWS_FOR_BLITZ);
                        let avoid = props.contains(&NamedProperties::CAN_AVOID_DODGING);
                        let report = |game: &mut Game, use_: SkillUse| {
                            game.report_list.add(ReportSkillUse::new(
                                Some(acting_id.clone()), skill, true, use_));
                        };
                        if game.rules == ffb_model::enums::Rules::Bb2020 {
                            // bb2020: if frenzy ... else if claws (no avoid-dodging arm)
                            if frenzy {
                                report(game, SkillUse::GAIN_FRENZY_FOR_BLITZ);
                            } else if claws {
                                report(game, SkillUse::GAIN_CLAWS_FOR_BLITZ);
                            }
                        } else {
                            // bb2025: three independent ifs
                            if frenzy { report(game, SkillUse::GAIN_FRENZY_FOR_BLITZ); }
                            if claws { report(game, SkillUse::GAIN_CLAWS_FOR_BLITZ); }
                            if avoid { report(game, SkillUse::AVOID_DODGING); }
                        }
                    }
                }
                game.field_model.target_selection_state = Some(ts);
                // Java also sets game.defenderId via the downstream block; keep the folded-target
                // channel in sync so the BlitzBlock sequence finds its defender.
                game.defender_id = Some(selected);
                StepOutcome::next()
            }
        }
    }

    /// The skill the client's FRENZIED_RUSH / SLASHING_NAILS action would send from this dialog
    /// (`UtilCards.hasUnusedSkillWithProperty` canGainFrenzyForBlitz, then canGainClawsForBlitz).
    /// A plain scan so OFFERING the skill does not register as a skill touch.
    fn unused_blitz_gain_skill(game: &Game, acting_id: &str) -> Option<SkillId> {
        let p = game.player(acting_id)?;
        [NamedProperties::CAN_GAIN_FRENZY_FOR_BLITZ, NamedProperties::CAN_GAIN_CLAWS_FOR_BLITZ]
            .iter()
            .find_map(|prop| p.all_skill_ids()
                .find(|id| id.properties().contains(prop) && !p.used_skills.contains(id)))
    }

    /// Java `hasStandingOpponents`: ANY opponent in bounds whose state `canBeBlocked`. No
    /// adjacency, no ordering - this only decides whether the dialog is shown at all. The
    /// candidate list the agent picks from is the separate, adjacency-based
    /// `standing_opponents` below (ParityRunner.pickBlockTarget).
    fn has_standing_opponents(game: &Game) -> bool {
        let opponent_team = game.inactive_team();
        opponent_team.players.iter().any(|op| {
            game.field_model.player_coordinate(&op.id)
                .map(|c| ffb_model::types::FieldCoordinateBounds::FIELD.is_in_bounds(c)).unwrap_or(false)
                && game.field_model.player_state(&op.id)
                    .map(|s| s.can_be_blocked()).unwrap_or(false)
        })
    }

    /// ParityRunner.pickBlockTarget candidate rule / ParityRunner.pickBlockTarget candidate rule: ADJACENT
    /// opponents whose state base is STANDING or MOVING. Coordinate-sorted so a single
    /// `actionRng` pick lands on the same player in both engines.
    fn standing_opponents(game: &Game, acting_id: &str) -> Vec<String> {
        let Some(coord) = game.field_model.player_coordinate(acting_id) else { return Vec::new() };
        let opponent_team = game.inactive_team();
        let mut out: Vec<String> = opponent_team.players.iter()
            .filter(|op| {
                game.field_model.player_coordinate(&op.id)
                    .map(|oc| oc.is_adjacent(coord))
                    .unwrap_or(false)
                    && game.field_model.player_state(&op.id)
                        .map(|s| s.can_be_blocked())
                        .unwrap_or(false)
            })
            .map(|op| op.id.clone())
            .collect();
        out.sort_by_key(|pid| {
            game.field_model.player_coordinate(pid)
                .map(|c| (c.x, c.y))
                .unwrap_or((i32::MAX, i32::MAX))
        });
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    fn blitz_test_player(id: &str) -> ffb_model::model::player::Player {
        use ffb_model::enums::{PlayerGender, PlayerType};
        ffb_model::model::player::Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            ..Default::default()
        }
    }

    #[test]
    fn start_returns_next_step() {
        let mut game = make_game();
        let mut step = StepSelectBlitzTarget::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn handle_select_player_stores_id() {
        let mut game = make_game();
        let mut step = StepSelectBlitzTarget::new();
        let action = Action::SelectPlayer { player_id: "p1".into() };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(step.selected_player_id.as_deref(), Some("p1"));
    }

    /// Java gates the dialog on `hasStandingOpponents` - ANY in-bounds opponent that
    /// `canBeBlocked` - while the harness's `pickBlockTarget` candidate list is ADJACENCY-based.
    /// The port collapsed both into the adjacency list, so a blitzer with no neighbour took the
    /// skip() path where Java shows the dialog and the harness answers it with EndTurn (lineman
    /// bb2025 seed 14). Pin the two predicates apart: a DISTANT standing opponent must still
    /// produce the prompt, with an EMPTY candidate list.
    #[test]
    fn distant_standing_opponent_prompts_with_no_candidates() {
        use ffb_model::enums::{PlayerState, PS_STANDING};
        use ffb_model::types::FieldCoordinate;
        let mut game = make_game();
        game.home_playing = true;
        game.team_home.players.push(blitz_test_player("h1"));
        game.team_away.players.push(blitz_test_player("a1"));

        let attacker = "h1".to_string();
        let far_opponent = "a1".to_string();
        game.acting_player.player_id = Some(attacker.clone());
        game.field_model.set_player_coordinate(&attacker, FieldCoordinate::new(2, 2));
        game.field_model.set_player_state(&attacker, PlayerState::new(PS_STANDING));
        // Far away: in bounds and blockable, but NOT adjacent.
        game.field_model.set_player_coordinate(&far_opponent, FieldCoordinate::new(20, 10));
        game.field_model.set_player_state(&far_opponent, PlayerState::new(PS_STANDING));

        let mut step = StepSelectBlitzTarget::new();
        let out = step.start(&mut game, &mut GameRng::new(0));

        assert_eq!(out.action, StepAction::Continue, "Java shows the dialog and waits");
        match out.prompt {
            Some(AgentPrompt::BlitzTarget { ref eligible_players, .. }) => {
                assert!(eligible_players.is_empty(),
                    "candidates are adjacency-based, so a distant opponent is not one");
            }
            other => panic!("expected a BlitzTarget prompt, got {other:?}"),
        }
        assert!(game.field_model.target_selection_state.is_none(),
            "the skip() path must NOT be taken while a blockable opponent exists");
    }

    /// The mirror case: with no blockable opponent anywhere, Java DOES skip.
    #[test]
    fn no_blockable_opponent_anywhere_skips() {
        use ffb_model::enums::{PlayerState, PS_STANDING};
        use ffb_model::types::FieldCoordinate;
        let mut game = make_game();
        game.home_playing = true;
        game.team_home.players.push(blitz_test_player("h1"));
        game.team_away.players.push(blitz_test_player("a1"));

        let attacker = "h1".to_string();
        game.acting_player.player_id = Some(attacker.clone());
        game.field_model.set_player_coordinate(&attacker, FieldCoordinate::new(2, 2));
        game.field_model.set_player_state(&attacker, PlayerState::new(PS_STANDING));
        // Every away player stays off the field (no coordinate), so none is in bounds.

        let mut step = StepSelectBlitzTarget::new();
        let out = step.start(&mut game, &mut GameRng::new(0));

        assert_eq!(out.action, StepAction::NextStep);
        assert!(game.field_model.target_selection_state.as_ref()
            .map(|ts| ts.is_skipped()).unwrap_or(false));
    }

    #[test]
    fn end_turn_pushes_end_player_action_sequence() {
        let mut game = make_game();
        let mut step = StepSelectBlitzTarget::new();
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "end_turn should push EndPlayerAction sequence");
    }

    #[test]
    fn end_player_action_pushes_sequence() {
        let mut game = make_game();
        let mut step = StepSelectBlitzTarget::new();
        step.end_player_action = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "end_player_action should push EndPlayerAction sequence");
    }

    #[test]
    fn handle_end_turn_sets_flags() {
        let mut game = make_game();
        let mut step = StepSelectBlitzTarget::new();
        step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert!(step.end_turn);
        assert!(step.check_forgo);
    }

    #[test]
    fn set_parameter_wiring() {
        let mut step = StepSelectBlitzTarget::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnEnd("END".into())));
        assert_eq!(step.goto_label_on_end, "END");
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.set_parameter(&StepParameter::EndPlayerAction(true)));
        assert!(step.set_parameter(&StepParameter::CheckForgo(true)));
        assert!(step.check_forgo);
    }

    /// Acting home blitzer `h1` (carrying `skill`, if any) next to a standing away player `a1`.
    fn blitz_game(rules: Rules, skill: Option<SkillId>) -> Game {
        use ffb_model::enums::{PlayerState, PS_STANDING};
        use ffb_model::model::skill_def::SkillWithValue;
        use ffb_model::types::FieldCoordinate;
        let mut game = Game::new(test_team("home", 0), test_team("away", 0), rules);
        game.home_playing = true;
        let mut h1 = blitz_test_player("h1");
        if let Some(s) = skill {
            h1.starting_skills.push(SkillWithValue { skill_id: s, value: None });
        }
        game.team_home.players.push(h1);
        game.team_away.players.push(blitz_test_player("a1"));
        game.acting_player.player_id = Some("h1".into());
        game.field_model.set_player_coordinate("h1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("h1", PlayerState::new(PS_STANDING));
        game.field_model.set_player_coordinate("a1", FieldCoordinate::new(6, 5));
        game.field_model.set_player_state("a1", PlayerState::new(PS_STANDING));
        game
    }

    fn skill_uses(game: &Game) -> usize {
        use ffb_model::report::report_id::ReportId;
        game.report_list.get_reports().iter().filter(|r| r.get_id() == ReportId::SKILL_USE).count()
    }

    #[test]
    fn select_opponent_adds_select_blitz_target_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = blitz_game(Rules::Bb2025, None);
        let mut step = StepSelectBlitzTarget::new();
        let action = Action::SelectPlayer { player_id: "a1".into() };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::SELECT_BLITZ_TARGET));
        assert_eq!(skill_uses(&game), 0);
    }

    /// Java StepSelectBlitzTarget: the dialog offers the unused blitz skill, CLIENT_USE_SKILL
    /// keeps the step waiting (SKIP_STEP), and the target selection then adds the skill to the
    /// TargetSelectionState, grants its enhancement (Frenzy, source "Frenzied Rush") and reports
    /// GAIN_FRENZY_FOR_BLITZ.
    #[test]
    fn frenzied_rush_is_offered_then_grants_frenzy_on_select() {
        let mut game = blitz_game(Rules::Bb2025, Some(SkillId::FrenziedRush));
        let mut step = StepSelectBlitzTarget::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        match out.prompt {
            Some(AgentPrompt::BlitzTarget { gain_skill, .. }) =>
                assert_eq!(gain_skill, Some(SkillId::FrenziedRush)),
            other => panic!("expected BlitzTarget, got {other:?}"),
        }
        let out = step.handle_command(
            &Action::UseSkill { skill_id: SkillId::FrenziedRush, use_skill: true },
            &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue, "the step keeps waiting for the target");
        match out.prompt {
            Some(AgentPrompt::BlitzTarget { gain_skill, .. }) => assert_eq!(gain_skill, None),
            other => panic!("expected BlitzTarget, got {other:?}"),
        }
        let out = step.handle_command(&Action::SelectPlayer { player_id: "a1".into() },
            &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        let h1 = game.player("h1").unwrap();
        assert!(h1.has_skill(SkillId::Frenzy), "Frenzy is granted for the blitz");
        assert!(h1.has_active_enhancement("Frenzied Rush"));
        assert!(!h1.used_skills.contains(&SkillId::FrenziedRush), "spent only at the end");
        let ts = game.field_model.target_selection_state.as_ref().unwrap();
        assert!(ts.get_used_skills().contains(&SkillId::FrenziedRush));
        assert_eq!(skill_uses(&game), 1);
    }

    #[test]
    fn slashing_nails_grants_claws_on_select() {
        let mut game = blitz_game(Rules::Bb2025, Some(SkillId::SlashingNails));
        let mut step = StepSelectBlitzTarget::new();
        step.used_skill = Some(SkillId::SlashingNails);
        step.handle_command(&Action::SelectPlayer { player_id: "a1".into() },
            &mut game, &mut GameRng::new(0));
        assert!(game.player("h1").unwrap().has_skill(SkillId::Claw));
        assert_eq!(skill_uses(&game), 1);
    }

    /// The end of the blitz (StepRemoveTargetSelectionState -> markSkillsTrackedOutsideOf
    /// ActivationAndRemoveEffects) spends Frenzied Rush once the player has acted and takes the
    /// granted Frenzy away again.
    #[test]
    fn blitz_end_spends_frenzied_rush_and_removes_frenzy() {
        let mut game = blitz_game(Rules::Bb2020, Some(SkillId::FrenziedRush));
        let mut step = StepSelectBlitzTarget::new();
        step.used_skill = Some(SkillId::FrenziedRush);
        step.handle_command(&Action::SelectPlayer { player_id: "a1".into() },
            &mut game, &mut GameRng::new(0));
        assert!(game.player("h1").unwrap().has_skill(SkillId::Frenzy));
        game.acting_player.has_blocked = true;
        crate::step::util_server_steps::mark_skills_tracked_outside_of_activation_and_remove_effects(&mut game);
        let h1 = game.player("h1").unwrap();
        assert!(!h1.has_skill(SkillId::Frenzy));
        assert!(!h1.has_active_enhancement("Frenzied Rush"));
        assert!(h1.used_skills.contains(&SkillId::FrenziedRush));
    }

    /// Without having acted, the effect is removed but the skill is NOT spent (Java: markUsed is
    /// gated on hasActed()).
    #[test]
    fn blitz_end_without_acting_keeps_frenzied_rush() {
        let mut game = blitz_game(Rules::Bb2025, Some(SkillId::FrenziedRush));
        game.player_mut("h1").unwrap().add_skill_enhancement("Frenzied Rush", &[SkillId::Frenzy]);
        crate::step::util_server_steps::mark_skills_tracked_outside_of_activation_and_remove_effects(&mut game);
        let h1 = game.player("h1").unwrap();
        assert!(!h1.has_skill(SkillId::Frenzy));
        assert!(!h1.used_skills.contains(&SkillId::FrenziedRush));
    }
}
