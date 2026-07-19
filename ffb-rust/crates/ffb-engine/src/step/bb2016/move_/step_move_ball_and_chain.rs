use ffb_model::enums::Direction;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::report::report_scatter_player::ReportScatterPlayer;
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use crate::action::Action;
use crate::drop_player_context::SteadyFootingContext;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.move.StepMoveBallAndChain.
///
/// Handles movement of a Ball-and-Chain player. The player *always* scatters one
/// square in a randomly-rolled direction (biased toward the direction of the
/// pre-supplied COORDINATE_TO, i.e. the square the player was attempting to move
/// to). If the scatter destination is out of bounds, the player is crowd-pushed.
/// If there is a player in the destination square, a block resolves. Otherwise,
/// the player moves there.
///
/// Init params: GOTO_LABEL_ON_END (mandatory), GOTO_LABEL_ON_FALL_DOWN (mandatory),
///              COORDINATE_FROM (mandatory), COORDINATE_TO (mandatory).
///
/// Logic (executeStep):
/// 1. Check movesRandomly property (Ball-and-Chain carrier). If !movesRandomly → NEXT_STEP.
/// 2. Roll a D8. Determine a base compass direction by comparing coordinateFrom to the
///    (pre-scatter) coordinateTo, then interpret the roll relative to that base direction
///    via ThrowInMechanic (this is *not* conditional on whether coordinateTo was preset —
///    Ball-and-Chain always scatters).
/// 3. Overwrite coordinateTo with the scatter result (1 square from coordinateFrom).
/// 4. addReport(ReportScatterPlayer).
/// 5. If out of bounds:
///    - Publish INJURY_TYPE(InjuryTypeCrowdPush)
///    - GOTO_LABEL(fGotoLabelOnFallDown)
/// 6. Publish COORDINATE_TO(fCoordinateTo) unconditionally.
/// 7. If there is a player at coordinateTo (blockDefenderId):
///    - currentMove += 1; setGoingForIt(isNextMoveGoingForIt)
///    - Publish BLOCK_DEFENDER_ID
///    - GOTO_LABEL(fGotoLabelOnEnd)
/// 8. Else: NEXT_STEP
pub struct StepMoveBallAndChain {
    /// Java: fGotoLabelOnEnd
    pub goto_label_on_end: String,
    /// Java: fGotoLabelOnFallDown
    pub goto_label_on_fall_down: String,
    /// Java: fCoordinateFrom
    pub coordinate_from: Option<FieldCoordinate>,
    /// Java: fCoordinateTo (pre-scatter: the intended target; post-scatter: the actual landing square)
    pub coordinate_to: Option<FieldCoordinate>,
}

impl StepMoveBallAndChain {
    pub fn new(goto_label_on_end: String, goto_label_on_fall_down: String) -> Self {
        Self {
            goto_label_on_end,
            goto_label_on_fall_down,
            coordinate_from: None,
            coordinate_to: None,
        }
    }
}

impl Default for StepMoveBallAndChain {
    fn default() -> Self { Self::new(String::new(), String::new()) }
}

impl Step for StepMoveBallAndChain {
    fn id(&self) -> StepId { StepId::MoveBallAndChain }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(v) => { self.goto_label_on_end = v.clone(); true }
            StepParameter::GotoLabelOnFallDown(v) => { self.goto_label_on_fall_down = v.clone(); true }
            StepParameter::CoordinateFrom(v) => { self.coordinate_from = Some(*v); true }
            StepParameter::CoordinateTo(v) => { self.coordinate_to = Some(*v); true }
            _ => false,
        }
    }
}

