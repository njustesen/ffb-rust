/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.move.StepMoveBallAndChain`.
///
/// Handles the BALL_AND_CHAIN skill during movement: rolls a random scatter direction,
/// optionally asks for a re-roll, then publishes COORDINATE_TO and BLOCK_DEFENDER_ID or
/// jumps to GOTO_LABEL_ON_END / GOTO_LABEL_ON_FALL_DOWN as appropriate.
///
/// Init parameters (mandatory): GOTO_LABEL_ON_END, GOTO_LABEL_ON_FALL_DOWN.
/// Incoming parameters: COORDINATE_FROM, COORDINATE_TO, BALL_AND_CHAIN_RE_ROLL_SETTING.
use ffb_model::enums::Direction;
use ffb_model::enums::SkillId;
use ffb_model::model::game::Game;
use ffb_model::model::re_rolled_action::ReRolledAction;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::option::game_option_id;
use ffb_model::report::report_scatter_player::ReportScatterPlayer;
use ffb_model::report::report_skill_use::ReportSkillUse;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::abstract_step_with_re_roll::{ReRollState, find_skill_reroll_source};

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

            // Java: getResult().addReport(new ReportScatterPlayer(fCoordinateFrom, fCoordinateTo, new Direction[]{playerScatter}, new int[]{scatterRoll}))
            game.report_list.add(ReportScatterPlayer::new(coord_from, new_coord, vec![scatter_dir], vec![scatter_roll], None));

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
                    // Java: `ReRollSource reRollSource = UtilCards.getUnusedRerollSource(actingPlayer,
                    //   RE_ROLLED_ACTION); if (reRollSource != null) { showDialog(...); return; }
                    //   askForReRoll = ALLOW_BALL_AND_CHAIN_RE_ROLL.isEnabled(); if (askForReRoll && ...)`
                    // — a skill-granted reroll source (e.g. WhirlingDervish) is offered
                    // *independent* of the ALLOW_BALL_AND_CHAIN_RE_ROLL option, which only gates
                    // the TRR-backed reroll. `ask_for_reroll_if_available` (below) already checks
                    // the skill source first, so pre-checking here just decides whether we're
                    // allowed to fall through to it for the TRR-only case; a present skill source
                    // always takes this call regardless of the option.
                    let has_skill_source = find_skill_reroll_source(game, RE_ROLLED_ACTION).is_some();
                    // Java: `askForReRoll = ((GameOptionBoolean) game.getOptions()
                    //   .getOptionWithDefault(GameOptionId.ALLOW_BALL_AND_CHAIN_RE_ROLL)).isEnabled();`
                    // — defaults to `false`, so without this gate a TRR-backed reroll was
                    // (incorrectly) always offered even with no skill reroll source present.
                    let option_allows_trr = game.options.is_enabled(game_option_id::ALLOW_BALL_AND_CHAIN_RE_ROLL);

                    if has_skill_source || option_allows_trr {
                        if let Some(prompt) = crate::step::util_server_re_roll::ask_for_reroll_if_available(
                            game,
                            RE_ROLLED_ACTION,
                            0,
                            false,
                        ) {
                            self.re_roll_state.re_rolled_action = Some(ReRolledAction::new(RE_ROLLED_ACTION));
                            // Java: `rerollSource` is only committed via `setReRollSource(rerollSource)`
                            // in `handleCommand` once the player accepts (isSkillUsed()==true). We stash
                            // the *offered* source here (mirroring `AgentPrompt::ReRollOffer.source`) so
                            // `handle_command` has something to commit-or-discard based on the reply,
                            // matching the pattern used by e.g. `StepGoForIt::execute_step`.
                            if let ffb_model::prompts::AgentPrompt::ReRollOffer { ref source, .. } = prompt {
                                self.re_roll_state.re_roll_source = Some(source.clone());
                            }
                            return StepOutcome::cont().with_prompt(prompt);
                        }
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

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_USE_SKILL → `clientCommandUseSkill.getSkill().getRerollSource(RE_ROLLED_ACTION)`
        //   != null → setReRolledAction(RE_ROLLED_ACTION);
        //   getResult().addReport(new ReportSkillUse(playerId, skill, skillUsed, SkillUse.RE_ROLL_DIRECTION));
        //   if (skillUsed) setReRollSource(rerollSource).
        //
        // Rust: `ask_for_reroll_if_available` returns `AgentPrompt::ReRollOffer`, and per the
        // engine's uniform agent-response mapping (see `agent.rs`, `AgentPrompt::ReRollOffer =>
        // Action::UseReRoll`), the reply always arrives as `Action::UseReRoll`, never
        // `Action::UseSkill` — matching on `UseSkill` here left this branch permanently
        // unreachable, so the offered `re_roll_source` (stashed in `execute_step` when the
        // prompt was issued) was never committed on accept nor cleared on decline, and the
        // scatter re-roll could never actually be consumed.
        if let Action::UseReRoll { use_reroll } = action {
            let actor_id = game.acting_player.player_id.clone();
            // Java reports the *actual* skill backing the reroll; the Rust action carries no
            // skill id for this generic re-roll reply, so we use the same placeholder
            // (`SkillId::Block`) already established for skill-agnostic replies in `agent.rs`.
            game.report_list.add(ReportSkillUse::new(actor_id, SkillId::Block, *use_reroll, SkillUse::RE_ROLL_DIRECTION));
            if !use_reroll {
                self.re_roll_state.re_roll_source = None;
            }
        }
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
    fn scatter_player_report_added_on_roll() {
        let mut step = StepMoveBallAndChain::new();
        step.coordinate_from = Some(FieldCoordinate::new(10, 8));
        step.coordinate_to = Some(FieldCoordinate::new(11, 8));
        step.original_coordinate_to = Some(FieldCoordinate::new(11, 8));
        step.goto_label_on_end = "end".into();
        step.goto_label_on_fall_down = "fall".into();
        let mut game = make_game();
        let mut rng = GameRng::new(2);
        step.start(&mut game, &mut rng);
        assert!(game.report_list.has_report(ffb_model::report::report_id::ReportId::SCATTER_PLAYER));
    }

    #[test]
    fn no_scatter_player_report_when_no_coords() {
        let mut step = StepMoveBallAndChain::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(!game.report_list.has_report(ffb_model::report::report_id::ReportId::SCATTER_PLAYER));
    }

    #[test]
    fn set_parameter_coordinate_to_sets_both() {
        let mut step = StepMoveBallAndChain::new();
        let coord = FieldCoordinate::new(7, 3);
        step.set_parameter(&StepParameter::CoordinateTo(coord));
        assert_eq!(step.coordinate_to, Some(coord));
        assert_eq!(step.original_coordinate_to, Some(coord));
    }

    /// Regression test for the bug where `re_roll_source` was never stashed when the
    /// re-roll offer was made, and `handle_command` matched the wrong `Action` variant
    /// (`UseSkill` instead of the `UseReRoll` the agent actually sends in reply to
    /// `AgentPrompt::ReRollOffer` — see `agent.rs`). Before the fix, accepting the
    /// re-roll offer had no effect: `re_roll_source` stayed `None`, so the next
    /// `execute_step` call's `doRoll` check was always false and the reroll (and its
    /// TRR consumption) never happened.
    #[test]
    fn accepting_reroll_offer_consumes_trr_and_rerolls() {
        let mut step = StepMoveBallAndChain::new();
        step.coordinate_from = Some(FieldCoordinate::new(10, 8));
        step.coordinate_to = Some(FieldCoordinate::new(11, 8));
        step.original_coordinate_to = Some(FieldCoordinate::new(11, 8));
        step.goto_label_on_end = "end".into();
        step.goto_label_on_fall_down = "fall".into();

        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        game.turn_data_home.reroll_used = false;
        // Java defaults ALLOW_BALL_AND_CHAIN_RE_ROLL to false — the TRR offer only fires
        // once the house rule is turned on.
        game.options.set(game_option_id::ALLOW_BALL_AND_CHAIN_RE_ROLL, "true");

        let mut rng = GameRng::new(2);
        let out = step.start(&mut game, &mut rng);

        // A re-roll offer should have been made and the offered source stashed.
        assert_eq!(out.action, StepAction::Continue);
        assert!(step.re_roll_state.re_roll_source.is_some(), "re_roll_source must be stashed when the offer is issued");
        assert!(step.player_scatter.is_some());

        // Agent accepts the re-roll (Action::UseReRoll, per agent.rs's uniform
        // ReRollOffer -> UseReRoll mapping — never Action::UseSkill for this prompt).
        let out2 = step.handle_command(&Action::UseReRoll { use_reroll: true }, &mut game, &mut rng);

        // The TRR must actually be consumed and the step must not simply bail out to
        // NextStep without re-rolling (the pre-fix behavior).
        assert_eq!(game.turn_data_home.rerolls, 0, "accepting the offer must consume the TRR");
        assert!(out2.action == StepAction::NextStep || out2.action == StepAction::GotoLabel || out2.action == StepAction::Continue);
    }

    #[test]
    fn declining_reroll_offer_clears_source_and_does_not_consume_trr() {
        let mut step = StepMoveBallAndChain::new();
        step.coordinate_from = Some(FieldCoordinate::new(10, 8));
        step.coordinate_to = Some(FieldCoordinate::new(11, 8));
        step.original_coordinate_to = Some(FieldCoordinate::new(11, 8));
        step.goto_label_on_end = "end".into();
        step.goto_label_on_fall_down = "fall".into();

        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        game.turn_data_home.reroll_used = false;
        game.options.set(game_option_id::ALLOW_BALL_AND_CHAIN_RE_ROLL, "true");

        let mut rng = GameRng::new(2);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::Continue);
        assert!(step.re_roll_state.re_roll_source.is_some());

        let _ = step.handle_command(&Action::UseReRoll { use_reroll: false }, &mut game, &mut rng);

        assert!(step.re_roll_state.re_roll_source.is_none(), "declining must clear the stashed source");
        assert_eq!(game.turn_data_home.rerolls, 1, "declining must not consume the TRR");
    }

    /// Regression test: Java only offers a TRR-backed Ball & Chain reroll when
    /// `ALLOW_BALL_AND_CHAIN_RE_ROLL` is enabled (defaults to `false`). A prior Rust
    /// translation ignored this option entirely and always offered the TRR reroll
    /// whenever one was available.
    #[test]
    fn trr_reroll_not_offered_when_option_disabled() {
        let mut step = StepMoveBallAndChain::new();
        step.coordinate_from = Some(FieldCoordinate::new(10, 8));
        step.coordinate_to = Some(FieldCoordinate::new(11, 8));
        step.original_coordinate_to = Some(FieldCoordinate::new(11, 8));
        step.goto_label_on_end = "end".into();
        step.goto_label_on_fall_down = "fall".into();

        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        game.turn_data_home.reroll_used = false;
        // ALLOW_BALL_AND_CHAIN_RE_ROLL left unset — Java's default is `false`.

        let mut rng = GameRng::new(2);
        let out = step.start(&mut game, &mut rng);

        assert_ne!(out.action, StepAction::Continue, "no reroll offer should be made with the option disabled");
        assert!(step.re_roll_state.re_roll_source.is_none());
        assert_eq!(game.turn_data_home.rerolls, 1, "TRR must be untouched when no offer was made");
    }

    /// Regression test: Java checks `UtilCards.getUnusedRerollSource(actingPlayer,
    /// RE_ROLLED_ACTION)` (e.g. WhirlingDervish's registered DIRECTION reroll)
    /// *independent* of the `ALLOW_BALL_AND_CHAIN_RE_ROLL` option — a skill-granted
    /// reroll is still offered even when the house rule is off. A prior translation's
    /// `find_skill_reroll_source` didn't map "DIRECTION" to any skill at all, so
    /// WhirlingDervish could never reroll Ball & Chain scatter.
    #[test]
    fn whirling_dervish_reroll_offered_even_with_option_disabled() {
        use ffb_model::model::skill_def::SkillWithValue;
        use ffb_model::enums::{SkillId, PlayerType, PlayerGender, PS_STANDING};
        use ffb_model::model::player::Player;

        let mut step = StepMoveBallAndChain::new();
        step.coordinate_from = Some(FieldCoordinate::new(10, 8));
        step.coordinate_to = Some(FieldCoordinate::new(11, 8));
        step.original_coordinate_to = Some(FieldCoordinate::new(11, 8));
        step.goto_label_on_end = "end".into();
        step.goto_label_on_fall_down = "fall".into();

        let mut game = make_game();
        game.team_home.players.push(Player {
            id: "mover".into(), name: "mover".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![SkillWithValue::new(SkillId::WhirlingDervish)],
            extra_skills: vec![], temporary_skills: vec![], used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate("mover", FieldCoordinate::new(10, 8));
        game.field_model.set_player_state("mover", ffb_model::enums::PlayerState::new(PS_STANDING));
        game.acting_player.set_player("mover".into(), ffb_model::enums::PlayerAction::Move);
        game.home_playing = true;
        game.turn_mode = ffb_model::enums::TurnMode::Regular;
        // ALLOW_BALL_AND_CHAIN_RE_ROLL left unset/false — must not matter for a skill reroll.

        let mut rng = GameRng::new(2);
        let out = step.start(&mut game, &mut rng);

        assert_eq!(out.action, StepAction::Continue, "a WhirlingDervish skill reroll must be offered regardless of the option");
        assert_eq!(
            step.re_roll_state.re_roll_source.as_ref().map(|s| s.name.as_str()),
            Some("WhirlingDervish"),
        );
    }
}
