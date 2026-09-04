/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.pass.StepInitPassing`.
///
/// Initialization step of the pass sequence (BB2016).
/// - CLIENT_PASS: sets pass coordinate + catcher + thrower.
/// - CLIENT_HAND_OVER: sets catcher + pass coordinate from player.
/// - CLIENT_ACTING_PLAYER (no id): end player action.
/// - CLIENT_END_TURN: end turn.
///
/// Init parameters: GOTO_LABEL_ON_END (mandatory), TARGET_COORDINATE (opt), CATCHER_ID (opt).
/// Publishes: CATCHER_ID, END_TURN, END_PLAYER_ACTION, TARGET_COORDINATE.
///
/// client-only: UtilRangeRuler.createRangeRuler — range ruler is client-side display only.
use ffb_model::enums::{PlayerAction, TurnMode};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_mechanics::bb2016::pass_mechanic::PassMechanic;
use ffb_mechanics::pass_mechanic::PassMechanic as PassMechanicTrait;
use ffb_model::prompts::agent_prompt::AgentPrompt;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepInitPassing` (bb2016/pass).
pub struct StepInitPassing {
    /// Java: `fGotoLabelOnEnd` — mandatory init param.
    goto_label_on_end: String,
    /// Java: `fCatcherId`
    catcher_id: Option<String>,
    /// Java: `fEndTurn`
    end_turn: bool,
    /// Java: `fEndPlayerAction`
    end_player_action: bool,
    /// Stores the TARGET_COORDINATE init param for resolution in start() when game is available.
    pending_target_coordinate: Option<ffb_model::types::FieldCoordinate>,
}

impl StepInitPassing {
    pub fn new() -> Self {
        Self {
            goto_label_on_end: String::new(),
            catcher_id: None,
            end_turn: false,
            end_player_action: false,
            pending_target_coordinate: None,
        }
    }

    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        // Bomb re-throw window -- see the mixed StepInitPassing for the full note. Java parks
        // here with no dialog; the prompt only surfaces that wait so the agent can decline.
        if game.thrower_id.is_none() || game.thrower_action.is_none() {
            if let Some(pid) = game.acting_player.player_id.clone() {
                // The park is NOT bomb-specific. `ParityRunner`'s INIT_PASSING handler gates on
                // `game.getThrower() == null && actingPlayer != null && getPlayerId() != null`
                // and nothing else, then answers with `sendPassAction` — the ordinary
                // teammate-coordinate pick, one actionRng draw. The bomb re-throw is only the
                // most common way to reach it; a BB2016 Take Root failure on a HAND_OVER_MOVE /
                // PASS_MOVE reaches it too, because `bb2016.StepTakeRoot.cancelPlayerAction`
                // (unlike its bb2020/bb2025 twins) reverts the action to HAND_OVER / PASS
                // WITHOUT setting the thrower, and `StepEndMoving` then pushes the Pass
                // sequence. Restricting the prompt to the bomb turn modes left that park with no
                // prompt at all and the driver stalled the game outright
                // (halfling bb2016 seed 1 idx 55).
                return StepOutcome::cont()
                    .with_prompt(AgentPrompt::BombRethrow { player_id: pid });
            }
            return StepOutcome::cont();
        }
        // Java: Player<?> catcher = game.getPlayerById(fCatcherId);
        //
        // The parameter is absent on the *_MOVE path: `StepInitMoving` publishes it, but the Pass
        // sequence is pushed afterwards by `StepEndMoving`, so it never arrives. Java derives the
        // catcher from the pass coordinate in its own CLIENT_PASS handler
        // (`fieldModel.getPlayer(game.getPassCoordinate())`), which is the same rule — apply it
        // here when nothing was handed in. A throw at an empty square still yields no catcher.
        let catcher_id = self.catcher_id.clone().or_else(|| {
            game.pass_coordinate
                .and_then(|c| game.field_model.player_at(c).cloned())
        });
        let catcher_exists = catcher_id.as_deref()
            .map(|id| game.player(id).is_some())
            .unwrap_or(false);
        // Java: catcher publish happens unconditionally before the end_turn/end_player_action/
        // blood_lust checks, so it must be present on every returned outcome below.
        let mut out = StepOutcome::next();
        if catcher_exists {
            out = out.publish(StepParameter::CatcherId(catcher_id.clone()));
        }
        if self.end_turn {
            out.action = crate::step::framework::StepAction::GotoLabel;
            out.goto_label = Some(self.goto_label_on_end.clone());
            return out.publish(StepParameter::EndTurn(true));
        }
        if self.end_player_action {
            out.action = crate::step::framework::StepAction::GotoLabel;
            out.goto_label = Some(self.goto_label_on_end.clone());
            return out.publish(StepParameter::EndPlayerAction(true));
        }
        // Java: if thrower==actingPlayer && isSufferingBloodLust && !hasFed → goto end
        let thrower_is_acting = game.thrower_id.is_some()
            && game.thrower_id == game.acting_player.player_id;
        if thrower_is_acting
            && game.acting_player.suffering_blood_lust
            && !game.acting_player.has_fed
        {
            out.action = crate::step::framework::StepAction::GotoLabel;
            out.goto_label = Some(self.goto_label_on_end.clone());
            return out;
        }

