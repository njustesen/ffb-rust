use ffb_model::enums::{Direction, SkillId, TurnMode};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::re_rolled_action::ReRolledAction;
use ffb_model::prompts::AgentPrompt;
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds};
use ffb_model::util::rng::GameRng;
use ffb_mechanics::bb2025::throw_in_mechanic::ThrowInMechanic;
use ffb_mechanics::throw_in_mechanic::ThrowInMechanic as ThrowInMechanicTrait;
use ffb_mechanics::modifiers::catch_context::CatchContext;
use ffb_mechanics::modifiers::catch_modifier_factory::CatchModifierFactory;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::step::abstract_step_with_re_roll::ReRollState;
use crate::step::framework::{Step, StepAction, StepOutcome};
use crate::step::framework::{CatchScatterThrowInMode, StepId, StepParameter, SequenceStep};
use crate::step::generator::common::SpikedBallApo;
use ffb_model::option::game_option_id;
use crate::step::util_server_catch_scatter_throw_in::UtilServerCatchScatterThrowIn;
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use ffb_model::report::report_catch_roll::ReportCatchRoll;
use ffb_model::report::report_scatter_ball::ReportScatterBall;
use ffb_model::report::report_throw_in::ReportThrowIn;
use ffb_model::report::report_skill_use::ReportSkillUse;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::enums::ReRollSource;
use crate::skill_behaviour::dispatch;

// Model's CatchScatterThrowInMode (SCREAMING_SNAKE_CASE) needed for CatchContext.

// ── Hook state ─────────────────────────────────────────────────────────────────

/// Java: StepCatchScatterThrowIn.StepState (the subset needed by the Catch/MonstrousMouth
/// skill-modifier hooks) — mutable state passed through executeStepHooks.
/// Exported so Catch/MonstrousMouth step-modifiers can downcast to it.
#[derive(Debug)]
pub struct StepCatchHookState {
    /// Java: state.catcher.getId() — carried as a player id for headless
    pub catcher_id: String,
    /// Java: state.rerollCatch — out-param, true when a skill grants an automatic catch reroll
    pub reroll_catch: bool,
    /// Java: step.setReRolledAction(...) — out-param
    pub re_rolled_action: Option<ReRolledAction>,
    /// Java: step.setReRollSource(...) — out-param
    pub re_roll_source: Option<ReRollSource>,
}

impl StepCatchHookState {
    pub fn new(catcher_id: String) -> Self {
        Self { catcher_id, reroll_catch: false, re_rolled_action: None, re_roll_source: None }
    }
}

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.shared.StepCatchScatterThrowIn.
///
/// Dispatches on CatchScatterThrowInMode to handle:
///   - Catch ball (regular, hand-off, accurate pass, kickoff, scatter, punt)
///   - Bomb catch variants (CATCH_BOMB, DEFLECTED_BOMB, …)
///   - Ball scatter (one bounce or 3-square scatter)
///   - Throw-in
///   - Failed-catch / failed-pick-up (with spiked-ball apo sub-sequence)
///   - Diving catch (3-phase: ASK_HOME → ASK_AWAY → PROCESS)
///
/// Uses StepAction::Repeat to loop: Java pushCurrentStepOnStack+NEXT_STEP
/// and Java REPEAT both map to Rust Repeat (driver re-calls start() on same instance).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DivingCatchPhase {
    AskHome,
    AskAway,
    Process,
}

impl Default for DivingCatchPhase {
    fn default() -> Self { DivingCatchPhase::AskHome }
}

pub struct StepCatchScatterThrowIn {
    /// Java: fCatcherId
    pub catcher_id: Option<String>,
    /// Java: fScatterBounds — computed at start of execute_step from TurnMode
    pub scatter_bounds: FieldCoordinateBounds,
    /// Java: fCatchScatterThrowInMode
    pub catch_scatter_throw_in_mode: Option<CatchScatterThrowInMode>,
    /// Java: fThrowInCoordinate
    pub throw_in_coordinate: Option<FieldCoordinate>,
    /// Java: fBombMode
    pub bomb_mode: bool,
    /// Java: evaluate (blast-it flag for catch re-evaluation)
    pub evaluate: bool,
    /// Java: phase (DivingCatchPhase) — default ASK_HOME
    pub phase: DivingCatchPhase,
    /// Java: divingCatchControlTeam
    pub diving_catch_control_team: Option<String>,
    /// Java: divingCatchers (List<String> player IDs)
    pub diving_catchers: Vec<String>,
    /// Java: roll (catch roll result)
    pub roll: i32,
    /// Java: usingModifyingSkill (Boolean tristate)
    pub using_modifying_skill: Option<bool>,
    /// Java: state.rerollCatch (from AbstractStepWithReRoll / hooks)
    pub reroll_catch: bool,
    /// Java: AbstractStepWithReRoll fields
    pub re_roll_state: ReRollState,
    /// Java: transient boolean repeat
    pub repeat: bool,
    /// Pending prompt (mirrors Java's game.dialogParameter). Set by catch_ball/diving_catch.
    pending_prompt: Option<AgentPrompt>,
    /// Events accumulated during this invocation of execute_step.
    pending_events: Vec<GameEvent>,
    /// Parameters to publish accumulated during this invocation.
    pending_published: Vec<StepParameter>,
    /// Set when SPIKED_BALL triggers in handle_failed_catch — push SpikedBallApo before scatter.
    pending_spiked_ball: bool,
}

impl StepCatchScatterThrowIn {
    pub fn new() -> Self {
        Self {
            catcher_id: None,
            scatter_bounds: FieldCoordinateBounds::FIELD,
            catch_scatter_throw_in_mode: None,
            throw_in_coordinate: None,
            bomb_mode: false,
            evaluate: false,
            phase: DivingCatchPhase::AskHome,
            diving_catch_control_team: None,
            diving_catchers: Vec::new(),
            roll: 0,
            using_modifying_skill: None,
            reroll_catch: false,
            re_roll_state: ReRollState::new(),
            repeat: false,
            pending_prompt: None,
            pending_events: Vec::new(),
            pending_published: Vec::new(),
            pending_spiked_ball: false,
        }
    }
}

impl Default for StepCatchScatterThrowIn {
    fn default() -> Self { Self::new() }
}

