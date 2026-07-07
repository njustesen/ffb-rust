use ffb_model::events::GameEvent;
use ffb_model::enums::{Direction, TurnMode};
use ffb_model::model::skill_use::SkillUse;
use ffb_model::report::mixed::report_hit_and_run::ReportHitAndRun;
use ffb_model::report::report_skill_use::ReportSkillUse;
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds, MoveSquare};
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::util_cards::UtilCards;
use ffb_model::util::util_player::UtilPlayer;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::sequences::pick_up_catch_scatter_sequence;
use crate::util::{ServerUtilBlock, UtilServerPlayerMove};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.block.StepHitAndRun.
/// After a block, the attacker may move one adjacent empty square with no opponent tackle zones.
pub struct StepHitAndRun {
    pub end_player_action: bool,
    pub end_turn: bool,
    pub coordinate: Option<FieldCoordinate>,
    pub saved_turn_mode: Option<TurnMode>,
}

impl StepHitAndRun {
    pub fn new() -> Self {
        Self { end_player_action: false, end_turn: false, coordinate: None, saved_turn_mode: None }
    }
}

impl Default for StepHitAndRun {
    fn default() -> Self { Self::new() }
}

impl Step for StepHitAndRun {
    fn id(&self) -> StepId { StepId::HitAndRun }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::HitAndRun { coord } => { self.coordinate = *coord; }
            Action::EndTurn => { self.end_turn = true; }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
            StepParameter::EndPlayerAction(v) => { self.end_player_action = *v; true }
            _ => false,
        }
    }
}

impl StepHitAndRun {
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        let acting_player_id = game.acting_player.player_id.clone();
        let attacker_state = acting_player_id.as_deref()
            .and_then(|id| game.field_model.player_state(id))
            .unwrap_or_default();

