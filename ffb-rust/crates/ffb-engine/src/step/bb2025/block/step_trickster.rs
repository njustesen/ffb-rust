use ffb_model::types::{FieldCoordinate, MoveSquare};
use ffb_model::enums::TurnMode;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_cards::UtilCards;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// Internal state machine phase for StepTrickster.
/// Java: ActionStatus (WAITING_FOR_SKILL_USE / SKILL_CHOICE_YES).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TricksterPhase {
    WaitingForSkillUse,
    SkillChoiceYes,
}

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.block.StepTrickster.
/// Before a block, defender with Trickster (canMoveBeforeBeingBlocked) may move to an adjacent
/// empty non-blocked square. Switches acting team to show eligible squares to the defender's coach.
pub struct StepTrickster {
    pub using_stab: bool,
    pub using_chainsaw: bool,
    pub using_vomit: bool,
    pub using_breathe_fire: bool,
    pub using_chomp: bool,
    pub last_turn_mode: Option<TurnMode>,
    pub eligible_squares: Vec<FieldCoordinate>,
    /// Java: usingTrickster (Boolean — tristate)
    pub using_trickster: Option<bool>,
    pub to_coordinate: Option<FieldCoordinate>,
    pub with_ball: bool,
    /// Java: attemptPickUp (Boolean — tristate)
    pub attempt_pick_up: Option<bool>,
    action_status: TricksterPhase,
}

impl StepTrickster {
    pub fn new() -> Self {
        Self {
            using_stab: false,
            using_chainsaw: false,
            using_vomit: false,
            using_breathe_fire: false,
            using_chomp: false,
            last_turn_mode: None,
            eligible_squares: Vec::new(),
            using_trickster: None,
            to_coordinate: None,
            with_ball: false,
            attempt_pick_up: None,
            action_status: TricksterPhase::WaitingForSkillUse,
        }
    }
}

impl Default for StepTrickster {
    fn default() -> Self { Self::new() }
}

impl Step for StepTrickster {
    fn id(&self) -> StepId { StepId::Trickster }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if self.action_status == TricksterPhase::SkillChoiceYes {
            // Waiting for pick-up choice (ball on destination square)
            if let Action::Acknowledge = action {
                // Java: CLIENT_PICK_UP_CHOICE → attemptPickUp
                self.attempt_pick_up = Some(true); // headless: decode from action — command parsing not yet ported
            }
        } else {
            match action {
                Action::UseSkill { skill_id: _, use_skill } => {
                    // Java: CLIENT_USE_SKILL from passive player with canMoveBeforeBeingBlocked
                    self.using_trickster = Some(*use_skill);
                }
                Action::TricksterMove { coord } => {
                    if self.eligible_squares.contains(coord) {
                        self.to_coordinate = Some(*coord);
                    }
                }
                Action::EndTurn => {
                    // Java: CLIENT_END_TURN → leave()
                    return self.leave(game);
                }
                _ => {}
            }
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::UsingStab(v) => { self.using_stab = *v; false } // Java: returns false!
            StepParameter::UsingChainsaw(v) => { self.using_chainsaw = *v; false }
            StepParameter::UsingVomit(v) => { self.using_vomit = *v; false }
            StepParameter::UsingBreatheFire(v) => { self.using_breathe_fire = *v; false }
            StepParameter::UsingChomp(v) => { self.using_chomp = *v; false }
            _ => false,
        }
    }
}

impl StepTrickster {
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        let defender_id = game.defender_id.clone();
        let defender_state = defender_id.as_deref()
            .and_then(|id| game.field_model.player_state(id))
            .unwrap_or_default();

        // If no defender or defender has no tackle zones, skip Trickster
        if defender_id.is_none() || !defender_state.has_tacklezones() {
            self.using_trickster = Some(false);
        }

        if self.using_trickster.is_none() {
            // Check if defender has Trickster skill and eligible squares exist
            let defender_has_trickster = defender_id.as_deref()
                .and_then(|id| game.player(id))
                .map(|p| p.has_skill_property(NamedProperties::CAN_MOVE_BEFORE_BEING_BLOCKED))
                .unwrap_or(false);

            let attacker_cancels = game.acting_player.player_id.as_deref()
                .and_then(|id| game.player(id))
                .map(|p| UtilCards::has_skill_to_cancel_property(p, NamedProperties::CAN_MOVE_BEFORE_BEING_BLOCKED))
                .unwrap_or(false);

            if defender_has_trickster
                && (self.using_chainsaw || self.using_vomit || self.using_stab
                    || self.using_breathe_fire || self.using_chomp || !attacker_cancels)
            {
                // Calculate eligible squares: adjacent to attacker, no player, not blocked for trickster
                let attacker_coord = game.acting_player.player_id.as_deref()
                    .and_then(|id| game.field_model.player_coordinate(id));
                if let Some(att_coord) = attacker_coord {
                    self.eligible_squares = game.field_model
                        .adjacent_on_pitch(att_coord)
                        .into_iter()
                        .filter(|&c| game.field_model.player_at(c).is_none())
                        .filter(|&c| !game.field_model.is_blocked_for_trickster(c))
                        .collect();
                }

                if self.eligible_squares.is_empty() {
                    return StepOutcome::next();
                }

                // Would show DialogSkillUse for Trickster to passive player — stub: cont
                return StepOutcome::cont();
            } else {
                return StepOutcome::next();
            }
        }