impl Step for StepCatchScatterThrowIn {
    fn id(&self) -> StepId { StepId::CatchScatterThrowIn }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::SelectPlayer {player_id } => {
                match self.phase {
                    DivingCatchPhase::AskHome => {
                        if !player_id.is_empty() {
                            self.diving_catchers.push(player_id.clone());
                            self.diving_catch_control_team = Some(game.team_home.id.clone());
                        }
                        self.phase = DivingCatchPhase::AskAway;
                    }
                    DivingCatchPhase::AskAway => {
                        if !player_id.is_empty() {
                            self.diving_catchers.push(player_id.clone());
                            if self.diving_catch_control_team.is_none() {
                                self.diving_catch_control_team = Some(game.team_away.id.clone());
                            }
                        }
                        self.phase = DivingCatchPhase::Process;
                    }
                    DivingCatchPhase::Process => {
                        self.catcher_id = Some(player_id.clone());
                    }
                }
            }
            Action::UseSkill { skill_id: _, use_skill } => {
                // Java: if skill hasProperty grantsCatchBonusToReceiver → evaluate=true
                // Java: else if rerollSource != null → use re-roll
                self.evaluate = *use_skill;
            }
            Action::UseReRoll { use_reroll: true } => {
                // Player accepted the re-roll offer
                if let Some(source) = self.re_roll_state.re_roll_source.clone() {
                    let pid = self.catcher_id.clone().unwrap_or_default();
                    use_reroll(game, &source, &pid);
                    self.roll = 0; // fresh roll
                }
            }
            Action::UseReRoll { use_reroll: false } => {
                // Player declined the re-roll
                self.re_roll_state.re_roll_source = None;
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::CatcherId(v) => { self.catcher_id = v.clone(); true }
            StepParameter::KickoffBounds(v) => { self.scatter_bounds = *v; true }
            StepParameter::CatchScatterThrowInMode(v) => {
                self.catch_scatter_throw_in_mode = Some(*v);
                true
            }
            StepParameter::ThrowInCoordinate(v) => { self.throw_in_coordinate = Some(*v); true }
            StepParameter::UsingModifyingSkill(v) => { self.using_modifying_skill = Some(*v); true }
            _ => false,
        }
    }
}

impl StepCatchScatterThrowIn {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: getResult().reset(); replay pending reports
        // Java: UtilServerDialog.hideDialog(getGameState())
        self.pending_events.clear();
        self.pending_published.clear();
        self.pending_prompt = None;

        if self.catch_scatter_throw_in_mode.is_none() {
            return StepOutcome::next();
        }

        // Java: fScatterBounds = TurnMode.KICKOFF → receiving half bounds, else FIELD.
        self.scatter_bounds = if game.turn_mode == TurnMode::Kickoff {
            if game.home_playing { FieldCoordinateBounds::HALF_AWAY } else { FieldCoordinateBounds::HALF_HOME }
        } else {
            FieldCoordinateBounds::FIELD
        };

        // Player under the regular ball (not bomb)
        let player_under_ball = game.field_model.ball_coordinate
            .and_then(|c| game.field_model.player_at(c))
            .map(|id| id.clone());

        let mut deflected_bomb = false;
        let mut deflected_pass = false;

