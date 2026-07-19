use ffb_model::enums::{Direction, Rules, TurnMode};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::re_rolled_action::ReRolledAction;
use ffb_model::prompts::AgentPrompt;
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds};
use ffb_model::util::rng::GameRng;
use ffb_mechanics::bb2016::throw_in_mechanic::ThrowInMechanic;
use ffb_mechanics::throw_in_mechanic::ThrowInMechanic as ThrowInMechanicTrait;
use ffb_mechanics::modifiers::catch_context::CatchContext;
use ffb_mechanics::modifiers::catch_modifier_factory::CatchModifierFactory;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::injury::injuryType::injury_type_stab::InjuryTypeStab;
use crate::step::abstract_step_with_re_roll::ReRollState;
use crate::step::framework::{Step, StepAction, StepOutcome};
use crate::step::framework::{CatchScatterThrowInMode, StepId, StepParameter, SequenceStep};
use crate::step::generator::common::SpikedBallApo;
use ffb_model::option::game_option_id;
use crate::step::util_server_catch_scatter_throw_in::UtilServerCatchScatterThrowIn;
use crate::step::util_server_injury::{drop_player, handle_injury};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use ffb_model::report::report_catch_roll::ReportCatchRoll;
use ffb_model::report::report_scatter_ball::ReportScatterBall;
use ffb_model::report::report_throw_in::ReportThrowIn;
use ffb_model::report::report_skill_use::ReportSkillUse;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::enums::{ApothecaryMode, ReRollSource, SkillId};
use crate::skill_behaviour::dispatch;

/// Java: StepCatchScatterThrowIn.StepState (the subset needed by the Catch/MonstrousMouth
/// skill-modifier hooks) — mutable state passed through executeStepHooks.
#[derive(Debug)]
pub struct StepCatchHookState {
    pub catcher_id: String,
    pub reroll_catch: bool,
    pub re_rolled_action: Option<ReRolledAction>,
    pub re_roll_source: Option<ReRollSource>,
}

impl StepCatchHookState {
    pub fn new(catcher_id: String) -> Self {
        Self { catcher_id, reroll_catch: false, re_rolled_action: None, re_roll_source: None }
    }
}