        if self.using_trickster == Some(true) {
            if self.to_coordinate.is_none() {
                // Show eligible squares to defender's coach
                self.last_turn_mode = Some(game.turn_mode);
                game.turn_mode = TurnMode::Trickster;
                // Java: game.setHomePlaying(!game.isHomePlaying()) — switch acting team
                game.home_playing = !game.home_playing;
                game.field_model.clear_move_squares();
                for &c in &self.eligible_squares {
                    game.field_model.add_move_square(MoveSquare::new(c, 0, 0));
                }
                return StepOutcome::cont();
            } else if self.action_status == TricksterPhase::WaitingForSkillUse {
                // Java: move defender and update state — then push current step for pick-up
                let def_id = defender_id.clone().unwrap_or_default();
                let to = self.to_coordinate.unwrap();

                // Java: fieldModel.replaceMultiBlockTargetCoordinate(defCoordinate, toCoordinate)
                if let Some(old_coord) = game.field_model.player_coordinate(&def_id) {
                    game.field_model.replace_multi_block_target_coordinate(old_coord, to);
                }

                // Check if defender has ball
                self.with_ball = game.field_model.ball_coordinate
                    .map(|bc| bc == game.field_model.player_coordinate(&def_id).unwrap_or(FieldCoordinate::new(-1, -1)))
                    .unwrap_or(false);

                self.action_status = TricksterPhase::SkillChoiceYes;
                // Java: getResult().setNextAction(NEXT_STEP_AND_REPEAT); getGameState().pushCurrentStepOnStack()
                // This means: advance + re-deliver same command. In Rust: just publish DefenderPosition and cont
                return StepOutcome::next()
                    .publish(StepParameter::DefenderPosition(to));
            } else {
                // Actually move the defender
                let def_id = defender_id.clone().unwrap_or_default();
                let to = self.to_coordinate.unwrap();
                game.field_model.set_player_coordinate(&def_id, to);

                let mut outcome = StepOutcome::next()
                    .publish(StepParameter::DefenderPosition(to));

                if self.with_ball {
                    game.field_model.ball_coordinate = Some(to);
                }

                // If ball on destination (not carrying), handle pick-up
                if !self.with_ball
                    && game.field_model.ball_coordinate == Some(to)
                    && game.field_model.ball_moving
                {
                    if self.attempt_pick_up.is_none() {
                        // Would show DialogPickUpChoice — stub: default to attempt
                        self.attempt_pick_up = Some(false);
                    }
                    if let Some(true) = self.attempt_pick_up {
                        // Java: publish AttemptPickUp=true, PlayerOnBallId=mover, PickUpOptional=false
                        let mover = game.acting_player.player_id.clone().unwrap_or_default();
                        outcome = outcome
                            .publish(StepParameter::AttemptPickUp(true))
                            .publish(StepParameter::PlayerOnBallId(mover))
                            .publish(StepParameter::PickUpOptional(false));
                    } else {
                        // Java: publish CatchScatterThrowInMode::ScatterBall
                        outcome = outcome.publish(StepParameter::CatchScatterThrowInMode(
                            crate::step::framework::CatchScatterThrowInMode::ScatterBall,
                        ));
                    }
                }

                outcome = outcome.publish(StepParameter::DefenderPosition(to));
                return self.leave_outcome(game, outcome);
            }
        }