        match self.catch_scatter_throw_in_mode {
            Some(CatchScatterThrowInMode::DeflectedBomb) => {
                deflected_bomb = true;
                // fall through to bomb handling
                self.handle_bomb(game, rng, deflected_bomb);
            }
            Some(CatchScatterThrowInMode::CatchBomb)
            | Some(CatchScatterThrowInMode::CatchAccurateBombEmptySquare)
            | Some(CatchScatterThrowInMode::CatchAccurateBomb) => {
                self.handle_bomb(game, rng, deflected_bomb);
            }
            Some(CatchScatterThrowInMode::Deflected) => {
                deflected_pass = true;
                // fall through to regular catch
                self.handle_regular_catch(game, rng, deflected_pass, &player_under_ball);
            }
            Some(CatchScatterThrowInMode::CatchAccuratePass)
            | Some(CatchScatterThrowInMode::CatchHandOff)
            | Some(CatchScatterThrowInMode::CatchKickoff)
            | Some(CatchScatterThrowInMode::CatchScatter)
            | Some(CatchScatterThrowInMode::CatchPunt) => {
                self.handle_regular_catch(game, rng, deflected_pass, &player_under_ball);
            }
            Some(CatchScatterThrowInMode::CatchThrowIn)
            | Some(CatchScatterThrowInMode::CatchAccuratePassEmptySquare)
            | Some(CatchScatterThrowInMode::CatchMissedPass) => {
                self.bomb_mode = false;
                if player_under_ball.is_some() {
                    self.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::CatchScatter);
                } else if let Some(coord) = game.field_model.ball_coordinate {
                    let new_mode = self.diving_catch(game, rng, coord);
                    self.catch_scatter_throw_in_mode = new_mode;
                }
            }
            Some(CatchScatterThrowInMode::ThrowIn) => {
                self.bomb_mode = false;
                if self.throw_in_coordinate.is_some() {
                    let new_mode = self.throw_in_ball(game, rng);
                    self.catch_scatter_throw_in_mode = new_mode;
                } else {
                    self.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::ScatterBall);
                }
            }
            Some(CatchScatterThrowInMode::FailedDeflectionConversion) => {
                deflected_pass = true;
                // fall through to failed catch
                self.handle_failed_catch(game, deflected_pass, &player_under_ball);
            }
            Some(CatchScatterThrowInMode::FailedCatch)
            | Some(CatchScatterThrowInMode::FailedPickUp) => {
                self.handle_failed_catch(game, deflected_pass, &player_under_ball);
            }
            Some(CatchScatterThrowInMode::ScatterBall) => {
                self.bomb_mode = false;
                if game.field_model.ball_in_play {
                    let new_mode = self.bounce_ball(game, rng);
                    self.catch_scatter_throw_in_mode = new_mode;
                } else {
                    self.catch_scatter_throw_in_mode = None;
                }
            }
            Some(CatchScatterThrowInMode::ThreeSquareScatter) => {
                self.bomb_mode = false;
                if game.field_model.ball_in_play {
                    let new_mode = self.scatter_ball(game, rng);
                    self.catch_scatter_throw_in_mode = new_mode;
                } else {
                    self.catch_scatter_throw_in_mode = None;
                }
            }
            None => {}
        }

        // Build the base outcome from accumulated events
        let events = std::mem::take(&mut self.pending_events);
        let published = std::mem::take(&mut self.pending_published);

        // Java: if (getReRolledAction() != null || game.getDialogParameter() != null) CONTINUE
        if self.re_roll_state.re_rolled_action.is_some() || self.pending_prompt.is_some() {
            let prompt = self.pending_prompt.take();
            let mut out = StepOutcome::cont().with_events(events);
            for p in published { out = out.publish(p); }
            if let Some(p) = prompt { out = out.with_prompt(p); }
            return out;
        }

        // Java: if (repeat) { repeat = false; REPEAT; return; }
        if self.repeat {
            self.repeat = false;
            return StepOutcome {
                action: StepAction::Repeat,
                goto_label: None,
                events,
                pushes: vec![],
                published,
                prompt: None,
                clear_stack: false,
            };
        }

        // Java: spiked-ball path → push SpikedBallApo then re-push CatchScatterThrowIn with next mode
        if self.pending_spiked_ball {
            self.pending_spiked_ball = false;
            let next_mode = self.catch_scatter_throw_in_mode.take()
                .unwrap_or(CatchScatterThrowInMode::ScatterBall);
            let scatter_seq = vec![SequenceStep {
                step_id: StepId::CatchScatterThrowIn,
                params: vec![StepParameter::CatchScatterThrowInMode(next_mode)],
                label: None,
            }];
            let mut out = StepOutcome::next().with_events(events);
            for p in published { out = out.publish(p); }
            out = out.push_seq(scatter_seq).push_seq(SpikedBallApo::build_sequence());
            return out;
        }

        // Java: if (fCatchScatterThrowInMode != null) pushCurrentStepOnStack → NEXT_STEP
        // Rust: StepAction::Repeat re-calls start() on the same instance (equivalent)
        if self.catch_scatter_throw_in_mode.is_some() {
            return StepOutcome {
                action: StepAction::Repeat,
                goto_label: None,
                events,
                pushes: vec![],
                published,
                prompt: None,
                clear_stack: false,
            };
        }

        // Terminal: mode is null
        let scatter_bounds = self.scatter_bounds;
        let catcher_id: Option<String>;
        if self.bomb_mode {
            catcher_id = if !game.field_model.bomb_moving {
                game.field_model.bomb_coordinate
                    .and_then(|c| game.field_model.player_at(c))
                    .map(|id| id.clone())
            } else {
                None
            };
        } else {
            let ball_coord = game.field_model.ball_coordinate;
            let catcher_at_ball = ball_coord
                .and_then(|c| game.field_model.player_at(c))
                .map(|id| id.clone());
            // Java: check QuickBite adjacents
            if let (Some(ref catcher), Some(ball)) = (&catcher_at_ball, ball_coord) {
                use ffb_model::util::util_player::UtilPlayer;
                let qb_opponents = UtilPlayer::find_adjacent_opposing_players_with_property(
                    game, catcher, ball,
                    NamedProperties::CAN_ATTACK_OPPONENT_FOR_BALL_AFTER_CATCH,
                    false,
                );
                if !qb_opponents.is_empty() {
                    use crate::step::generator::mixed::quick_bite::QuickBite;
                    let seq = QuickBite::build_sequence();
                    let mut out2 = StepOutcome::next().with_events(events.clone());
                    for p in published.clone() { out2 = out2.publish(p); }
                    out2 = out2.publish(StepParameter::CatcherId(catcher_at_ball.clone()));
                    self.deactivate_cards(game);
                    return out2.push_seq(seq);
                }
            }
            catcher_id = catcher_at_ball;
        }

        let mut out = StepOutcome::next().with_events(events);
        for p in published { out = out.publish(p); }
        out = out.publish(StepParameter::CatcherId(catcher_id));

        // Java: deactivateCards()
        self.deactivate_cards(game);

        // Java: if kickoff && !fScatterBounds.isInBounds(ballCoord) → TOUCHBACK
        if game.turn_mode == TurnMode::Kickoff {
            let ball_oob = game.field_model.ball_coordinate
                .map(|c| !scatter_bounds.is_in_bounds(c))
                .unwrap_or(false);
            if ball_oob {
                out = out.publish(StepParameter::Touchback(true));
            }
        }

        out
    }

    /// Java: handle CATCH_BOMB / CATCH_ACCURATE_BOMB / CATCH_ACCURATE_BOMB_EMPTY_SQUARE / DEFLECTED_BOMB
    fn handle_bomb(&mut self, game: &mut Game, rng: &mut GameRng, deflected_bomb: bool) {
        self.bomb_mode = true;
        if self.catcher_id.is_none() {
            self.catcher_id = game.field_model.bomb_coordinate
                .and_then(|c| game.field_model.player_at(c))
                .map(|id| id.clone());
        }
        if let Some(ref cid) = self.catcher_id.clone() {
            let catcher_state = game.field_model.player_state(cid);
            let has_tz = catcher_state.map(|s| s.has_tacklezones()).unwrap_or(false);
            if has_tz && game.field_model.bomb_moving {
                let new_mode = self.catch_ball(game, rng);
                self.catch_scatter_throw_in_mode = new_mode;
            } else {
                self.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::ScatterBall);
            }
        } else {
            if self.catch_scatter_throw_in_mode == Some(CatchScatterThrowInMode::CatchAccurateBomb) {
                self.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::CatchBomb);
            }
            let bomb_coord = game.field_model.bomb_coordinate.unwrap_or(FieldCoordinate::new(0, 0));
            let new_mode = self.diving_catch(game, rng, bomb_coord);
            self.catch_scatter_throw_in_mode = new_mode;
        }
        let mode = self.catch_scatter_throw_in_mode;
        if mode == Some(CatchScatterThrowInMode::FailedCatch)
            || mode == Some(CatchScatterThrowInMode::ScatterBall)
        {
            if deflected_bomb {
                let new_mode = self.scatter_bomb(game, rng);
                self.catch_scatter_throw_in_mode = new_mode;
            } else {
                game.field_model.bomb_moving = true;
                self.catch_scatter_throw_in_mode = None;
            }
        }
    }

    /// Java: handle CATCH_ACCURATE_PASS / CATCH_HAND_OFF / CATCH_KICKOFF / CATCH_SCATTER / CATCH_PUNT / DEFLECTED
    fn handle_regular_catch(&mut self, game: &mut Game, rng: &mut GameRng, deflected_pass: bool, player_under_ball: &Option<String>) {
        self.bomb_mode = false;
        if self.catcher_id.is_none() {
            self.catcher_id = player_under_ball.clone();
        }
        if let Some(ref cid) = self.catcher_id.clone() {
            let catcher_state = game.field_model.player_state(cid);
            let has_tz = catcher_state.map(|s| s.has_tacklezones()).unwrap_or(false);
            if game.field_model.ball_in_play && game.field_model.ball_moving {
                if has_tz {
                    let new_mode = self.catch_ball(game, rng);
                    // Java has two branches:
                    //   1. result==null && deflectedPass && passState.isDeflectionSuccessful() → setInterceptionSuccessful(true)
                    //      In bb2025, StepIntercept uses setInterceptionSuccessful() directly; setDeflectionSuccessful() is
                    //      never called, so isDeflectionSuccessful() always returns false — this branch is dead code.
                    //   2. result==FAILED_CATCH && deflectedPass → FAILED_DEFLECTION_CONVERSION  [implemented below]
                    if new_mode == Some(CatchScatterThrowInMode::FailedCatch) && deflected_pass {
                        self.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::FailedDeflectionConversion);
                    } else {
                        self.catch_scatter_throw_in_mode = new_mode;
                    }
                } else {
                    self.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::FailedCatch);
                }
            } else {
                self.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::ScatterBall);
            }
        } else if self.catch_scatter_throw_in_mode == Some(CatchScatterThrowInMode::CatchKickoff) {
            if let Some(coord) = game.field_model.ball_coordinate {
                let new_mode = self.diving_catch(game, rng, coord);
                self.catch_scatter_throw_in_mode = new_mode;
            }
        } else {
            self.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::ScatterBall);
        }
    }

    /// Java: handle FAILED_CATCH / FAILED_PICK_UP / FAILED_DEFLECTION_CONVERSION
    fn handle_failed_catch(&mut self, game: &mut Game, deflected_pass: bool, player_under_ball: &Option<String>) {
        self.bomb_mode = false;
        let ball_in_play = game.field_model.ball_in_play;

        // Java: spiked-ball check (UtilGameOption.isOptionEnabled(SPIKED_BALL)
        //        || game.isActive(NamedProperties.droppedBallCausesArmourRoll))
        if player_under_ball.is_some() && ball_in_play
            && (game.options.is_enabled(game_option_id::SPIKED_BALL)
                || game.is_active(NamedProperties::DROPPED_BALL_CAUSES_ARMOUR_ROLL))
        {
            self.pending_spiked_ball = true;
            self.catch_scatter_throw_in_mode = if deflected_pass {
                Some(CatchScatterThrowInMode::ThreeSquareScatter)
            } else {
                Some(CatchScatterThrowInMode::ScatterBall)
            };
            return;
        }

        if deflected_pass {
            self.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::ThreeSquareScatter);
            return;
        }

        // Fall through to SCATTER_BALL
        self.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::ScatterBall);
    }

    /// Java: bounceBall() — roll 1×d8, advance ball 1 square, check player/bounds.
    fn bounce_ball(&mut self, game: &mut Game, rng: &mut GameRng) -> Option<CatchScatterThrowInMode> {
        self.re_roll_state = ReRollState::new();
        self.catcher_id = None;

        let roll = rng.d8();
        let direction = Direction::for_roll(roll).unwrap_or(Direction::North);
        let ball_coord_start = game.field_model.ball_coordinate.unwrap_or(FieldCoordinate::new(0, 0));
        let ball_coord_end = UtilServerCatchScatterThrowIn::find_scatter_coordinate(ball_coord_start, direction, 1);

        let last_valid = if self.scatter_bounds.is_in_bounds(ball_coord_end) { ball_coord_end } else { ball_coord_start };

        self.pending_events.push(GameEvent::ScatterBall {
            from: ball_coord_start,
            directions: vec![roll],
        });
        game.report_list.add(ReportScatterBall::new(vec![direction], vec![roll], false));

        game.field_model.ball_coordinate = Some(ball_coord_end);
        game.field_model.ball_moving = true;

        if self.scatter_bounds.is_in_bounds(ball_coord_end) {
            if let Some(pid) = game.field_model.player_at(ball_coord_end).map(|id| id.clone()) {
                let has_tz = game.field_model.player_state(&pid)
                    .map(|s| s.has_tacklezones()).unwrap_or(false);
                if has_tz {
                    self.catcher_id = Some(pid);
                    return Some(CatchScatterThrowInMode::CatchScatter);
                } else {
                    return Some(CatchScatterThrowInMode::FailedCatch);
                }
            }
            // No player at landing square — ball rests
            None
        } else {
            game.field_model.out_of_bounds = true;
            if self.scatter_bounds == FieldCoordinateBounds::FIELD {
                self.throw_in_coordinate = Some(last_valid);
                Some(CatchScatterThrowInMode::ThrowIn)
            } else {
                // Kickoff bounds — touchback
                self.pending_published.push(StepParameter::Touchback(true));
                None
            }
        }
    }

    /// Java: scatterBall() — up to 3×d8 scatters, check OOB/player.
    fn scatter_ball(&mut self, game: &mut Game, rng: &mut GameRng) -> Option<CatchScatterThrowInMode> {
        self.re_roll_state = ReRollState::new();
        self.catcher_id = None;

        let mut last_valid = game.field_model.ball_coordinate.unwrap_or(FieldCoordinate::new(0, 0));
        let mut rolls = Vec::new();
        let mut directions: Vec<Direction> = Vec::new();
        let mut in_bounds = true;

        while in_bounds && rolls.len() < 3 {
            let roll = rng.d8();
            let direction = Direction::for_roll(roll).unwrap_or(Direction::North);
            let next = UtilServerCatchScatterThrowIn::find_scatter_coordinate(last_valid, direction, 1);
            if self.scatter_bounds.is_in_bounds(next) {
                last_valid = next;
                rolls.push(roll);
                directions.push(direction);
            } else {
                in_bounds = false;
            }
        }

        self.pending_events.push(GameEvent::ScatterBall {
            from: game.field_model.ball_coordinate.unwrap_or(FieldCoordinate::new(0, 0)),
            directions: rolls.clone(),
        });
        game.report_list.add(ReportScatterBall::new(directions, rolls.clone(), false));

        game.field_model.ball_coordinate = Some(last_valid);
        game.field_model.ball_moving = true;

        if in_bounds {
            if let Some(pid) = game.field_model.player_at(last_valid).map(|id| id.clone()) {
                let has_tz = game.field_model.player_state(&pid)
                    .map(|s| s.has_tacklezones()).unwrap_or(false);
                if has_tz {
                    self.catcher_id = Some(pid);
                    return Some(CatchScatterThrowInMode::CatchScatter);
                } else {
                    return Some(CatchScatterThrowInMode::FailedCatch);
                }
            }
            Some(CatchScatterThrowInMode::ScatterBall)
        } else {
            self.throw_in_coordinate = Some(last_valid);
            Some(CatchScatterThrowInMode::ThrowIn)
        }
    }

    /// Java: scatterBomb() — up to 3×d8 scatters using bomb_coordinate.
    fn scatter_bomb(&mut self, game: &mut Game, rng: &mut GameRng) -> Option<CatchScatterThrowInMode> {
        self.re_roll_state = ReRollState::new();
        self.catcher_id = None;

        let mut last_valid = game.field_model.bomb_coordinate.unwrap_or(FieldCoordinate::new(0, 0));
        let mut rolls = Vec::new();
        let mut directions: Vec<Direction> = Vec::new();
        let mut in_bounds = true;

        while in_bounds && rolls.len() < 3 {
            let roll = rng.d8();
            let direction = Direction::for_roll(roll).unwrap_or(Direction::North);
            let next = UtilServerCatchScatterThrowIn::find_scatter_coordinate(last_valid, direction, 1);
            if self.scatter_bounds.is_in_bounds(next) {
                last_valid = next;
                rolls.push(roll);
                directions.push(direction);
            } else {
                in_bounds = false;
            }
        }

        self.pending_events.push(GameEvent::ScatterBall {
            from: game.field_model.bomb_coordinate.unwrap_or(FieldCoordinate::new(0, 0)),
            directions: rolls.clone(),
        });
        game.report_list.add(ReportScatterBall::new(directions, rolls.clone(), false));

        game.field_model.bomb_coordinate = Some(last_valid);
        game.field_model.bomb_moving = true;

        if in_bounds {
            if let Some(pid) = game.field_model.player_at(last_valid).map(|id| id.clone()) {
                let has_tz = game.field_model.player_state(&pid)
                    .map(|s| s.has_tacklezones()).unwrap_or(false);
                if has_tz {
                    self.catcher_id = Some(pid);
                    return Some(CatchScatterThrowInMode::CatchBomb);
                }
            }
            None
        } else {
            game.field_model.bomb_coordinate = None;
            game.field_model.bomb_moving = false;
            None
        }
    }

    /// Java: throwInBall() — corner/sideline roll, 2d6 distance, advance step-by-step.
    fn throw_in_ball(&mut self, game: &mut Game, rng: &mut GameRng) -> Option<CatchScatterThrowInMode> {
        let mechanic = ThrowInMechanic::new();
        let start = self.throw_in_coordinate.unwrap_or(FieldCoordinate::new(0, 0));
        self.catcher_id = None;

        let is_corner = mechanic.is_corner_throw_in(start);
        let direction_roll = if is_corner { rng.d3() } else { rng.d6() };
        let direction = mechanic.interpret_throw_in_direction_roll(start, direction_roll);
        let d1 = rng.d6();
        let d2 = rng.d6();
        let distance = mechanic.distance(&[d1, d2]);

        let mut ball_coord_end = start;
        let mut last_valid = start;
        for i in 0..distance {
            ball_coord_end = UtilServerCatchScatterThrowIn::find_scatter_coordinate(start, direction, i);
            if FieldCoordinateBounds::FIELD.is_in_bounds(ball_coord_end) {
                last_valid = ball_coord_end;
            }
        }

        self.pending_events.push(GameEvent::ThrowIn {
            coord: start,
            direction: direction_roll,
            distance,
        });
        game.report_list.add(ReportThrowIn::new(direction, direction_roll, vec![d1, d2]));

        game.field_model.ball_moving = true;

        if ball_coord_end == last_valid {
            game.field_model.out_of_bounds = false;
            game.field_model.ball_coordinate = Some(last_valid);
            self.throw_in_coordinate = None;
            Some(CatchScatterThrowInMode::CatchThrowIn)
        } else {
            game.field_model.ball_coordinate = None;
            self.throw_in_coordinate = Some(last_valid);
            Some(CatchScatterThrowInMode::ThrowIn)
        }
    }

    /// Java: deactivateCards() — deactivate WHILE_HOLDING_THE_BALL cards for non-carriers.
    fn deactivate_cards(&self, game: &mut Game) {
        crate::util::util_server_cards::UtilServerCards::deactivate_while_holding_ball(game);
    }

    /// Java: divingCatch(pCoordinate) — 3-phase ASK_HOME → ASK_AWAY → PROCESS.
    ///
    /// Shows dialogs for each team with DivingCatch-skilled adjacent players.
    /// In PROCESS phase, attempts catch_ball() for each declared catcher.
    /// Returns the updated catch_scatter_throw_in_mode.
    fn diving_catch(&mut self, game: &mut Game, rng: &mut GameRng, coord: FieldCoordinate) -> Option<CatchScatterThrowInMode> {
        if self.phase == DivingCatchPhase::AskHome {
            let home_team = game.team_home.clone();
            let home_catchers = UtilServerCatchScatterThrowIn::find_diving_catchers(game, &home_team, coord);
            if !home_catchers.is_empty() {
                // Java: DialogParameterDivingCatch (declare home-team diving catchers)
                self.pending_prompt = Some(AgentPrompt::SwarmingPlayers {
                    team_id: home_team.id.clone(),
                    eligible_players: home_catchers.iter().map(|p| p.id.clone()).collect(),
                });
                return self.catch_scatter_throw_in_mode;
            } else {
                self.phase = DivingCatchPhase::AskAway;
            }
        }
        if self.phase == DivingCatchPhase::AskAway {
            let away_team = game.team_away.clone();
            let away_catchers = UtilServerCatchScatterThrowIn::find_diving_catchers(game, &away_team, coord);
            if !away_catchers.is_empty() {
                // Java: DialogParameterDivingCatch (declare away-team diving catchers)
                self.pending_prompt = Some(AgentPrompt::SwarmingPlayers {
                    team_id: away_team.id.clone(),
                    eligible_players: away_catchers.iter().map(|p| p.id.clone()).collect(),
                });
                return self.catch_scatter_throw_in_mode;
            } else {
                self.phase = DivingCatchPhase::Process;
            }
        }
        if self.phase == DivingCatchPhase::Process {
            let catcher_id_set = self.catcher_id.is_some();
            let in_divers = self.catcher_id.as_ref()
                .map(|id| self.diving_catchers.contains(id))
                .unwrap_or(false);
            let has_reroll_source = self.re_roll_state.re_roll_source.is_some();

            if (catcher_id_set && (has_reroll_source || in_divers)) || self.evaluate {
                let cid = self.catcher_id.clone().unwrap_or_default();
                self.diving_catchers.retain(|id| id != &cid);
                if self.re_roll_state.re_roll_source.is_none() && !self.evaluate {
                    self.re_roll_state.re_rolled_action = None;
                    self.pending_events.push(GameEvent::SkillUse {
                        player_id: cid.clone(),
                        skill_id: SkillId::DivingCatch as u16,
                        used: true,
                    });
                    game.report_list.add(ReportSkillUse::new(
                        Some(cid.clone()),
                        SkillId::DivingCatch,
                        true,
                        ffb_model::model::skill_use::SkillUse::CATCH_BALL,
                    ));
                }
                let mode = self.catch_ball(game, rng);
                let current_mode = self.catch_scatter_throw_in_mode;
                if mode.is_none() || mode == current_mode {
                    return mode;
                }
                self.re_roll_state.re_rolled_action = None;
                self.re_roll_state.re_roll_source = None;
            }
            self.re_roll_state.re_rolled_action = None;
            if self.diving_catchers.is_empty() {
                return Some(CatchScatterThrowInMode::ScatterBall);
            }
            if self.diving_catchers.len() == 1 {
                self.repeat = true;
                self.catcher_id = self.diving_catchers.first().cloned();
            } else {
                // Multiple diving catchers — ask coach to pick one
                let ctrl_team = self.diving_catch_control_team.clone()
                    .unwrap_or_else(|| game.team_home.id.clone());
                // Java: DialogParameterDivingCatch (coach picks which declared diver attempts)
                self.pending_prompt = Some(AgentPrompt::PlayerChoice {
                    eligible_players: self.diving_catchers.clone(),
                    reason: ctrl_team,
                });
            }
        }
        self.catch_scatter_throw_in_mode
    }

    /// Java: catchBall() — AG roll, modifier lookup, success/fail/reroll paths.
    ///
    /// Returns None on success (ball placed), SCATTER_BALL on preventCatch,
    /// current mode (loop) when re-roll offered, or FAILED_CATCH on final failure.
    fn catch_ball(&mut self, game: &mut Game, rng: &mut GameRng) -> Option<CatchScatterThrowInMode> {
        let mode = self.catch_scatter_throw_in_mode?;
        let cid = match self.catcher_id.clone() {
            Some(id) => id,
            None => return Some(CatchScatterThrowInMode::ScatterBall),
        };
        let prevent_catch = game.player(&cid)
            .map(|p| p.has_skill_property(NamedProperties::PREVENT_CATCH))
            .unwrap_or(false);
        if prevent_catch {
            return Some(CatchScatterThrowInMode::ScatterBall);
        }

        let catcher_coord = game.field_model.player_coordinate(&cid);

        // Java: if (ReRolledActions.CATCH == getReRolledAction()) { try useReRoll or doRoll=false }
        let mut do_roll = true;
        let already_rerolled = self.re_roll_state.re_rolled_action
            .as_ref().map(|a| a.name == "CATCH").unwrap_or(false);
        if already_rerolled {
            let source_opt = self.re_roll_state.re_roll_source.clone();
            let consumed = source_opt.as_ref()
                .map(|s| use_reroll(game, s, &cid))
                .unwrap_or(false);
            if !consumed {
                do_roll = false;
            }
        }

        if do_roll || self.evaluate {
            let factory = CatchModifierFactory::for_rules(game.rules);
            let model_mode = to_model_mode(mode);
            let catcher = game.player(&cid).map(|p| p as *const _);
            let min_roll = if let Some(ptr) = catcher {
                let player = unsafe { &*ptr };
                let ctx = CatchContext::new(game, Some(player), model_mode, None);
                let mods = factory.find_applicable(&ctx);
                let skill_mods = factory.find_skill_modifiers(&ctx);
                let all: Vec<&ffb_mechanics::modifiers::catch_modifier::CatchModifier> = mods.iter().copied().chain(skill_mods.iter()).collect();
                CatchModifierFactory::minimum_roll_catch(player, &all)
            } else {
                2
            };

            if do_roll && self.roll == 0 {
                self.roll = rng.d6();
            }
            let rerolled = already_rerolled && self.re_roll_state.re_roll_source.is_some();
            let successful = DiceInterpreter::is_skill_roll_successful(self.roll, min_roll);

            self.pending_events.push(GameEvent::CatchRoll {
                player_id: cid.clone(),
                target: min_roll,
                roll: self.roll,
                success: successful,
                rerolled: rerolled || self.evaluate,
            });
            game.report_list.add(ReportCatchRoll::new(
                Some(cid.clone()),
                successful,
                self.roll,
                min_roll,
                rerolled || self.evaluate,
                vec![],
                mode.is_bomb(),
            ));
            self.evaluate = false;

            if successful {
                if mode.is_bomb() {
                    game.field_model.bomb_coordinate = catcher_coord;
                    game.field_model.bomb_moving = false;
                } else {
                    game.field_model.ball_coordinate = catcher_coord;
                    game.field_model.ball_moving = false;
                }
                self.re_roll_state.re_rolled_action = None;
                // Java: if hand-off/accurate-pass/punt and wrong team → publish EndTurn(true)
                let wrong_team = match mode {
                    CatchScatterThrowInMode::CatchHandOff
                    | CatchScatterThrowInMode::CatchAccuratePass
                    | CatchScatterThrowInMode::CatchPunt => {
                        game.turn_mode != TurnMode::DumpOff && {
                            let home_has = game.team_home.has_player(&cid);
                            let away_has = game.team_away.has_player(&cid);
                            (game.home_playing && away_has) || (!game.home_playing && home_has)
                        }
                    }
                    _ => false,
                };
                if wrong_team {
                    self.pending_published.push(StepParameter::EndTurn(true));
                }
                return None;
            }

            // Failure path
            if !already_rerolled {
                // Java: boolean stopProcessing = getGameState().executeStepHooks(this, state);
                //       if (state.rerollCatch && (!mode.isBomb() || catchForBombs.isEnabled())) {
                //         ...successfulWithBlastIt branch is client-only, skipped here...
                //         return catchBall();
                //       }
                let mut hook_state = StepCatchHookState::new(cid.clone());
                dispatch::execute_step_hooks(game, rng, StepId::CatchScatterThrowIn, &mut hook_state);
                self.re_roll_state.re_rolled_action = hook_state.re_rolled_action.or(self.re_roll_state.re_rolled_action.clone());
                self.re_roll_state.re_roll_source = hook_state.re_roll_source.or(self.re_roll_state.re_roll_source.clone());

                let catch_works_for_bombs = game.options
                    .get_option_with_default(game_option_id::GameOptionId::CATCH_WORKS_FOR_BOMBS)
                    .get_value_as_string() == "true";
                if hook_state.reroll_catch && (!mode.is_bomb() || catch_works_for_bombs) {
                    // client-only: blast-it dialog (grantsCatchBonusToReceiver + Hail Mary pass) skipped in headless mode
                    return self.catch_ball(game, rng);
                }

                if let Some(prompt) = ask_for_reroll_if_available(game, "CATCH", min_roll, false) {
                    self.re_roll_state.re_rolled_action = Some(ReRolledAction::new("CATCH"));
                    self.re_roll_state.re_roll_source = Some(ffb_model::enums::ReRollSource::new("TRR"));
                    self.roll = 0;
                    self.pending_prompt = Some(prompt);
                    return self.catch_scatter_throw_in_mode;
                }
            }
        }

        // Final failure
        self.re_roll_state.re_rolled_action = None;
        if let Some(coord) = catcher_coord {
            if self.phase != DivingCatchPhase::Process {
                if mode.is_bomb() {
                    game.field_model.bomb_coordinate = Some(coord);
                    game.field_model.bomb_moving = true;
                } else {
                    game.field_model.ball_coordinate = Some(coord);
                    game.field_model.ball_moving = true;
                }
            }
        }
        self.roll = 0;
        Some(CatchScatterThrowInMode::FailedCatch)
    }
}

