/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.pass.StepInitPassing`.
///
/// Initialization step of the pass sequence.  Waits for a CLIENT_PASS,
/// CLIENT_HAND_OVER, or CLIENT_ACTING_PLAYER / CLIENT_END_TURN command and sets
/// up the thrower, catcher and pass coordinate on the game object accordingly.
///
/// Init parameters (mandatory): GOTO_LABEL_ON_END.
/// Optional init: TARGET_COORDINATE, CATCHER_ID.
/// Sets: CATCHER_ID, END_TURN, END_PLAYER_ACTION (all published for downstream).
///
/// Java: `@RulesCollection(BB2020, BB2025)`, extends `AbstractStep`.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::enums::TurnMode;
use ffb_model::prompts::agent_prompt::AgentPrompt;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepInitPassing` (mixed/pass, BB2020 + BB2025).
#[derive(Debug, Default)]
pub struct StepInitPassing {
    /// Java: `fGotoLabelOnEnd`
    pub goto_label_on_end: String,
    /// Java: `fCatcherId`
    pub catcher_id: Option<String>,
    /// Java: `fEndTurn`
    pub end_turn: bool,
    /// Java: `fEndPlayerAction`
    pub end_player_action: bool,
    /// Rust bridging: the pass target the agent chose at activation time, threaded through the
    /// Pass generator's TARGET_COORDINATE init param. Java instead receives a CLIENT_PASS command
    /// carrying this coordinate; consuming the param here plays that command's role.
    pub target_coordinate: Option<ffb_model::types::FieldCoordinate>,
}

impl StepInitPassing {
    pub fn new() -> Self { Self::default() }

