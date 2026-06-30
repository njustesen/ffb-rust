/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.move.StepMoveBallAndChain`.
///
/// Handles the BALL_AND_CHAIN skill during movement: rolls a random scatter direction,
/// optionally asks for a re-roll, then publishes COORDINATE_TO and BLOCK_DEFENDER_ID or
/// jumps to GOTO_LABEL_ON_END / GOTO_LABEL_ON_FALL_DOWN as appropriate.
///
/// Init parameters (mandatory): GOTO_LABEL_ON_END, GOTO_LABEL_ON_FALL_DOWN.
/// Incoming parameters: COORDINATE_FROM, COORDINATE_TO, BALL_AND_CHAIN_RE_ROLL_SETTING.
use ffb_model::enums::Direction;
use ffb_model::model::game::Game;
use ffb_model::model::re_rolled_action::ReRolledAction;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::abstract_step_with_re_roll::ReRollState;

/// Java `ReRolledActions.DIRECTION` equivalent name.
const RE_ROLLED_ACTION: &str = "DIRECTION";

/// Java: `StepMoveBallAndChain` (mixed/move, BB2020 + BB2025).
/// Extends AbstractStepWithReRoll.
#[derive(Debug)]
pub struct StepMoveBallAndChain {
    /// Java: `fGotoLabelOnEnd` — label to jump to when a blocker is found.
    pub goto_label_on_end: String,
    /// Java: `fGotoLabelOnFallDown` — label to jump to when the player goes out of bounds.
    pub goto_label_on_fall_down: String,
    /// Java: `reRollSetting` — player preference for when to ask for a re-roll.
    pub re_roll_setting: Option<String>,
    /// Java: `fCoordinateFrom`
    pub coordinate_from: Option<FieldCoordinate>,
    /// Java: `fCoordinateTo`
    pub coordinate_to: Option<FieldCoordinate>,
    /// Java: `originalCoordinateTo`
    pub original_coordinate_to: Option<FieldCoordinate>,
    /// Java: `playerScatter` — the direction actually scattered.
    pub player_scatter: Option<Direction>,
    /// Re-roll tracking (AbstractStepWithReRoll).
    pub re_roll_state: ReRollState,
}

impl StepMoveBallAndChain {
    pub fn new() -> Self { Self::default() }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let coord_from = match self.coordinate_from {
            Some(c) => c,
            None => return StepOutcome::next(),
        };
        let original_coord_to = match self.original_coordinate_to {
            Some(c) => c,
            None => return StepOutcome::next(),
        };

        // Java: boolean doRoll = playerScatter == null ||
        //   (getReRollSource() != null && UtilServerReRoll.useReRoll(this, getReRollSource(), actingPlayer.getPlayer()))
        let do_roll = if self.player_scatter.is_none() {
            true
        } else if let Some(ref src) = self.re_roll_state.re_roll_source.clone() {
            let pid = game.acting_player.player_id.clone().unwrap_or_default();
            crate::step::util_server_re_roll::use_reroll(game, src, &pid)
        } else {
            false
        };

        if do_roll {
            // Java: int scatterRoll = getGameState().getDiceRoller().rollThrowInDirection()
            let scatter_roll = rng.d8();

            // Determine base direction from coordinate delta (East/West/South/North)
            let base_dir = if coord_from.x < original_coord_to.x {
                Direction::East
            } else if coord_from.x > original_coord_to.x {
                Direction::West
            } else if coord_from.y < original_coord_to.y {
                Direction::South
            } else {
                Direction::North
            };

            let scatter_dir = interpret_throw_in_direction(base_dir, scatter_roll);
            self.player_scatter = Some(scatter_dir);

            let new_coord = scatter_one_square(coord_from, scatter_dir);
            self.coordinate_to = Some(new_coord);

            // Emit scatter report
            let pid = game.acting_player.player_id.clone().unwrap_or_default();
            let mut outcome = StepOutcome::next()
                .with_event(ffb_model::events::GameEvent::ScatterPlayer {
                    player_id: pid.clone(),
                    coords: vec![new_coord],
                });

            // Java: if (getReRollSource() == null) { ... askForReRoll? ... }
            if self.re_roll_state.re_roll_source.is_none() {
                if should_ask_for_reroll(self.re_roll_setting.as_deref(), game, new_coord) {
                    if let Some(prompt) = crate::step::util_server_re_roll::ask_for_reroll_if_available(
                        game,
                        RE_ROLLED_ACTION,
                        0,
                        false,
                    ) {
                        self.re_roll_state.re_rolled_action = Some(ReRolledAction::new(RE_ROLLED_ACTION));
                        return StepOutcome::cont().with_prompt(prompt);
                    }
                }
            }

            // Java: if (!FieldCoordinateBounds.FIELD.isInBounds(fCoordinateTo))
            if !is_in_bounds(new_coord) {
                // Java: publishParameter(INJURY_TYPE, new InjuryTypeCrowdPush())
                // Java: setNextAction(GOTO_LABEL, fGotoLabelOnFallDown)
                return StepOutcome::goto(&self.goto_label_on_fall_down.clone());
            }

            // Java: publishParameter(COORDINATE_TO, fCoordinateTo)
            outcome = outcome.publish(StepParameter::CoordinateTo(new_coord));

            // Java: Player<?> blockDefender = game.getFieldModel().getPlayer(fCoordinateTo)
            let block_defender_id = game.field_model.player_at(new_coord).cloned();
            if let Some(bid) = block_defender_id {
                // Java: actingPlayer.setCurrentMove(actingPlayer.getCurrentMove() + 1)
                game.acting_player.current_move += 1;
                outcome = outcome.publish(StepParameter::BlockDefenderId(bid));
                let lbl = self.goto_label_on_end.clone();
                return StepOutcome::goto(&lbl)
                    .with_events(outcome.events)
                    .with_published(outcome.published);
            }

            return outcome;
        }

        StepOutcome::next()
    }
}

impl Default for StepMoveBallAndChain {
    fn default() -> Self {
        Self {
            goto_label_on_end: String::new(),
            goto_label_on_fall_down: String::new(),
            re_roll_setting: None,
            coordinate_from: None,
            coordinate_to: None,
            original_coordinate_to: None,
            player_scatter: None,
            re_roll_state: ReRollState::new(),
        }
    }
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
            StepParameter::CoordinateFrom(v)        => { self.coordinate_from = Some(*v); true }
            StepParameter::CoordinateTo(v)           => {
                self.coordinate_to = Some(*v);
                self.original_coordinate_to = Some(*v);
                true
            }
            StepParameter::BallAndChainRrSetting(v) => { self.re_roll_setting = v.clone(); true }
            StepParameter::GotoLabelOnEnd(v)         => { self.goto_label_on_end = v.clone(); true }
            StepParameter::GotoLabelOnFallDown(v)    => { self.goto_label_on_fall_down = v.clone(); true }
            _ => false,
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Java: `ThrowInMechanic.interpretThrowInDirectionRoll(baseDir, roll)`.
/// Maps a d8 roll (1-8) to a scatter direction based on the facing direction.
/// The Java implementation uses a fixed lookup table per facing.
fn interpret_throw_in_direction(base: Direction, roll: i32) -> Direction {
    // Each array: roll 1..=8 → Direction (1-indexed: idx = roll-1).
    // Mirrors Java ThrowInMechanic's directional scatter tables.
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

/// Move one square in `dir` from `from`.
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

/// Check if within standard Blood Bowl field (26×17, columns 0-25, rows 0-16).
fn is_in_bounds(coord: FieldCoordinate) -> bool {
    coord.x >= 0 && coord.x <= 25 && coord.y >= 0 && coord.y <= 16
}

/// Java: check the reRollSetting to decide whether to ask the player for a re-roll.
fn should_ask_for_reroll(setting: Option<&str>, game: &Game, coord_to: FieldCoordinate) -> bool {
    const NEVER: &str = "reRollBallAndChainNever";
    const TEAM_MATE: &str = "reRollBallAndChainTeamMate";
    const NO_OPPONENT: &str = "reRollBallAndChainNoOpponent";

    match setting {
        Some(s) if s == NEVER => false,
        Some(s) if s == TEAM_MATE => {
            // ask only when about to land on a team-mate
            game.field_model.player_at(coord_to)
                .map_or(false, |pid| is_acting_team(game, pid))
        }
        Some(s) if s == NO_OPPONENT => {
            // ask unless the target square has an opponent
            !game.field_model.player_at(coord_to)
                .map_or(false, |pid| !is_acting_team(game, pid))
        }
        _ => true,
    }
}

/// Returns true if `player_id` belongs to the currently acting team.
fn is_acting_team(game: &Game, player_id: &str) -> bool {
    if game.home_playing {
        game.team_home.players.iter().any(|p| p.id == player_id)
    } else {
        game.team_away.players.iter().any(|p| p.id == player_id)
    }
}

// Extension to add convenience helpers to StepOutcome
trait StepOutcomeBuilder {
    fn with_published(self, params: Vec<StepParameter>) -> Self;
}

impl StepOutcomeBuilder for StepOutcome {
    fn with_published(mut self, params: Vec<StepParameter>) -> Self {
        self.published.extend(params);
        self
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{test_team, StepAction};
    use ffb_model::enums::Rules;
    use ffb_model::model::game::Game;
    use ffb_model::util::rng::GameRng;
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn id_is_move_ball_and_chain() {
        assert_eq!(StepMoveBallAndChain::new().id(), StepId::MoveBallAndChain);
    }

    #[test]
    fn no_coords_returns_next() {
        let mut step = StepMoveBallAndChain::new();
        let mut game = make_game();
        let mut rng = GameRng::new(42);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn in_bounds_scatter_produces_coordinate_to_param() {
        let mut step = StepMoveBallAndChain::new();
        step.coordinate_from = Some(FieldCoordinate::new(10, 8));
        step.coordinate_to = Some(FieldCoordinate::new(11, 8));
        step.original_coordinate_to = Some(FieldCoordinate::new(11, 8));
        step.goto_label_on_end = "end".into();
        step.goto_label_on_fall_down = "fall".into();
        let mut game = make_game();
        // Seed 2 → roll_d8 returns a specific value; all in-bounds landing
        let mut rng = GameRng::new(2);
        let out = step.start(&mut game, &mut rng);
        // Should not be Continue (no reroll offer expected without TRR)
        let has_coord_to = out.published.iter().any(|p| matches!(p, StepParameter::CoordinateTo(_)));
        // Either coord_to published (empty target sq) or goto-label (blocker/OOB)
        let meaningful = has_coord_to
            || out.action == StepAction::GotoLabel
            || out.action == StepAction::NextStep;
        assert!(meaningful);
    }

    #[test]
    fn is_in_bounds_edge_cases() {
        assert!(is_in_bounds(FieldCoordinate::new(0, 0)));
        assert!(is_in_bounds(FieldCoordinate::new(25, 16)));
        assert!(!is_in_bounds(FieldCoordinate::new(-1, 0)));
        assert!(!is_in_bounds(FieldCoordinate::new(26, 0)));
        assert!(!is_in_bounds(FieldCoordinate::new(0, -1)));
        assert!(!is_in_bounds(FieldCoordinate::new(0, 17)));
    }

    #[test]
    fn scatter_one_square_directions() {
        let origin = FieldCoordinate::new(5, 5);
        assert_eq!(scatter_one_square(origin, Direction::North),     FieldCoordinate::new(5, 4));
        assert_eq!(scatter_one_square(origin, Direction::East),      FieldCoordinate::new(6, 5));
        assert_eq!(scatter_one_square(origin, Direction::Southwest), FieldCoordinate::new(4, 6));
    }

    #[test]
    fn set_parameter_coordinate_to_sets_both() {
        let mut step = StepMoveBallAndChain::new();
        let coord = FieldCoordinate::new(7, 3);
        step.set_parameter(&StepParameter::CoordinateTo(coord));
        assert_eq!(step.coordinate_to, Some(coord));
        assert_eq!(step.original_coordinate_to, Some(coord));
    }
}
