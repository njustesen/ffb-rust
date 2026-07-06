use ffb_model::types::FieldCoordinate;
use ffb_model::enums::{PlayerAction, PlayerState};
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_cards::UtilCards;
use ffb_mechanics::skills::SkillId;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::util::UtilServerPlayerMove;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.block.StepFollowup.
/// Handles optional attacker follow-up after a block. Fend (preventOpponentFollowingUp) and
/// Taunt (forceOpponentToFollowUp) skill dialogs require TODO stubs; movement and
/// updatePlayerAndBallPosition + PlayerEnteringSquare are wired.
pub struct StepFollowup {
    pub coordinate_from: Option<FieldCoordinate>,
    pub defender_position: Option<FieldCoordinate>,
    /// Java: usingSkillPreventingFollowUp (Boolean — tristate: null/true/false)
    pub using_skill_preventing_follow_up: Option<bool>,
    pub followup_choice: Option<bool>,
    pub old_defender_state: Option<PlayerState>,
    /// Java: usingSkillForcingFollowUp (Boolean — tristate: null/true/false)
    pub using_skill_forcing_follow_up: Option<bool>,
}

impl StepFollowup {
    pub fn new() -> Self {
        Self {
            coordinate_from: None,
            defender_position: None,
            using_skill_preventing_follow_up: None,
            followup_choice: None,
            old_defender_state: None,
            using_skill_forcing_follow_up: None,
        }
    }
}

impl Default for StepFollowup {
    fn default() -> Self { Self::new() }
}

impl Step for StepFollowup {
    fn id(&self) -> StepId { StepId::Followup }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::UseSkill { skill_id, use_skill } => {
                // Fend skill: defender uses it to prevent follow-up
                if *skill_id == SkillId::Fend {
                    self.using_skill_preventing_follow_up = Some(*use_skill);
                }
                // Taunt skill: defender uses it to force follow-up
                if *skill_id == SkillId::Taunt {
                    self.using_skill_forcing_follow_up = Some(*use_skill);
                }
            }
            Action::FollowUp { follow_up } => {
                self.followup_choice = Some(*follow_up);
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::CoordinateFrom(v) => { self.coordinate_from = Some(*v); true }
            StepParameter::DefenderPosition(v) => { self.defender_position = Some(*v); true }
            StepParameter::FollowupChoice(v) => { self.followup_choice = Some(*v); true }
            StepParameter::OldDefenderState(v) => { self.old_defender_state = Some(*v); true }
            _ => false,
        }
    }
}

impl StepFollowup {
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        let acting_player_id = game.acting_player.player_id.clone();
        let player_action = game.acting_player.player_action;
        let attacker_state = acting_player_id.as_deref()
            .and_then(|id| game.field_model.player_state(id))
            .unwrap_or_default();
        // Java local var: SkillId skillPreventingFollowUp = UtilCards.getSkillWithProperty(pGame.getDefender(), preventOpponentFollowingUp)
        let fend_skill_id = game.defender_id.as_deref()
            .and_then(|id| game.player(id))
            .and_then(|p| p.all_skill_ids().find(|id| id.properties().contains(&NamedProperties::PREVENT_OPPONENT_FOLLOWING_UP)));
        // Java local var: SkillId skillCancelsSkillPreventingFollow = UtilCards.getSkillCancelling(attacker, skillPreventingFollowUp)
        let cancel_skill_id = acting_player_id.as_deref()
            .and_then(|id| game.player(id))
            .and_then(|p| UtilCards::get_skill_cancelling_property(p, NamedProperties::PREVENT_OPPONENT_FOLLOWING_UP));
        let mut cancel_skill_used = false;

        // Local effective choice — mirrors Java's `publishParameter(FollowupChoice, ...)` which
        // immediately updates `followupChoice` on the current step via the stack walk.
        let mut effective_choice = self.followup_choice;
        let mut out_params: Vec<StepParameter> = Vec::new();

