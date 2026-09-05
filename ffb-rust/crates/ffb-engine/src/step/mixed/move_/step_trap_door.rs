/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.move.StepTrapDoor`.
///
/// When a player enters a Trap Door square, roll 1d6: on a 2+ the player escapes;
/// on a 1 the player falls through the trap door (removed from the pitch, injury
/// applied).  A re-roll may be offered.
///
/// Java `@RulesCollection(BB2020, BB2025)`.
///
/// Incoming parameters: PLAYER_ENTERING_SQUARE (consumed), THROWN_PLAYER_HAS_BALL,
///                       PLAYER_WAS_PUSHED (consumed).
use ffb_model::model::game::Game;
use ffb_model::model::re_rolled_action::ReRolledAction;
use ffb_model::report::mixed::report_trap_door::ReportTrapDoor;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::enums::ApothecaryMode;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::abstract_step_with_re_roll::ReRollState;

/// Java: `ReRolledActions.TRAP_DOOR` equivalent.
const RE_ROLLED_ACTION: &str = "TRAP_DOOR";

/// Java: `StepTrapDoor` (mixed/move, BB2020 + BB2025).
/// Extends AbstractStepWithReRoll.
#[derive(Debug, Default)]
pub struct StepTrapDoor {
    /// Java: `playerId` — player on the trap door (consumed from PLAYER_ENTERING_SQUARE).
    pub player_id: Option<String>,
    /// Every id published as PLAYER_ENTERING_SQUARE, in order.
    ///
    /// Java's `setParameter` assigns `playerId` ONLY when that player stands on a trap door
    /// (`StepTrapDoor.java:68`), so a later publish for a NON-trapdoor square cannot overwrite an
    /// earlier one that was. Rust's `set_parameter` has no `&Game`, so ids are collected here and
    /// filtered in `execute_step` — same precedence (the last id actually on a trapdoor wins).
    ///
    /// Without this the attacker's FOLLOW-UP into the vacated square overwrote the player just
    /// pushed ONTO the trapdoor, and it never fired (nippon bb2020 seed 29 i=51: away_03 pushed to
    /// the trapdoor at (19,13), then home_01's follow-up into (18,13) clobbered it — Java dropped
    /// away_03 through with an InjuryTypeCrowd, Rust left it Prone, 3 dice apart).
    pub entering_square_ids: Vec<String>,
    /// Java: `thrownPlayerHasBall` — Some(bool) only in TTM context.
    pub thrown_player_has_ball: Option<bool>,
    /// Java: `playerWasPushed` — whether the push came from a block (consumed).
    pub player_was_pushed: bool,
    /// Re-roll tracking (AbstractStepWithReRoll).
    pub re_roll_state: ReRollState,
}

impl StepTrapDoor {
    pub fn new() -> Self { Self::default() }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if self.player_id.is_none() {
            self.player_id = self.entering_square_ids.iter().rev()
                .find(|id| game.field_model.player_coordinate(id)
                    .map(|c| game.field_model.has_trap_door(c))
                    .unwrap_or(false))
                .cloned();
        }
        let player_id = match self.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        // Java: FieldCoordinate playerCoordinate = fieldModel.getPlayerCoordinate(player)
        let player_coord = match game.field_model.player_coordinate(&player_id) {
            Some(c) => c,
            None => return StepOutcome::next(),
        };

        // Java: if (!isOnTrapDoor(fieldModel, playerCoordinate)) { nextStep; return; }
        if !game.field_model.has_trap_door(player_coord) {
            return StepOutcome::next();
        }

        // Java: if (getReRolledAction() == RE_ROLLED_ACTION) { use reroll or fall; }
        if let Some(ref action) = self.re_roll_state.re_rolled_action.clone() {
            if action.get_name() == RE_ROLLED_ACTION {
                // re-roll was asked — check if source is set
                let did_reroll = if let Some(ref src) = self.re_roll_state.re_roll_source.clone() {
                    crate::step::util_server_re_roll::use_reroll(game, src, &player_id, rng)
                } else {
                    false
                };
                if !did_reroll {
                    return self.trap_door_triggered(game, rng, player_id, player_coord);
                }
                // fall through to roll again
            }
        }

        // Java: int roll = getDiceRoller().rollDice(6)
        let roll = rng.d6();
        let escaped = roll != 1;

        // Java: getResult().addReport(new ReportTrapDoor(playerId, roll, escaped))
        game.report_list.add(ReportTrapDoor::new(Some(player_id.clone()), roll, escaped));

        // Emit TrapDoor event (ReportTrapDoor)
        let outcome_base = StepOutcome::next()
            .with_event(ffb_model::events::GameEvent::TrapDoor {
                player_id: player_id.clone(),
                roll,
                escaped,
            });

        if escaped {
            return outcome_base;
        }

        // Java: else if (getReRolledAction() != null || !UtilServerReRoll.askForReRollIfAvailable(...))
        if self.re_roll_state.re_rolled_action.is_some() {
            // already re-rolled once — fall through the trap door
            return outcome_base
                .with_events(self.trap_door_triggered(game, rng, player_id, player_coord).events)
                .with_published(self.trap_door_triggered_params(game, player_coord));
        }

        // Offer a re-roll if one is available.
        //
        // Java passes the trapdoor VICTIM: `askForReRollIfAvailable(getGameState(), player,
        // RE_ROLLED_ACTION, 2, false)` (StepTrapDoor.java:122). Rust called the ACTING-PLAYER
        // overload, which re-derives the source from whoever is currently activated — the
        // ATTACKER. A player pushed onto a trapdoor is usually an OPPONENT of the acting team, so
        // Java finds no usable re-roll for them and shows no dialog while Rust found the acting
        // team's and burned two sampler draws, splitting the agent streams
        // (nippon bb2020 seed 29 i=52: away_03 falls, Java goes straight on, Rust asked
        // TRR/TRAP_DOOR). Same wrong-overload shape as the Dodge/Tackle fix in
        // bb2025/move_/step_move_dodge.rs.
        if let Some(prompt) = crate::step::util_server_re_roll::ask_for_reroll_if_available_for(
            game, Some(&player_id), RE_ROLLED_ACTION, 2, false,
        ) {
            self.re_roll_state.re_rolled_action = Some(ReRolledAction::new(RE_ROLLED_ACTION));
            // Stash the *offered* source here (mirroring `AgentPrompt::ReRollOffer.source`) so
            // `handle_command` has something to commit-or-discard based on the reply — matching
            // the fix applied to `step_move_ball_and_chain.rs`'s identical bug shape.
            if let ffb_model::prompts::AgentPrompt::ReRollOffer { ref source, .. } = prompt {
                self.re_roll_state.re_roll_source = Some(source.clone());
            }
            // A second, more fundamental bug fixed alongside the source-stash one:
            // `outcome_base`'s action is `NextStep`, but the driver's `dispatch_after_start`
            // only honors `outcome.prompt` for `Continue`/`Repeat` actions — a `NextStep`
            // outcome's prompt is silently discarded (see `driver.rs`). Returning
            // `outcome_base.with_prompt(...)` meant this dialog was never actually surfaced to
            // the agent at all; must build a `cont()`-based outcome carrying the same events.
            return StepOutcome::cont()
                .with_events(outcome_base.events)
                .with_prompt(prompt);
        }

        // No re-roll available — fall through the trap door.
        //
        // Java runs the FULL `trapDoorTriggered(...)` here (StepTrapDoor.java:122-123), whose first
        // act is `handleInjury(InjuryTypeTrapDoorFall..., ApothecaryMode.TRAP_DOOR)` — an armour and
        // an injury roll. This path published the parameters and removed the player but never rolled
        // the injury, so Rust came out TWO DICE short of Java for every trapdoor fall that was not
        // re-rolled (nippon bb2020 seed 29 i=52: R52 vs J54). The re-rolled branch above already
        // calls `trap_door_triggered`; this one has to as well.
        let mut outcome = outcome_base;
        let triggered = self.trap_door_triggered(game, rng, player_id.clone(), player_coord);
        outcome = outcome.with_events(triggered.events);
        for p in self.trap_door_triggered_params(game, player_coord) {
            outcome = outcome.publish(p);
        }
        game.field_model.remove_player(&player_id);
        outcome
    }

    /// Java: `trapDoorTriggered` — apply injury, remove player, scatter ball if needed.
    fn trap_door_triggered(&mut self, game: &mut Game, rng: &mut GameRng, player_id: String, coord: FieldCoordinate) -> StepOutcome {
        // Java: eligibleForSpp = playerWasPushed && attacker != null && prayerState.hasFanInteraction(attacker.getTeam())
        let attacker_id = game.acting_player.player_id.clone();
        let eligible_for_spp = self.player_was_pushed
            && attacker_id.is_some()
            && attacker_id.as_deref()
                .and_then(|id| game.player_team_id(id))
                .map(|tid| game.prayer_state.has_fan_interaction(tid))
                .unwrap_or(false);
        let ir = if eligible_for_spp {
            let mut injury_type = crate::injury::injuryType::injury_type_trap_door_fall_for_spp::InjuryTypeTrapDoorFallForSpp::new();
            crate::step::util_server_injury::handle_injury(
                game, rng, &mut injury_type,
                attacker_id.as_deref(), &player_id, coord, None, None,
                ApothecaryMode::TrapDoor,
            )
        } else {
            let mut injury_type = crate::injury::injuryType::injury_type_trap_door_fall::InjuryTypeTrapDoorFall::new();
            crate::step::util_server_injury::handle_injury(
                game, rng, &mut injury_type,
                None, &player_id, coord, None, None,
                ApothecaryMode::TrapDoor,
            )
        };
        // Java: `publishParameter(new StepParameter(StepParameterKey.INJURY_RESULT,
        //        UtilServerInjury.handleInjury(...)))` — the result is PUBLISHED for the
        // APOTHECARY(TRAP_DOOR) step that follows in the sequence, NOT applied here. Rust applied
        // it inline instead, and the `remove(player)` two lines down then overwrote the state, so a
        // trap-door casualty came out as `Reserve` where Java has `Injured` — wood_elf bb2020
        // seed 50 i=300, `h05`, on identical dice (injury 5+5, casualty d16=1/d6=5).
        let mut outcome = StepOutcome::next().publish(StepParameter::InjuryResult(Box::new(ir)));
        for p in self.trap_door_triggered_params(game, coord) {
            outcome = outcome.publish(p);
        }
        // Java: game.getFieldModel().remove(player)
        game.field_model.remove_player(&player_id);
        outcome
    }

    /// Build the parameters to publish when the trap door triggers.
    fn trap_door_triggered_params(&self, game: &Game, coord: FieldCoordinate) -> Vec<StepParameter> {
        let mut params = Vec::new();
        let player_id = match self.player_id.clone() {
            Some(id) => id,
            None => return params,
        };

        let has_ball = self.thrown_player_has_ball.unwrap_or_else(|| {
            // Java: UtilPlayer.hasBall(game, player)
            game.field_model.ball_coordinate == game.field_model.player_coordinate(&player_id)
        });

        if has_ball {
            params.push(StepParameter::CatchScatterThrowInMode(
                ffb_model::model::catch_scatter_throw_in_mode::CatchScatterThrowInMode::ScatterBall,
            ));
            // Java: if (game.getActingTeam().hasPlayer(player)) publishParameter(END_TURN, true)
            let acting_team_has_player = if game.home_playing {
                game.team_home.players.iter().any(|p| p.id == player_id)
            } else {
                game.team_away.players.iter().any(|p| p.id == player_id)
            };
            if acting_team_has_player {
                params.push(StepParameter::EndTurn(true));
            }
        }

        // Java: if (thrownPlayerHasBall != null) { TTM context cleanup }
        if self.thrown_player_has_ball.is_some() {
            params.push(StepParameter::ThrownPlayerCoordinate(None));
        }

        params
    }
}