    fn execute_step(&mut self, game: &mut Game) -> StepOutcome {
        // Rust bridging: a TARGET_COORDINATE init param stands in for Java's CLIENT_PASS
        // command — apply the same state setup its handler performs. Java sets the thrower
        // UNCONDITIONALLY (dump-off → defender, else acting player); only overwriting when
        // unset left a stale thrower from an earlier pass in the same turn, which made
        // StepEndPassing treat the current activation as a foreign dump-off and end the game.
        if let Some(coord) = self.target_coordinate.take() {
            game.pass_coordinate = Some(coord);
            self.catcher_id = game.field_model.player_at(coord).cloned();
            let is_dump_off = game.defender_id.is_some()
                && game.defender_action == Some(ffb_model::enums::PlayerAction::DumpOff);
            if is_dump_off {
                game.thrower_id = game.defender_id.clone();
                game.thrower_action = game.defender_action;
            } else {
                game.thrower_id = game.acting_player.player_id.clone();
                game.thrower_action = game.acting_player.player_action;
            }
        }
        // Java's StepInitPassing simply PARKS here waiting for a client command. The bomb
        // re-throw window (TurnMode BombHome/BombAway) is the one place nothing ever declared
        // the action -- the engine made the bomb's catcher the acting player -- so no
        // activation prompt is pending and the step would spin forever. Surface the wait so
        // the agent can answer, mirroring ParityRunner's INIT_PASSING case.
        if game.thrower_id.is_none() || game.thrower_action.is_none() {
            if let Some(pid) = game.acting_player.player_id.clone() {
                // ParityRunner's INIT_PASSING case answers ANY thrower==null park with
                // sendPassAction — its gate is the unset thrower, not the turn mode. The
                // bomb re-throw window (Bomb* modes) was the first such park; the second
                // All You Can Eat bomb is another (REGULAR mode, acting action THROW_BOMB,
                // fresh Pass sequence pushed by StepEndBomb with no folded target —
                // halfling seed 5 i=14 parked promptless and the game ended early).
                let acting_is_bomb = game.acting_player.player_action
                    .map(|a| a.is_bomb())
                    .unwrap_or(false);
                if acting_is_bomb
                    || matches!(
                        game.turn_mode,
                        TurnMode::BombHome | TurnMode::BombAway
                            | TurnMode::BombHomeBlitz | TurnMode::BombAwayBlitz
                    )
                {
                    return StepOutcome::cont()
                        .with_prompt(AgentPrompt::BombRethrow { player_id: pid });
                }
            }
            return StepOutcome::cont();
        }

        // Java: catcher publish
        let mut outcome = StepOutcome::next();
        if let Some(ref cid) = self.catcher_id {
            outcome = outcome.publish(StepParameter::CatcherId(Some(cid.clone())));
        }

        if self.end_turn {
            outcome = outcome.publish(StepParameter::EndTurn(true));
            return StepOutcome::goto(&self.goto_label_on_end.clone())
                .with_events(outcome.events)
                .with_published(outcome.published);
        }

        if self.end_player_action {
            outcome = outcome.publish(StepParameter::EndPlayerAction(true));
            return StepOutcome::goto(&self.goto_label_on_end.clone())
                .with_events(outcome.events)
                .with_published(outcome.published);
        }

        // Java: (game.getThrower() == actingPlayer.getPlayer()) && actingPlayer.isSufferingBloodLust() && !actingPlayer.hasFed()
        let thrower_is_acting = game.thrower_id.is_some() && game.thrower_id == game.acting_player.player_id;
        if thrower_is_acting
            && game.acting_player.suffering_blood_lust
            && !game.acting_player.has_fed
        {
            return StepOutcome::goto(&self.goto_label_on_end.clone())
                .with_events(outcome.events)
                .with_published(outcome.published);
        }

        // Java mixed StepInitPassing: it only advances to the pass roll when the throw is IN RANGE
        // (findPassingDistance != null) and thrower==actingPlayer; an out-of-range target leaves the
        // step WAITING, and the parity agent then ends the turn (turnover) with the ball unmoved.
        // The Rust translation was missing this gate and always advanced, so StepPass auto-fumbled the
        // out-of-range throw AND scattered the ball (an extra d8) where Java rolls nothing (seed 23
        // i=264: away_08 at (8,11) throwing to out-of-range (18,2) — Java keeps the ball at (8,11),
        // Rust bounced it to (8,10)). Use the SAME range function StepPass uses so the two steps agree.
        // client-only: full range ruler logic — RangeRuler is client-side display.
        use ffb_model::enums::PlayerAction;
        let thrower_action = game.thrower_action;
        let thrower_coordinate = game.thrower_id.as_deref()
            .and_then(|id| game.field_model.player_coordinate(id));
        let passing_distance_valid = match (thrower_coordinate, game.pass_coordinate) {
            (Some(tc), Some(pc)) => match ffb_model::util::passing::passing_distance(tc, pc) {
                // Java PassMechanic.findPassingDistance nulls out a Long Pass / Long Bomb in a
                // Blizzard (a Throw Team-Mate is gated the same way, but that is a separate step).
                // The pure table lookup here was missing that weather gate, so an out-of-range
                // Long Pass thrown in a Blizzard was treated as valid and executed, whereas the
                // stock engine refuses it and the turn ends with the ball unmoved (human seed 16
                // i=74: an Ogre ball-carrier's Long Pass in a Blizzard).
                Some(d) => !(game.field_model.weather == ffb_model::enums::Weather::Blizzard
                    && matches!(d, ffb_model::enums::PassingDistance::LongPass
                        | ffb_model::enums::PassingDistance::LongBomb)),
                None => false,
            },
            _ => false,
        };
        let catcher_exists = self.catcher_id.as_deref()
            .map(|id| game.player(id).is_some())
            .unwrap_or(false);

        // Java: HAND_OVER — thrower==actingPlayer && catcher != null (no range check).
        if thrower_action == Some(PlayerAction::HandOver) && thrower_is_acting && catcher_exists {
            game.acting_player.has_passed = true;
            game.concession_possible = false;
            game.turn_data_mut().hand_over_used = true;
            game.turn_data_mut().turn_started = true;
            return outcome;
        }
        // Java: THROW_BOMB (thrower==actingPlayer) / HAIL_MARY_BOMB — range-gated.
        if (passing_distance_valid && thrower_is_acting && thrower_action == Some(PlayerAction::ThrowBomb))
            || thrower_action == Some(PlayerAction::HailMaryBomb)
        {
            if thrower_is_acting {
                game.acting_player.has_passed = true;
            }
            game.turn_data_mut().turn_started = true;
            game.concession_possible = false;
            return outcome;
        }
        // Java: PASS (thrower==actingPlayer) / HAIL_MARY_PASS — range-gated.
        if (passing_distance_valid && thrower_is_acting && thrower_action == Some(PlayerAction::Pass))
            || thrower_action == Some(PlayerAction::HailMaryPass)
        {
            game.acting_player.has_passed = true;
            game.turn_data_mut().turn_started = true;
            game.concession_possible = false;
            game.turn_data_mut().pass_used = true;
            return outcome;
        }
        // Java: (THROW_BOMB || DUMP_OFF) / HAIL_MARY_BOMB — no thrower==actingPlayer requirement.
        if (passing_distance_valid
            && matches!(thrower_action, Some(PlayerAction::ThrowBomb) | Some(PlayerAction::DumpOff)))
            || thrower_action == Some(PlayerAction::HailMaryBomb)
        {
            if thrower_is_acting {
                game.acting_player.has_passed = true;
            }
            return outcome;
        }

        // No branch matched (the throw is out of range). Java's StepInitPassing leaves the step
        // WAITING and the parity agent (ParityRunner INIT_PASSING handler) unconditionally injects
        // ClientCommandEndTurn — a turnover with the ball unmoved and no roll. Both reference agents
        // always end the turn here, so end it directly (goto the end label + publish EndTurn) rather
        // than waiting for a prompt the headless runner cannot surface. This produces the identical
        // observable result: turnover, ball stays at the thrower, zero dice (seed 23 i=264).
        outcome = outcome.publish(StepParameter::EndTurn(true));
        StepOutcome::goto(&self.goto_label_on_end.clone())
            .with_events(outcome.events)
            .with_published(outcome.published)
    }
}

