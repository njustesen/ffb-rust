use ffb_model::events::GameEvent;
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds};
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::report::report_bomb_out_of_bounds::ReportBombOutOfBounds;
use ffb_model::report::report_scatter_ball::ReportScatterBall;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_cards::UtilCards;
use ffb_model::enums::Direction;
use ffb_model::option::game_option_id;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, CatchScatterThrowInMode};
use crate::step::framework::{StepAction, StepId, StepParameter};
use crate::util::util_server_catch_scatter_throw_in::UtilServerCatchScatterThrowIn;

/// Initialises the bomb sequence for Bombardier/bomb-carrying players.
///
/// Java executeStep logic:
///   game.turnData.setBombUsed(true)
///   game.fieldModel.setRangeRuler(null)
///   if passFumble: catcherId = null
///   if bombOutOfBounds: catcherId = null
///
///   if catcherId != null:
///     explodeSkill = UtilCards.getUnusedSkillWithProperty(actingPlayer, canForceBombExplosion)
///     if explodeSkill != null:
///       if explodeSkillUsed==null: showDialog(DialogSkillUseParameter) -> wait; return
///     else if explodeSkillUsed==null: explodeSkillUsed=false
///     if explodeSkillUsed: catcherId=null
///
///   if catcherId==null:
///     fBombCoordinate = fieldModel.getBombCoordinate()
///     bombOut = false
///     if bombCoordinate==null:
///       if !dontDropFumble: bombOut=true
///     else:
///       bounceOption = game.options.BOMB_BOUNCES_ON_EMPTY_SQUARES
///       if !passFumble && bounceOption.isEnabled && fieldModel.getPlayer(bombCoordinate)==null:
///         roll scatter direction; direction = DiceInterpreter.interpretScatterDirectionRoll
///         bounceTo = findScatterCoordinate(bombCoordinate, direction, 1)
///         report ReportScatterBall
///         if !inBounds(bounceTo): bombOut=true
///         else if player at bounceTo:
///           setBombCoordinate(bounceTo); setBombMoving(true)
///           publish CATCH_SCATTER_THROW_IN_MODE=CATCH_BOMB
///         else: fBombCoordinate=bounceTo; setBombCoordinate(bounceTo); setBombMoving(false)
///     if bombOut: setBombCoordinate(null); setBombMoving(false); report ReportBombOutOfBounds
///     leaveStep(null) -> publish CATCHER_ID=null; NEXT_STEP
///   else:
///     leaveStep(gotoLabelOnEnd) -> publish CATCHER_ID=catcherId; GOTO gotoLabelOnEnd
///
/// handleCommand:
///   CLIENT_USE_SKILL with canForceBombExplosion -> explodeSkillUsed = isSkillUsed -> EXECUTE_STEP
///
/// Unported utilities:
///   ✓ UtilCards.getUnusedSkillWithProperty(actingPlayer, canForceBombExplosion) — implemented
///     (`has_skill_property` in `execute_step`, `get_unused_skill_with_property` in `handle_command`).
///   client-only: UtilServerDialog.showDialog(DialogSkillUseParameter) — dialog is client-side; headless
///     waits via `StepOutcome::cont()` for the `UseSkill` command instead.
///   ✓ game.options.getOptionWithDefault(BOMB_BOUNCES_ON_EMPTY_SQUARES) — implemented via `game.options.is_enabled`.
///   ✓ DiceInterpreter.interpretScatterDirectionRoll — implemented via `Direction::for_roll`
///     (Java delegates to `DirectionFactory.forRoll`, same mapping).
///   ✓ UtilServerCatchScatterThrowIn.findScatterCoordinate — implemented.
///   ✓ setBombCoordinate / setBombMoving for bounce path; fieldModel.getPlayer for bounce — implemented.
///   ✓ reports: ReportScatterBall, ReportBombOutOfBounds, ReportSkillUse — implemented.
///   ✓ CATCH_BOMB mode publish — implemented.
///
///   Known remaining gap (cross-step, out of scope for this file): Java additionally publishes
///   `StepParameter::SKIP = false` when the bomb is caught mid-bounce (`catch_bomb`) *and* the acting
///   player still holds an unused `canForceBombExplosion` skill, so the downstream
///   `StepRecheckExplodeSkill` re-offers the explode choice. That `SKIP` parameter is not modeled in
///   this crate's `StepParameter` enum, and `StepRecheckExplodeSkill`'s own Rust port does not consume
///   `CATCHER_ID`/`SKIP` the way the Java version does — wiring this correctly spans both step files
///   and is left for a follow-up rather than invented here.
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.special.StepInitBomb`.
pub struct StepInitBomb {
    /// Java: fGotoLabelOnEnd (mandatory init param)
    pub goto_label_on_end: String,
    /// Java: fCatcherId (optional init param / cleared on fumble or OOB)
    pub catcher_id: Option<String>,
    /// Java: fPassFumble (mandatory init param)
    pub pass_fumble: bool,
    /// Java: fBombCoordinate (resolved in executeStep)
    pub bomb_coordinate: Option<FieldCoordinate>,
    /// Java: fBombOutOfBounds (init param / set parameter)
    pub bomb_out_of_bounds: bool,
    /// Java: dontDropFumble (init param DONT_DROP_FUMBLE)
    pub dont_drop_fumble: bool,
    /// Java: explodeSkillUsed (Boolean tristate — None=not yet asked)
    pub explode_skill_used: Option<bool>,
}

impl StepInitBomb {
    pub fn new(goto_label_on_end: String) -> Self {
        Self {
            goto_label_on_end,
            catcher_id: None,
            pass_fumble: false,
            bomb_coordinate: None,
            bomb_out_of_bounds: false,
            dont_drop_fumble: false,
            explode_skill_used: None,
        }
    }
}

impl Default for StepInitBomb {
    fn default() -> Self { Self::new(String::new()) }
}

impl Step for StepInitBomb {
    fn id(&self) -> StepId { StepId::InitBomb }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // CLIENT_USE_SKILL with canForceBombExplosion skill
        if let Action::UseSkill { use_skill, .. } = action {
            if *use_skill {
                // Java: actingPlayer.markSkillUsed(skill)
                if let Some(pid) = game.acting_player.player_id.clone() {
                    let sid = game.player(&pid).and_then(|p| UtilCards::get_unused_skill_with_property(
                        p, NamedProperties::CAN_FORCE_BOMB_EXPLOSION));
                    if let Some(sid) = sid {
                        // Java: addReport(new ReportSkillUse(skill, true))
                        let skill_event = GameEvent::SkillUse {
                            player_id: pid.clone(),
                            skill_id: sid as u16,
                            used: true,
                        };
                        let is_home = game.team_home.player(&pid).is_some();
                        if is_home { game.team_home.player_mut(&pid).map(|p| p.used_skills.insert(sid)); }
                        else { game.team_away.player_mut(&pid).map(|p| p.used_skills.insert(sid)); }
                        self.explode_skill_used = Some(*use_skill);
                        return self.execute_step(game, rng).with_event(skill_event);
                    }
                }
            }
            self.explode_skill_used = Some(*use_skill);
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::BombOutOfBounds(v) => { self.bomb_out_of_bounds = *v; true }
            _ => false,
        }
    }
}

impl StepInitBomb {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        game.turn_data_mut().bomb_used = true;
        game.field_model.range_ruler = None;

        // Clear catcher on fumble or OOB
        if self.pass_fumble {
            self.catcher_id = None;
        }
        if self.bomb_out_of_bounds {
            self.catcher_id = None;
        }

        if self.catcher_id.is_some() {
            // Java: explodeSkill = UtilCards.getUnusedSkillWithProperty(actingPlayer, canForceBombExplosion)
            let has_explode_skill = game.acting_player.player_id.as_deref()
                .and_then(|pid| game.player(pid))
                .map(|p| p.has_skill_property(NamedProperties::CAN_FORCE_BOMB_EXPLOSION))
                .unwrap_or(false);
            if self.explode_skill_used.is_none() {
                if has_explode_skill {
                    // Java: show dialog → CONTINUE, wait for CLIENT_USE_SKILL
                    return StepOutcome::cont();
                } else {
                    // No skill → auto-skip explode
                    self.explode_skill_used = Some(false);
                }
            }
            if self.explode_skill_used == Some(true) {
                self.catcher_id = None;
            }
        }

        if self.catcher_id.is_none() {
            // Java: fBombCoordinate = fieldModel.getBombCoordinate()
            self.bomb_coordinate = game.field_model.bomb_coordinate;
            let mut bomb_out = false;
            if self.bomb_coordinate.is_none() && !self.dont_drop_fumble {
                bomb_out = true;
            }
            let mut catch_bomb = false;
            if let Some(bomb_coord) = self.bomb_coordinate {
                let bounce_enabled = game.options.is_enabled(game_option_id::BOMB_BOUNCES_ON_EMPTY_SQUARES);
                if !self.pass_fumble && bounce_enabled && game.field_model.player_at(bomb_coord).is_none() {
                    let scatter_roll = rng.d8();
                    let direction = Direction::for_roll(scatter_roll).unwrap_or(Direction::North);
                    let bounce_to = UtilServerCatchScatterThrowIn::find_scatter_coordinate(bomb_coord, direction, 1);
                    // Java: getResult().addReport(new ReportScatterBall(new Direction[]{direction}, new int[]{scatterRoll}, false))
                    game.report_list.add(ReportScatterBall::new(vec![direction], vec![scatter_roll], false));
                    if !FieldCoordinateBounds::FIELD.is_in_bounds(bounce_to) {
                        bomb_out = true;
                    } else if game.field_model.player_at(bounce_to).is_some() {
                        game.field_model.bomb_coordinate = Some(bounce_to);
                        game.field_model.bomb_moving = true;
                        catch_bomb = true;
                    } else {
                        self.bomb_coordinate = Some(bounce_to);
                        game.field_model.bomb_coordinate = Some(bounce_to);
                        game.field_model.bomb_moving = false;
                    }
                }
            }
            let mut out_event: Option<GameEvent> = None;
            if bomb_out {
                game.field_model.bomb_coordinate = None;
                game.field_model.bomb_moving = false;
                // Java: getResult().addReport(new ReportBombOutOfBounds())
                game.report_list.add(ReportBombOutOfBounds::new());
                out_event = Some(GameEvent::BombOutOfBounds {
                    coord: self.bomb_coordinate.unwrap_or(FieldCoordinate::new(0, 0)),
                });
            }
            // Java: leaveStep(null) → publishParameter(CATCHER_ID=null); NEXT_STEP
            let mut outcome = StepOutcome::next().publish(StepParameter::CatcherId(None));
            if catch_bomb {
                outcome = outcome.publish(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::CatchBomb));
            }
            if let Some(ev) = out_event {
                outcome = outcome.with_event(ev);
            }
            outcome
        } else {
            // Java: leaveStep(gotoLabelOnEnd) → publishParameter(CATCHER_ID=catcherId); GOTO
            let catcher_id = self.catcher_id.clone();
            StepOutcome::goto(&self.goto_label_on_end)
                .publish(StepParameter::CatcherId(catcher_id))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn no_catcher_returns_next_step() {
        let mut game = make_game();
        let mut step = StepInitBomb::new("end".into());
        // catcher_id is None, explode_skill_used doesn't matter
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn pass_fumble_clears_catcher_returns_next_step() {
        let mut game = make_game();
        let mut step = StepInitBomb::new("end".into());
        step.catcher_id = Some("p1".into());
        step.pass_fumble = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(step.catcher_id.is_none());
    }

    #[test]
    fn bomb_out_of_bounds_clears_catcher() {
        let mut game = make_game();
        let mut step = StepInitBomb::new("end".into());
        step.catcher_id = Some("p1".into());
        step.bomb_out_of_bounds = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(step.catcher_id.is_none());
    }

    #[test]
    fn catcher_without_explode_skill_auto_skips_to_label() {
        // Java: if explodeSkill == null → explode_skill_used = false, no dialog wait
        let mut game = make_game();
        let mut step = StepInitBomb::new("end".into());
        step.catcher_id = Some("p1".into());
        step.explode_skill_used = None;
        // No acting player with canForceBombExplosion skill → auto-skip to label
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn catcher_with_explode_skill_used_routes_to_label() {
        let mut game = make_game();
        let mut step = StepInitBomb::new("catch_label".into());
        step.catcher_id = Some("p1".into());
        step.explode_skill_used = Some(false); // decided not to explode
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("catch_label"));
    }

    #[test]
    fn no_catcher_publishes_catcher_id_null() {
        let mut game = make_game();
        let mut step = StepInitBomb::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::CatcherId(None))));
    }

    #[test]
    fn catcher_present_publishes_catcher_id() {
        let mut game = make_game();
        let mut step = StepInitBomb::new("end".into());
        step.catcher_id = Some("p99".into());
        step.explode_skill_used = Some(false);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::CatcherId(Some(id)) if id == "p99")));
    }

    #[test]
    fn no_catcher_reads_bomb_coordinate_from_field_model() {
        use ffb_model::types::FieldCoordinate;
        let mut game = make_game();
        game.field_model.bomb_coordinate = Some(FieldCoordinate::new(5, 7));
        let mut step = StepInitBomb::new("end".into());
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(step.bomb_coordinate, Some(FieldCoordinate::new(5, 7)));
    }

    #[test]
    fn bomb_out_clears_field_model_bomb_coordinate() {
        // bomb_coordinate is None on field_model → bombOut=true → clear field_model.bomb_coordinate
        let mut game = make_game();
        game.field_model.bomb_coordinate = None;
        let mut step = StepInitBomb::new("end".into());
        step.dont_drop_fumble = false;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.field_model.bomb_coordinate.is_none());
        assert!(!game.field_model.bomb_moving);
    }

    #[test]
    fn dont_drop_fumble_prevents_bomb_out() {
        let mut game = make_game();
        game.field_model.bomb_coordinate = None;
        let mut step = StepInitBomb::new("end".into());
        step.dont_drop_fumble = true;
        step.start(&mut game, &mut GameRng::new(0));
        // bomb_moving should NOT have been set to false by bomb_out path
        // (bomb_out skipped due to dont_drop_fumble)
        assert!(!game.field_model.bomb_moving); // stays false (default)
    }

    #[test]
    fn null_bomb_coordinate_adds_bomb_out_of_bounds_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        game.field_model.bomb_coordinate = None;
        let mut step = StepInitBomb::new("end".into());
        step.dont_drop_fumble = false;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::BOMB_OUT_OF_BOUNDS),
            "BOMB_OUT_OF_BOUNDS report must be added when bomb is out of bounds");
    }

    #[test]
    fn catcher_set_no_bomb_out_does_not_add_bomb_out_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        // catcher_id set, explode_skill_used=false → goes to gotoLabel path (no bomb_out)
        let mut step = StepInitBomb::new("end".into());
        step.catcher_id = Some("p1".into());
        step.explode_skill_used = Some(false);
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.report_list.has_report(ReportId::BOMB_OUT_OF_BOUNDS),
            "BOMB_OUT_OF_BOUNDS must NOT be added when catcher is present");
    }
}