/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.StepCatchScatterThrowIn`.
///
/// NOTE: this used to `pub use` the BB2025-shared implementation, on the (incorrect) theory
/// that the BB2016 Java class was structurally identical to the BB2025 one. Diffing the two
/// Java sources shows they are NOT identical — BB2025 added diving-catch multi-select
/// (3-phase ASK_HOME/ASK_AWAY/PROCESS vs. BB2016's single boolean choice), CATCH_PUNT,
/// DEFLECTED/THREE_SQUARE_SCATTER, "blast it"/evaluate catch-bonus handling, corner throw-ins,
/// and (since BB2020) changed a bounced ball landing on a no-tacklezone player from
/// `SCATTER_BALL` (BB2016) to `FAILED_CATCH` (BB2020+). Reusing the BB2025 shared step for
/// BB2016 games silently applied all of these edition-specific rule changes to BB2016 games.
/// This file is now a faithful, standalone translation of the BB2016 Java source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DivingCatchChoice {
    Cancelled,
    Declined,
    Accepted,
}

pub struct StepCatchScatterThrowIn {
    /// Java: fCatcherId
    pub catcher_id: Option<String>,
    /// Java: fScatterBounds
    pub scatter_bounds: FieldCoordinateBounds,
    /// Java: fCatchScatterThrowInMode
    pub catch_scatter_throw_in_mode: Option<CatchScatterThrowInMode>,
    /// Java: fThrowInCoordinate
    pub throw_in_coordinate: Option<FieldCoordinate>,
    /// Java: fBombMode
    pub bomb_mode: bool,
    /// Java: fDivingCatchChoice (Boolean, tri-state via Option<bool>)
    pub diving_catch_choice: Option<bool>,
    /// Java: state.rerollCatch
    pub reroll_catch: bool,
    /// Java: AbstractStepWithReRoll fields
    pub re_roll_state: ReRollState,
    pending_prompt: Option<AgentPrompt>,
    pending_events: Vec<GameEvent>,
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
            diving_catch_choice: None,
            reroll_catch: false,
            re_roll_state: ReRollState::new(),
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
        // Java: handleCommand — only CLIENT_PLAYER_CHOICE(DIVING_CATCH) is consumed here.
        if let Action::SelectPlayer { player_id } = action {
            self.diving_catch_choice = Some(!player_id.is_empty());
            self.catcher_id = if player_id.is_empty() { None } else { Some(player_id.clone()) };
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::CatchScatterThrowInMode(v) => {
                self.catch_scatter_throw_in_mode = Some(*v);
                true
            }
            StepParameter::ThrowInCoordinate(v) => { self.throw_in_coordinate = Some(*v); true }
            _ => false,
        }
    }
}

impl StepCatchScatterThrowIn {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
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

        let player_under_ball = game.field_model.ball_coordinate
            .and_then(|c| game.field_model.player_at(c))
            .map(|id| id.clone());

        match self.catch_scatter_throw_in_mode {
            Some(CatchScatterThrowInMode::CatchBomb)
            | Some(CatchScatterThrowInMode::CatchAccurateBombEmptySquare)
            | Some(CatchScatterThrowInMode::CatchAccurateBomb)
            | Some(CatchScatterThrowInMode::DeflectedBomb) => {
                self.handle_bomb(game, rng);
            }
            Some(CatchScatterThrowInMode::CatchAccuratePass)
            | Some(CatchScatterThrowInMode::CatchHandOff)
            | Some(CatchScatterThrowInMode::CatchScatter)
            | Some(CatchScatterThrowInMode::Deflected) => {
                self.handle_regular_catch(game, rng, &player_under_ball);
            }
            Some(CatchScatterThrowInMode::CatchKickoff)
            | Some(CatchScatterThrowInMode::CatchThrowIn)
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
            Some(CatchScatterThrowInMode::FailedCatch)
            | Some(CatchScatterThrowInMode::FailedPickUp) => {
                self.handle_failed_catch(game, rng, &player_under_ball);
            }
            Some(CatchScatterThrowInMode::ScatterBall) => {
                self.bomb_mode = false;
                if game.field_model.ball_in_play {
                    let new_mode = self.scatter_ball(game, rng);
                    self.catch_scatter_throw_in_mode = new_mode;
                } else {
                    self.catch_scatter_throw_in_mode = None;
                }
            }
            // BB2016 has no CATCH_PUNT / THREE_SQUARE_SCATTER / FAILED_DEFLECTION_CONVERSION.
            _ => {}
        }

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
            self.diving_catch_choice = None;
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
        let catcher_id: Option<String> = if self.bomb_mode {
            if !game.field_model.bomb_moving {
                game.field_model.bomb_coordinate
                    .and_then(|c| game.field_model.player_at(c))
                    .map(|id| id.clone())
            } else {
                None
            }
        } else {
            game.field_model.ball_coordinate
                .and_then(|c| game.field_model.player_at(c))
                .map(|id| id.clone())
        };

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
    fn handle_bomb(&mut self, game: &mut Game, rng: &mut GameRng) {
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
            game.field_model.bomb_moving = true;
            self.catch_scatter_throw_in_mode = None;
        }
    }

    /// Java: handle CATCH_ACCURATE_PASS / CATCH_HAND_OFF / CATCH_SCATTER / DEFLECTED
    fn handle_regular_catch(&mut self, game: &mut Game, rng: &mut GameRng, player_under_ball: &Option<String>) {
        self.bomb_mode = false;
        if self.catcher_id.is_none() {
            self.catcher_id = player_under_ball.clone();
        }
        if let Some(ref cid) = self.catcher_id.clone() {
            let catcher_state = game.field_model.player_state(cid);
            let has_tz = catcher_state.map(|s| s.has_tacklezones()).unwrap_or(false);
            if has_tz && game.field_model.ball_in_play && game.field_model.ball_moving {
                let new_mode = self.catch_ball(game, rng);
                self.catch_scatter_throw_in_mode = new_mode;
            } else {
                self.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::ScatterBall);
            }
        } else {
            self.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::ScatterBall);
        }
    }

    /// Java: handle FAILED_CATCH / FAILED_PICK_UP — spiked-ball injury roll on the player
    /// standing under the ball, then fall through to a regular scatter.
    fn handle_failed_catch(&mut self, game: &mut Game, rng: &mut GameRng, player_under_ball: &Option<String>) {
        self.bomb_mode = false;
        let ball_in_play = game.field_model.ball_in_play;

        if let Some(cid) = player_under_ball.clone() {
            if ball_in_play
                && (game.options.is_enabled(game_option_id::SPIKED_BALL)
                    || game.is_active(NamedProperties::DROPPED_BALL_CAUSES_ARMOUR_ROLL))
            {
                let ball_coord = game.field_model.ball_coordinate.unwrap_or(FieldCoordinate::new(0, 0));
                let mut injury_type = InjuryTypeStab::new(false);
                let injury_result = handle_injury(
                    game, rng, &mut injury_type, None, &cid,
                    ball_coord, None, None, ApothecaryMode::Catcher,
                );
                let armor_broken = injury_result.injury_context().is_armor_broken();
                self.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::ScatterBall);
                self.pending_spiked_ball = true;
                if armor_broken {
                    for param in drop_player(game, &cid, false) {
                        self.pending_published.push(param);
                    }
                }
                self.pending_published.push(StepParameter::InjuryResult(Box::new(injury_result)));
                return;
            }
        }
        self.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::ScatterBall);
    }

    /// Java: bounceBall()/scatterBall() (BB2016 has only the single-bounce version — no
    /// 3-square scatter variant exists in this edition). Landing on a player without
    /// tacklezones returns SCATTER_BALL (continue bouncing) — NOT FAILED_CATCH; the
    /// FAILED_CATCH-on-no-tacklezones rule change was introduced in BB2020.
    fn scatter_ball(&mut self, game: &mut Game, rng: &mut GameRng) -> Option<CatchScatterThrowInMode> {
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
                    // Java bb2016: `return CatchScatterThrowInMode.SCATTER_BALL;`
                    return Some(CatchScatterThrowInMode::ScatterBall);
                }
            }
            None
        } else {
            game.field_model.out_of_bounds = true;
            if self.scatter_bounds == FieldCoordinateBounds::FIELD {
                self.throw_in_coordinate = Some(last_valid);
                Some(CatchScatterThrowInMode::ThrowIn)
            } else {
                self.pending_published.push(StepParameter::Touchback(true));
                None
            }
        }
    }

    /// Java: throwInBall() — direction roll (always non-corner in BB2016), 2d6 distance,
    /// advance step-by-step.
    fn throw_in_ball(&mut self, game: &mut Game, rng: &mut GameRng) -> Option<CatchScatterThrowInMode> {
        let mechanic = ThrowInMechanic::new();
        let start = self.throw_in_coordinate.unwrap_or(FieldCoordinate::new(0, 0));
        self.catcher_id = None;

        let direction_roll = rng.d6();
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

    /// Java: divingCatch(pCoordinate) — single boolean choice: try home team's diving
    /// catchers first, then away's; if both sides have eligible catchers, the attempt is
    /// automatically cancelled (Java: `SkillUse.CANCEL_DIVING_CATCH`). Unlike BB2025's
    /// 3-phase ASK_HOME/ASK_AWAY/PROCESS + multi-select, BB2016 only ever asks one team,
    /// and only ever one catcher.
    fn diving_catch(&mut self, game: &mut Game, rng: &mut GameRng, coord: FieldCoordinate) -> Option<CatchScatterThrowInMode> {
        if self.diving_catch_choice.is_none() {
            self.catcher_id = None;
            let home_team = game.team_home.clone();
            let away_team = game.team_away.clone();
            let home_catchers = UtilServerCatchScatterThrowIn::find_diving_catchers(game, &home_team, coord);
            let away_catchers = UtilServerCatchScatterThrowIn::find_diving_catchers(game, &away_team, coord);
            if !home_catchers.is_empty() && !away_catchers.is_empty() {
                self.diving_catch_choice = Some(false);
                game.report_list.add(ReportSkillUse::new(
                    None,
                    SkillId::DivingCatch,
                    false,
                    SkillUse::CANCEL_DIVING_CATCH,
                ));
            } else if !home_catchers.is_empty() {
                self.pending_prompt = Some(AgentPrompt::SwarmingPlayers {
                    team_id: home_team.id.clone(),
                    eligible_players: home_catchers.iter().map(|p| p.id.clone()).collect(),
                });
                return self.catch_scatter_throw_in_mode;
            } else if !away_catchers.is_empty() {
                self.pending_prompt = Some(AgentPrompt::SwarmingPlayers {
                    team_id: away_team.id.clone(),
                    eligible_players: away_catchers.iter().map(|p| p.id.clone()).collect(),
                });
                return self.catch_scatter_throw_in_mode;
            } else {
                self.diving_catch_choice = Some(false);
            }
        }
        if let Some(choice) = self.diving_catch_choice {
            if choice {
                if let Some(cid) = self.catcher_id.clone() {
                    if self.re_roll_state.re_roll_source.is_none() {
                        self.pending_events.push(GameEvent::SkillUse {
                            player_id: cid.clone(),
                            skill_id: SkillId::DivingCatch as u16,
                            used: true,
                        });
                        game.report_list.add(ReportSkillUse::new(
                            Some(cid),
                            SkillId::DivingCatch,
                            true,
                            SkillUse::CATCH_BALL,
                        ));
                    }
                }
                return self.catch_ball(game, rng);
            } else {
                return Some(CatchScatterThrowInMode::ScatterBall);
            }
        }
        self.catch_scatter_throw_in_mode
    }

    /// Java: catchBall() — AG roll, modifier lookup, success/fail/reroll paths.
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

        if do_roll {
            let factory = CatchModifierFactory::for_rules(Rules::Bb2016);
            let catcher = game.player(&cid).map(|p| p as *const _);
            let min_roll = if let Some(ptr) = catcher {
                let player = unsafe { &*ptr };
                let ctx = CatchContext::new(game, Some(player), mode, None);
                let mods = factory.find_applicable(&ctx);
                let skill_mods = factory.find_skill_modifiers(&ctx);
                let all: Vec<&ffb_mechanics::modifiers::catch_modifier::CatchModifier> = mods.iter().copied().chain(skill_mods.iter()).collect();
                CatchModifierFactory::minimum_roll_catch(player, &all)
            } else {
                2
            };

            let rerolled = already_rerolled && self.re_roll_state.re_roll_source.is_some();
            let roll = rng.d6();
            let successful = DiceInterpreter::is_skill_roll_successful(roll, min_roll);

            self.pending_events.push(GameEvent::CatchRoll {
                player_id: cid.clone(),
                target: min_roll,
                roll,
                success: successful,
                rerolled,
            });
            game.report_list.add(ReportCatchRoll::new(
                Some(cid.clone()),
                successful,
                roll,
                min_roll,
                rerolled,
                vec![],
                mode.is_bomb(),
            ));

            if successful {
                if mode.is_bomb() {
                    game.field_model.bomb_coordinate = catcher_coord;
                    game.field_model.bomb_moving = false;
                } else {
                    game.field_model.ball_coordinate = catcher_coord;
                    game.field_model.ball_moving = false;
                }
                self.re_roll_state.re_rolled_action = None;
                // Java: if hand-off/accurate-pass and wrong team → publish EndTurn(true)
                let wrong_team = match mode {
                    CatchScatterThrowInMode::CatchHandOff
                    | CatchScatterThrowInMode::CatchAccuratePass => {
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
                let mut hook_state = StepCatchHookState::new(cid.clone());
                dispatch::execute_step_hooks(game, rng, StepId::CatchScatterThrowIn, &mut hook_state);
                self.re_roll_state.re_rolled_action = hook_state.re_rolled_action.or(self.re_roll_state.re_rolled_action.clone());
                self.re_roll_state.re_roll_source = hook_state.re_roll_source.or(self.re_roll_state.re_roll_source.clone());

                if hook_state.reroll_catch {
                    return self.catch_ball(game, rng);
                }

                if let Some(prompt) = ask_for_reroll_if_available(game, "CATCH", min_roll, false) {
                    self.re_roll_state.re_rolled_action = Some(ReRolledAction::new("CATCH"));
                    self.re_roll_state.re_roll_source = Some(ffb_model::enums::ReRollSource::new("TRR"));
                    self.pending_prompt = Some(prompt);
                    return self.catch_scatter_throw_in_mode;
                }
            }
        }

        // Final failure
        self.re_roll_state.re_rolled_action = None;
        if let Some(coord) = catcher_coord {
            if mode.is_bomb() {
                game.field_model.bomb_coordinate = Some(coord);
                game.field_model.bomb_moving = true;
            } else {
                game.field_model.ball_coordinate = Some(coord);
                game.field_model.ball_moving = true;
            }
        }
        Some(CatchScatterThrowInMode::FailedCatch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{PlayerGender, PlayerType, PS_PRONE, PS_STANDING, PlayerState, Rules};
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::model::Player;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2016)
    }

    fn make_player(id: &str) -> Player {
        Player {
            id: id.into(), name: id.into(), nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: Vec::<SkillWithValue>::new(),
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        }
    }

    #[test]
    fn no_mode_returns_next() {
        let mut step = StepCatchScatterThrowIn::new();
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn mode_parameter_accepted() {
        let mut step = StepCatchScatterThrowIn::new();
        let accepted = step.set_parameter(&StepParameter::CatchScatterThrowInMode(
            CatchScatterThrowInMode::CatchScatter,
        ));
        assert!(accepted);
        assert_eq!(step.catch_scatter_throw_in_mode, Some(CatchScatterThrowInMode::CatchScatter));
    }

    #[test]
    fn throw_in_coordinate_parameter_accepted() {
        let mut step = StepCatchScatterThrowIn::new();
        let coord = FieldCoordinate::new(3, 3);
        let accepted = step.set_parameter(&StepParameter::ThrowInCoordinate(coord));
        assert!(accepted);
        assert_eq!(step.throw_in_coordinate, Some(coord));
    }

    #[test]
    fn scatter_ball_no_ball_in_play_returns_next() {
        let mut step = StepCatchScatterThrowIn::new();
        let mut game = make_game();
        game.field_model.ball_in_play = false;
        step.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::ScatterBall);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(step.catch_scatter_throw_in_mode.is_none());
    }

    /// Regression test for the core bug found in this file: it used to `pub use` the
    /// BB2025-shared implementation, which (since BB2020) returns `FAILED_CATCH` when a
    /// bounced ball lands on a player without tacklezones (e.g. prone). BB2016 Java's
    /// `scatterBall()` returns `SCATTER_BALL` in that case (the ball keeps bouncing) — there
    /// is no "must attempt a catch while prone" rule in BB2016.
    #[test]
    fn scatter_ball_onto_prone_player_keeps_bouncing_not_failed_catch() {
        let mut game = make_game();
        let start = FieldCoordinate::new(12, 7);
        game.field_model.ball_coordinate = Some(start);
        game.field_model.ball_in_play = true;
        game.field_model.ball_moving = true;

        // Place a prone (no-tacklezone) player at every square adjacent to `start` so that
        // regardless of the scatter direction rolled, the ball lands on a prone player.
        let dirs = [(-1,-1),(0,-1),(1,-1),(-1,0),(1,0),(-1,1),(0,1),(1,1)];
        for (i, (dx, dy)) in dirs.iter().enumerate() {
            let id = format!("p{i}");
            let mut p = make_player(&id);
            p.id = id.clone();
            game.team_home.players.push(p);
            let coord = FieldCoordinate::new(start.x + dx, start.y + dy);
            game.field_model.set_player_coordinate(&id, coord);
            game.field_model.set_player_state(&id, PlayerState::new(PS_PRONE));
        }

        let mut step = StepCatchScatterThrowIn::new();
        step.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::ScatterBall);
        let mode = step.scatter_ball(&mut game, &mut GameRng::new(1));

        assert_eq!(mode, Some(CatchScatterThrowInMode::ScatterBall),
            "BB2016: bouncing onto a no-tacklezone player must return SCATTER_BALL, not FAILED_CATCH");
    }

    #[test]
    fn scatter_ball_onto_standing_player_returns_catch_scatter() {
        let mut game = make_game();
        let start = FieldCoordinate::new(12, 7);
        game.field_model.ball_coordinate = Some(start);
        game.field_model.ball_in_play = true;
        game.field_model.ball_moving = true;

        let dirs = [(-1,-1),(0,-1),(1,-1),(-1,0),(1,0),(-1,1),(0,1),(1,1)];
        for (i, (dx, dy)) in dirs.iter().enumerate() {
            let id = format!("p{i}");
            let mut p = make_player(&id);
            p.id = id.clone();
            game.team_home.players.push(p);
            let coord = FieldCoordinate::new(start.x + dx, start.y + dy);
            game.field_model.set_player_coordinate(&id, coord);
            game.field_model.set_player_state(&id, PlayerState::new(PS_STANDING));
        }

        let mut step = StepCatchScatterThrowIn::new();
        step.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::ScatterBall);
        let mode = step.scatter_ball(&mut game, &mut GameRng::new(1));

        assert_eq!(mode, Some(CatchScatterThrowInMode::CatchScatter));
        assert!(step.catcher_id.is_some());
    }

    #[test]
    fn diving_catch_single_choice_not_multi_phase() {
        // BB2016 has no `phase`/`DivingCatchPhase` field at all (unlike BB2025's 3-phase
        // ASK_HOME/ASK_AWAY/PROCESS state machine) — a single decline resolves the whole
        // diving-catch attempt directly to a plain ball scatter.
        let mut game = make_game();
        // No players anywhere near the ball, so `divingCatch` finds no eligible catchers
        // on either side and immediately resolves `fDivingCatchChoice = false` in one call.
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(12, 7));
        let mut step = StepCatchScatterThrowIn::new();
        step.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::CatchKickoff);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(step.catch_scatter_throw_in_mode, Some(CatchScatterThrowInMode::ScatterBall));
        assert!(matches!(out.action, StepAction::Repeat | StepAction::NextStep));
    }

    #[test]
    fn no_catch_punt_or_three_square_scatter_variants_used() {
        // BB2016 never sets CATCH_PUNT / THREE_SQUARE_SCATTER / FAILED_DEFLECTION_CONVERSION;
        // feeding those modes in should hit the catch-all no-op arm (mode left unchanged).
        let mut game = make_game();
        let mut step = StepCatchScatterThrowIn::new();
        step.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::CatchPunt);
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Falls through the catch-all `_ => {}` arm, mode remains CatchPunt, so Repeat.
        assert_eq!(out.action, StepAction::Repeat);
        assert_eq!(step.catch_scatter_throw_in_mode, Some(CatchScatterThrowInMode::CatchPunt));
    }

    #[test]
    fn failed_catch_transitions_to_scatter_ball() {
        let mut game = make_game();
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(12, 7));
        game.field_model.ball_in_play = true;
        let mut step = StepCatchScatterThrowIn::new();
        step.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::FailedCatch);
        let _out = step.start(&mut game, &mut GameRng::new(0));
        assert_ne!(step.catch_scatter_throw_in_mode, Some(CatchScatterThrowInMode::FailedCatch));
    }

    #[test]
    fn throw_in_no_coordinate_becomes_scatter_ball() {
        let mut game = make_game();
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(12, 7));
        game.field_model.ball_in_play = true;
        let mut step = StepCatchScatterThrowIn::new();
        step.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::ThrowIn);
        step.throw_in_coordinate = None;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_ne!(step.catch_scatter_throw_in_mode, Some(CatchScatterThrowInMode::ThrowIn));
        assert!(matches!(out.action, StepAction::Repeat | StepAction::NextStep));
    }

    #[test]
    fn catch_ball_with_standing_player_emits_catch_roll() {
        let mut game = make_game();
        let coord = FieldCoordinate::new(12, 7);
        game.field_model.ball_coordinate = Some(coord);
        game.field_model.ball_in_play = true;
        game.field_model.ball_moving = true;
        let mut player = make_player("h1");
        player.agility = 4;
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

    #[test]
    fn catcher_id_parameter_still_handled_via_set_parameter_rejected() {
        // BB2016's Step doesn't accept a raw CatcherId step-parameter (unlike BB2025's shared
        // step, which does) — Java's `setParameter` only recognizes CATCH_SCATTER_THROW_IN_MODE
        // and THROW_IN_COORDINATE.
        let mut step = StepCatchScatterThrowIn::new();
        let accepted = step.set_parameter(&StepParameter::CatcherId(Some("p1".to_string())));
        assert!(!accepted);
    }
}