/// Convert framework's CatchScatterThrowInMode (PascalCase) to model's (SCREAMING_SNAKE_CASE).
fn to_model_mode(m: CatchScatterThrowInMode) -> CatchScatterThrowInMode {
    match m {
        CatchScatterThrowInMode::CatchAccurateBomb => CatchScatterThrowInMode::CatchAccurateBomb,
        CatchScatterThrowInMode::CatchAccurateBombEmptySquare => CatchScatterThrowInMode::CatchAccurateBombEmptySquare,
        CatchScatterThrowInMode::CatchAccuratePass => CatchScatterThrowInMode::CatchAccuratePass,
        CatchScatterThrowInMode::CatchAccuratePassEmptySquare => CatchScatterThrowInMode::CatchAccuratePassEmptySquare,
        CatchScatterThrowInMode::CatchBomb => CatchScatterThrowInMode::CatchBomb,
        CatchScatterThrowInMode::CatchHandOff => CatchScatterThrowInMode::CatchHandOff,
        CatchScatterThrowInMode::CatchKickoff => CatchScatterThrowInMode::CatchKickoff,
        CatchScatterThrowInMode::CatchMissedPass => CatchScatterThrowInMode::CatchMissedPass,
        CatchScatterThrowInMode::CatchPunt => CatchScatterThrowInMode::CatchPunt,
        CatchScatterThrowInMode::CatchScatter => CatchScatterThrowInMode::CatchScatter,
        CatchScatterThrowInMode::CatchThrowIn => CatchScatterThrowInMode::CatchThrowIn,
        CatchScatterThrowInMode::Deflected => CatchScatterThrowInMode::Deflected,
        CatchScatterThrowInMode::DeflectedBomb => CatchScatterThrowInMode::DeflectedBomb,
        CatchScatterThrowInMode::FailedCatch => CatchScatterThrowInMode::FailedCatch,
        CatchScatterThrowInMode::FailedPickUp => CatchScatterThrowInMode::FailedPickUp,
        CatchScatterThrowInMode::FailedDeflectionConversion => CatchScatterThrowInMode::FailedDeflectionConversion,
        CatchScatterThrowInMode::ScatterBall => CatchScatterThrowInMode::ScatterBall,
        CatchScatterThrowInMode::ThreeSquareScatter => CatchScatterThrowInMode::ThreeSquareScatter,
        CatchScatterThrowInMode::ThrowIn => CatchScatterThrowInMode::ThrowIn,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{CatchScatterThrowInMode, StepAction, StepParameter};
    use ffb_model::enums::{Rules, PS_STANDING, PlayerState};
    use ffb_model::model::{Game, Player};
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    fn make_player(id: &str, agility: i32) -> Player {
        Player {
            id: id.into(), name: id.into(), nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        }
    }

    // ── basic parameter/mode tests ────────────────────────────────────────────────

    #[test]
    fn start_without_mode_returns_next() {
        let mut game = make_game();
        let mut step = StepCatchScatterThrowIn::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn mode_parameter_accepted() {
        let mut step = StepCatchScatterThrowIn::default();
        let accepted = step.set_parameter(&StepParameter::CatchScatterThrowInMode(
            CatchScatterThrowInMode::CatchScatter,
        ));
        assert!(accepted);
        assert_eq!(step.catch_scatter_throw_in_mode, Some(CatchScatterThrowInMode::CatchScatter));
    }

    #[test]
    fn throw_in_coordinate_parameter_accepted() {
        let mut step = StepCatchScatterThrowIn::default();
        let coord = FieldCoordinate::new(5, 3);
        let accepted = step.set_parameter(&StepParameter::ThrowInCoordinate(coord));
        assert!(accepted);
        assert_eq!(step.throw_in_coordinate, Some(coord));
    }

    #[test]
    fn diving_catch_phase_progression_via_select_player() {
        let mut game = make_game();
        let mut step = StepCatchScatterThrowIn::new();
        step.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::CatchKickoff);
        assert_eq!(step.phase, DivingCatchPhase::AskHome);

        step.handle_command(&Action::SelectPlayer {player_id: "p1".into() }, &mut game, &mut GameRng::new(0));
        assert_eq!(step.phase, DivingCatchPhase::AskAway);
        assert!(step.diving_catchers.contains(&"p1".to_string()));

        step.handle_command(&Action::SelectPlayer {player_id: "".into() }, &mut game, &mut GameRng::new(0));
        assert_eq!(step.phase, DivingCatchPhase::Process);
    }

    // ── scatter_ball / bounce_ball tests ─────────────────────────────────────────

    #[test]
    fn scatter_ball_no_ball_returns_next() {
        let mut game = make_game();
        let mut step = StepCatchScatterThrowIn::new();
        step.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::ScatterBall);
        game.field_model.ball_in_play = false;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(step.catch_scatter_throw_in_mode.is_none());
    }

    #[test]
    fn scatter_ball_in_play_loops_until_terminal() {
        let mut game = make_game();
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(12, 7));
        game.field_model.ball_in_play = true;
        game.field_model.ball_moving = true;
        let mut step = StepCatchScatterThrowIn::new();
        step.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::ScatterBall);
        // One step: bounce ball → some mode (THROW_IN, FAILED_CATCH, or None)
        let out = step.start(&mut game, &mut GameRng::new(42));
        // Mode should have advanced (no longer ScatterBall — it ran once)
        assert_ne!(step.catch_scatter_throw_in_mode, Some(CatchScatterThrowInMode::ScatterBall),
            "ScatterBall should have been processed (not remain as-is)");
        // Outcome should be Repeat (if mode still set) or NextStep
        assert!(matches!(out.action, StepAction::Repeat | StepAction::NextStep));
    }

    #[test]
    fn scatter_ball_emits_scatter_ball_event() {
        let mut game = make_game();
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(12, 7));
        game.field_model.ball_in_play = true;
        game.field_model.ball_moving = true;
        let mut step = StepCatchScatterThrowIn::new();
        step.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::ScatterBall);
        let out = step.start(&mut game, &mut GameRng::new(1));
        assert!(out.events.iter().any(|e| matches!(e, GameEvent::ScatterBall { .. })));
    }

    // ── throw_in tests ────────────────────────────────────────────────────────────

    #[test]
    fn throw_in_no_coordinate_becomes_scatter_ball() {
        let mut game = make_game();
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(12, 7));
        game.field_model.ball_in_play = true;
        let mut step = StepCatchScatterThrowIn::new();
        step.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::ThrowIn);
        step.throw_in_coordinate = None;
        // Should transition to ScatterBall (then Repeat since still set)
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_ne!(step.catch_scatter_throw_in_mode, Some(CatchScatterThrowInMode::ThrowIn));
        assert!(matches!(out.action, StepAction::Repeat | StepAction::NextStep));
    }

    #[test]
    fn throw_in_with_coordinate_emits_throw_in_event() {
        let mut game = make_game();
        game.field_model.ball_in_play = true;
        game.field_model.ball_moving = true;
        let mut step = StepCatchScatterThrowIn::new();
        step.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::ThrowIn);
        step.throw_in_coordinate = Some(FieldCoordinate::new(0, 7)); // sideline
        let out = step.start(&mut game, &mut GameRng::new(10));
        assert!(out.events.iter().any(|e| matches!(e, GameEvent::ThrowIn { .. })));
    }

    // ── catch_ball tests ─────────────────────────────────────────────────────────

    #[test]
    fn catch_ball_no_catcher_returns_next() {
        let mut game = make_game();
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(12, 7));
        game.field_model.ball_in_play = true;
        game.field_model.ball_moving = true;
        let mut step = StepCatchScatterThrowIn::new();
        step.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::CatchScatter);
        // No catcher_id and no player under ball → SCATTER_BALL → Repeat
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::Repeat | StepAction::NextStep));
    }

    #[test]
    fn catch_ball_with_standing_player_emits_catch_roll() {
        let mut game = make_game();
        let coord = FieldCoordinate::new(12, 7);
        game.field_model.ball_coordinate = Some(coord);
        game.field_model.ball_in_play = true;
        game.field_model.ball_moving = true;
        // Add a player at the ball location
        let player = make_player("h1", 4);
        game.team_home.players.push(player);
        game.field_model.set_player_coordinate("h1", coord);
        game.field_model.set_player_state("h1", PlayerState::new(PS_STANDING));

        let mut step = StepCatchScatterThrowIn::new();
        step.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::CatchScatter);
        step.catcher_id = Some("h1".into());

        let out = step.start(&mut game, &mut GameRng::new(99));
        assert!(out.events.iter().any(|e| matches!(e, GameEvent::CatchRoll { .. })),
            "expected CatchRoll event, got {:?}", out.events);
    }

    // ── failed_catch tests ────────────────────────────────────────────────────────

    #[test]
    fn failed_catch_transitions_to_scatter_ball() {
        let mut game = make_game();
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(12, 7));
        game.field_model.ball_in_play = true;
        let mut step = StepCatchScatterThrowIn::new();
        step.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::FailedCatch);
        let _out = step.start(&mut game, &mut GameRng::new(0));
        // FailedCatch → ScatterBall (then Repeat)
        assert_ne!(step.catch_scatter_throw_in_mode, Some(CatchScatterThrowInMode::FailedCatch));
    }

    #[test]
    fn failed_deflection_conversion_transitions_to_three_square_scatter() {
        let mut game = make_game();
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(12, 7));
        game.field_model.ball_in_play = true;
        let mut step = StepCatchScatterThrowIn::new();
        step.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::FailedDeflectionConversion);
        let _out = step.start(&mut game, &mut GameRng::new(0));
        assert_ne!(step.catch_scatter_throw_in_mode,
            Some(CatchScatterThrowInMode::FailedDeflectionConversion));
    }

    // ── spiked ball tests ─────────────────────────────────────────────────────────

    #[test]
    fn spiked_ball_option_enabled_with_player_pushes_sequences() {
        let mut game = make_game();
        let coord = FieldCoordinate::new(12, 7);
        game.field_model.ball_coordinate = Some(coord);
        game.field_model.ball_in_play = true;
        game.options.set(ffb_model::option::game_option_id::SPIKED_BALL, "true");
        // Add a player at the ball coordinate
        let p = make_player("p1", 3);
        game.team_home.players.push(p);
        game.field_model.set_player_coordinate("p1", coord);
        game.field_model.set_player_state("p1", PlayerState::new(PS_STANDING));

        let mut step = StepCatchScatterThrowIn::new();
        step.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::FailedCatch);
        let out = step.start(&mut game, &mut GameRng::new(0));

        // Should push sequences (SpikedBallApo + CatchScatterThrowIn) and return NextStep
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 2, "expected 2 pushed sequences (scatter + spiked_ball_apo)");
    }

    #[test]
    fn spiked_ball_option_disabled_no_push() {
        let mut game = make_game();
        let coord = FieldCoordinate::new(12, 7);
        game.field_model.ball_coordinate = Some(coord);
        game.field_model.ball_in_play = true;
        // SPIKED_BALL not set
        let p = make_player("p1", 3);
        game.team_home.players.push(p);
        game.field_model.set_player_coordinate("p1", coord);
        game.field_model.set_player_state("p1", PlayerState::new(PS_STANDING));

        let mut step = StepCatchScatterThrowIn::new();
        step.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::FailedCatch);
        let out = step.start(&mut game, &mut GameRng::new(0));

        // No spiked ball → normal Repeat
        assert_eq!(out.pushes.len(), 0);
    }

    #[test]
    fn catch_ball_adds_catch_roll_report() {
        let mut game = make_game();
        let coord = FieldCoordinate::new(12, 7);
        game.field_model.ball_coordinate = Some(coord);
        game.field_model.ball_in_play = true;
        game.field_model.ball_moving = true;
        let player = make_player("h1", 4);
        game.team_home.players.push(player);
        game.field_model.set_player_coordinate("h1", coord);
        game.field_model.set_player_state("h1", PlayerState::new(PS_STANDING));
        let mut step = StepCatchScatterThrowIn::new();
        step.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::CatchScatter);
        step.catcher_id = Some("h1".into());
        step.start(&mut game, &mut GameRng::new(99));
        assert!(game.report_list.has_report(ffb_model::report::report_id::ReportId::CATCH_ROLL));
    }

    #[test]
    fn bounce_ball_adds_scatter_ball_report() {
        let mut game = make_game();
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(12, 7));
        game.field_model.ball_in_play = true;
        game.field_model.ball_moving = true;
        let mut step = StepCatchScatterThrowIn::new();
        step.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::ScatterBall);
        step.start(&mut game, &mut GameRng::new(1));
        assert!(game.report_list.has_report(ffb_model::report::report_id::ReportId::SCATTER_BALL));
    }

    // ── terminal path ────────────────────────────────────────────────────────────

    #[test]
    fn terminal_publishes_catcher_id() {
        let mut game = make_game();
        // Ball resting (ball_in_play = true, ball_moving = false, no player)
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(12, 7));
        game.field_model.ball_in_play = true;
        game.field_model.ball_moving = false;
        let mut step = StepCatchScatterThrowIn::new();
        step.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::ScatterBall);
        // Force scatter to bounce OOB via seeded RNG that goes north off the top boundary
        // But for this test, set ball_in_play = false so ScatterBall clears mode immediately
        game.field_model.ball_in_play = false;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        let has_catcher_id = out.published.iter().any(|p| matches!(p, StepParameter::CatcherId(_)));
        assert!(has_catcher_id, "expected CatcherId to be published in terminal path");
    }
}