impl StepMoveBallAndChain {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: if (!actingPlayer.getPlayer().hasSkillProperty(MOVES_RANDOMLY)) NEXT_STEP
        let moves_randomly = game.acting_player.player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill_property(NamedProperties::MOVES_RANDOMLY))
            .unwrap_or(false);

        if !moves_randomly {
            return StepOutcome::next();
        }

        let coordinate_from = match self.coordinate_from {
            Some(c) => c,
            None => return StepOutcome::next(),
        };
        // Java uses fCoordinateTo directly (no null-guard in the source — a Ball-and-Chain
        // player is always dispatched with a pending target square). We guard defensively.
        let bias_coordinate_to = match self.coordinate_to {
            Some(c) => c,
            None => return StepOutcome::next(),
        };

        // Java: int scatterRoll = getGameState().getDiceRoller().rollThrowInDirection();
        // Always rolled — the scatter is unconditional, never skipped because coordinateTo
        // was pre-supplied by the caller (that value only biases the direction below).
        let scatter_roll = rng.d8();

        // Java: compare fCoordinateFrom to fCoordinateTo (pre-scatter) to pick a base direction
        let base_dir = if coordinate_from.x < bias_coordinate_to.x {
            Direction::East
        } else if coordinate_from.x > bias_coordinate_to.x {
            Direction::West
        } else if coordinate_from.y < bias_coordinate_to.y {
            Direction::South
        } else {
            Direction::North
        };

        let player_scatter = interpret_throw_in_direction(base_dir, scatter_roll);

        // Java: fCoordinateTo = UtilServerCatchScatterThrowIn.findScatterCoordinate(fCoordinateFrom, playerScatter, 1)
        let coordinate_to = scatter_one_square(coordinate_from, player_scatter);
        self.coordinate_to = Some(coordinate_to);

        // Java: getResult().addReport(new ReportScatterPlayer(coordinateFrom, coordinateTo, [playerScatter], [scatterRoll]))
        game.report_list.add(ReportScatterPlayer::new(
            coordinate_from,
            coordinate_to,
            vec![player_scatter],
            vec![scatter_roll],
            None,
        ));

        // Java: if (!FieldCoordinateBounds.FIELD.isInBounds(fCoordinateTo))
        if !FieldCoordinateBounds::FIELD.is_in_bounds(coordinate_to) {
            // Java: publishParameter(INJURY_TYPE, new InjuryTypeCrowdPush()) + GOTO_LABEL_ON_FALL_DOWN
            let label = self.goto_label_on_fall_down.clone();
            let ctx = SteadyFootingContext::from_injury_type_name("InjuryTypeCrowdPush".into());
            return StepOutcome::goto(&label)
                .publish(StepParameter::SteadyFootingContext(Box::new(ctx)));
        }

        // Java: publishParameter(COORDINATE_TO, fCoordinateTo) — unconditional
        let mut outcome = StepOutcome::next()
            .publish(StepParameter::CoordinateTo(coordinate_to));

        // Java: Player<?> blockDefender = game.getFieldModel().getPlayer(fCoordinateTo)
        let block_defender_id = game.field_model.player_at(coordinate_to).cloned();

        if let Some(defender_id) = block_defender_id {
            // Java: actingPlayer.setCurrentMove(actingPlayer.getCurrentMove() + 1)
            game.acting_player.current_move += 1;
            // Java: actingPlayer.setGoingForIt(UtilPlayer.isNextMoveGoingForIt(game))
            game.acting_player.goes_for_it = UtilPlayer::is_next_move_going_for_it(game);
            let label = self.goto_label_on_end.clone();
            outcome = outcome.publish(StepParameter::BlockDefenderId(defender_id));
            return StepOutcome::goto(&label).publish_all(outcome.published);
        }

        outcome
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Java: ThrowInMechanic.interpretThrowInDirectionRoll(baseDir, roll).
/// Maps a d8 roll (1-8) to a scatter direction based on the base (facing) direction.
fn interpret_throw_in_direction(base: Direction, roll: i32) -> Direction {
    let dirs_east: [Direction; 8] = [
        Direction::Northeast, Direction::North, Direction::Northwest,
        Direction::East, Direction::East,
        Direction::Southeast, Direction::South, Direction::Southwest,
    ];
    let dirs_west: [Direction; 8] = [
        Direction::Southwest, Direction::South, Direction::Southeast,
        Direction::West, Direction::West,
        Direction::Northwest, Direction::North, Direction::Northeast,
    ];
    let dirs_south: [Direction; 8] = [
        Direction::Southeast, Direction::East, Direction::Northeast,
        Direction::South, Direction::South,
        Direction::Southwest, Direction::West, Direction::Northwest,
    ];
    let dirs_north: [Direction; 8] = [
        Direction::Northwest, Direction::West, Direction::Southwest,
        Direction::North, Direction::North,
        Direction::Northeast, Direction::East, Direction::Southeast,
    ];
    let idx = ((roll - 1).max(0).min(7)) as usize;
    match base {
        Direction::East | Direction::Northeast | Direction::Southeast => dirs_east[idx],
        Direction::West | Direction::Northwest | Direction::Southwest => dirs_west[idx],
        Direction::South => dirs_south[idx],
        Direction::North => dirs_north[idx],
    }
}