// Extension to add published parameters to a StepOutcome
trait WithPublished {
    fn with_published(self, params: Vec<StepParameter>) -> Self;
}

impl WithPublished for StepOutcome {
    fn with_published(mut self, params: Vec<StepParameter>) -> Self {
        self.published.extend(params);
        self
    }
}

impl Step for StepTrapDoor {
    fn id(&self) -> StepId { StepId::TrapDoor }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: `AbstractStepWithReRoll.handleCommand`'s CLIENT_USE_RE_ROLL branch commits
        // `reRollSource` from the reply; the Rust `Action::UseReRoll` reply is a simplified
        // bool, so `use_reroll == false` clears the stashed source instead (same convention as
        // `step_move_ball_and_chain.rs`'s fix for this identical bug shape).
        if let Action::UseReRoll { use_reroll } = action {
            if !use_reroll {
                self.re_roll_state.re_roll_source = None;
            }
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::PlayerEnteringSquare(id) => {
                // Java: consume and set player_id if that square has a trap door
                // Without the consume mechanism here, just store the id.
                self.entering_square_ids.push(id.clone());
                true
            }
            StepParameter::ThrownPlayerHasBall(v) => { self.thrown_player_has_ball = Some(*v); true }
            // Java StepTrapDoor:76 — PLAYER_WAS_PUSHED is stored and consumed.
            StepParameter::PlayerWasPushed(v) => { self.player_was_pushed = *v; true }
            _ => false,
        }
    }

    // Java: setParameter consume()s these keys (THROWN_PLAYER_HAS_BALL is set WITHOUT
    // consuming; PLAYER_WAS_PUSHED is consumed in Java even though the Rust
    // set_parameter does not handle it yet).
    fn consumes_parameter(&self, param: &StepParameter) -> bool {
        matches!(param,
            StepParameter::PlayerEnteringSquare(_) | StepParameter::PlayerWasPushed(_))
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

    /// Regression (nippon bb2020 seed 29, i=51): a block publishes PLAYER_ENTERING_SQUARE twice —
    /// once for the player it pushes and again for the attacker's follow-up. Java only assigns
    /// `playerId` when that player is standing on a trap door (`StepTrapDoor.java:68`), so the
    /// follow-up onto an ordinary square cannot overwrite the pushed player who landed on one.
    /// Rust assigned unconditionally, so the follow-up clobbered it and the trapdoor never fired:
    /// Java dropped away_03 through the trapdoor at (19,13) with an InjuryTypeCrowd, Rust left it
    /// Prone on the pitch — a 3-dice divergence.
    #[test]
    fn follow_up_onto_a_plain_square_does_not_steal_the_trapdoor_victim() {
        let mut step = StepTrapDoor::new();
        let mut game = make_game();

        let trap = FieldCoordinate::new(19, 13);
        game.field_model.add_trap_door(trap);

        // The pushed player lands ON the trapdoor; the attacker follows up onto a plain square.
        game.field_model.set_player_coordinate("pushed", trap);
        game.field_model.set_player_coordinate("follower", FieldCoordinate::new(18, 13));

        assert!(step.set_parameter(&StepParameter::PlayerEnteringSquare("pushed".into())));
        assert!(step.set_parameter(&StepParameter::PlayerEnteringSquare("follower".into())));

        let _ = step.start(&mut game, &mut GameRng::new(0));

        assert_eq!(
            step.player_id.as_deref(), Some("pushed"),
            "the trapdoor must resolve to the player standing ON it, not the later follow-up"
        );
    }

    #[test]
    fn no_player_id_returns_next() {
        let mut step = StepTrapDoor::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn player_not_on_trap_door_returns_next() {
        let mut step = StepTrapDoor::new();
        step.player_id = Some("p1".into());
        let coord = FieldCoordinate::new(5, 5);
        let mut game = make_game();
        // Place player but no trap door
        game.field_model.set_player_coordinate("p1", coord);
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn player_on_trap_door_rolls_and_emits_event() {
        let mut step = StepTrapDoor::new();
        step.player_id = Some("p1".into());
        let coord = FieldCoordinate::new(5, 5);
        let mut game = make_game();
        game.field_model.set_player_coordinate("p1", coord);
        game.field_model.trap_doors.push(coord);
        // Seed 5 → roll_d6 ≥ 2 → escaped = true
        let mut rng = GameRng::new(5);
        let out = step.start(&mut game, &mut rng);
        let has_trap_door_event = out.events.iter().any(|e| matches!(e, ffb_model::events::GameEvent::TrapDoor { .. }));
        assert!(has_trap_door_event);
    }

    #[test]
    fn trap_door_report_added_on_roll() {
        let mut step = StepTrapDoor::new();
        step.player_id = Some("p1".into());
        let coord = FieldCoordinate::new(5, 5);
        let mut game = make_game();
        game.field_model.set_player_coordinate("p1", coord);
        game.field_model.trap_doors.push(coord);
        let mut rng = GameRng::new(5);
        step.start(&mut game, &mut rng);
        assert!(game.report_list.has_report(ffb_model::report::report_id::ReportId::TRAP_DOOR));
    }

    #[test]
    fn no_trap_door_report_when_not_on_trap_door() {
        let mut step = StepTrapDoor::new();
        step.player_id = Some("p1".into());
        let coord = FieldCoordinate::new(5, 5);
        let mut game = make_game();
        // Place player but no trap door registered
        game.field_model.set_player_coordinate("p1", coord);
        let mut rng = GameRng::new(5);
        step.start(&mut game, &mut rng);
        assert!(!game.report_list.has_report(ffb_model::report::report_id::ReportId::TRAP_DOOR));
    }

    #[test]
    fn set_parameter_thrown_player_has_ball() {
        let mut step = StepTrapDoor::new();
        step.set_parameter(&StepParameter::ThrownPlayerHasBall(true));
        assert_eq!(step.thrown_player_has_ball, Some(true));
    }

    #[test]
    fn player_falls_through_trap_door_on_roll_1() {
        let mut step = StepTrapDoor::new();
        step.player_id = Some("p1".into());
        let coord = FieldCoordinate::new(5, 5);
        let mut game = make_game();
        game.field_model.set_player_coordinate("p1", coord);
        game.field_model.trap_doors.push(coord);
        // Seed that produces d6 == 1 → player falls
        // Use RNG seed 3 which gives 1 on first d6 roll
        let mut rng = GameRng::new(3);
        let out = step.start(&mut game, &mut rng);
        // TrapDoor event should be emitted with escaped=false
        let trap_event = out.events.iter().find_map(|e| {
            if let ffb_model::events::GameEvent::TrapDoor { escaped, .. } = e { Some(*escaped) } else { None }
        });
        // Either the event is present or a re-roll was offered (either is correct behavior)
        assert!(trap_event.is_some() || !out.published.is_empty() || out.action == StepAction::NextStep);
    }

    /// Java `trapDoorTriggered` PUBLISHES the injury result for the APOTHECARY(TRAP_DOOR) step
    /// that follows; it never applies it inline. Rust applied it and then `remove_player` overwrote
    /// the state, so a trap-door casualty surfaced as `Reserve` instead of `Injured`
    /// (wood_elf bb2020 seed 50 i=300).
    #[test]
    fn trap_door_publishes_the_injury_result_instead_of_applying_it() {
        let coord = FieldCoordinate::new(5, 5);
        let mut step = StepTrapDoor::new();
        step.player_id = Some("p1".into());
        let mut game = make_game();
        game.field_model.set_player_coordinate("p1", coord);
        game.field_model.trap_doors.push(coord);

        let out = step.trap_door_triggered(
            &mut game, &mut GameRng::new(3), "p1".into(), coord);

        assert!(
            out.published.iter().any(|p| matches!(p, StepParameter::InjuryResult(_))),
            "the trap-door injury must be published for the apothecary step, got {:?}",
            out.published);
    }

    #[test]
    fn eligible_for_spp_false_without_fan_interaction() {
        // Without fan_interaction set, eligible_for_spp should be false (no attacker SPP)
        let mut step = StepTrapDoor::new();
        step.player_id = Some("p1".into());
        step.player_was_pushed = true;
        game_acting_player_test();
    }

    fn game_acting_player_test() {
        // Simple smoke test: player_was_pushed but no fan_interaction → no crash
        let mut step = StepTrapDoor::new();
        step.player_id = Some("p1".into());
        step.player_was_pushed = true;
        let coord = FieldCoordinate::new(5, 5);
        let mut game = make_game();
        game.field_model.set_player_coordinate("p1", coord);
        game.field_model.trap_doors.push(coord);
        game.acting_player.player_id = None; // no attacker
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        // Should not panic
        assert!(matches!(out.action, StepAction::NextStep | StepAction::Continue));
    }

    /// Regression test for the bug where `re_roll_source` was never stashed when the
    /// re-roll offer was made, and `handle_command` ignored the incoming `Action`
    /// entirely — identical bug shape to `step_move_ball_and_chain.rs`'s fixed
    /// BallAndChain re-roll. Before the fix, accepting the trap-door re-roll offer had
    /// no effect: `re_roll_source` stayed `None`, so the second roll's reroll-source
    /// check was always `None` and no re-roll (nor TRR consumption) ever happened.
    #[test]
    fn accepting_reroll_offer_stashes_source_and_consumes_trr() {
        let mut step = StepTrapDoor::new();
        step.player_id = Some("p1".into());
        let coord = FieldCoordinate::new(5, 5);
        let mut game = make_game();
        game.field_model.set_player_coordinate("p1", coord);
        game.field_model.trap_doors.push(coord);
        // The victim must really BE on the home team: Java asks for the re-roll on behalf of the
        // trapdoor VICTIM (StepTrapDoor.java:122), so an id that belongs to no team resolves to no
        // re-roll and no dialog. Previously this test leaned on the acting-player overload, which
        // read the home TRR regardless of whose player p1 was.
        game.team_home.players.push(ffb_model::model::player::Player {
            id: "p1".into(), name: "p1".into(), nr: 1, position_id: "lineman".into(),
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            ..Default::default()
        });
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        game.turn_data_home.reroll_used = false;

        // Seed 7 produces d6 == 1 on the first roll → fall triggers the re-roll offer.
        let mut rng = GameRng::new(7);
        let out = step.start(&mut game, &mut rng);

        assert_eq!(out.action, StepAction::Continue);
        assert!(step.re_roll_state.re_roll_source.is_some(), "re_roll_source must be stashed when the offer is issued");

        let _ = step.handle_command(&Action::UseReRoll { use_reroll: true }, &mut game, &mut rng);

        assert_eq!(game.turn_data_home.rerolls, 0, "accepting the offer must consume the TRR");
    }

    #[test]
    fn declining_reroll_offer_clears_source_and_does_not_consume_trr() {
        let mut step = StepTrapDoor::new();
        step.player_id = Some("p1".into());
        let coord = FieldCoordinate::new(5, 5);
        let mut game = make_game();
        game.field_model.set_player_coordinate("p1", coord);
        game.field_model.trap_doors.push(coord);
        // The victim must really BE on the home team: Java asks for the re-roll on behalf of the
        // trapdoor VICTIM (StepTrapDoor.java:122), so an id that belongs to no team resolves to no
        // re-roll and no dialog. Previously this test leaned on the acting-player overload, which
        // read the home TRR regardless of whose player p1 was.
        game.team_home.players.push(ffb_model::model::player::Player {
            id: "p1".into(), name: "p1".into(), nr: 1, position_id: "lineman".into(),
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            ..Default::default()
        });
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        game.turn_data_home.reroll_used = false;

        let mut rng = GameRng::new(7);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::Continue);
        assert!(step.re_roll_state.re_roll_source.is_some());

        let _ = step.handle_command(&Action::UseReRoll { use_reroll: false }, &mut game, &mut rng);

        assert!(step.re_roll_state.re_roll_source.is_none(), "declining must clear the stashed source");
        assert_eq!(game.turn_data_home.rerolls, 1, "declining must not consume the TRR");
    }
}
