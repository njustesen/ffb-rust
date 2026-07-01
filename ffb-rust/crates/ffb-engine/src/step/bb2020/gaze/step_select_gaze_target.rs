/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.gaze.StepSelectGazeTarget` (BB2020).
///
/// Prompts the active coach to select a Hypnotic Gaze target, handles cancellation, and
/// dispatches skill sub-sequences (Treacherous, RaidingParty, BalefulHex, LookIntoMyEyes,
/// BlackInk, ThenIStartedBlastin, CatchOfTheDay) when the coach uses a matching skill.
///
/// DEFERREDs:
///  - DEFERRED(dialog): DialogSelectGazeTargetParameter / DialogConfirmEndActionParameter —
///      waiting for dialog infrastructure (UtilServerDialog.showDialog) to be ported.
///  - DEFERRED(clear-stack): stack clear before EndPlayerAction — the step has no stack reference;
///      waiting for driver-level stack-clear support.
///  - DEFERRED(reportSkillUse): ReportSkillUse (GAIN_FRENZY_FOR_BLITZ) — skill-use tracking not yet ported.
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2020.gaze.StepSelectGazeTarget`.
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::enums::TurnMode;
use ffb_model::enums::PlayerState;
use ffb_model::model::property::NamedProperties;
use ffb_model::model::target_selection_state::TargetSelectionState;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::bb2020::EndPlayerAction;
use crate::step::generator::bb2020::end_player_action::EndPlayerActionParams;
use crate::step::generator::bb2020::{
    Treacherous, treacherous::TreacherousParams,
    RaidingParty, raiding_party::RaidingPartyParams,
    BalefulHex, baleful_hex::BalefulHexParams,
    LookIntoMyEyes, look_into_my_eyes::LookIntoMyEyesParams,
    BlackInk, black_ink::BlackInkParams,
    ThenIStartedBlastin,
    CatchOfTheDay, catch_of_the_day::CatchOfTheDayParams,
};
use crate::step::generator::sequence::{SequenceStep, labels};

pub struct StepSelectGazeTarget {
    pub goto_label_on_end: String,
    pub selected_player_id: Option<String>,
    pub confirmed: bool,
    pub end_player_action: bool,
    pub end_turn: bool,
    /// Skill name used (from a UseSkill action that triggered a skill sub-sequence).
    pub used_skill: Option<String>,
}

impl StepSelectGazeTarget {
    pub fn new() -> Self {
        Self {
            goto_label_on_end: String::new(),
            selected_player_id: None,
            confirmed: false,
            end_player_action: false,
            end_turn: false,
            used_skill: None,
        }
    }
}

impl Default for StepSelectGazeTarget {
    fn default() -> Self { Self::new() }
}