        // Java: UtilCards.getUnusedSkillWithProperty(actingPlayer, canMoveAfterBlock)
        let has_hit_and_run_skill = acting_player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill_property(NamedProperties::CAN_MOVE_AFTER_BLOCK))
            .unwrap_or(false);

        if has_hit_and_run_skill && !attacker_state.is_pinned() {
            if self.end_turn || self.end_player_action {
                self.reset_state(game);
                return StepOutcome::next();
            }

            let eligible_squares = self.find_squares(game);
            if eligible_squares.is_empty() {
                return StepOutcome::next();
            }

            if self.coordinate.is_none() {
                // Java: getResult().addReport(new ReportSkillUse(actingPlayer.getPlayerId(), skill, true, SkillUse.MOVE_SQUARE))
                {
                    let skill_id = acting_player_id.as_deref()
                        .and_then(|id| game.player(id))
                        .and_then(|p| UtilCards::get_unused_skill_with_property(p, NamedProperties::CAN_MOVE_AFTER_BLOCK));
                    if let Some(sid) = skill_id {
                        game.report_list.add(ReportSkillUse::new(
                            acting_player_id.clone(),
                            sid,
                            true,
                            SkillUse::MOVE_SQUARE,
                        ));
                    }
                }
                // Show eligible squares to agent; set HIT_AND_RUN turn mode
                if game.turn_mode != TurnMode::HitAndRun {
                    self.saved_turn_mode = game.last_turn_mode;
                    game.last_turn_mode = Some(game.turn_mode);
                    game.turn_mode = TurnMode::HitAndRun;
                }
                game.field_model.clear_move_squares();
                for coord in self.find_squares(game) {
                    game.field_model.add_move_square(MoveSquare::new(coord, 0, 0));
                }
                return StepOutcome::cont();
            } else {
                // Move the player
                let mut hit_and_run_event: Option<GameEvent> = None;
                if let (Some(ref attacker_id), Some(dest)) = (acting_player_id, self.coordinate) {
                    // Java: updatePlayerAndBallPosition — ball follows player if at old coord.
                    let old_pos = game.field_model.player_coordinate(attacker_id);
                    if !game.field_model.ball_moving {
                        if let (Some(old), Some(ball)) = (old_pos, game.field_model.ball_coordinate) {
                            if old == ball {
                                game.field_model.ball_coordinate = Some(dest);
                            }
                        }
                    }
                    let direction = old_pos
                        .and_then(|o| Direction::from_coords(o.x, o.y, dest.x, dest.y))
                        .unwrap_or(Direction::North);
                    // Java: getResult().addReport(new ReportHitAndRun(actingPlayer.getPlayerId(), direction))
                    game.report_list.add(ReportHitAndRun::new(
                        Some(attacker_id.clone()),
                        Some(direction),
                    ));
                    game.field_model.set_player_coordinate(attacker_id, dest);
                    hit_and_run_event = Some(GameEvent::HitAndRun { player_id: attacker_id.clone(), direction });
                    // Java: actingPlayer.markSkillUsed(canMoveAfterBlock)
                    let sid = game.player(attacker_id).and_then(|p| UtilCards::get_unused_skill_with_property(
                        p, NamedProperties::CAN_MOVE_AFTER_BLOCK));
                    if let Some(sid) = sid {
                        let is_home = game.team_home.player(attacker_id).is_some();
                        if is_home { game.team_home.player_mut(attacker_id).map(|p| p.used_skills.insert(sid)); }
                        else { game.team_away.player_mut(attacker_id).map(|p| p.used_skills.insert(sid)); }
                    }
                }
                self.reset_state(game);
                // Java: push PickUp(goto SCATTER_BALL on fail) + CatchScatterThrowIn sequence
                let out = StepOutcome::next().push_seq(pick_up_catch_scatter_sequence());
                if let Some(ev) = hit_and_run_event { out.with_event(ev) } else { out }
            }
        } else {
            StepOutcome::next()
        }
    }

    /// Java: StepHitAndRun.findSquares — adjacent, no player, no opponent in tackle zone.
    fn find_squares(&self, game: &Game) -> Vec<FieldCoordinate> {
        let attacker_id = match game.acting_player.player_id.as_deref() {
            Some(id) => id,
            None => return Vec::new(),
        };
        let player_coord = match game.field_model.player_coordinate(attacker_id) {
            Some(c) => c,
            None => return Vec::new(),
        };
        let other_team = game.inactive_team();
        game.field_model
            .adjacent_on_pitch(player_coord)
            .into_iter()
            .filter(|&c| game.field_model.player_at(c).is_none())
            // Java: !ArrayTool.isProvided(UtilPlayer.findAdjacentPlayers(game, otherTeam, coord))
            .filter(|&c| {
                UtilPlayer::find_adjacent_players_with_tacklezones(game, other_team, c, false).is_empty()
            })
            .collect()
    }

    /// Java: StepHitAndRun.resetState — restores TurnMode, updates move squares.
    fn reset_state(&mut self, game: &mut Game) {
        if game.turn_mode == TurnMode::HitAndRun {
            game.turn_mode = game.last_turn_mode.unwrap_or(TurnMode::Regular);
            if let Some(saved) = self.saved_turn_mode.take() {
                game.last_turn_mode = Some(saved);
            }
        }
        // Java: UtilServerPlayerMove.updateMoveSquares(getGameState(), false)
        UtilServerPlayerMove::update_move_squares(game, false);
        // Java: ServerUtilBlock.updateDiceDecorations(getGameState())
        ServerUtilBlock::update_dice_decorations(game);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{Rules, PlayerType, PlayerGender, SkillId};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::report::report_id::ReportId;
    use ffb_model::types::FieldCoordinate;
    use std::collections::HashSet;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn no_hit_and_run_skill_returns_next_step() {
        // No acting player in game → no canMoveAfterBlock skill → NEXT_STEP
        let mut step = StepHitAndRun::new();
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn end_turn_with_skill_returns_next_step() {
        let mut step = StepHitAndRun::new();
        step.end_turn = true;
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Without skill the step just returns NEXT_STEP
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_end_turn_accepted() {
        let mut step = StepHitAndRun::new();
        step.set_parameter(&StepParameter::EndTurn(true));
        assert!(step.end_turn);
    }

    #[test]
    fn set_parameter_end_player_action_accepted() {
        let mut step = StepHitAndRun::new();
        step.set_parameter(&StepParameter::EndPlayerAction(true));
        assert!(step.end_player_action);
    }

    #[test]
    fn hit_and_run_command_sets_coordinate() {
        let mut step = StepHitAndRun::new();
        let coord = FieldCoordinate::new(6, 6);
        let mut game = make_game();
        step.handle_command(
            &Action::HitAndRun { coord: Some(coord) },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(step.coordinate, Some(coord));
    }

    // ── report wiring ─────────────────────────────────────────────────────────

    fn add_hit_and_run_player(game: &mut Game, id: &str, coord: FieldCoordinate) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![SkillWithValue { skill_id: SkillId::HitAndRun, value: None }],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate(id, coord);
    }

    /// When coordinate is None, the step shows the dialog and adds ReportSkillUse (MOVE_SQUARE).
    #[test]
    fn no_coordinate_adds_skill_use_report() {
        let mut step = StepHitAndRun::new();
        let coord = FieldCoordinate::new(10, 7);
        let mut game = make_game();
        add_hit_and_run_player(&mut game, "p1", coord);
        game.acting_player.player_id = Some("p1".into());
        // coordinate is None — step shows dialog
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::SKILL_USE), "ReportSkillUse should appear when showing hit-and-run dialog");
    }

    /// When coordinate is set, the step moves the player and adds ReportHitAndRun.
    #[test]
    fn with_coordinate_adds_hit_and_run_report() {
        let mut step = StepHitAndRun::new();
        let from = FieldCoordinate::new(10, 7);
        let dest = FieldCoordinate::new(11, 7);
        let mut game = make_game();
        add_hit_and_run_player(&mut game, "p1", from);
        game.acting_player.player_id = Some("p1".into());
        step.coordinate = Some(dest);
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::HIT_AND_RUN), "ReportHitAndRun should appear after player moves");
    }
}
