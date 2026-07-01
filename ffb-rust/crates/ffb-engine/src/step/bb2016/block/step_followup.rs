/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.block.StepFollowup`.
///
/// Step in block sequence to handle followup.
///
/// Expects stepParameter DEFENDER_POSITION to be set by a preceding step.
/// Expects stepParameter FOLLOWUP_CHOICE to be set by a preceding step.
/// Expects stepParameter OLD_DEFENDER_STATE to be set by a preceding step.
///
/// Sets stepParameter COORDINATE_FROM for all steps on the stack.
/// Sets stepParameter FOLLOWUP_CHOICE for all steps on the stack.
use ffb_model::enums::{PlayerAction, PlayerState};
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_cards::UtilCards;
use ffb_mechanics::skills::SkillId;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::util::UtilServerPlayerMove;

/// Java: `StepFollowup` (bb2016/block).
pub struct StepFollowup {
    /// Java: `coordinateFrom`
    pub coordinate_from: Option<FieldCoordinate>,
    /// Java: `defenderPosition`
    pub defender_position: Option<FieldCoordinate>,
    /// Java: `usingSkillPreventingFollowUp` (Boolean — tristate: null/true/false)
    pub using_skill_preventing_follow_up: Option<bool>,
    /// Java: `followupChoice` (Boolean — tristate: null/true/false)
    pub followup_choice: Option<bool>,
    /// Java: `oldDefenderState`
    pub old_defender_state: Option<PlayerState>,
}

impl StepFollowup {
    pub fn new() -> Self {
        Self {
            coordinate_from: None,
            defender_position: None,
            using_skill_preventing_follow_up: None,
            followup_choice: None,
            old_defender_state: None,
        }
    }

    fn execute_step(&mut self, game: &mut Game) -> StepOutcome {
        let acting_player_id = game.acting_player.player_id.clone();
        let player_action = game.acting_player.player_action;
        let attacker_state = acting_player_id.as_deref()
            .and_then(|id| game.field_model.player_state(id))
            .unwrap_or_default();

        let mut out_params: Vec<StepParameter> = Vec::new();

        // Java: if (attackerState.isRooted()) publishParameter(FOLLOWUP_CHOICE, false)
        if attacker_state.is_rooted() {
            self.followup_choice = Some(false);
            out_params.push(StepParameter::FollowupChoice(false));
        }
        // Java: if (actingPlayer.getPlayerAction() == MULTIPLE_BLOCK)
        //   publishParameter(FOLLOWUP_CHOICE, false)
        if player_action == Some(PlayerAction::MultipleBlock) {
            self.followup_choice = Some(false);
            out_params.push(StepParameter::FollowupChoice(false));
        }

        if self.followup_choice.is_none() {
            let defender_state = game.defender_id.as_deref()
                .and_then(|id| game.field_model.player_state(id))
                .unwrap_or_default();
            let old_defender_state = self.old_defender_state.unwrap_or_default();

            // Java: Skill skillPreventsFollowingUp =
            //   game.getDefender().getSkillWithProperty(NamedProperties.preventOpponentFollowingUp)
            let defender_has_fend = game.defender_id.as_deref()
                .and_then(|id| game.player(id))
                .map(|p| p.has_skill_property(NamedProperties::PREVENT_OPPONENT_FOLLOWING_UP))
                .unwrap_or(false);

            // Java: if (skillPreventsFollowingUp != null && !defenderState.isProneOrStunned()
            //   && !((oldDefenderState != null) && oldDefenderState.isProneOrStunned()))
            if defender_has_fend
                && !defender_state.is_prone_or_stunned()
                && !old_defender_state.is_prone_or_stunned()
            {
                // Java: Skill skillCancelsSkillPreventingFollow = UtilCards.getSkillCancelling(actingPlayer, fendSkill)
                // Check if attacker has a skill (e.g. Juggernaut) that cancels the fend property.
                let attacker_cancels_fend = acting_player_id.as_deref()
                    .and_then(|id| game.player(id))
                    .map(|p| UtilCards::has_skill_to_cancel_property(p, NamedProperties::PREVENT_OPPONENT_FOLLOWING_UP))
                    .unwrap_or(false);
                if self.using_skill_preventing_follow_up.is_none() && attacker_cancels_fend {
                    let action = game.acting_player.player_action.unwrap_or(PlayerAction::Block);
                    let is_blitz_or_blocks_during_move = action == PlayerAction::Blitz
                        || action == PlayerAction::BlitzMove
                        || (action == PlayerAction::Move && acting_player_id.as_deref()
                            .and_then(|id| game.player(id))
                            .map(|p| p.has_skill_property(NamedProperties::BLOCKS_DURING_MOVE))
                            .unwrap_or(false));
                    if is_blitz_or_blocks_during_move {
                        // Java: usingSkillPreventingFollowUp = false; cancelSkillUsed = true; report CANCEL_FEND
                        self.using_skill_preventing_follow_up = Some(false);
                    }
                }

                // Java: boolean defenderHasTacklezones = oldDefenderState.hasTacklezones()
                let defender_had_tacklezones = old_defender_state.has_tacklezones();

                if self.using_skill_preventing_follow_up.is_none() {
                    if !defender_had_tacklezones {
                        // Java: report NO_TACKLEZONE; usingSkillPreventingFollowUp = false
                        self.using_skill_preventing_follow_up = Some(false);
                    } else {
                        // Java: UtilServerDialog.showDialog(... DialogSkillUseParameter(defenderId, skill, 0) ...) → CONTINUE
                        return build_outcome(out_params, StepOutcome::cont());
                    }
                } else {
                    if self.using_skill_preventing_follow_up == Some(true) {
                        // Java: publishParameter(FOLLOWUP_CHOICE, false)
                        self.followup_choice = Some(false);
                        out_params.push(StepParameter::FollowupChoice(false));
                    }
                    // Java: if (!cancelSkillUsed) report skillUse (STAY_AWAY_FROM_OPPONENT / true/false)
                }
            } else {
                self.using_skill_preventing_follow_up = Some(false);
            }

            // Java: if (usingSkillPreventingFollowUp != null && !usingSkillPreventingFollowUp
            //   && activePlayer.hasSkillProperty(forceFollowup))
            //   publishParameter(FOLLOWUP_CHOICE, true)
            if self.using_skill_preventing_follow_up == Some(false) {
                let attacker_force_followup = acting_player_id.as_deref()
                    .and_then(|id| game.player(id))
                    .map(|p| p.has_skill_property(NamedProperties::FORCE_FOLLOWUP))
                    .unwrap_or(false);
                if attacker_force_followup {
                    self.followup_choice = Some(true);
                    out_params.push(StepParameter::FollowupChoice(true));
                }
            }

            // Java: if ((followupChoice == null) && (usingSkillPreventingFollowUp != null))
            //   UtilServerDialog.showDialog(... DialogFollowupChoiceParameter ...) → CONTINUE
            if self.followup_choice.is_none() && self.using_skill_preventing_follow_up.is_some() {
                return build_outcome(out_params, StepOutcome::cont());
            }
        }

        // Java: if (followupChoice != null) { ... }
        if let Some(choice) = self.followup_choice {
            let mut outcome = StepOutcome::next();
            for p in out_params {
                outcome = outcome.publish(p);
            }

            if choice {
                // Java: followupCoordinate = defenderPosition
                // Java: publishParameter(COORDINATE_FROM, playerCoordinate(actingPlayer))
                // Java: updatePlayerAndBallPosition(actingPlayer, followupCoordinate)
                // Java: publishParameter(PLAYER_ENTERING_SQUARE, actingPlayerId)
                // Java: updateMoveSquares(gameState, false)
                // Java: if (BLITZ) { trackNumber = new TrackNumber(coordinateFrom, currentMove - 1); add }
                // Java: getResult().setSound(SoundId.STEP)
                if let (Some(ref attacker_id), Some(followup_coord)) =
                    (acting_player_id.clone(), self.defender_position)
                {
                    let current_coord = game.field_model.player_coordinate(attacker_id)
                        .unwrap_or(FieldCoordinate::new(0, 0));
                    outcome = outcome.publish(StepParameter::CoordinateFrom(current_coord));

                    // Java: updatePlayerAndBallPosition — ball follows if carried
                    if !game.field_model.ball_moving {
                        if let (Some(ball), Some(old_pos)) = (game.field_model.ball_coordinate, Some(current_coord)) {
                            if ball == old_pos {
                                game.field_model.ball_coordinate = Some(followup_coord);
                            }
                        }
                    }
                    game.field_model.set_player_coordinate(attacker_id, followup_coord);
                    UtilServerPlayerMove::update_move_squares(game, false);
                    outcome = outcome.publish(StepParameter::PlayerEnteringSquare(attacker_id.clone()));
                }
            } else {
                // Java: publishParameter(COORDINATE_FROM, null) → null/zero coord
                outcome = outcome.publish(StepParameter::CoordinateFrom(FieldCoordinate::new(0, 0)));
            }

            // Java: publishParameter(DEFENDER_POSITION,
            //   game.getFieldModel().getPlayerCoordinate(game.getDefender()))
            let defender_pos = game.defender_id.as_deref()
                .and_then(|id| game.field_model.player_coordinate(id));
            if let Some(pos) = defender_pos {
                outcome = outcome.publish(StepParameter::DefenderPosition(pos));
            }
            // Java: getResult().setNextAction(StepAction.NEXT_STEP)
            return outcome;
        }

        build_outcome(out_params, StepOutcome::cont())
    }
}

fn build_outcome(params: Vec<StepParameter>, mut base: StepOutcome) -> StepOutcome {
    for p in params {
        base = base.publish(p);
    }
    base
}

impl Default for StepFollowup {
    fn default() -> Self { Self::new() }
}

impl Step for StepFollowup {
    fn id(&self) -> StepId { StepId::Followup }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: if (commandStatus == UNHANDLED_COMMAND) {
        //   case CLIENT_USE_SKILL: if (skill.hasProperty(preventOpponentFollowingUp))
        //     usingSkillPreventingFollowUp = isSkillUsed
        //   case CLIENT_FOLLOWUP_CHOICE:
        //     publishParameter(FOLLOWUP_CHOICE, isChoiceFollowup) }
        match action {
            Action::UseSkill { skill_id, use_skill } => {
                // Java: useSkillCommand.getSkill().hasSkillProperty(preventOpponentFollowingUp)
                if *skill_id == SkillId::Fend {
                    self.using_skill_preventing_follow_up = Some(*use_skill);
                }
            }
            Action::FollowUp { follow_up } => {
                // Java: publishParameter(FOLLOWUP_CHOICE, followupChoiceCommand.isChoiceFollowup())
                // The publish updates our own followupChoice via the set_parameter path.
                self.followup_choice = Some(*follow_up);
            }
            _ => {}
        }
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            // Java: COORDINATE_FROM
            StepParameter::CoordinateFrom(v) => { self.coordinate_from = Some(*v); true }
            // Java: DEFENDER_POSITION
            StepParameter::DefenderPosition(v) => { self.defender_position = Some(*v); true }
            // Java: FOLLOWUP_CHOICE
            StepParameter::FollowupChoice(v) => { self.followup_choice = Some(*v); true }
            // Java: OLD_DEFENDER_STATE
            StepParameter::OldDefenderState(v) => { self.old_defender_state = Some(*v); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{Rules, PS_STANDING};
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    #[test]
    fn id_is_followup() {
        assert_eq!(StepFollowup::new().id(), StepId::Followup);
    }

    #[test]
    fn followup_false_publishes_coordinate_from_and_next() {
        // Java: followupChoice = false → publishParameter(COORDINATE_FROM, null) → NEXT_STEP
        let mut step = StepFollowup::new();
        step.followup_choice = Some(false);
        step.using_skill_preventing_follow_up = Some(false);
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::CoordinateFrom(_))));
    }

    #[test]
    fn no_choice_no_defender_stays_cont() {
        // Java: when no followupChoice and no fend skill dialog needed → CONTINUE (show dialog)
        let mut step = StepFollowup::new();
        step.using_skill_preventing_follow_up = Some(false);
        // No followupChoice set yet
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Should wait for followup dialog → CONTINUE
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn set_parameter_accepted() {
        let mut step = StepFollowup::new();
        let coord = FieldCoordinate::new(3, 7);
        assert!(step.set_parameter(&StepParameter::DefenderPosition(coord)));
        assert!(step.set_parameter(&StepParameter::FollowupChoice(true)));
        assert!(step.set_parameter(&StepParameter::OldDefenderState(PlayerState::new(PS_STANDING))));
        assert!(step.set_parameter(&StepParameter::CoordinateFrom(coord)));
        assert_eq!(step.defender_position, Some(coord));
        assert_eq!(step.followup_choice, Some(true));
        assert!(step.old_defender_state.is_some());
        assert!(step.coordinate_from.is_some());
    }

    #[test]
    fn multiple_block_forces_no_followup() {
        // Java: if (playerAction == MULTIPLE_BLOCK) publishParameter(FOLLOWUP_CHOICE, false)
        let mut step = StepFollowup::new();
        step.using_skill_preventing_follow_up = Some(false);
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::MultipleBlock);
        let out = step.start(&mut game, &mut GameRng::new(0));
        // followupChoice forced to false → NEXT_STEP
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn followup_true_returns_next_step() {
        // Java: followupChoice = true → NEXT_STEP (no acting player set → skips move)
        let mut step = StepFollowup::new();
        step.followup_choice = Some(true);
        step.defender_position = Some(FieldCoordinate::new(6, 5));
        step.using_skill_preventing_follow_up = Some(false);
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn juggernaut_cancels_fend_on_blitz() {
        use ffb_model::enums::{PlayerType, PlayerGender, SkillId};
        use ffb_model::model::player::Player;
        use ffb_model::model::skill_def::SkillWithValue;
        // Setup: defender has Fend, attacker has Juggernaut on a Blitz action.
        let mut step = StepFollowup::new();
        step.old_defender_state = Some(PlayerState::new(PS_STANDING)); // PS_STANDING has tacklezones
        let mut game = make_game();
        // Add attacker with Juggernaut
        game.team_home.players.push(Player {
            id: "att".into(), name: "att".into(), nr: 1, position_id: "p".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![SkillWithValue { skill_id: SkillId::Juggernaut, value: None }],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
        });
        game.field_model.set_player_coordinate("att", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("att", PlayerState::new(PS_STANDING));
        game.acting_player.player_id = Some("att".into());
        game.acting_player.player_action = Some(PlayerAction::Blitz);
        // Add defender with Fend
        game.team_away.players.push(Player {
            id: "def".into(), name: "def".into(), nr: 1, position_id: "p".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![SkillWithValue { skill_id: SkillId::Fend, value: None }],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
        });
        game.field_model.set_player_coordinate("def", FieldCoordinate::new(6, 5));
        game.field_model.set_player_state("def", PlayerState::new(PS_STANDING));
        game.defender_id = Some("def".into());
        // Juggernaut should auto-cancel Fend → using_skill_preventing_follow_up = Some(false)
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(step.using_skill_preventing_follow_up, Some(false));
    }
}