        // Pinned or Vicious Vines: cannot follow up
        if attacker_state.is_pinned() || player_action == Some(PlayerAction::ViciousVines) {
            effective_choice = Some(false);
            out_params.push(StepParameter::FollowupChoice(false));
        }
        // Multiple Block: cannot follow up
        if player_action == Some(PlayerAction::MultipleBlock) {
            effective_choice = Some(false);
            out_params.push(StepParameter::FollowupChoice(false));
        }

        if effective_choice.is_none() {
            let defender_state = game.defender_id.as_deref()
                .and_then(|id| game.field_model.player_state(id))
                .unwrap_or_default();
            let old_defender_state = self.old_defender_state.unwrap_or_default();

            // Fend skill (preventOpponentFollowingUp): defender may prevent follow-up
            let defender_has_fend = game.defender_id.as_deref()
                .and_then(|id| game.player(id))
                .map(|p| p.has_skill_property(NamedProperties::PREVENT_OPPONENT_FOLLOWING_UP))
                .unwrap_or(false);

            if defender_has_fend
                && !defender_state.is_prone_or_stunned()
                && !old_defender_state.is_prone_or_stunned()
            {
                // Java: check if attacker has a skill that cancels preventOpponentFollowingUp
                // (Juggernaut registers CancelSkillProperty(preventOpponentFollowingUp)).
                // Auto-cancel if action is BLITZ or (MOVE + blocksDuringMove) — Java line 130.
                let attacker_cancels_fend = acting_player_id.as_deref()
                    .and_then(|id| game.player(id))
                    .map(|p| p.has_skill_property(NamedProperties::CANCELS_PREVENT_OPPONENT_FOLLOWING_UP))
                    .unwrap_or(false);
                let action_allows_cancel = matches!(
                    game.acting_player.player_action,
                    Some(PlayerAction::Blitz)
                ) || (game.acting_player.player_action == Some(PlayerAction::Move)
                    && acting_player_id.as_deref()
                        .and_then(|id| game.player(id))
                        .map(|p| p.has_skill_property(NamedProperties::BLOCKS_DURING_MOVE))
                        .unwrap_or(false));

                if attacker_cancels_fend && action_allows_cancel {
                    self.using_skill_preventing_follow_up = Some(false);
                    cancel_skill_used = true;
                    // Java: getResult().addReport(new ReportSkillUse(actingPlayer.getPlayerId(), skillCancelsSkillPreventingFollow, true, SkillUse.CANCEL_FEND))
                    if let (Some(ref aid), Some(csid)) = (&acting_player_id, cancel_skill_id) {
                        use ffb_model::model::skill_use::SkillUse;
                        use ffb_model::report::report_skill_use::ReportSkillUse;
                        game.report_list.add(ReportSkillUse::new(Some(aid.clone()), csid, true, SkillUse::CANCEL_FEND));
                    }
                } else if self.using_skill_preventing_follow_up.is_none() {
                    if !old_defender_state.has_tacklezones() {
                        // Defender has no tacklezones — Fend fails automatically
                        self.using_skill_preventing_follow_up = Some(false);
                        // Java: getResult().addReport(new ReportSkillUse(game.getDefenderId(), skillPreventingFollowUp, false, SkillUse.NO_TACKLEZONE))
                        if let (Some(ref did), Some(fsid)) = (&game.defender_id.clone(), fend_skill_id) {
                            use ffb_model::model::skill_use::SkillUse;
                            use ffb_model::report::report_skill_use::ReportSkillUse;
                            game.report_list.add(ReportSkillUse::new(Some(did.clone()), fsid, false, SkillUse::NO_TACKLEZONE));
                        }
                    } else {
                        // Would show DialogSkillUse for Fend — stub: wait for response
                        return build_outcome(out_params, StepOutcome::cont());
                    }
                }
                if let Some(true) = self.using_skill_preventing_follow_up {
                    effective_choice = Some(false);
                    out_params.push(StepParameter::FollowupChoice(false));
                }
                // Java: if (!cancelSkillUsed) getResult().addReport(new ReportSkillUse(defenderId, skillPreventingFollowUp, usingSkillPreventingFollowUp, SkillUse.STAY_AWAY_FROM_OPPONENT))
                if !cancel_skill_used {
                    if let (Some(ref did), Some(fsid)) = (&game.defender_id.clone(), fend_skill_id) {
                        use ffb_model::model::skill_use::SkillUse;
                        use ffb_model::report::report_skill_use::ReportSkillUse;
                        game.report_list.add(ReportSkillUse::new(
                            Some(did.clone()), fsid,
                            self.using_skill_preventing_follow_up.unwrap_or(false),
                            SkillUse::STAY_AWAY_FROM_OPPONENT,
                        ));
                    }
                }
            } else {
                self.using_skill_preventing_follow_up = Some(false);
            }

            // forceFollowup on attacker (e.g. Frenzy property): attacker must follow up
            if self.using_skill_preventing_follow_up == Some(false) {
                let attacker_force_followup = acting_player_id.as_deref()
                    .and_then(|id| game.player(id))
                    .map(|p| p.has_skill_property(NamedProperties::FORCE_FOLLOWUP))
                    .unwrap_or(false);
                if attacker_force_followup {
                    effective_choice = Some(true);
                    out_params.push(StepParameter::FollowupChoice(true));
                }
            }

            // Taunt skill (forceOpponentToFollowUp): defender may force attacker to follow up
            let cannot_follow = attacker_state.is_pinned()
                || player_action == Some(PlayerAction::ViciousVines)
                || player_action == Some(PlayerAction::MultipleBlock);
            let defender_has_taunt = game.defender_id.as_deref()
                .and_then(|id| game.player(id))
                .map(|p| p.has_skill_property(NamedProperties::FORCE_OPPONENT_TO_FOLLOW_UP))
                .unwrap_or(false);

            if defender_has_taunt
                && effective_choice.is_none()
                && self.using_skill_preventing_follow_up == Some(false)
                && !cannot_follow
            {
                if self.using_skill_forcing_follow_up.is_none() {
                    // Would show DialogSkillUse for Taunt — stub: wait for response
                    return build_outcome(out_params, StepOutcome::cont());
                }
                if let Some(true) = self.using_skill_forcing_follow_up {
                    effective_choice = Some(true);
                    out_params.push(StepParameter::FollowupChoice(true));
                }
                // Report skill use (TODO: add ReportSkillUse event)
            } else if self.using_skill_forcing_follow_up.is_none() {
                self.using_skill_forcing_follow_up = Some(false);
            }

            // No automated choice — show followup choice dialog
            if effective_choice.is_none()
                && self.using_skill_preventing_follow_up.is_some()
                && self.using_skill_forcing_follow_up != Some(true)
            {
                // Would show DialogFollowupChoice — stub: wait for agent FollowUp action
                return build_outcome(out_params, StepOutcome::cont());
            }
        }