        // using_trickster == Some(false)
        StepOutcome::next()
    }

    fn leave(&mut self, game: &mut Game) -> StepOutcome {
        game.field_model.clear_move_squares();
        // Java: game.setHomePlaying(!game.isHomePlaying()) — switch back
        game.home_playing = !game.home_playing;
        if let Some(mode) = self.last_turn_mode {
            game.turn_mode = mode;
        }
        StepOutcome::next()
    }

    fn leave_outcome(&mut self, game: &mut Game, base: StepOutcome) -> StepOutcome {
        game.field_model.clear_move_squares();
        // Java: game.setHomePlaying(!game.isHomePlaying()) — switch back
        game.home_playing = !game.home_playing;
        if let Some(mode) = self.last_turn_mode {
            game.turn_mode = mode;
        }
        base
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
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn no_defender_skips_trickster() {
        let mut step = StepTrickster::new();
        let mut game = make_game();
        // No defender set → using_trickster forced to false → NEXT_STEP
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(step.using_trickster, Some(false));
    }

    #[test]
    fn using_trickster_false_returns_next_step() {
        let mut step = StepTrickster::new();
        step.using_trickster = Some(false);
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_using_flags_not_consumed() {
        // Java: these params return false (not consumed) — they peek but don't own the param
        let mut step = StepTrickster::new();
        let result = step.set_parameter(&StepParameter::UsingStab(true));
        assert!(!result); // Java returns false for all skill flags
        assert!(step.using_stab); // but the value is still stored
    }

    #[test]
    fn end_turn_command_calls_leave_and_returns_next_step() {
        let mut step = StepTrickster::new();
        step.using_trickster = Some(true);
        step.last_turn_mode = Some(TurnMode::Regular);
        let mut game = make_game();
        game.turn_mode = TurnMode::Trickster;
        let out = step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        // leave() restores TurnMode and returns NEXT_STEP
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.turn_mode, TurnMode::Regular);
    }

    #[test]
    fn trickster_move_to_ineligible_square_ignored() {
        let mut step = StepTrickster::new();
        step.using_trickster = Some(true);
        // eligible_squares is empty → any move is not in the list
        let mut game = make_game();
        step.handle_command(
            &Action::TricksterMove { coord: FieldCoordinate::new(3, 3) },
            &mut game,
            &mut GameRng::new(0),
        );
        assert!(step.to_coordinate.is_none()); // not added since not in eligible_squares
    }

    #[test]
    fn trickster_move_to_eligible_square_accepted() {
        let mut step = StepTrickster::new();
        let coord = FieldCoordinate::new(4, 4);
        step.eligible_squares = vec![coord];
        step.using_trickster = Some(true);
        let mut game = make_game();
        step.handle_command(
            &Action::TricksterMove { coord },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(step.to_coordinate, Some(coord));
    }

    /// blocked_for_trickster squares are filtered out of eligible squares.
    #[test]
    fn blocked_for_trickster_squares_excluded_from_eligibles() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender, PS_STANDING, PlayerState, PlayerAction, SkillId};

        let mut game = make_game();

        // Attacker at (5,5)
        let attacker = Player {
            id: "att".into(), name: "a".into(), nr: 1, position_id: "p".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        };
        let defender = Player {
            id: "def".into(), name: "d".into(), nr: 2, position_id: "p".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            // Trickster skill = canMoveBeforeBeingBlocked
            starting_skills: vec![ffb_model::model::skill_def::SkillWithValue { skill_id: SkillId::Trickster, value: None }],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        };
        game.team_home.players.push(attacker);
        game.field_model.set_player_coordinate("att", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("att", PlayerState::new(PS_STANDING));
        game.acting_player.set_player("att".into(), PlayerAction::Block);

        game.team_away.players.push(defender);
        game.field_model.set_player_coordinate("def", FieldCoordinate::new(6, 5));
        game.field_model.set_player_state("def", PlayerState::new(PS_STANDING));
        game.defender_id = Some("def".into());

        // Block (6,6) for trickster
        game.field_model.blocked_for_trickster_coordinates.insert(FieldCoordinate::new(6, 6));

        let mut step = StepTrickster::new();
        let _out = step.start(&mut game, &mut GameRng::new(0));

        // No eligible square should be (6,6)
        assert!(!step.eligible_squares.contains(&FieldCoordinate::new(6, 6)));
    }

    /// After leave(), move_squares are cleared.
    #[test]
    fn leave_clears_move_squares() {
        let mut step = StepTrickster::new();
        step.using_trickster = Some(true);
        step.last_turn_mode = Some(TurnMode::Regular);
        let mut game = make_game();
        game.turn_mode = TurnMode::Trickster;
        // Populate a stale move square
        game.field_model.add_move_square(MoveSquare::new(FieldCoordinate::new(3, 3), 0, 0));
        step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert!(game.field_model.move_squares.is_empty());
    }

    /// replace_multi_block_target_coordinate replaces old coord with new.
    #[test]
    fn replace_multi_block_target_coord_works() {
        let mut game = make_game();
        let old = FieldCoordinate::new(3, 3);
        let new = FieldCoordinate::new(4, 4);
        game.field_model.multi_block_target_coordinates.insert(old);
        game.field_model.replace_multi_block_target_coordinate(old, new);
        assert!(!game.field_model.multi_block_target_coordinates.contains(&old));
        assert!(game.field_model.multi_block_target_coordinates.contains(&new));
    }
}