        let thrower_action = game.thrower_action;
        let thrower_coordinate = game.thrower_id.as_deref()
            .and_then(|id| game.field_model.player_coordinate(id));

        // Java: HAND_OVER branch — requires thrower==actingPlayer && catcher != null.
        if thrower_action == Some(PlayerAction::HandOver)
            && thrower_is_acting
            && catcher_exists
        {
            game.acting_player.has_passed = true;
            game.concession_possible = false;
            game.turn_data_mut().hand_over_used = true;
            game.turn_data_mut().turn_started = true;
            return out;
        }

        let mechanic = PassMechanic::new();
        let pass_coord = game.pass_coordinate;
        let passing_distance_valid = pass_coord.is_some()
            && mechanic.find_passing_distance(game, thrower_coordinate, pass_coord, false).is_some();

        // Java: THROW_BOMB (thrower==actingPlayer) / HAIL_MARY_BOMB branch.
        if (passing_distance_valid
            && thrower_is_acting
            && thrower_action == Some(PlayerAction::ThrowBomb))
            || thrower_action == Some(PlayerAction::HailMaryBomb)
        {
            game.acting_player.has_passed = true;
            game.turn_data_mut().turn_started = true;
            game.concession_possible = false;
            // client-only: UtilRangeRuler.createRangeRuler — client display only (THROW_BOMB case)
            return out;
        }

        // Java: PASS (thrower==actingPlayer) / HAIL_MARY_PASS branch.
        if (passing_distance_valid
            && thrower_is_acting
            && thrower_action == Some(PlayerAction::Pass))
            || thrower_action == Some(PlayerAction::HailMaryPass)
        {
            game.acting_player.has_passed = true;
            game.turn_data_mut().turn_started = true;
            game.concession_possible = false;
            game.turn_data_mut().pass_used = true;
            // client-only: UtilRangeRuler.createRangeRuler — client display only (PASS case)
            return out;
        }

        // Java: (THROW_BOMB || DUMP_OFF) / HAIL_MARY_BOMB branch — no thrower==actingPlayer requirement
        // for the distance check; hasPassed only set if thrower==actingPlayer.
        if (passing_distance_valid
            && matches!(thrower_action, Some(PlayerAction::ThrowBomb) | Some(PlayerAction::DumpOff)))
            || thrower_action == Some(PlayerAction::HailMaryBomb)
        {
            if thrower_is_acting {
                game.acting_player.has_passed = true;
            }
            // client-only: UtilRangeRuler.createRangeRuler — client display only (non-HAIL_MARY_BOMB case)
            return out;
        }

        // Java: no branch matched (the throw is out of range) — `executeStep()` returns without
        // calling `setNextAction`, so StepInitPassing stays the current step, WAITING. Both
        // reference agents then end the turn: ParityRunner's step dispatch has no INIT_PASSING
        // case, so the stuck step falls to its `default:` branch and injects
        // `ClientCommandEndTurn` (which StepInitPassing.handleCommand turns into
        // `fEndTurn = true` → `publishParameter(END_TURN, true)` + `GOTO_LABEL(fGotoLabelOnEnd)`).
        // Produce that same observable result directly — turnover, ball unmoved, zero dice —
        // rather than waiting for a prompt the headless driver cannot surface. Identical to the
        // shared `mixed::pass::step_init_passing` handling of this case.
        out.action = crate::step::framework::StepAction::GotoLabel;
        out.goto_label = Some(self.goto_label_on_end.clone());
        out.publish(StepParameter::EndTurn(true))
    }
}

impl Default for StepInitPassing {
    fn default() -> Self { Self::new() }
}

