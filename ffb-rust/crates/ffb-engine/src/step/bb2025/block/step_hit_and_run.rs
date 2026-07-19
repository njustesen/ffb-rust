use ffb_model::events::GameEvent;
use ffb_model::enums::{Direction, TurnMode};
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::report::mixed::report_hit_and_run::ReportHitAndRun;
use ffb_model::report::report_skill_use::ReportSkillUse;
use ffb_model::types::{FieldCoordinate, MoveSquare};
use ffb_model::util::util_cards::UtilCards;
use ffb_model::util::util_player::UtilPlayer;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::sequences::pick_up_catch_scatter_sequence;
use crate::util::UtilServerPlayerMove;
use crate::util::server_util_block::ServerUtilBlock;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.block.StepHitAndRun.
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
                // Show eligible squares to agent; set HIT_AND_RUN turn mode
                if game.turn_mode != TurnMode::HitAndRun {
                    self.saved_turn_mode = game.last_turn_mode;
                    game.last_turn_mode = Some(game.turn_mode);
                    game.turn_mode = TurnMode::HitAndRun;
                }
                // Java: getResult().addReport(new ReportSkillUse(actingPlayer.getPlayerId(), skill, true, SkillUse.MOVE_SQUARE))
                if let Some(ref attacker_id) = acting_player_id {
                    let skill_id = game.player(attacker_id).and_then(|p| p.all_skill_ids().find(|s| {
                        s.properties().contains(&NamedProperties::CAN_MOVE_AFTER_BLOCK)
                    }));
                    if let Some(sid) = skill_id {
                        game.report_list.add(ReportSkillUse::new(
                            Some(attacker_id.clone()), sid, true, SkillUse::MOVE_SQUARE,
                        ));
                    }
                }
                game.field_model.clear_move_squares();
                for c in &eligible_squares {
                    game.field_model.add_move_square(MoveSquare::new(*c, 0, 0));
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
                    game.field_model.set_player_coordinate(attacker_id, dest);
                    // Java: getResult().addReport(new ReportHitAndRun(actingPlayer.getPlayerId(), direction))
                    game.report_list.add(ReportHitAndRun::new(
                        Some(attacker_id.clone()), Some(direction),
                    ));
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
            // Note: uses the plain findAdjacentPlayers (no tacklezone filter) — any adjacent
            // opponent (even prone/stunned) disqualifies the square, unlike findAdjacentPlayersWithTacklezones.
            .filter(|&c| {
                UtilPlayer::find_adjacent_players(game, other_team, c).is_empty()
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
        UtilServerPlayerMove::update_move_squares(game, game.acting_player.jumping);
        ServerUtilBlock::update_dice_decorations(game);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
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

    /// When eligible squares exist, a ReportSkillUse(MOVE_SQUARE) is added to report_list.
    #[test]
    fn eligible_squares_adds_report_skill_use_move_square() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender, PS_STANDING, PlayerState, PlayerAction, SkillId};
        use ffb_model::report::report_id::ReportId;

        let mut game = make_game();
        let player = Player {
            id: "attacker".into(), name: "a".into(), nr: 1, position_id: "p".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![ffb_model::model::skill_def::SkillWithValue { skill_id: SkillId::HitAndRun, value: None }],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        game.team_home.players.push(player);
        game.field_model.set_player_coordinate("attacker", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("attacker", PlayerState::new(PS_STANDING));
        game.acting_player.set_player("attacker".into(), PlayerAction::Block);

        let mut step = StepHitAndRun::new();
        step.start(&mut game, &mut GameRng::new(0));

        assert!(game.report_list.has_report(ReportId::SKILL_USE),
            "expected SKILL_USE report for MOVE_SQUARE when eligible squares exist");
    }

    /// When a coordinate is chosen and player moves, ReportHitAndRun is added to report_list.
    #[test]
    fn selecting_coordinate_adds_report_hit_and_run() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender, PS_STANDING, PlayerState, PlayerAction, SkillId};
        use ffb_model::report::report_id::ReportId;

        let mut game = make_game();
        let player = Player {
            id: "attacker".into(), name: "a".into(), nr: 1, position_id: "p".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![ffb_model::model::skill_def::SkillWithValue { skill_id: SkillId::HitAndRun, value: None }],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        game.team_home.players.push(player);
        game.field_model.set_player_coordinate("attacker", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("attacker", PlayerState::new(PS_STANDING));
        game.acting_player.set_player("attacker".into(), PlayerAction::Block);
        game.turn_mode = TurnMode::HitAndRun;

        let dest = FieldCoordinate::new(6, 5);
        let mut step = StepHitAndRun::new();
        step.coordinate = Some(dest);
        step.start(&mut game, &mut GameRng::new(0));

        assert!(game.report_list.has_report(ReportId::HIT_AND_RUN),
            "expected HIT_AND_RUN report after player moves");
    }

    /// When eligible squares exist and no coordinate is chosen yet,
    /// clearMoveSquares + addMoveSquare are called for each eligible square.
    #[test]
    fn eligible_squares_populate_move_squares_and_set_hit_and_run_mode() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender, PS_STANDING, PlayerState, PlayerAction, SkillId};

        let mut game = make_game();

        let player = Player {
            id: "attacker".into(), name: "a".into(), nr: 1, position_id: "p".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            // HitAndRun skill has "canMoveAfterBlock" property
            starting_skills: vec![ffb_model::model::skill_def::SkillWithValue { skill_id: SkillId::HitAndRun, value: None }], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };

        game.team_home.players.push(player);
        game.field_model.set_player_coordinate("attacker", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("attacker", PlayerState::new(PS_STANDING));
        game.acting_player.set_player("attacker".into(), PlayerAction::Block);

        // Pre-fill a stale move square so we can verify clear was called
        game.field_model.add_move_square(MoveSquare::new(FieldCoordinate::new(0, 0), 1, 1));

        let mut step = StepHitAndRun::new();
        let out = step.start(&mut game, &mut GameRng::new(0));

        // Step should wait for coordinate selection
        assert_eq!(out.action, StepAction::Continue);
        // TurnMode should now be HitAndRun
        assert_eq!(game.turn_mode, TurnMode::HitAndRun);
        // Stale move square (0,0) should be gone; eligible squares should be present
        assert!(game.field_model.get_move_square(FieldCoordinate::new(0, 0)).is_none());
        // At least one eligible adjacent square should have been added
        assert!(!game.field_model.move_squares.is_empty());
    }

    /// Java: StepHitAndRun.findSquares uses the plain UtilPlayer.findAdjacentPlayers (no
    /// tacklezone filter), so a square adjacent to a PRONE opponent must still be excluded —
    /// unlike findAdjacentPlayersWithTacklezones, which would ignore prone/stunned opponents.
    #[test]
    fn find_squares_excludes_square_adjacent_to_prone_opponent() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender, PS_STANDING, PS_PRONE, PlayerState, PlayerAction, SkillId};

        let mut game = make_game();

        let attacker = Player {
            id: "attacker".into(), name: "a".into(), nr: 1, position_id: "p".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![ffb_model::model::skill_def::SkillWithValue { skill_id: SkillId::HitAndRun, value: None }],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        game.team_home.players.push(attacker);
        game.field_model.set_player_coordinate("attacker", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("attacker", PlayerState::new(PS_STANDING));
        game.acting_player.set_player("attacker".into(), PlayerAction::Block);

        // Opponent lying prone (no tacklezones) adjacent to (6,5), which is itself adjacent
        // to the attacker at (5,5). (6,5) is empty, so under a tacklezone-only filter it
        // would wrongly be treated as eligible.
        let opponent = Player {
            id: "opponent".into(), name: "o".into(), nr: 2, position_id: "p".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        game.team_away.players.push(opponent);
        game.field_model.set_player_coordinate("opponent", FieldCoordinate::new(7, 5));
        game.field_model.set_player_state("opponent", PlayerState::new(PS_PRONE));

        let step = StepHitAndRun::new();
        let squares = step.find_squares(&game);
        assert!(
            !squares.contains(&FieldCoordinate::new(6, 5)),
            "square adjacent to a prone opponent must be excluded, matching Java's plain findAdjacentPlayers (no tacklezone filter)"
        );
    }

    /// Selecting a coordinate causes the player to move to that square (move_squares cleared via reset_state).
    #[test]
    fn selecting_coordinate_moves_player_and_clears_hit_and_run_mode() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender, PS_STANDING, PlayerState, PlayerAction, SkillId};

        let mut game = make_game();

        let player = Player {
            id: "attacker".into(), name: "a".into(), nr: 1, position_id: "p".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![ffb_model::model::skill_def::SkillWithValue { skill_id: SkillId::HitAndRun, value: None }], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };

        game.team_home.players.push(player);
        game.field_model.set_player_coordinate("attacker", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("attacker", PlayerState::new(PS_STANDING));
        game.acting_player.set_player("attacker".into(), PlayerAction::Block);

        game.turn_mode = TurnMode::HitAndRun;

        let dest = FieldCoordinate::new(6, 5);
        let mut step = StepHitAndRun::new();
        step.coordinate = Some(dest);
        let out = step.start(&mut game, &mut GameRng::new(0));

        // Player should have moved to dest
        assert_eq!(game.field_model.player_coordinate("attacker"), Some(dest));
        // TurnMode restored (HitAndRun cleared)
        assert_ne!(game.turn_mode, TurnMode::HitAndRun);
        // Step advances
        assert_eq!(out.action, StepAction::NextStep);
    }
}