// Extension to add published parameters to a StepOutcome (mirrors the helper in step_trap_door)
trait WithPublished {
    fn with_published(self, params: Vec<StepParameter>) -> Self;
}

impl WithPublished for StepOutcome {
    fn with_published(mut self, params: Vec<StepParameter>) -> Self {
        self.published.extend(params);
        self
    }
}

impl Step for StepInitPassing {
    fn id(&self) -> StepId { StepId::InitPassing }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::Pass { coord } => {
                // Java: CLIENT_PASS
                game.pass_coordinate = Some(*coord);
                // Java: catcher = fieldModel.getPlayer(passCoordinate)
                self.catcher_id = game.field_model.player_at(*coord).cloned();
                // Java: setThrowerId / setThrowerAction (use actingPlayer defaults)
                if game.thrower_id.is_none() {
                    game.thrower_id = game.acting_player.player_id.clone();
                }
                if game.thrower_action.is_none() {
                    game.thrower_action = game.acting_player.player_action;
                }
                self.execute_step(game)
            }
            Action::EndTurn => {
                // Java: CLIENT_END_TURN → fEndTurn = true
                self.end_turn = true;
                self.execute_step(game)
            }
            Action::ActivatePlayer {player_id, .. } if player_id.is_empty() => {
                // Java: CLIENT_ACTING_PLAYER with null playerId → fEndPlayerAction = true
                self.end_player_action = true;
                self.execute_step(game)
            }
            _ => StepOutcome::cont(),
        }
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(v)    => { self.goto_label_on_end = v.clone(); true }
            StepParameter::CatcherId(v)          => { self.catcher_id = v.clone(); true }
            StepParameter::EndTurn(v)            => { self.end_turn = *v; true }
            StepParameter::EndPlayerAction(v)    => { self.end_player_action = *v; true }
            StepParameter::TargetCoordinate(v)   => { self.target_coordinate = Some(*v); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{test_team, StepAction};
    use ffb_model::enums::{Rules, PlayerAction};
    use ffb_model::model::game::Game;
    use ffb_model::util::rng::GameRng;
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    /// Add a player to the home roster and place it on the field, so `game.player(id)` and
    /// `field_model.player_coordinate(id)` both resolve (needed for the range/catcher checks).
    fn add_field_player(game: &mut Game, id: &str, x: i32, y: i32) {
        let mut p = ffb_model::model::player::Player::default();
        p.id = id.into();
        game.team_home.players.push(p);
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(x, y));
    }

    #[test]
    fn no_thrower_waits_for_command() {
        let mut step = StepInitPassing::new();
        step.goto_label_on_end = "end".into();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn end_turn_set_goes_to_label() {
        let mut step = StepInitPassing::new();
        step.goto_label_on_end = "end".into();
        let mut game = make_game();
        game.thrower_id = Some("p1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        let mut rng = GameRng::new(0);
        let out = step.handle_command(&Action::EndTurn, &mut game, &mut rng);
        assert_eq!(out.action, StepAction::GotoLabel);
        let has_end_turn = out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true)));
        assert!(has_end_turn);
    }

    #[test]
    fn end_player_action_via_activate_empty_goes_to_label() {
        let mut step = StepInitPassing::new();
        step.goto_label_on_end = "end".into();
        let mut game = make_game();
        game.thrower_id = Some("p1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        let mut rng = GameRng::new(0);
        // Java: CLIENT_ACTING_PLAYER with null playerId → endPlayerAction
        let out = step.handle_command(
            &Action::ActivatePlayer {player_id: "".into(),
                player_action: crate::action::PlayerActionChoice::Move,
                block_defender_id: None },
            &mut game,
            &mut rng,
        );
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn with_thrower_and_no_flags_returns_next() {
        let mut step = StepInitPassing::new();
        step.goto_label_on_end = "end".into();
        let mut game = make_game();
        add_field_player(&mut game, "p1", 5, 5);
        game.thrower_id = Some("p1".into());
        game.acting_player.player_id = Some("p1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        game.pass_coordinate = Some(FieldCoordinate::new(6, 6)); // in range (QuickPass)
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    /// Regression: the mixed StepInitPassing previously set `has_moved` (a proxy,
    /// wrong field) instead of `has_passed` (Java: `actingPlayer.setHasPassed(true)`),
    /// and never set `turn_data.pass_used` / `concession_possible`. Mirrors the
    /// already-correct bb2016 sibling translation.
    #[test]
    fn pass_action_sets_has_passed_and_pass_used_and_clears_concession() {
        let mut step = StepInitPassing::new();
        step.goto_label_on_end = "end".into();
        let mut game = make_game();
        add_field_player(&mut game, "p1", 5, 5);
        game.thrower_id = Some("p1".into());
        game.acting_player.player_id = Some("p1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        game.pass_coordinate = Some(FieldCoordinate::new(6, 6)); // in range (QuickPass)
        game.concession_possible = true;
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
        assert!(game.acting_player.has_passed, "has_passed must be set (Java: actingPlayer.setHasPassed(true))");
        assert!(!game.acting_player.has_moved, "has_moved must NOT be touched by this step");
        assert!(game.turn_data().pass_used, "turn_data.pass_used must be set for PASS action");
        assert!(game.turn_data().turn_started);
        assert!(!game.concession_possible);
    }

    #[test]
    fn hand_over_action_sets_hand_over_used() {
        let mut step = StepInitPassing::new();
        step.goto_label_on_end = "end".into();
        let mut game = make_game();
        add_field_player(&mut game, "p1", 5, 5);
        add_field_player(&mut game, "c1", 6, 6);
        step.catcher_id = Some("c1".into()); // hand-over requires a catcher (no range check)
        game.thrower_id = Some("p1".into());
        game.acting_player.player_id = Some("p1".into());
        game.thrower_action = Some(PlayerAction::HandOver);
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(game.acting_player.has_passed);
        assert!(game.turn_data().hand_over_used);
    }

    /// Regression: Java guards the blood-lust check with
    /// `game.getThrower() == actingPlayer.getPlayer()`. The mixed translation
    /// previously dropped this guard and applied the check unconditionally
    /// whenever `acting_player.suffering_blood_lust` was set, even if the acting
    /// player was not the thrower (e.g. during a Dump-Off where the defender is
    /// the thrower). It also hard-coded `has_fed = false` instead of reading the
    /// real `acting_player.has_fed` field.
    #[test]
    fn blood_lust_check_ignored_when_thrower_is_not_acting_player() {
        let mut step = StepInitPassing::new();
        step.goto_label_on_end = "end".into();
        let mut game = make_game();
        // Thrower is the defender (dump-off scenario); acting player is someone else.
        add_field_player(&mut game, "defender", 5, 5);
        game.thrower_id = Some("defender".into());
        game.thrower_action = Some(PlayerAction::DumpOff);
        game.pass_coordinate = Some(FieldCoordinate::new(6, 6)); // in range (DumpOff advances)
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.suffering_blood_lust = true;
        game.acting_player.has_fed = false;
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        // Must NOT goto label, since thrower != acting player.
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn blood_lust_thrower_is_acting_and_not_fed_goes_to_label() {
        let mut step = StepInitPassing::new();
        step.goto_label_on_end = "end".into();
        let mut game = make_game();
        game.thrower_id = Some("p1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.suffering_blood_lust = true;
        game.acting_player.has_fed = false;
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn out_of_range_pass_ends_the_turn_without_advancing() {
        // Java's mixed StepInitPassing only advances when findPassingDistance != null; an out-of-range
        // target leaves it waiting and the parity agent ends the turn (turnover, ball unmoved, 0 dice).
        // Rust ends the turn directly here (goto end label + EndTurn) rather than auto-fumbling and
        // bouncing the ball in StepPass (seed 23 i=264).
        let mut step = StepInitPassing::new();
        step.goto_label_on_end = "end".into();
        let mut game = make_game();
        add_field_player(&mut game, "p1", 8, 11);
        game.thrower_id = Some("p1".into());
        game.acting_player.player_id = Some("p1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        game.pass_coordinate = Some(FieldCoordinate::new(18, 2)); // dx=10 dy=9 → out of range
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::GotoLabel, "out-of-range pass gotos the end label");
        assert_eq!(out.goto_label.as_deref(), Some("end"));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))),
            "out-of-range pass ends the turn (turnover)");
        assert!(!game.turn_data().pass_used, "no pass was actually thrown");
    }

    #[test]
    fn long_pass_in_a_blizzard_is_out_of_range_and_ends_the_turn() {
        // Java PassMechanic.findPassingDistance nulls out a Long Pass / Long Bomb in a Blizzard, so
        // the stock engine refuses the throw and the turn ends with the ball unmoved. The pure table
        // lookup treats it as a valid Long Pass; the weather gate must override that (human seed 16
        // i=74: an Ogre at (12,6) throwing a Long Pass to (5,10) — dx=7, dy=4 = LongPass, in a Blizzard).
        use ffb_model::enums::Weather;
        let mut step = StepInitPassing::new();
        step.goto_label_on_end = "end".into();
        let mut game = make_game();
        game.field_model.weather = Weather::Blizzard;
        add_field_player(&mut game, "p1", 12, 6);
        game.thrower_id = Some("p1".into());
        game.acting_player.player_id = Some("p1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        game.pass_coordinate = Some(FieldCoordinate::new(5, 10)); // dx=7 dy=4 → LongPass, blizzard → out of range
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::GotoLabel, "a Long Pass in a Blizzard gotos the end label");
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))),
            "a Long Pass in a Blizzard is out of range → turn ends (turnover)");
        assert!(!game.turn_data().pass_used, "no pass was actually thrown");
    }

    #[test]
    fn short_pass_in_a_blizzard_is_still_in_range() {
        // Only Long Pass / Long Bomb are gated by a Blizzard; a Quick/Short pass is unaffected.
        use ffb_model::enums::Weather;
        let mut step = StepInitPassing::new();
        step.goto_label_on_end = "end".into();
        let mut game = make_game();
        game.field_model.weather = Weather::Blizzard;
        add_field_player(&mut game, "p1", 5, 5);
        game.thrower_id = Some("p1".into());
        game.acting_player.player_id = Some("p1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        game.pass_coordinate = Some(FieldCoordinate::new(6, 6)); // QuickPass — allowed in a Blizzard
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep, "a Quick/Short pass is unaffected by a Blizzard");
    }

    #[test]
    fn blood_lust_thrower_is_acting_and_fed_continues() {
        let mut step = StepInitPassing::new();
        step.goto_label_on_end = "end".into();
        let mut game = make_game();
        add_field_player(&mut game, "p1", 5, 5);
        game.thrower_id = Some("p1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        game.pass_coordinate = Some(FieldCoordinate::new(6, 6)); // in range (QuickPass)
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.suffering_blood_lust = true;
        game.acting_player.has_fed = true;
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }
}
