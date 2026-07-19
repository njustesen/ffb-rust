use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::report::report_skill_use::ReportSkillUse;
use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::util::UtilServerPlayerMove;
use crate::util::server_util_block::ServerUtilBlock;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.move.StepMove.
///
/// NOTE: bb2020's guard checks `playerState.isRooted()` (NOT the broader isPinned() used by the
/// bb2025 sibling) -- this is a genuine bb2020/bb2025 rules divergence, not a translation error.
/// Physically moves the acting player one square: updates the field model,
/// increments currentMove (×2 for jumping), optionally moves the ball if carried,
/// publishes PLAYER_ENTERING_SQUARE.
///
/// client-only: TrackNumber animation not ported — no Rust FieldModel.track_numbers field.
pub struct StepMove {
    /// Java: fCoordinateFrom
    pub coordinate_from: Option<FieldCoordinate>,
    /// Java: fCoordinateTo
    pub coordinate_to: Option<FieldCoordinate>,
    /// Java: fMoveStackSize — size of remaining move stack (0 = last step)
    pub move_stack_size: i32,
}

impl StepMove {
    pub fn new() -> Self {
        Self { coordinate_from: None, coordinate_to: None, move_stack_size: 0 }
    }
}

impl Default for StepMove {
    fn default() -> Self { Self::new() }
}

impl Step for StepMove {
    fn id(&self) -> StepId { StepId::Move }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::CoordinateFrom(v) => { self.coordinate_from = Some(*v); true }
            StepParameter::CoordinateTo(v) => { self.coordinate_to = Some(*v); true }
            StepParameter::MoveStack(v) => { self.move_stack_size = v.len() as i32; true }
            _ => false,
        }
    }
}

impl StepMove {
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        let Some(ref attacker_id) = game.acting_player.player_id.clone() else {
            return StepOutcome::next();
        };

        // Java: `if (!playerState.isRooted()) { ... }` -- note this checks isRooted() specifically,
        // NOT the broader isPinned() (which is isChomped() || isRooted()).
        let is_rooted = game.field_model.player_state(attacker_id)
            .map(|s| s.is_rooted())
            .unwrap_or(false);
        if is_rooted {
            return StepOutcome::next();
        }

        let Some(to) = self.coordinate_to else {
            return StepOutcome::next();
        };

        let jump_inc = if game.acting_player.jumping { 2 } else { 1 };
        game.acting_player.current_move += jump_inc;

        // Java: possibleFreeRushes = 2 + (canMakeAnExtraGfi ? 1 : 0)
        // Java: if currentMove > movementWithModifiers + possibleFreeRushes && extraGfiOnceSkill.isPresent()
        let extra_gfi_once_skill = game.acting_player.player_id.as_deref()
            .and_then(|id| game.player(id))
            .and_then(|p| p.skill_id_with_property(NamedProperties::CAN_MAKE_AN_EXTRA_GFI_ONCE));
        if let Some(skill_id) = extra_gfi_once_skill {
            let possible_free_rushes = 2 + if game.acting_player.player_id.as_deref()
                .and_then(|id| game.player(id))
                .map(|p| p.has_skill_property(NamedProperties::CAN_MAKE_AN_EXTRA_GFI))
                .unwrap_or(false) { 1 } else { 0 };
            let movement = game.acting_player.player_id.as_deref()
                .and_then(|id| game.player(id))
                .map(|p| p.movement_with_modifiers())
                .unwrap_or(0);
            if game.acting_player.current_move > movement + possible_free_rushes {
                // Java: actingPlayer.markSkillUsed(extraGfiOnceSkill.get())
                if let Some(pid) = game.acting_player.player_id.clone() {
                    if let Some(p) = game.team_home.player_mut(&pid).or_else(|| game.team_away.player_mut(&pid)) {
                        p.used_skills.insert(skill_id);
                    }
                }
                game.report_list.add(ReportSkillUse::new(
                    game.acting_player.player_id.clone(),
                    skill_id,
                    true,
                    SkillUse::RUSH_ADDITIONAL_SQUARE_ONCE,
                ));
            }
        }

        let old_pos = game.field_model.player_coordinate(attacker_id);
        let ball_position_updated = if !game.field_model.ball_moving {
            if let (Some(old), Some(ball)) = (old_pos, game.field_model.ball_coordinate) {
                if old == ball {
                    game.field_model.ball_coordinate = Some(to);
                    true
                } else { false }
            } else { false }
        } else { false };
        game.field_model.set_player_coordinate(attacker_id, to);