/// Move one square in `dir` from `from` (Java: UtilServerCatchScatterThrowIn.findScatterCoordinate(from, dir, 1)).
fn scatter_one_square(from: FieldCoordinate, dir: Direction) -> FieldCoordinate {
    let (dx, dy): (i32, i32) = match dir {
        Direction::North     => (0, -1),
        Direction::Northeast => (1, -1),
        Direction::East      => (1,  0),
        Direction::Southeast => (1,  1),
        Direction::South     => (0,  1),
        Direction::Southwest => (-1, 1),
        Direction::West      => (-1, 0),
        Direction::Northwest => (-1,-1),
    };
    FieldCoordinate::new(from.x + dx, from.y + dy)
}

// Extension to add convenience helpers to StepOutcome
trait StepOutcomeBuilder {
    fn publish_all(self, params: Vec<StepParameter>) -> Self;
}

impl StepOutcomeBuilder for StepOutcome {
    fn publish_all(mut self, params: Vec<StepParameter>) -> Self {
        self.published.extend(params);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::Rules;
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::report::report_id::ReportId;
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;
    use std::collections::HashSet;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2016)
    }

    fn add_player(game: &mut Game, id: &str, coord: FieldCoordinate) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 4, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
                    ..Default::default()
});
        game.field_model.set_player_coordinate(id, coord);
    }

    fn add_ball_and_chain_player(game: &mut Game, id: &str, coord: FieldCoordinate) {
        use ffb_model::model::skill_def::SkillWithValue;
        use ffb_model::enums::SkillId;
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 2, position_id: "ballchain".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 2, strength: 5, agility: 1, passing: 6, armour: 9,
            starting_skills: vec![SkillWithValue { skill_id: SkillId::BallAndChain, value: None }],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
                    ..Default::default()
});
        game.field_model.set_player_coordinate(id, coord);
        game.acting_player.player_id = Some(id.into());
    }

    #[test]
    fn non_ball_and_chain_player_returns_next_step() {
        let mut game = make_game();
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(6, 5);
        add_player(&mut game, "p1", from);
        game.acting_player.player_id = Some("p1".into());
        let mut step = StepMoveBallAndChain::new("end".into(), "fall".into());
        step.coordinate_from = Some(from);
        step.coordinate_to = Some(to);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn out_of_bounds_target_gotos_fall_down_label() {
        // Center-field carrier scattering toward the near edge with a seed that rolls
        // a direction driving it off the board.
        let mut game = make_game();
        let from = FieldCoordinate::new(0, 5);
        let bias_to = FieldCoordinate::new(1, 5); // east bias
        add_ball_and_chain_player(&mut game, "carrier", from);
        let mut step = StepMoveBallAndChain::new("end".into(), "fall".into());
        step.coordinate_from = Some(from);
        step.coordinate_to = Some(bias_to);
        // Try a handful of seeds to find one that scatters west/off the left edge.
        let mut found_oob = false;
        for seed in 0..64u64 {
            let mut s = StepMoveBallAndChain::new("end".into(), "fall".into());
            s.coordinate_from = Some(from);
            s.coordinate_to = Some(bias_to);
            let out = s.start(&mut game, &mut GameRng::new(seed));
            if out.action == StepAction::GotoLabel && out.goto_label.as_deref() == Some("fall") {
                found_oob = true;
                break;
            }
        }
        assert!(found_oob, "expected at least one seed to scatter the carrier out of bounds from x=0");
    }

    #[test]
    fn scatter_always_rolls_even_with_preset_coordinate_to() {
        // Regression test: previously, when COORDINATE_TO was already set (the normal
        // case — it holds the player's intended destination before the mandatory
        // scatter), the step short-circuited and used it directly as the final
        // landing square with no scatter roll and no ReportScatterPlayer data.
        // Ball-and-Chain must *always* scatter one square from coordinateFrom.
        let mut game = make_game();
        let from = FieldCoordinate::new(10, 8);
        let intended_to = FieldCoordinate::new(11, 8); // preset — east bias
        add_ball_and_chain_player(&mut game, "carrier", from);
        let mut step = StepMoveBallAndChain::new("end".into(), "fall".into());
        step.coordinate_from = Some(from);
        step.coordinate_to = Some(intended_to);
        step.start(&mut game, &mut GameRng::new(2));
        assert!(game.report_list.has_report(ReportId::SCATTER_PLAYER),
            "SCATTER_PLAYER report must always be added, even with a preset COORDINATE_TO");
        // The final coordinate_to must be exactly one square from coordinate_from
        // (a scatter result), not the untouched preset value further away.
        let final_to = step.coordinate_to.expect("coordinate_to must be set after scatter");
        let dist = (final_to.x - from.x).abs().max((final_to.y - from.y).abs());
        assert_eq!(dist, 1, "scatter must land exactly one square from coordinateFrom");
    }

    #[test]
    fn occupied_target_square_gotos_end_label() {
        // Place a defender directly east of the carrier's scatter origin so that
        // whichever direction the scatter mechanic resolves to when biased east,
        // it is likely to land on the defender for at least one seed.
        let mut game = make_game();
        let from = FieldCoordinate::new(5, 5);
        let bias_to = FieldCoordinate::new(6, 5);
        add_ball_and_chain_player(&mut game, "carrier", from);
        add_player(&mut game, "defender", FieldCoordinate::new(6, 5));
        let mut found_block = false;
        for seed in 0..64u64 {
            let mut s = StepMoveBallAndChain::new("end".into(), "fall".into());
            s.coordinate_from = Some(from);
            s.coordinate_to = Some(bias_to);
            let out = s.start(&mut game, &mut GameRng::new(seed));
            if out.action == StepAction::GotoLabel && out.goto_label.as_deref() == Some("end") {
                found_block = true;
                break;
            }
        }
        assert!(found_block, "expected at least one seed to scatter the carrier onto the defender's square");
    }

    #[test]
    fn set_parameter_goto_label_on_end_accepted() {
        let mut step = StepMoveBallAndChain::new("old".into(), "fall".into());
        assert!(step.set_parameter(&StepParameter::GotoLabelOnEnd("new".into())));
        assert_eq!(step.goto_label_on_end, "new");
    }

    #[test]
    fn set_parameter_goto_label_on_fall_down_accepted() {
        let mut step = StepMoveBallAndChain::new("end".into(), "old".into());
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFallDown("new".into())));
        assert_eq!(step.goto_label_on_fall_down, "new");
    }

    #[test]
    fn set_parameter_coordinate_from_accepted() {
        let mut step = StepMoveBallAndChain::new("end".into(), "fall".into());
        let coord = FieldCoordinate::new(5, 5);
        assert!(step.set_parameter(&StepParameter::CoordinateFrom(coord)));
        assert_eq!(step.coordinate_from, Some(coord));
    }

    #[test]
    fn set_parameter_coordinate_to_accepted() {
        let mut step = StepMoveBallAndChain::new("end".into(), "fall".into());
        let coord = FieldCoordinate::new(6, 5);
        assert!(step.set_parameter(&StepParameter::CoordinateTo(coord)));
        assert_eq!(step.coordinate_to, Some(coord));
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepMoveBallAndChain::new("end".into(), "fall".into());
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn no_coordinate_from_returns_next_step() {
        let mut game = make_game();
        let mut step = StepMoveBallAndChain::new("end".into(), "fall".into());
        step.coordinate_from = None;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn scatter_player_report_added_with_preset_coordinate() {
        let mut game = make_game();
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(6, 5);
        add_ball_and_chain_player(&mut game, "carrier", from);
        let mut step = StepMoveBallAndChain::new("end".into(), "fall".into());
        step.coordinate_from = Some(from);
        step.coordinate_to = Some(to);
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::SCATTER_PLAYER),
            "should have SCATTER_PLAYER report when ball-and-chain player moves");
    }
}