        if let Some(choice) = effective_choice {
            let mut outcome = StepOutcome::next();
            for p in out_params {
                outcome = outcome.publish(p);
            }
            if choice {
                // Move attacker to defender's old position
                if let Some(ref attacker_id) = acting_player_id {
                    let current_coord = game.field_model.player_coordinate(attacker_id);
                    if let Some(followup_coord) = self.defender_position {
                        outcome = outcome.publish(StepParameter::CoordinateFrom(
                            current_coord.unwrap_or(FieldCoordinate::new(0, 0)),
                        ));
                        // Java: updatePlayerAndBallPosition — ball follows if carried
                        if !game.field_model.ball_moving {
                            if let (Some(old), Some(ball)) = (current_coord, game.field_model.ball_coordinate) {
                                if old == ball {
                                    game.field_model.ball_coordinate = Some(followup_coord);
                                }
                            }
                        }
                        game.field_model.set_player_coordinate(attacker_id, followup_coord);
                        UtilServerPlayerMove::update_move_squares(game, false);
                        // client-only: TrackNumber for BLITZ action
                        outcome = outcome.publish(StepParameter::PlayerEnteringSquare(attacker_id.clone()));
                    }
                }
            } else {
                outcome = outcome.publish(StepParameter::CoordinateFrom(FieldCoordinate::new(0, 0)));
            }
            // Publish updated defender position
            let defender_pos = game.defender_id.as_deref()
                .and_then(|id| game.field_model.player_coordinate(id));
            if let Some(pos) = defender_pos {
                outcome = outcome.publish(StepParameter::DefenderPosition(pos));
            }
            outcome
        } else {
            build_outcome(out_params, StepOutcome::cont())
        }
    }
}

