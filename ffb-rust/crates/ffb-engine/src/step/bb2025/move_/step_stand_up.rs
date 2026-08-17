use ffb_model::enums::{PS_PRONE, PlayerAction, PlayerState};
use ffb_model::enums::ReRollSource;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use ffb_model::events::GameEvent;
use ffb_model::util::util_player::UtilPlayer;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::abstract_step_with_re_roll::ReRollState;
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use ffb_model::model::target_selection_state::TargetSelectionState;

/// Minimum MA to stand up for free. Java: Constant.MINIMUM_MOVE_TO_STAND_UP = 3.
const MINIMUM_MOVE_TO_STAND_UP: i32 = 3;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.move.StepStandUp.
///
/// Stand-up sequence for a prone player.
///
/// If MA ≥ MINIMUM_MOVE_TO_STAND_UP (3) or player has `canStandUpForFree`: free stand-up, NEXT_STEP.
/// Otherwise: roll d6 ≥ 4 (with optional +modifier from allowStandUpAssists).
///   Success → NEXT_STEP; Failure → publish END_PLAYER_ACTION + GOTO failure label.
///
/// The outer guard is `actingPlayer.isStandingUp() && !actingPlayer.hasMoved()`, or
/// `reRolledAction == STAND_UP`.  If not standing up → NEXT_STEP immediately.
///
/// Re-roll: TRR offered via ReRollOffer prompt on first failure (no skill re-roll for StandUp in BB2025).
///
/// Init params: GOTO_LABEL_ON_FAILURE (mandatory).
///
/// setTurnStarted(true) and setConcessionPossible(false) are wired; isPinned() guard is wired.
/// DiceInterpreter::is_stand_up_successful is wired; allowStandUpAssists → findStandUpAssists wired.
///
/// handleFailedStandUp: per-action turn data flags wired (BB2025: separate pass/ttm, adds punt).
/// TargetSelectionState.failed() wired in Blitz/BlitzMove/KickEmBlitz branch.
pub struct StepStandUp {
    /// Java: fGotoLabelOnFailure
    pub goto_label_on_failure: String,
    /// Java: AbstractStepWithReRoll fields
    pub re_roll_state: ReRollState,
    /// Persisted roll for re-roll path
    roll: i32,
}

impl StepStandUp {
    pub fn new(goto_label_on_failure: String) -> Self {
        Self { goto_label_on_failure, re_roll_state: ReRollState::new(), roll: 0 }
    }
}

impl Step for StepStandUp {
    fn id(&self) -> StepId { StepId::StandUp }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if let Action::UseReRoll { use_reroll: false } = action {
            self.re_roll_state.re_roll_source = None;
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(v) => { self.goto_label_on_failure = v.clone(); true }
            _ => false,
        }
    }
}

impl StepStandUp {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: game.getTurnData().setTurnStarted(true)
        game.turn_data_mut().turn_started = true;

        let already_rerolled = self.re_roll_state.re_rolled_action
            .as_ref().map(|a| a.name == "STAND_UP").unwrap_or(false);

        // Java outer guard: (actingPlayer.isStandingUp() && !actingPlayer.hasMoved()) || STAND_UP == reRolledAction
        if !already_rerolled {
            if !game.acting_player.standing_up {
                return StepOutcome::next();
            }
            if game.acting_player.has_moved {
                return StepOutcome::next();
            }
        }

        // Java: game.setConcessionPossible(false)
        game.concession_possible = false;