impl Step for StepSelectGazeTarget {
    fn id(&self) -> StepId { StepId::SelectGazeTarget }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::SelectPlayer { player_id } => {
                // Java CLIENT_TARGET_SELECTED: set selectedPlayerId, EXECUTE_STEP
                self.selected_player_id = Some(player_id.clone());
            }
            Action::EndTurn => {
                // Java CLIENT_END_TURN (checkCommandIsFromCurrentPlayer omitted): set endTurn, EXECUTE_STEP
                self.end_turn = true;
            }
            Action::UseSkill { skill_id, use_skill } if *use_skill => {
                // Java CLIENT_USE_SKILL (isSkillUsed = true): push current step + route to sub-sequence.
                // Java: getGameState().pushCurrentStepOnStack() + push generator sequence + NEXT_STEP.
                // Rust: push a fresh SelectGazeTarget (labeled SELECT, same goto_label_on_end) below
                //       the sub-sequence so the driver returns here after the sub-sequence finishes.
                let self_return_seq = vec![SequenceStep {
                    step_id: StepId::SelectGazeTarget,
                    label: Some(labels::SELECT.to_string()),
                    params: vec![StepParameter::GotoLabelOnEnd(self.goto_label_on_end.clone())],
                }];

                let props = skill_id.properties();
                let sub_seq = if props.contains(&NamedProperties::CAN_STAB_TEAM_MATE_FOR_BALL) {
                    // Java: Treacherous generator, failureLabel = SELECT
                    Some(Treacherous::build_sequence(&TreacherousParams { failure_label: labels::SELECT.into() }))
                } else if props.contains(&NamedProperties::CAN_MOVE_OPEN_TEAM_MATE) {
                    // Java: RaidingParty generator, failureLabel = SELECT, successLabel = SELECT
                    Some(RaidingParty::build_sequence(&RaidingPartyParams {
                        failure_label: labels::SELECT.into(),
                        success_label: labels::SELECT.into(),
                    }))
                } else if props.contains(&NamedProperties::CAN_MAKE_OPPONENT_MISS_TURN) {
                    // Java: BalefulHex generator, failureLabel = SELECT
                    Some(BalefulHex::build_sequence(&BalefulHexParams { failure_label: labels::SELECT.into() }))
                } else if props.contains(&NamedProperties::CAN_STEAL_BALL_FROM_OPPONENT) {
                    // Java: LookIntoMyEyes generator, push_select = false, goto_on_end = goto_label_on_end
                    Some(LookIntoMyEyes::build_sequence(&LookIntoMyEyesParams {
                        push_select: false,
                        goto_on_end: self.goto_label_on_end.clone(),
                    }))
                } else if props.contains(&NamedProperties::CAN_GAZE_AUTOMATICALLY) {
                    // Java: BlackInk generator, failureLabel = goto_label_on_end, old_player_state = acting player state
                    let old_state = game.acting_player.player_id.as_deref()
                        .and_then(|pid| game.field_model.player_state(pid));
                    Some(BlackInk::build_sequence(&BlackInkParams {
                        failure_label: self.goto_label_on_end.clone(),
                        old_player_state: old_state,
                    }))
                } else if props.contains(&NamedProperties::CAN_BLAST_REMOTE_PLAYER) {
                    // Java: ThenIStartedBlastin generator (no params)
                    Some(ThenIStartedBlastin::build_sequence())
                } else if props.contains(&NamedProperties::CAN_GET_BALL_ON_GROUND) {
                    // Java: CatchOfTheDay generator, failureLabel = goto_label_on_end
                    Some(CatchOfTheDay::build_sequence(&CatchOfTheDayParams {
                        failure_label: self.goto_label_on_end.clone(),
                    }))
                } else {
                    // Unknown skill property — store name for debugging, no sub-sequence
                    self.used_skill = Some(format!("{skill_id:?}"));
                    None
                };

                if let Some(seq) = sub_seq {
                    // Push self-return first (goes below sub-sequence), then sub-sequence on top.
                    // This mirrors Java's pushCurrentStepOnStack() + pushSequence() + NEXT_STEP.
                    let mut outcome = StepOutcome::next()
                        .push_seq(self_return_seq)
                        .push_seq(seq);
                    return outcome;
                }
            }
            Action::UseSkill { .. } => {
                // use_skill = false: skill not used, just execute step
            }
            Action::Acknowledge => {
                // Java CLIENT_CONFIRM: set confirmed = true, EXECUTE_STEP
                self.confirmed = true;
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(v) => { self.goto_label_on_end = v.clone(); true }
            StepParameter::EndPlayerAction(v) => { self.end_player_action = *v; true }
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
            _ => false,
        }
    }
}