        // Java: if (ballPositionUpdated) { playerResult.setRushing(rushing + deltaX) }
        if ball_position_updated {
            let from_x = self.coordinate_from.map(|c| c.x).unwrap_or(to.x);
            let delta_x = if game.home_playing { to.x - from_x } else { from_x - to.x };
            let is_home = game.team_home.player(attacker_id).is_some();
            let pr = if is_home {
                game.game_result.home.player_results.entry(attacker_id.clone()).or_default()
            } else {
                game.game_result.away.player_results.entry(attacker_id.clone()).or_default()
            };
            pr.rushing += delta_x;
        }

        game.acting_player.goes_for_it = UtilPlayer::is_next_move_going_for_it(game);

        // Java: `if (fMoveStackSize == 0) { UtilServerPlayerMove.updateMoveSquares(getGameState(), false); }`
        if self.move_stack_size == 0 {
            UtilServerPlayerMove::update_move_squares(game, false);
        }
        // Java: `ServerUtilBlock.updateDiceDecorations(getGameState());` -- always, regardless of move stack size.
        ServerUtilBlock::update_dice_decorations(game);

        StepOutcome::next()
            .publish(StepParameter::PlayerEnteringSquare(attacker_id.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepParameter;
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn no_acting_player_returns_next_step() {
        let mut game = make_game();
        let mut step = StepMove::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, crate::step::framework::StepAction::NextStep);
    }

    #[test]
    fn moves_player_to_coordinate_to() {
        let mut game = make_game();
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(6, 5);
        game.field_model.set_player_coordinate("p1", from);
        game.acting_player.player_id = Some("p1".into());
        let mut step = StepMove::new();
        step.coordinate_from = Some(from);
        step.coordinate_to = Some(to);
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.field_model.player_coordinate("p1"), Some(to));
    }