impl Step for StepInitPassing {
    fn id(&self) -> StepId { StepId::InitPassing }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java init(): if TARGET_COORDINATE set, resolve pass coordinate + catcher + thrower
        if let Some(coord) = self.pending_target_coordinate.take() {
            game.pass_coordinate = Some(coord);
            self.catcher_id = game.field_model.player_at(coord).map(|id| id.clone());
            game.thrower_id = game.acting_player.player_id.clone();
            game.thrower_action = game.acting_player.player_action;
        }
        if std::env::var("FFB_BOMB").is_ok() {
            eprintln!(
                "RINITPASS acting={:?} act={:?} thrower={:?} pass_coord={:?} defender={:?}",
                game.acting_player.player_id, game.acting_player.player_action,
                game.thrower_id, game.pass_coordinate, game.defender_id
            );
        }
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::Pass { coord } => {
                game.pass_coordinate = Some(*coord);
                let catcher = game.field_model.player_at(*coord).map(|id| id.clone());
                self.catcher_id = catcher.clone();
                // Java: if defender != null && defenderAction == DUMP_OFF → thrower = defender
                if game.defender_id.is_some()
                    && matches!(game.defender_action, Some(PlayerAction::DumpOff))
                {
                    game.thrower_id = game.defender_id.clone();
                    game.thrower_action = game.defender_action;
                } else {
                    game.thrower_id = game.acting_player.player_id.clone();
                    game.thrower_action = game.acting_player.player_action;
                }
                self.execute_step(game)
            }
            Action::EndTurn => {
                self.end_turn = true;
                self.execute_step(game)
            }
            // Java: CLIENT_ACTING_PLAYER with no player_id → fEndPlayerAction = true → executeStep
            Action::ActivatePlayer { player_id, .. } if player_id.is_empty() => {
                self.end_player_action = true;
                self.execute_step(game)
            }
            _ => StepOutcome::cont(),
        }
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(s) => { self.goto_label_on_end = s.clone(); true }
            StepParameter::CatcherId(v)      => { self.catcher_id = v.clone(); true }
            StepParameter::EndTurn(v)        => { self.end_turn = *v; true }
            StepParameter::EndPlayerAction(v)=> { self.end_player_action = *v; true }
            StepParameter::TargetCoordinate(c) => {
                // Store for resolution in start() when game is available.
                self.pending_target_coordinate = Some(*c);
                true
            }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    /// Adds a minimal player to the home team at the given coordinate, mirroring the
    /// helper pattern used in step_intercept.rs tests.
    fn add_player(game: &mut Game, id: &str, coord: ffb_model::types::FieldCoordinate) {
        use ffb_model::enums::{PlayerGender, PlayerType, PlayerState, PS_STANDING};
        use ffb_model::model::player::Player;
        let player = Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: std::collections::HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        game.team_home.players.push(player);
        game.field_model.set_player_coordinate(id, coord);
        game.field_model.set_player_state(id, PlayerState::new(PS_STANDING));
    }

    #[test]
    fn no_thrower_returns_continue() {
        let mut game = make_game();
        let mut step = StepInitPassing::new();
        step.goto_label_on_end = "end".into();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::Continue));
    }

    /// Java `StepInitPassing.executeStep()` gates the PASS branch on
    /// `mechanic.findPassingDistance(game, throwerCoordinate, game.getPassCoordinate(), false) != null`
    /// using the **BB2016** `PassMechanic` throwing-range table. When the target is out of range no
    /// branch matches, `executeStep` returns without `setNextAction`, and the step stays waiting —
    /// at which point ParityRunner's `default:` dispatch injects `ClientCommandEndTurn`, so the
    /// observable result is a turnover with the ball unmoved and ZERO dice rolled.
    ///
    /// bb2016 games used to be driven through the shared `mixed`/bb2025 pass steps, whose range
    /// check reads the bb2020+ table — an out-of-range bb2016 throw was accepted there, rolling the
    /// accuracy d6 and offering an interception that stock Java never rolls (underworld bb2016
    /// seed 72 i=74: away_03 at (12,9) throwing to (25,7), 13 squares).
    #[test]
    fn out_of_range_pass_ends_the_turn_without_rolling() {
        use ffb_model::types::FieldCoordinate;
        let mut game = make_game();
        add_player(&mut game, "thrower", FieldCoordinate::new(12, 9));
        game.home_playing = true;
        game.acting_player.player_id = Some("thrower".into());
        game.acting_player.player_action = Some(ffb_model::enums::PlayerAction::Pass);
        game.thrower_id = Some("thrower".into());
        game.thrower_action = Some(ffb_model::enums::PlayerAction::Pass);
        // 13 squares along x, 2 along y — off the end of the bb2016 throwing-range table.
        game.pass_coordinate = Some(FieldCoordinate::new(25, 7));

        let mut step = StepInitPassing::new();
        step.goto_label_on_end = "end".into();
        let out = step.start(&mut game, &mut GameRng::new(0));

        assert!(matches!(out.action, StepAction::GotoLabel),
            "an out-of-range throw must not advance into the Pass step");
        assert_eq!(out.goto_label.as_deref(), Some("end"));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))),
            "the stuck step is resolved by the agents' EndTurn injection → turnover");
        assert!(!game.acting_player.has_passed,
            "Java only sets hasPassed inside a matched branch");
        assert!(!game.turn_data().pass_used,
            "an out-of-range throw never consumes the team's pass action");
    }

    /// The null-thrower park is not bomb-specific. `ParityRunner`'s INIT_PASSING handler is
    /// ```java
    /// ActingPlayer bombAp = game.getActingPlayer();
    /// if (game.getThrower() == null && bombAp != null && bombAp.getPlayerId() != null) {
    ///     sendPassAction(game, gameState, bombAp.getPlayerId());
    /// } else { ...EndTurn... }
    /// ```
    /// — no turn-mode condition at all. A BB2016 Take Root failure on a HAND_OVER_MOVE reverts the
    /// action to HAND_OVER without setting the thrower, and `StepEndMoving` then pushes the Pass
    /// sequence, so the park is reached in a REGULAR turn.
    #[test]
    fn a_null_thrower_park_prompts_in_a_regular_turn_too() {
        use ffb_model::types::FieldCoordinate;
        for mode in [TurnMode::Regular, TurnMode::BombHome] {
            let mut game = make_game();
            add_player(&mut game, "p1", FieldCoordinate::new(12, 7));
            game.turn_mode = mode;
            game.home_playing = true;
            game.acting_player.player_id = Some("p1".into());
            game.acting_player.player_action = Some(ffb_model::enums::PlayerAction::HandOver);
            game.thrower_id = None;
            game.thrower_action = None;

            let mut step = StepInitPassing::new();
            step.goto_label_on_end = "end".into();
            let out = step.start(&mut game, &mut GameRng::new(0));

            assert!(matches!(out.prompt, Some(AgentPrompt::BombRethrow { .. })),
                "{mode:?}: the park must surface a prompt, not stall the driver");
        }
    }

    #[test]
    fn end_turn_goto_label() {
        let mut game = make_game();
        game.thrower_id = Some("p1".into());
        game.thrower_action = Some(ffb_model::enums::PlayerAction::Pass);
        let mut step = StepInitPassing::new();
        step.goto_label_on_end = "end".into();
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::GotoLabel));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn end_player_action_goto_label() {
        let mut game = make_game();
        game.thrower_id = Some("p1".into());
        game.thrower_action = Some(ffb_model::enums::PlayerAction::Pass);
        let mut step = StepInitPassing::new();
        step.goto_label_on_end = "end".into();
        step.end_player_action = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::GotoLabel));
    }

    #[test]
    fn set_parameter_goto_label_on_end() {
        let mut step = StepInitPassing::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnEnd("x".into())));
        assert_eq!(step.goto_label_on_end, "x");
    }

    #[test]
    fn blood_lust_thrower_not_fed_goto_label() {
        let mut game = make_game();
        game.thrower_id = Some("p1".into());
        game.thrower_action = Some(ffb_model::enums::PlayerAction::Pass);
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.suffering_blood_lust = true;
        game.acting_player.has_fed = false;
        let mut step = StepInitPassing::new();
        step.goto_label_on_end = "end".into();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::GotoLabel));
        assert_eq!(out.goto_label.as_deref(), Some("end"));
    }

    #[test]
    fn blood_lust_thrower_already_fed_continues() {
        let mut game = make_game();
        game.thrower_id = Some("p1".into());
        // HailMaryPass bypasses the thrower==actingPlayer / passing-distance checks entirely
        // (Java: `|| (HAIL_MARY_PASS == throwerAction)`), so this isolates the blood-lust check.
        game.thrower_action = Some(ffb_model::enums::PlayerAction::HailMaryPass);
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.suffering_blood_lust = true;
        game.acting_player.has_fed = true;
        let mut step = StepInitPassing::new();
        step.goto_label_on_end = "end".into();
        let out = step.start(&mut game, &mut GameRng::new(0));
        // has_fed=true → does NOT goto label; falls through to NextStep
        assert!(matches!(out.action, StepAction::NextStep));
    }

    #[test]
    fn pass_action_sets_has_passed_and_pass_used() {
        // Java: PASS branch requires thrower==actingPlayer AND a valid passing distance
        // (mechanic.findPassingDistance != null). Set up a 1-square Quick Pass.
        let mut game = make_game();
        use ffb_model::types::FieldCoordinate;
        add_player(&mut game, "p1", FieldCoordinate::new(1, 7));
        game.acting_player.player_id = Some("p1".into());
        game.thrower_id = Some("p1".into());
        game.thrower_action = Some(ffb_model::enums::PlayerAction::Pass);
        game.pass_coordinate = Some(FieldCoordinate::new(2, 7));
        let mut step = StepInitPassing::new();
        step.goto_label_on_end = "end".into();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
        assert!(game.acting_player.has_passed);
        assert!(game.turn_data().pass_used);
        assert!(!game.concession_possible);
    }

    #[test]
    fn hand_over_action_sets_hand_over_used() {
        // Java: HAND_OVER branch requires thrower==actingPlayer AND catcher != null.
        let mut game = make_game();
        use ffb_model::types::FieldCoordinate;
        add_player(&mut game, "p1", FieldCoordinate::new(1, 7));
        add_player(&mut game, "catcher1", FieldCoordinate::new(2, 7));
        game.acting_player.player_id = Some("p1".into());
        game.thrower_id = Some("p1".into());
        game.thrower_action = Some(ffb_model::enums::PlayerAction::HandOver);
        let mut step = StepInitPassing::new();
        step.goto_label_on_end = "end".into();
        step.catcher_id = Some("catcher1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
        assert!(game.acting_player.has_passed);
        assert!(game.turn_data().hand_over_used);
    }

    #[test]
    fn hand_over_without_catcher_does_not_advance() {
        // Java: `(PlayerAction.HAND_OVER == throwerAction) && (thrower == actingPlayer.getPlayer())
        //         && (catcher != null)` — with no resolvable catcher, the HAND_OVER branch must NOT
        // fire, and since no other branch matches HAND_OVER either, `executeStep()` returns without
        // calling `setNextAction` and hand_over_used must remain false. Java's step then WAITS; both
        // reference agents resolve that by injecting `ClientCommandEndTurn` (ParityRunner has no
        // INIT_PASSING case), which handleCommand turns into END_TURN + GOTO_LABEL(gotoLabelOnEnd).
        // The step reproduces that terminal result directly, so the assertion here is
        // GotoLabel + EndTurn rather than Continue — the *engine-visible* effects Java has
        // (no hand_over_used, no has_passed) are unchanged, which is what this test guards.
        let mut game = make_game();
        use ffb_model::types::FieldCoordinate;
        add_player(&mut game, "p1", FieldCoordinate::new(1, 7));
        game.acting_player.player_id = Some("p1".into());
        game.thrower_id = Some("p1".into());
        game.thrower_action = Some(ffb_model::enums::PlayerAction::HandOver);
        let mut step = StepInitPassing::new();
        step.goto_label_on_end = "end".into();
        step.catcher_id = None; // no catcher resolved
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::GotoLabel), "expected GotoLabel, got {:?}", out.action);
        assert_eq!(out.goto_label.as_deref(), Some("end"));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
        assert!(!game.turn_data().hand_over_used);
        assert!(!game.acting_player.has_passed);
    }

    #[test]
    fn dump_off_defender_becomes_thrower() {
        let mut game = make_game();
        game.turn_mode = TurnMode::DumpOff;
        game.defender_id = Some("d1".into());
        game.defender_action = Some(ffb_model::enums::PlayerAction::DumpOff);
        let mut step = StepInitPassing::new();
        step.goto_label_on_end = "end".into();
        use ffb_model::types::FieldCoordinate;
        let action = crate::action::Action::Pass { coord: FieldCoordinate::new(8, 7) };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        // Thrower should be the defender, not the acting player
        assert_eq!(game.thrower_id.as_deref(), Some("d1"));
        assert_eq!(game.thrower_action, Some(ffb_model::enums::PlayerAction::DumpOff));
    }

    #[test]
    fn pass_action_non_dump_off_uses_acting_player() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(ffb_model::enums::PlayerAction::Pass);
        game.defender_id = None;
        let mut step = StepInitPassing::new();
        step.goto_label_on_end = "end".into();
        use ffb_model::types::FieldCoordinate;
        let action = crate::action::Action::Pass { coord: FieldCoordinate::new(10, 7) };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(game.thrower_id.as_deref(), Some("p1"));
    }
}
