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
                self.selected_player_id = Some(player_id.clone());
                // Java: addReport(new ReportSelectBlitzTarget(actingPlayer.getPlayerId(), selectedPlayerId))
                // when the selected player is not on the acting team.
                let is_opponent = game.inactive_team().player(player_id).is_some();
                if is_opponent {
                    let acting_id = game.acting_player.player_id.clone();
                    let attacker_id = acting_id.clone().unwrap_or_default();
                    game.report_list.add(ReportSelectBlitzTarget::new(
                        acting_id,
                        Some(player_id.clone()),
                    ));
                    // Java: if usedSkill has canGainFrenzyForBlitz → ReportSkillUse(GAIN_FRENZY_FOR_BLITZ)
                    //        else if canGainClawsForBlitz → ReportSkillUse(GAIN_CLAWS_FOR_BLITZ)
                    //        else if canAvoidDodging → ReportSkillUse(AVOID_DODGING)
                    if let Some(skill) = self.used_skill {
                        if skill.properties().contains(&NamedProperties::CAN_GAIN_FRENZY_FOR_BLITZ) {
                            game.report_list.add(ReportSkillUse::new(
                                Some(attacker_id),
                                skill,
                                true,
                                SkillUse::GAIN_FRENZY_FOR_BLITZ,
                            ));
                        } else if skill.properties().contains(&NamedProperties::CAN_GAIN_CLAWS_FOR_BLITZ) {
                            game.report_list.add(ReportSkillUse::new(
                                Some(attacker_id),
                                skill,
                                true,
                                SkillUse::GAIN_CLAWS_FOR_BLITZ,
                            ));
                        } else if skill.properties().contains(&NamedProperties::CAN_AVOID_DODGING) {
                            game.report_list.add(ReportSkillUse::new(
                                Some(attacker_id),
                                skill,
                                true,
                                SkillUse::AVOID_DODGING,
                            ));
                        }
                    }
                }
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
            None => return StepOutcome::next(),
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
                StepOutcome::cont().with_prompt(AgentPrompt::BlitzTarget {
                    attacker_id: acting_id,
                    eligible_players: targets,
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
                for skill in self.used_skill.iter() {
                    ts.add_used_skill(*skill);
                }
                game.field_model.target_selection_state = Some(ts);
                // Java also sets game.defenderId via the downstream block; keep the folded-target
                // channel in sync so the BlitzBlock sequence finds its defender.
                game.defender_id = Some(selected);
                StepOutcome::next()
            }
        }
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

    #[test]
    fn select_opponent_adds_select_blitz_target_report() {
        use ffb_model::model::player::Player;
        use ffb_model::report::report_id::ReportId;
        let home = test_team("home", 0);
        let mut away = test_team("away", 0);
        away.players.push(Player { id: "away_p1".into(), name: "A".into(), ..Default::default() });
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.home_playing = true;
        let mut step = StepSelectBlitzTarget::new();
        let action = Action::SelectPlayer { player_id: "away_p1".into() };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::SELECT_BLITZ_TARGET));
    }

    #[test]
    fn select_opponent_with_frenzied_rush_adds_skill_use_report() {
        use ffb_model::model::player::Player;
        use ffb_model::report::report_id::ReportId;
        let home = test_team("home", 0);
        let mut away = test_team("away", 0);
        away.players.push(Player { id: "away_p1".into(), name: "A".into(), ..Default::default() });
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.home_playing = true;
        let mut step = StepSelectBlitzTarget::new();
        step.used_skill = Some(SkillId::FrenziedRush);
        let action = Action::SelectPlayer { player_id: "away_p1".into() };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::SELECT_BLITZ_TARGET));
        assert!(game.report_list.has_report(ReportId::SKILL_USE));
    }
}