impl StepSelectGazeTarget {
    fn execute_step(&self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: result.setNextAction(StepAction.CONTINUE)  (default)

        // Case 1: end player action or end turn triggered
        // Java: game.setTurnMode(game.getLastTurnMode()); clear stack; push EndPlayerAction; NEXT_STEP
        if self.end_player_action || self.end_turn {
            game.turn_mode = game.last_turn_mode.unwrap_or(game.turn_mode);
            // DEFERRED(clear-stack): Java clears the step stack before pushing EndPlayerAction —
            //   no stack reference is available inside execute_step(); waiting for driver-level support.
            let params = EndPlayerActionParams {
                feeding_allowed: false,
                end_player_action: true,
                end_turn: self.end_turn,
            };
            let seq = EndPlayerAction::build_sequence(&params);
            return StepOutcome::next().push_seq(seq);
        }

        // Case 2: no player selected yet
        // Java: game.setTurnMode(SELECT_GAZE_TARGET); showDialog(SelectGazeTarget); CONTINUE
        if self.selected_player_id.is_none() {
            game.turn_mode = TurnMode::SelectGazeTarget;
            // DEFERRED(dialog): DialogSelectGazeTargetParameter — waiting for dialog infrastructure.
            return StepOutcome::cont();
        }

        let selected_id = self.selected_player_id.clone().unwrap();
        let acting_player_id = game.acting_player.player_id.clone().unwrap_or_default();

        // Case 3: acting player selected themselves (cancel gaze or confirm end action)
        // Java: if selectedPlayerId.equals(game.getActingPlayer().getPlayerId())
        if selected_id == acting_player_id {
            let has_acted = game.acting_player.has_acted;
            if has_acted && !self.confirmed {
                // Java: showDialog(ConfirmEndAction); CONTINUE
                // DEFERRED(dialog): DialogConfirmEndActionParameter — waiting for dialog infrastructure.
                return StepOutcome::cont();
            } else {
                // Java: game.setTurnMode(lastTurnMode); setTargetSelectionState(cancel()); GOTO_LABEL
                game.turn_mode = game.last_turn_mode.unwrap_or(game.turn_mode);
                game.field_model.target_selection_state =
                    Some({ let mut s = TargetSelectionState::default(); s.cancel(); s });
                if self.goto_label_on_end.is_empty() {
                    return StepOutcome::next();
                }
                return StepOutcome::goto(&self.goto_label_on_end);
            }
        }

        // Case 4: selected player is on the opponent team (not acting team)
        // Java: !game.getActingTeam().hasPlayer(game.getPlayerById(selectedPlayerId))
        let is_on_active_team = game.active_team().player(&selected_id).is_some();
        if !is_on_active_team {
            game.turn_mode = game.last_turn_mode.unwrap_or(game.turn_mode);

            // Java: PlayerState newState = getPlayerState(targetPlayer).changeSelectedGazeTarget(true);
            //       setPlayerState(targetPlayer, newState);
            if let Some(current_state) = game.field_model.player_state(&selected_id) {
                let new_state: PlayerState = current_state.change_selected_gaze_target(true);
                game.field_model.set_player_state(&selected_id, new_state);
            }

            // Java: setTargetSelectionState(new TargetSelectionState(selectedPlayerId).select().commit())
            let mut ts = TargetSelectionState::new(selected_id.clone());
            ts.select();
            ts.commit();
            game.field_model.target_selection_state = Some(ts);

            // Java: getResult().addReport(new ReportSelectGazeTarget(actingPlayer.getId(), selectedId))
            let attacker_id = game.acting_player.player_id.clone().unwrap_or_default();
            return StepOutcome::next().with_event(GameEvent::SelectGazeTarget {
                attacker_id,
                defender_id: selected_id,
            });
        }

        // Case 5: selected player is on the acting team (own teammate, not self)
        // Java: game.setTurnMode(lastTurnMode); NEXT_STEP
        game.turn_mode = game.last_turn_mode.unwrap_or(game.turn_mode);
        StepOutcome::next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{Rules, PS_STANDING};
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn start_no_player_selected_returns_continue() {
        let mut game = make_game();
        let mut step = StepSelectGazeTarget::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
        assert_eq!(game.turn_mode, TurnMode::SelectGazeTarget);
    }

    #[test]
    fn end_turn_pushes_end_player_action_sequence() {
        let mut game = make_game();
        let mut step = StepSelectGazeTarget::new();
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1);
    }