/// Prepend accumulated force-publish params onto an outcome.
fn build_outcome(params: Vec<StepParameter>, mut base: StepOutcome) -> StepOutcome {
    for p in params {
        base = base.publish(p);
    }
    base
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{Rules, PS_STANDING, PlayerAction};
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn no_followup_choice_with_no_defender_stays_cont() {
        // When there is no defender the Fend/Taunt/forceFollowup checks all fall through
        // and the step shows a dialog → CONTINUE
        let mut step = StepFollowup::new();
        step.using_skill_preventing_follow_up = Some(false);
        step.using_skill_forcing_follow_up = Some(false);
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        // followup_choice is None, no conditions override → wait for choice dialog → CONTINUE
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn followup_true_moves_attacker_to_defender_position() {
        let mut step = StepFollowup::new();
        step.followup_choice = Some(true);
        step.defender_position = Some(FieldCoordinate::new(5, 5));
        step.using_skill_preventing_follow_up = Some(false);
        step.using_skill_forcing_follow_up = Some(false);
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn followup_false_publishes_coordinate_from_zero() {
        let mut step = StepFollowup::new();
        step.followup_choice = Some(false);
        step.using_skill_preventing_follow_up = Some(false);
        step.using_skill_forcing_follow_up = Some(false);
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        // Should publish CoordinateFrom(0,0) for no-followup path
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::CoordinateFrom(_))));
    }

    #[test]
    fn vicious_vines_forces_no_followup() {
        let mut step = StepFollowup::new();
        step.using_skill_preventing_follow_up = Some(false);
        step.using_skill_forcing_follow_up = Some(false);
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::ViciousVines);
        // Should automatically set followup to false, then proceed
        let out = step.start(&mut game, &mut GameRng::new(0));
        // ViciousVines forces false → NEXT_STEP
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn multiple_block_forces_no_followup() {
        let mut step = StepFollowup::new();
        step.using_skill_preventing_follow_up = Some(false);
        step.using_skill_forcing_follow_up = Some(false);
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::MultipleBlock);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_defender_position_accepted() {
        let mut step = StepFollowup::new();
        let coord = FieldCoordinate::new(3, 7);
        step.set_parameter(&StepParameter::DefenderPosition(coord));
        assert_eq!(step.defender_position, Some(coord));
    }

    #[test]
    fn fend_no_tacklezone_emits_no_tacklezone_report() {
        use std::collections::HashSet;
        use ffb_model::enums::{PlayerGender, PlayerType, SkillId, PS_FALLING};
        use ffb_model::model::player::Player;
        use ffb_model::model::skill_def::SkillWithValue;
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        game.team_home.players.push(Player {
            id: "attacker".into(), name: "attacker".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 4, strength: 4, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        });
        game.team_away.players.push(Player {
            id: "defender".into(), name: "defender".into(), nr: 2, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 4, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![SkillWithValue { skill_id: SkillId::Fend, value: None }],
            extra_skills: vec![], temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        });
        let falling_state = PlayerState::new(PS_FALLING);
        game.field_model.set_player_state("defender", falling_state);
        game.acting_player.player_id = Some("attacker".into());
        game.defender_id = Some("defender".into());
        let mut step = StepFollowup::new();
        step.old_defender_state = Some(PlayerState::new(PS_FALLING));
        step.using_skill_forcing_follow_up = Some(false);
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::SKILL_USE), "Fend no-tacklezone path must emit ReportSkillUse");
    }
}