    #[test]
    fn publishes_player_entering_square() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        let mut step = StepMove::new();
        step.coordinate_to = Some(FieldCoordinate::new(6, 5));
        let out = step.start(&mut game, &mut GameRng::new(0));
        let has = out.published.iter().any(|p| matches!(p, StepParameter::PlayerEnteringSquare(id) if id == "p1"));
        assert!(has, "PlayerEnteringSquare must be published");
    }

    #[test]
    fn increments_current_move_by_one_for_non_jumping() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.current_move = 2;
        game.acting_player.jumping = false;
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        let mut step = StepMove::new();
        step.coordinate_to = Some(FieldCoordinate::new(6, 5));
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.acting_player.current_move, 3);
    }

    #[test]
    fn increments_current_move_by_two_for_jumping() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.current_move = 0;
        game.acting_player.jumping = true;
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        let mut step = StepMove::new();
        step.coordinate_to = Some(FieldCoordinate::new(7, 5));
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.acting_player.current_move, 2);
    }

    #[test]
    fn rooted_player_returns_next_step_without_moving() {
        use ffb_model::enums::{PlayerState, PS_STANDING};
        let mut game = make_game();
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(6, 5);
        game.acting_player.player_id = Some("p1".into());
        game.field_model.set_player_coordinate("p1", from);
        // Rooted state means is_pinned() = true.
        game.field_model.set_player_state("p1", PlayerState::new(PS_STANDING).change_rooted(true));
        let mut step = StepMove::new();
        step.coordinate_to = Some(to);
        step.start(&mut game, &mut GameRng::new(0));
        // Pinned player should not have moved.
        assert_eq!(game.field_model.player_coordinate("p1"), Some(from));
    }

    // Java StepMove.executeStep(): `if (!playerState.isRooted())` guards the move logic -- it checks
    // isRooted() specifically, not the broader isPinned() (isChomped() || isRooted()). Before the fix,
    // this file used is_pinned(), which would incorrectly block movement for a chomped-but-not-rooted
    // player. This test would have failed before the fix (player stayed at `from`).
    #[test]
    fn chomped_but_not_rooted_player_still_moves() {
        use ffb_model::enums::{PlayerState, PS_STANDING};
        let mut game = make_game();
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(6, 5);
        game.acting_player.player_id = Some("p1".into());
        game.field_model.set_player_coordinate("p1", from);
        // Chomped but NOT rooted: Java's isRooted() guard must not treat this as blocked.
        game.field_model.set_player_state("p1", PlayerState::new(PS_STANDING).change_chomped(true));
        let mut step = StepMove::new();
        step.coordinate_to = Some(to);
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.field_model.player_coordinate("p1"), Some(to),
            "chomped-but-not-rooted player must still move (Java only checks isRooted())");
    }

    #[test]
    fn ball_moves_with_carrier() {
        let mut game = make_game();
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(6, 5);
        game.acting_player.player_id = Some("p1".into());
        game.home_playing = true;
        game.field_model.set_player_coordinate("p1", from);
        game.field_model.ball_coordinate = Some(from);
        game.field_model.ball_moving = false;
        let mut step = StepMove::new();
        step.coordinate_from = Some(from);
        step.coordinate_to = Some(to);
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.field_model.ball_coordinate, Some(to));
    }

    #[test]
    fn ball_does_not_move_when_ball_moving() {
        let mut game = make_game();
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(6, 5);
        let ball_pos = FieldCoordinate::new(10, 5);
        game.acting_player.player_id = Some("p1".into());
        game.field_model.set_player_coordinate("p1", from);
        game.field_model.ball_coordinate = Some(ball_pos);
        game.field_model.ball_moving = true; // ball is already in transit
        let mut step = StepMove::new();
        step.coordinate_to = Some(to);
        step.start(&mut game, &mut GameRng::new(0));
        // Ball position should not change.
        assert_eq!(game.field_model.ball_coordinate, Some(ball_pos));
    }

    #[test]
    fn extra_gfi_once_skill_adds_skill_use_report() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender, SkillId};
        use ffb_model::model::skill_def::SkillWithValue;
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        game.team_home.players.push(Player {
            id: "p1".into(), name: "p1".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            // SureFeet has canMakeAnExtraGfiOnce
            starting_skills: vec![SkillWithValue::new(SkillId::SureFeet)],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(6, 5);
        game.acting_player.player_id = Some("p1".into());
        // movement=6, possibleFreeRushes=2, threshold=8; set current_move=8 so after +1=9 > 8
        game.acting_player.current_move = 8;
        game.field_model.set_player_coordinate("p1", from);
        let mut step = StepMove::new();
        step.coordinate_from = Some(from);
        step.coordinate_to = Some(to);
        step.start(&mut game, &mut GameRng::new(0));
        assert!(
            game.report_list.has_report(ReportId::SKILL_USE),
            "ReportSkillUse(RUSH_ADDITIONAL_SQUARE_ONCE) must be added"
        );
    }

    #[test]
    fn extra_gfi_once_not_triggered_within_free_rushes() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender, SkillId};
        use ffb_model::model::skill_def::SkillWithValue;
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        game.team_home.players.push(Player {
            id: "p1".into(), name: "p1".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![SkillWithValue::new(SkillId::SureFeet)],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(6, 5);
        game.acting_player.player_id = Some("p1".into());
        // movement=6, possibleFreeRushes=2, threshold=8; current_move=5 → after +1=6 ≤ 8
        game.acting_player.current_move = 5;
        game.field_model.set_player_coordinate("p1", from);
        let mut step = StepMove::new();
        step.coordinate_from = Some(from);
        step.coordinate_to = Some(to);
        step.start(&mut game, &mut GameRng::new(0));
        assert!(
            !game.report_list.has_report(ReportId::SKILL_USE),
            "no SKILL_USE report when within free rushes"
        );
    }

    // Java: `if (fMoveStackSize == 0) { UtilServerPlayerMove.updateMoveSquares(getGameState(), false); }`
    // -- this call was entirely missing from the Rust translation. Before the fix, move_squares would
    // stay empty after finishing a move, so this test would have failed.
    #[test]
    fn last_step_of_move_recomputes_move_squares() {
        use ffb_model::enums::PlayerAction;
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Move);
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(6, 5);
        game.field_model.set_player_coordinate("p1", from);
        let mut step = StepMove::new();
        step.coordinate_to = Some(to);
        step.move_stack_size = 0; // last step of the move
        step.start(&mut game, &mut GameRng::new(0));
        assert!(
            !game.field_model.move_squares.is_empty(),
            "move_squares must be recomputed at the last step of a move"
        );
    }

    #[test]
    fn rushing_yards_added_to_player_result_when_carrying_ball() {
        use ffb_model::enums::{PlayerType, PlayerGender};
        use ffb_model::model::player::Player;
        use std::collections::HashSet;
        let mut game = make_game();
        game.team_home.players.push(Player {
            id: "carrier".into(), name: "c".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        let from = FieldCoordinate::new(5, 7);
        let to = FieldCoordinate::new(8, 7);
        game.acting_player.player_id = Some("carrier".into());
        game.home_playing = true;
        game.field_model.set_player_coordinate("carrier", from);
        game.field_model.ball_coordinate = Some(from);
        game.field_model.ball_moving = false;
        let mut step = StepMove::new();
        step.coordinate_from = Some(from);
        step.coordinate_to = Some(to);
        step.start(&mut game, &mut GameRng::new(0));
        // delta_x = to.x - from.x = 8 - 5 = 3 (home playing)
        let pr = game.game_result.home.player_results.get("carrier").unwrap();
        assert_eq!(pr.rushing, 3);
    }
}