    #[test]
    fn end_player_action_pushes_end_player_action_sequence() {
        let mut game = make_game();
        let mut step = StepSelectGazeTarget::new();
        step.end_player_action = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1);
    }

    #[test]
    fn selecting_self_with_has_acted_returns_continue() {
        let mut game = make_game();
        game.acting_player.player_id = Some("actor".into());
        game.acting_player.has_acted = true;
        let mut step = StepSelectGazeTarget::new();
        step.selected_player_id = Some("actor".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        // has_acted=true, confirmed=false → show confirm dialog → Continue
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn selecting_self_without_has_acted_returns_goto() {
        let mut game = make_game();
        game.acting_player.player_id = Some("actor".into());
        game.acting_player.has_acted = false;
        let mut step = StepSelectGazeTarget::new();
        step.goto_label_on_end = "END".into();
        step.selected_player_id = Some("actor".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label, Some("END".into()));
    }

    #[test]
    fn selecting_opponent_returns_next() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};

        let mut game = make_game();
        game.home_playing = true;
        game.acting_player.player_id = Some("home_p1".into());

        // Add an away player as the target
        let pid = "away_p1".to_string();
        game.team_away.players.push(Player {
            id: pid.clone(), name: "p1".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
        });
        game.field_model.set_player_coordinate(&pid, FieldCoordinate::new(10, 7));
        game.field_model.set_player_state(&pid, PlayerState::new(PS_STANDING));

        let mut step = StepSelectGazeTarget::new();
        step.selected_player_id = Some(pid.clone());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        // The target should have the selected gaze target bit set
        let state = game.field_model.player_state(&pid).unwrap();
        assert!(state.is_selected_gaze_target());
    }

    #[test]
    fn selecting_own_teammate_returns_next() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};

        let mut game = make_game();
        game.home_playing = true;
        game.acting_player.player_id = Some("home_p1".into());

        // Add a home player (same team) as the target
        let pid = "home_p2".to_string();
        game.team_home.players.push(Player {
            id: pid.clone(), name: "p2".into(), nr: 2, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
        });

        let mut step = StepSelectGazeTarget::new();
        step.selected_player_id = Some(pid.clone());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_goto_label_on_end() {
        let mut step = StepSelectGazeTarget::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnEnd("MY_LABEL".into())));
        assert_eq!(step.goto_label_on_end, "MY_LABEL");
    }

    #[test]
    fn set_parameter_end_player_action() {
        let mut step = StepSelectGazeTarget::new();
        assert!(step.set_parameter(&StepParameter::EndPlayerAction(true)));
        assert!(step.end_player_action);
    }

    #[test]
    fn set_parameter_end_turn() {
        let mut step = StepSelectGazeTarget::new();
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.end_turn);
    }

    #[test]
    fn handle_select_player_sets_selected_id() {
        let mut game = make_game();
        game.acting_player.player_id = Some("actor".into());
        let mut step = StepSelectGazeTarget::new();
        // SelectPlayer with an away player id — no away player registered so falls to teammate path
        let out = step.handle_command(
            &Action::SelectPlayer { player_id: "target_p".into() },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(step.selected_player_id.as_deref(), Some("target_p"));
        // Not in any team → treated as active-team player → next
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn handle_end_turn_sets_end_turn() {
        let mut game = make_game();
        let mut step = StepSelectGazeTarget::new();
        let out = step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert!(step.end_turn);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn handle_acknowledge_sets_confirmed() {
        let mut game = make_game();
        game.acting_player.player_id = Some("actor".into());
        game.acting_player.has_acted = true;
        let mut step = StepSelectGazeTarget::new();
        step.selected_player_id = Some("actor".into());
        // First call without confirm → Continue
        let out1 = step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        assert!(step.confirmed);
        // After confirm and has_acted, goto_label_on_end is empty → NextStep
        assert_eq!(out1.action, StepAction::NextStep);
    }

    #[test]
    fn end_turn_sequence_starts_with_remove_target_selection_state() {
        let mut game = make_game();
        let mut step = StepSelectGazeTarget::new();
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Verify the first sequence step is RemoveTargetSelectionState (from EndPlayerAction BB2020)
        assert_eq!(out.pushes[0][0].step_id, StepId::RemoveTargetSelectionState);
    }

    #[test]
    fn selecting_opponent_emits_select_gaze_target_event() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender, PlayerState};
        use std::collections::HashSet;
        let mut game = make_game();
        game.home_playing = true;
        game.acting_player.player_id = Some("gazer".into());
        game.team_away.players.push(Player {
            id: "target".into(), name: "Target".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
        });
        game.field_model.set_player_coordinate("target", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("target", PlayerState::new(PS_STANDING));
        let mut step = StepSelectGazeTarget::new();
        step.selected_player_id = Some("target".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.events.iter().any(|e| matches!(e, GameEvent::SelectGazeTarget {
            defender_id, ..
        } if defender_id == "target")));
    }
}