        // Java: rollStandUp = player.getMovementWithModifiers() < 3 && !canStandUpForFree
        let roll_stand_up_needed = game.acting_player.player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| {
                p.movement_with_modifiers() < MINIMUM_MOVE_TO_STAND_UP
                    && !p.has_skill_property(NamedProperties::CAN_STAND_UP_FOR_FREE)
            })
            .unwrap_or(true);

        if !roll_stand_up_needed {
            // MA >= 3 or has canStandUpForFree — stand up for free. Set the base PRONE→STANDING (the
            // deleted engine.rs did this in its StandUp dispatch; Java reaches STANDING via the
            // standing-up flow). current_move already holds the STAND_UP_COST from the activation.
            if let Some(pid) = game.acting_player.player_id.clone() {
                if let Some(ps) = game.field_model.player_state(&pid) {
                    game.field_model.set_player_state(&pid, ps.change_base(ffb_model::enums::PS_STANDING));
                }
            }
            // Java's free-stand-up branch (`StepStandUp.java:136-138`) sets ONLY `setHasMoved(true)`
            // — `standingUp` deliberately stays TRUE. Only the ROLLED stand-up's success path
            // (`:114-115`) clears it. Rust cleared it here too, and that flag is read later:
            // `FoulAppearanceBehaviour.handleFailure` reverts the attacker to PRONE when
            // `isStandingUp()`, so a blitzer who stood up for free and then failed Foul Appearance
            // must go back down. With the flag cleared the revert was skipped and the blitzer stayed
            // standing — nurgle bb2020 seed 2 i=33, `a01` Prone in Java, Standing in Rust, same dice.
            // Re-entry is still prevented by the `has_moved` half of the outer guard, as in Java.
            game.acting_player.has_moved = true;
            return StepOutcome::next();
        }

        // Java: if (STAND_UP == reRolledAction) { if (source == null || !useReRoll) → fail }
        if already_rerolled {
            let pid = game.acting_player.player_id.as_deref().unwrap_or("").to_owned();
            let source_opt = self.re_roll_state.re_roll_source.clone();
            let consumed = source_opt
                .as_ref()
                .map(|s| use_reroll(game, s, &pid))
                .unwrap_or(false);
            if !consumed {
                // Java: `if (STAND_UP == getReRolledAction()) { if (source == null ||
                // !UtilServerReRoll.useReRoll(...)) rollStandUp = false; }` — clearing rollStandUp
                // BEFORE the roll routes to the trailing
                // `if (!rollStandUp) { setPlayerState(PRONE, inactive); publish END_PLAYER_ACTION;
                // GOTO failure; }` block, which does NOT touch the per-action used flags. This is the
                // path a DECLINED stand-up re-roll takes (the harness always declines team re-rolls),
                // so a declined re-roll must NOT consume the team's Blitz/Pass/HandOver/Foul.
                // wood_elf bb2016 seed 14: away_01's stand-up BLITZ re-roll is declined at i=34;
                // Java keeps blitzUsed FALSE (verified with a gated blitzUsedA print — it only flips
                // after away_02's own blitz reaches target selection at si=41), so away_02 can still
                // blitz at i=39 where Rust offered it only Move.
                // EDITION-SPECIFIC. bb2020/bb2025 `StepStandUp` call `handleFailedStandUp(...)` right
                // here alongside `rollStandUp = false`; **bb2016 does NOT** — its pre-roll site only
                // clears `rollStandUp`, so control falls to the trailing
                // `if (!rollStandUp) { setPlayerState(PRONE, inactive); publish END_PLAYER_ACTION;
                // GOTO failure; }` block and the per-action used-flag switch (which bb2016 keeps
                // INLINE in the post-roll failure branch) never runs.
                if game.rules == ffb_model::enums::Rules::Bb2016 {
                    return self.end_stand_up_without_flags(game);
                }
                return self.fail_stand_up(game);
            }
            // Roll was reset to 0 when the re-roll offer was issued; a fresh d6 is rolled below
        }

        // Java: int roll = rollSkill()
        if self.roll == 0 {
            self.roll = rng.d6();
        }

        // Java: if (player.hasSkillProperty(allowStandUpAssists)) modifier = findStandUpAssists(game, player)
        let modifier = game.acting_player.player_id.as_deref()
            .and_then(|id| game.player(id))
            .filter(|p| p.has_skill_property(NamedProperties::ALLOW_STAND_UP_ASSISTS))
            .map(|_| {
                let id = game.acting_player.player_id.as_deref().unwrap_or("");
                UtilPlayer::find_stand_up_assists(game, id)
            })
            .unwrap_or(0);
        let successful = DiceInterpreter::is_stand_up_successful(self.roll, modifier);

        // Java line 110-112: boolean reRolled = ...; addReport(new ReportStandUpRoll(...))
        {
            use ffb_model::report::report_stand_up_roll::ReportStandUpRoll;
            let re_rolled = self.re_roll_state.re_rolled_action.as_ref()
                .map(|a| a.name == "STAND_UP").unwrap_or(false)
                && self.re_roll_state.re_roll_source.is_some();
            game.report_list.add(ReportStandUpRoll::new(
                game.acting_player.player_id.clone(),
                successful,
                self.roll,
                modifier,
                re_rolled,
            ));
        }

        // Coverage: `GameEvent::StandUpRoll` had no construction site in the engine, so a roll that
        // fires whenever a sub-MA-3 player stands up read as 0 across 8,700 games. Report-only.
        let roll_event = GameEvent::StandUpRoll {
            player_id: game.acting_player.player_id.clone().unwrap_or_default(),
            target: i32::max(2, 4 - modifier),
            roll: self.roll,
            success: successful,
        };

        if successful {
            game.acting_player.has_moved = true;
            game.acting_player.standing_up = false;
            // Mirror the free stand-up path (above): a successful stand-up leaves the
            // player STANDING. Java reaches STANDING via the standing-up flow and only
            // reverts to PRONE on FAILURE (fail_stand_up / Java line 131 changeBase(PRONE));
            // it never re-sets STANDING here because the player is already standing by
            // this point. The Rust roll path had no such earlier set, so without this a
            // low-MA (<3) player who SUCCEEDS the stand-up roll stayed PRONE in the field
            // model (undead seed 4 i=160: away_02 stood up + rushed but stayed Prone,
            // diverging from Java's Standing at the next activation's state hash).
            if let Some(pid) = game.acting_player.player_id.clone() {
                if let Some(ps) = game.field_model.player_state(&pid) {
                    game.field_model.set_player_state(&pid, ps.change_base(ffb_model::enums::PS_STANDING));
                }
            }
            // Java: only checked in the `successful` branch, and — unlike the failure
            // path — does NOT publish END_PLAYER_ACTION when redirecting to the failure
            // label (`getResult().setNextAction(GOTO_LABEL, ...)` with no publishParameter call).
            let is_pinned = game.acting_player.player_id.as_deref()
                .and_then(|id| game.field_model.player_state(id))
                .map(|s| s.is_pinned())
                .unwrap_or(false);
            if is_pinned {
                let label = self.goto_label_on_failure.clone();
                StepOutcome::goto(&label).with_event(roll_event)
            } else {
                StepOutcome::next().with_event(roll_event)
            }
        } else {
            // Java: if (reRolledAction == STAND_UP || !askForReRollIfAvailable(...)) → handleFailedStandUp
            if already_rerolled {
                return self.fail_stand_up(game).with_event(roll_event);
            }
            let minimum_roll = i32::max(2, 4 - modifier);
            if let Some(prompt) = ask_for_reroll_if_available(game, "STAND_UP", minimum_roll, false) {
                use ffb_model::model::re_rolled_action::ReRolledAction;
                self.re_roll_state.re_rolled_action = Some(ReRolledAction::new("STAND_UP"));
                self.re_roll_state.re_roll_source = Some(ReRollSource::new("TRR"));
                self.roll = 0; // reset so the re-roll gets a fresh d6
                return StepOutcome::cont().with_prompt(prompt).with_event(roll_event);
            }
            self.fail_stand_up(game).with_event(roll_event)
        }
    }

    /// Java's trailing `if (!rollStandUp) { ... }` block ONLY: drop to PRONE + inactive, publish
    /// END_PLAYER_ACTION, goto the failure label. Deliberately does NOT run `handle_failed_stand_up`
    /// (the per-action used-flag switch), which in Java lives inside the ROLL's failure branch.
    fn end_stand_up_without_flags(&self, game: &mut Game) -> StepOutcome {
        if let Some(pid) = game.acting_player.player_id.clone() {
            let current = game.field_model.player_state(&pid).unwrap_or_else(|| PlayerState::new(PS_PRONE));
            let new_state = current.change_base(PS_PRONE).change_active(false);
            game.field_model.set_player_state(&pid, new_state);
        }
        let label = self.goto_label_on_failure.clone();
        StepOutcome::goto(&label)
            .publish(StepParameter::EndPlayerAction(true))
    }

    fn fail_stand_up(&self, game: &mut Game) -> StepOutcome {
        // Java: setPlayerState(playerState.changeBase(PRONE).changeActive(false)) — this
        // mutates the player's *existing* PlayerState (preserving flags such as confused,
        // rooted, hypnotized, usedPro, …), not a freshly constructed PRONE state.
        if let Some(pid) = game.acting_player.player_id.clone() {
            let current = game.field_model.player_state(&pid).unwrap_or_else(|| PlayerState::new(PS_PRONE));
            let new_state = current.change_base(PS_PRONE).change_active(false);
            game.field_model.set_player_state(&pid, new_state);
        }
        self.handle_failed_stand_up(game);
        let label = self.goto_label_on_failure.clone();
        StepOutcome::goto(&label)
            .publish(StepParameter::EndPlayerAction(true))
    }

    fn handle_failed_stand_up(&self, game: &mut Game) {
        let player_action = game.acting_player.player_action;
        match player_action {
            Some(PlayerAction::Blitz)
            | Some(PlayerAction::BlitzMove)
            | Some(PlayerAction::KickEmBlitz) => {
                game.turn_data_mut().blitz_used = true;
                // Java: if (getFieldModel().getTargetSelectionState() != null) → .failed()
                if let Some(ref mut ts) = game.field_model.target_selection_state {
                    ts.failed();
                }
            }
            Some(PlayerAction::KickTeamMate)
            | Some(PlayerAction::KickTeamMateMove) => {
                game.turn_data_mut().ktm_used = true;
            }
            Some(PlayerAction::Pass)
            | Some(PlayerAction::PassMove) => {
                game.turn_data_mut().pass_used = true;
            }
            Some(PlayerAction::ThrowTeamMate)
            | Some(PlayerAction::ThrowTeamMateMove) => {
                game.turn_data_mut().ttm_used = true;
            }
            Some(PlayerAction::HandOver)
            | Some(PlayerAction::HandOverMove) => {
                game.turn_data_mut().hand_over_used = true;
            }
            Some(PlayerAction::Foul)
            | Some(PlayerAction::FoulMove) => {
                let pid = game.acting_player.player_id.clone();
                let allows_extra_foul = pid.as_deref()
                    .and_then(|id| game.player(id))
                    .map(|p| p.has_skill_property(NamedProperties::ALLOWS_ADDITIONAL_FOUL))
                    .unwrap_or(false);
                if !allows_extra_foul {
                    game.turn_data_mut().foul_used = true;
                }
            }
            Some(PlayerAction::SecureTheBall) => {
                game.turn_data_mut().secure_the_ball_used = true;
            }
            Some(PlayerAction::Punt)
            | Some(PlayerAction::PuntMove) => {
                game.turn_data_mut().punt_used = true;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{Rules, TurnMode};
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    /// Java bb2016 `StepStandUp` has TWO `reRolledAction == STAND_UP` sites and they differ:
    ///
    /// - **PRE-roll**: `if (STAND_UP == getReRolledAction()) { if (source == null ||
    ///   !useReRoll(...)) rollStandUp = false; }` — a DECLINED re-roll clears `rollStandUp` BEFORE the
    ///   roll, so control reaches only the trailing
    ///   `if (!rollStandUp) { setPlayerState(PRONE, inactive); publish END_PLAYER_ACTION; GOTO failure }`
    ///   block. The per-action used-flag switch is NOT run.
    /// - **POST-roll**: `else { if ((getReRolledAction() == STAND_UP) || !askForReRollIfAvailable(...))
    ///   { rollStandUp = false; switch (playerAction) { … setBlitzUsed(true) … } } }` — having already
    ///   re-rolled and failed AGAIN, the flags ARE set.
    ///
    /// **and the pre-roll site is EDITION-SPECIFIC**: bb2020/bb2025 `StepStandUp` call
    /// `handleFailedStandUp(...)` there alongside `rollStandUp = false`, while **bb2016 does not**.
    ///
    /// The parity harness always declines team re-rolls, so the pre-roll path is the common one: under
    /// bb2016 a declined stand-up re-roll must NOT consume the team's Blitz. wood_elf bb2016 seed 14:
    /// away_01's stand-up blitz re-roll is declined at i=34 and Java keeps `blitzUsed` false, so away_02
    /// can still blitz at i=39 — Rust was consuming the blitz and offering away_02 only Move. Applying
    /// the bb2016 behaviour to bb2025 regressed wood_elf bb2025 from 0 to 17 fails, hence the gate.
    #[test]
    fn declined_stand_up_reroll_does_not_consume_the_action() {
        use ffb_model::enums::{PlayerAction, PS_PRONE};

        fn setup() -> (Game, StepStandUp) {
            let mut game = make_game();
            game.home_playing = true;
            game.turn_mode = ffb_model::enums::TurnMode::Regular;
            game.team_home.players.push(ffb_model::model::player::Player {
                id: "tree".into(), name: "tree".into(), nr: 1, position_id: "treeman".into(),
                movement: 2, strength: 6, agility: 1, passing: 5, armour: 10, // MA < 3 → roll required
                ..Default::default()
            });
            game.field_model.set_player_coordinate("tree", ffb_model::types::FieldCoordinate::new(5, 5));
            game.field_model.set_player_state("tree", PlayerState::new(PS_PRONE));
            game.acting_player.player_id = Some("tree".into());
            game.acting_player.player_action = Some(PlayerAction::Blitz);
            game.acting_player.standing_up = true;
            game.turn_data_home.rerolls = 2;
            game.turn_data_home.reroll_used = false;
            (game, StepStandUp::new(String::new()))
        }

        // bb2025: a DECLINED re-roll DOES consume the action (handleFailedStandUp is called there).
        let (mut game, mut step) = setup();
        step.re_roll_state.re_rolled_action =
            Some(ffb_model::model::re_rolled_action::ReRolledAction::new("STAND_UP"));
        step.re_roll_state.re_roll_source = None;
        let _ = step.start(&mut game, &mut GameRng::new(0));
        assert!(game.turn_data_home.blitz_used,
            "bb2020/bb2025 call handleFailedStandUp at the pre-roll declined-re-roll site");

        // bb2016: the same path must NOT consume the team's Blitz.
        let (mut game, mut step) = setup();
        game.rules = Rules::Bb2016;
        step.re_roll_state.re_rolled_action =
            Some(ffb_model::model::re_rolled_action::ReRolledAction::new("STAND_UP"));
        step.re_roll_state.re_roll_source = None; // the harness declines
        let _ = step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.turn_data_home.blitz_used,
            "a DECLINED stand-up re-roll must not consume the team's Blitz");
        assert_eq!(game.field_model.player_state("tree").unwrap().base(), PS_PRONE);

        // No re-roll state at all → normal roll; on failure the flags DO get set (post-roll branch).
        let (mut game, mut step) = setup();
        game.turn_data_home.rerolls = 0; // no re-roll available → straight to the failure branch
        let mut seed = 0u64;
        while GameRng::new(seed).d6() >= 4 { seed += 1; assert!(seed < 500); }
        let _ = step.start(&mut game, &mut GameRng::new(seed));
        assert!(game.turn_data_home.blitz_used,
            "a stand-up that simply FAILS with no re-roll available still consumes the Blitz");
    }

    #[test]
    fn not_standing_up_returns_next_step_immediately() {
        let mut game = make_game();
        game.acting_player.standing_up = false;
        let mut step = StepStandUp::new("fail".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.is_empty());
    }

    #[test]
    fn already_moved_returns_next_step_immediately() {
        let mut game = make_game();
        game.acting_player.standing_up = true;
        game.acting_player.has_moved = true;
        let mut step = StepStandUp::new("fail".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    /// Java clears `standingUp` ONLY on the ROLLED stand-up's success path
    /// (`StepStandUp.java:114-115`). The free stand-up (`:136-138`) sets `hasMoved` and leaves
    /// `standingUp` TRUE, because `FoulAppearanceBehaviour.handleFailure` reads it later to send a
    /// blitzer who failed Foul Appearance back to PRONE. Rust cleared it on both paths.
    #[test]
    fn rolled_success_clears_standing_up_but_free_stand_up_does_not() {
        for seed in 0u64..200 {
            let mut g = make_game();
            g.acting_player.standing_up = true;
            let rolls = g.acting_player.player_id.as_deref()
                .and_then(|id| g.player(id))
                .map(|p| p.movement_with_modifiers() < MINIMUM_MOVE_TO_STAND_UP
                    && !p.has_skill_property(NamedProperties::CAN_STAND_UP_FOR_FREE))
                .unwrap_or(true);
            let mut step = StepStandUp::new("fail".into());
            let out = step.start(&mut g, &mut GameRng::new(seed));
            if out.action != StepAction::NextStep {
                continue;
            }
            if rolls {
                assert!(!g.acting_player.standing_up,
                    "a ROLLED stand-up success clears standingUp (Java :115)");
            } else {
                assert!(g.acting_player.standing_up,
                    "a FREE stand-up leaves standingUp set (Java :136-138) so a later Foul \
                     Appearance failure can revert the player to PRONE");
            }
            assert!(g.acting_player.has_moved);
            return;
        }
        panic!("no seed produced a successful stand-up");
    }

    #[test]
    fn failure_goes_to_failure_label_with_end_player_action() {
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        game.acting_player.standing_up = true;
        let mut step = StepStandUp::new("fail".into());
        step.roll = 1; // guaranteed fail (need >= 4)
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))));
    }

    #[test]
    fn failure_with_trr_offers_reroll_prompt() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        game.acting_player.standing_up = true;
        let mut step = StepStandUp::new("fail".into());
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
        assert!(out.prompt.is_some());
    }

    #[test]
    fn accept_reroll_then_success_returns_next_step() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        game.acting_player.standing_up = true;
        let mut step = StepStandUp::new("fail".into());
        step.roll = 1;
        let _offer = step.start(&mut game, &mut GameRng::new(0));
        step.roll = 6; // success on re-roll
        let out = step.handle_command(&Action::UseReRoll { use_reroll: true }, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn decline_reroll_goes_to_failure_label() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        game.acting_player.standing_up = true;
        let mut step = StepStandUp::new("fail".into());
        step.roll = 1;
        let _offer = step.start(&mut game, &mut GameRng::new(0));
        let out = step.handle_command(&Action::UseReRoll { use_reroll: false }, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn set_parameter_goto_label_on_failure_accepted() {
        let mut step = StepStandUp::new("old".into());
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("new".into())));
        assert_eq!(step.goto_label_on_failure, "new");
    }

    #[test]
    fn failed_stand_up_blitz_sets_blitz_used() {
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        game.acting_player.standing_up = true;
        game.acting_player.player_action = Some(PlayerAction::Blitz);
        let mut step = StepStandUp::new("fail".into());
        step.roll = 1;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.turn_data_home.blitz_used);
    }

    #[test]
    fn failed_stand_up_pass_sets_pass_used() {
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        game.acting_player.standing_up = true;
        game.acting_player.player_action = Some(PlayerAction::Pass);
        let mut step = StepStandUp::new("fail".into());
        step.roll = 1;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.turn_data_home.pass_used);
        assert!(!game.turn_data_home.ttm_used);
    }

    #[test]
    fn failed_stand_up_throw_team_mate_sets_ttm_used() {
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        game.acting_player.standing_up = true;
        game.acting_player.player_action = Some(PlayerAction::ThrowTeamMate);
        let mut step = StepStandUp::new("fail".into());
        step.roll = 1;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.turn_data_home.ttm_used);
        assert!(!game.turn_data_home.pass_used);
    }

    #[test]
    fn failed_stand_up_punt_sets_punt_used() {
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        game.acting_player.standing_up = true;
        game.acting_player.player_action = Some(PlayerAction::Punt);
        let mut step = StepStandUp::new("fail".into());
        step.roll = 1;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.turn_data_home.punt_used);
    }

    #[test]
    fn roll_emits_stand_up_roll_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        game.acting_player.standing_up = true;
        let mut step = StepStandUp::new("fail".into());
        step.roll = 4;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::STAND_UP_ROLL));
    }

    #[test]
    fn pinned_and_successful_goes_to_label_without_end_player_action() {
        // Java: `if (playerState.isPinned()) { setNextAction(GOTO_LABEL, ...); }` — no
        // publishParameter(END_PLAYER_ACTION) call in this branch (unlike the failure path).
        use ffb_model::enums::PlayerState;
        let mut game = make_game();
        game.acting_player.standing_up = true;
        game.acting_player.player_id = Some("p1".into());
        game.field_model.set_player_state("p1", PlayerState::new(PS_PRONE).change_rooted(true));
        let mut step = StepStandUp::new("fail".into());
        step.roll = 6; // guaranteed success
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
        assert!(
            !out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))),
            "Java's pinned-and-successful branch never publishes END_PLAYER_ACTION"
        );
    }

    #[test]
    fn pinned_and_failed_roll_still_offers_reroll() {
        // Java only checks isPinned() in the `successful` branch; a failed roll for a
        // pinned player must still go through the normal reroll-offer path, not short-
        // circuit straight to the failure label.
        use ffb_model::enums::PlayerState;
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        game.acting_player.standing_up = true;
        game.acting_player.player_id = Some("p1".into());
        game.field_model.set_player_state("p1", PlayerState::new(PS_PRONE).change_rooted(true));
        let mut step = StepStandUp::new("fail".into());
        step.roll = 1; // guaranteed fail
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue, "pinned player should still be offered a re-roll on failure");
        assert!(out.prompt.is_some());
    }

    #[test]
    fn failure_preserves_other_player_state_flags() {
        // Java: setPlayerState(playerState.changeBase(PRONE).changeActive(false)) mutates
        // the *existing* PlayerState (base mask for PRONE is 0xfff00, preserving upper
        // flag bits like rooted/confused/usedPro), not a fresh PRONE-only state.
        use ffb_model::enums::PlayerState;
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        game.acting_player.standing_up = true;
        game.acting_player.player_id = Some("p1".into());
        game.field_model.set_player_state(
            "p1",
            PlayerState::new(PS_PRONE).change_rooted(true),
        );
        let mut step = StepStandUp::new("fail".into());
        step.roll = 1; // guaranteed fail
        step.start(&mut game, &mut GameRng::new(0));
        let state = game.field_model.player_state("p1").unwrap();
        assert!(state.is_rooted(), "rooted flag must survive a failed stand-up");
        assert!(!state.is_active(), "active flag must be cleared on failed stand-up");
        assert_eq!(state.base(), PS_PRONE);
    }

    #[test]
    fn failed_roll_still_emits_stand_up_roll_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        game.acting_player.standing_up = true;
        let mut step = StepStandUp::new("fail".into());
        step.roll = 1;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::STAND_UP_ROLL));
    }
}
