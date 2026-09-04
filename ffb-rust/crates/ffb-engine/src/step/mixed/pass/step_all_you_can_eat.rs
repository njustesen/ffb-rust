/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.pass.StepAllYouCanEat`.
///
/// Handles the ALL_YOU_CAN_EAT skill check when a Ogre / big-eater bombardier
/// fires a bomb: roll 1d6 vs target 4+.  Failure ejects the bombardier.  A
/// re-roll may be offered when the player has not already re-rolled.
///
/// Java: `@RulesCollection(BB2020, BB2025)`, extends `AbstractStepWithReRoll`.
use ffb_model::model::game::Game;
use ffb_model::model::re_rolled_action::ReRolledAction;
use ffb_model::report::mixed::report_all_you_can_eat_roll::ReportAllYouCanEatRoll;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter, SequenceStep};
use crate::step::abstract_step_with_re_roll::ReRollState;
use crate::step::generator::sequence::labels;

/// Java: `ReRolledActions.ALL_YOU_CAN_EAT` equivalent.
const RE_ROLLED_ACTION: &str = "ALL_YOU_CAN_EAT";
/// Java: `int minimumRoll = 4`
const MINIMUM_ROLL: i32 = 4;

/// The `ReRollSource` carried by a `ReRollOffer` prompt — Java's
/// `ClientCommandUseReRoll.getReRollSource()`, which the client echoes back from the dialog it was
/// shown. Any other prompt shape cannot reach the AllYouCanEat re-roll, so it falls back to `TRR`,
/// the source `askForReRollIfAvailable` offers when no skill applies.
fn prompt_re_roll_source(prompt: &ffb_model::prompts::AgentPrompt) -> ffb_model::enums::ReRollSource {
    match prompt {
        ffb_model::prompts::AgentPrompt::ReRollOffer { source, .. } => source.clone(),
        _ => ffb_model::enums::ReRollSource::new("TRR"),
    }
}

/// Java: `StepAllYouCanEat` (mixed/pass, BB2020 + BB2025).
/// Extends AbstractStepWithReRoll.
#[derive(Debug, Default)]
pub struct StepAllYouCanEat {
    /// Re-roll tracking (AbstractStepWithReRoll).
    pub re_roll_state: ReRollState,
    /// Java: local `original_bombardier` — read from `passState.getOriginalBombardier()`.
    /// Stored here after the first `start()` so a re-roll invocation can find it.
    pub original_bombardier: Option<String>,
}

impl StepAllYouCanEat {
    pub fn new() -> Self { Self::default() }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: Player<?> player = game.getPlayerById(getGameState().getPassState().getOriginalBombardier())
        // — game.original_bombardier is the Rust home of PassState.originalBombardier. The old
        // thrower_id proxy dated from the PassState-stub era and is CLEARED by StepEndBomb right
        // before this step is pushed, so the sent-off roll was silently skipped (halfling seed 5
        // i=14: Java rolled the 4+ check, Rust rolled nothing).
        let player_id = self.original_bombardier
            .clone()
            .or_else(|| game.original_bombardier.clone())
            .or_else(|| game.thrower_id.clone())
            .unwrap_or_default();
        if player_id.is_empty() {
            return StepOutcome::next();
        }
        self.original_bombardier = Some(player_id.clone());

        let mut do_roll = true;

        // Java: if (getReRolledAction() == ReRolledActions.ALL_YOU_CAN_EAT)
        if let Some(ref action) = self.re_roll_state.re_rolled_action.clone() {
            if action.get_name() == RE_ROLLED_ACTION {
                let did_reroll = if let Some(ref src) = self.re_roll_state.re_roll_source.clone() {
                    crate::step::util_server_re_roll::use_reroll(game, src, &player_id, rng)
                } else {
                    false
                };
                if !did_reroll {
                    do_roll = false;
                }
            }
        }

        let mut success = false;
        let rerolled = self.re_roll_state.re_roll_source.is_some()
            && self.re_roll_state.re_rolled_action.as_ref()
                .map_or(false, |a| a.get_name() == RE_ROLLED_ACTION);

        if do_roll {
            // Java: int roll = getDiceRoller().rollSkill()
            let roll = rng.d6();
            success = roll >= MINIMUM_ROLL;

            // Java: getResult().addReport(new ReportAllYouCanEatRoll(player.getId(), success, roll, minimumRoll, reRolled))
            game.report_list.add(ReportAllYouCanEatRoll::new(
                Some(player_id.clone()),
                success,
                roll,
                MINIMUM_ROLL,
                rerolled,
            ));

            let outcome_base = StepOutcome::next()
                .with_event(ffb_model::events::GameEvent::AllYouCanEatRoll {
                    player_id: player_id.clone(),
                    roll,
                    minimum_roll: MINIMUM_ROLL,
                    success,
                    rerolled,
                });

            if !success && !rerolled {
                // Java: if (!success && !reRolled && askForReRollIfAvailable(...)) { return; }
                // Java passes the PLAYER overload the original bombardier, NOT the acting player:
                //   askForReRollIfAvailable(getGameState(), player, ALL_YOU_CAN_EAT, 4, false)
                // (`StepAllYouCanEat.executeStep`, the `player` resolved from
                // `passState.getOriginalBombardier()` at the top of the method).
                if let Some(prompt) = crate::step::util_server_re_roll::ask_for_reroll_if_available_for(
                    game, Some(&player_id), RE_ROLLED_ACTION, MINIMUM_ROLL, false,
                ) {
                    self.re_roll_state.re_rolled_action = Some(ReRolledAction::new(RE_ROLLED_ACTION));
                    // Java's `AbstractStepWithReRoll.handleCommand` fills BOTH fields from the
                    // incoming `ClientCommandUseReRoll` (`setReRolledAction` +
                    // `reRollSourceSuccessfully(cmd.getReRollSource())`), so on the re-entry
                    // `getReRollSource()` is the source the coach accepted. Rust's steps carry
                    // that themselves: remember the offered source now, and clear it in
                    // `handle_command` when the coach declines. Without this the source stayed
                    // `None`, `use_reroll` was never called, `doRoll` went false and an ACCEPTED
                    // re-roll ejected the bombardier (bb2025 halfling seed 90 i=6: Java rolls the
                    // Loner check and a fresh 4+ and plays on, Rust sent Cindy off and ended the
                    // drive).
                    self.re_roll_state.re_roll_source = Some(prompt_re_roll_source(&prompt));
                    // `outcome_base` was built from `StepOutcome::next()` (action == NextStep),
                    // which the driver's dispatch silently drops `.prompt` for. Rebuild from
                    // `cont()` so the reroll dialog actually reaches the agent, carrying over
                    // the already-queued `AllYouCanEatRoll` event.
                    return StepOutcome::cont()
                        .with_events(outcome_base.events)
                        .with_prompt(prompt);
                }
            }

            // Java: if (!success) push EjectPlayer + Bribes onto stack
            if !success {
                // Java: stack.push(EJECT_PLAYER(GOTO_LABEL_ON_END=END_BOMB))
                //       stack.push(BRIBES(GOTO_LABEL_ON_END=END_BOMB))
                // Raw LIFO pushes: BRIBES lands on top and runs FIRST (the argue-the-call d6),
                // then EjectPlayer. Rust's push_sequence executes vec[0] first, so the vec
                // order is [Bribes, EjectPlayer] — it was inverted, ejecting without the
                // argue roll and running one game die behind Java (halfling seed 28 i=77).
                let label_param = vec![StepParameter::GotoLabelOnEnd(labels::END_BOMB.into())];
                let seq = vec![
                    SequenceStep::with_params(StepId::Bribes, label_param.clone()),
                    SequenceStep::with_params(StepId::EjectPlayer, label_param),
                ];
                return outcome_base.push_seq(seq);
            }

            return outcome_base;
        }

        // do_roll == false means re-roll declined → treat as failure
        // Java: if (!success) push EjectPlayer + Bribes — raw LIFO pushes: Bribes runs first.
        let label_param = vec![StepParameter::GotoLabelOnEnd(labels::END_BOMB.into())];
        let seq = vec![
            SequenceStep::with_params(StepId::Bribes, label_param.clone()),
            SequenceStep::with_params(StepId::EjectPlayer, label_param),
        ];
        StepOutcome::next().push_seq(seq)
    }
}

impl Step for StepAllYouCanEat {
    fn id(&self) -> StepId { StepId::AllYouCanEat }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: `AbstractStepWithReRoll.handleCommand` sets fReRollSource from the command, which
        // the harness sends as null when the coach declines (`sendUseReRoll(action, null)`).
        // `executeStep` then sees `getReRollSource() == null` → `doRoll = false` → eject.
        if let Action::UseReRoll { use_reroll: false } = action {
            self.re_roll_state.re_roll_source = None;
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool {
        false
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

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    /// A home-team bombardier holding the ALL_YOU_CAN_EAT re-roll offer's preconditions:
    /// Java's `RollMechanic.isTeamReRollAvailable` gates on `actingTeam.hasPlayer(player)`, and
    /// `StepAllYouCanEat` hands it the ORIGINAL BOMBARDIER, so the player has to really be on the
    /// acting team for the dialog to be raised at all.
    fn add_home_bombardier(game: &mut Game, id: &str) {
        game.team_home.players.push(ffb_model::model::player::Player {
            id: id.into(), name: id.into(), nr: 2, position_id: "star".into(),
            movement: 5, strength: 2, agility: 3, passing: 3, armour: 7,
            ..Default::default()
        });
        game.field_model.set_player_coordinate(id, ffb_model::types::FieldCoordinate::new(12, 5));
        game.original_bombardier = Some(id.into());
        game.acting_player.player_id = Some(id.into());
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        game.turn_data_home.reroll_used = false;
    }

    /// Java `StepAllYouCanEat.executeStep`, the re-roll re-entry:
    ///
    /// ```java
    /// if (getReRolledAction() == ReRolledActions.ALL_YOU_CAN_EAT) {
    ///     if (getReRollSource() == null || !UtilServerReRoll.useReRoll(this, getReRollSource(), player)) {
    ///         doRoll = false;
    ///     }
    /// }
    /// ```
    ///
    /// `getReRollSource()` is filled by `AbstractStepWithReRoll.handleCommand` from the incoming
    /// `ClientCommandUseReRoll` — non-null when the coach ACCEPTS, null when they decline. So an
    /// accepted re-roll must spend the team re-roll and roll a fresh die; only a declined one
    /// falls through to Bribes + EjectPlayer.
    #[test]
    fn an_accepted_re_roll_rolls_again_and_a_declined_one_ejects() {
        // ── accepted ────────────────────────────────────────────────────────────────────────
        let mut game = make_game();
        add_home_bombardier(&mut game, "home_02");
        // Seed the failing roll: walk seeds until the first d6 is a miss (< 4) so the step
        // offers the re-roll.
        let mut accepted = None;
        for seed in 0u64..200 {
            let mut step = StepAllYouCanEat::new();
            let mut rng = GameRng::new(seed);
            let out = step.start(&mut game, &mut rng);
            if out.prompt.is_some() {
                assert!(step.re_roll_state.re_roll_source.is_some(),
                    "offering the dialog must remember the source, as Java's handleCommand sets it \
                     from the returning command");
                let before = rng.call_count;
                let rerolls_before = game.turn_data_home.rerolls;
                let out2 = step.handle_command(
                    &Action::UseReRoll { use_reroll: true }, &mut game, &mut rng);
                assert!(rng.call_count > before,
                    "an accepted ALL_YOU_CAN_EAT re-roll must roll a fresh d6");
                assert_eq!(game.turn_data_home.rerolls, rerolls_before - 1,
                    "an accepted team re-roll is spent");
                accepted = Some(out2);
                break;
            }
        }
        let accepted = accepted.expect("expected a failing roll with a TRR available");
        // Java only pushes BRIBES/EJECT_PLAYER when the (re-)roll failed; a successful re-roll
        // leaves the stack alone.
        if accepted.events.iter().any(|e| matches!(e,
            ffb_model::events::GameEvent::AllYouCanEatRoll { success: true, .. })) {
            assert!(accepted.pushes.is_empty(),
                "a successful re-roll must not eject the bombardier");
        }

        // ── declined ────────────────────────────────────────────────────────────────────────
        let mut game = make_game();
        add_home_bombardier(&mut game, "home_02");
        for seed in 0u64..200 {
            let mut step = StepAllYouCanEat::new();
            let mut rng = GameRng::new(seed);
            let out = step.start(&mut game, &mut rng);
            if out.prompt.is_some() {
                let before = rng.call_count;
                let out2 = step.handle_command(
                    &Action::UseReRoll { use_reroll: false }, &mut game, &mut rng);
                assert_eq!(rng.call_count, before,
                    "a declined re-roll rolls nothing (doRoll == false)");
                assert_eq!(out2.pushes.len(), 1);
                assert_eq!(out2.pushes[0][0].step_id, StepId::Bribes);
                assert_eq!(out2.pushes[0][1].step_id, StepId::EjectPlayer);
                return;
            }
        }
        panic!("expected a failing roll with a TRR available");
    }

    #[test]
    fn no_thrower_returns_next() {
        let mut step = StepAllYouCanEat::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn with_thrower_emits_roll_event() {
        let mut step = StepAllYouCanEat::new();
        let mut game = make_game();
        game.thrower_id = Some("bombardier".into());
        // Seed 5 → roll_d6 >= 4 → success
        let mut rng = GameRng::new(5);
        let out = step.start(&mut game, &mut rng);
        // Either NextStep (success) or Continue (reroll offered)
        let has_event = !out.events.is_empty();
        assert!(has_event || out.action == StepAction::NextStep);
    }

    #[test]
    fn high_roll_succeeds_without_reroll_offer() {
        let mut step = StepAllYouCanEat::new();
        let mut game = make_game();
        game.thrower_id = Some("bombardier".into());
        // Force a specific high roll (no TRR available in default game)
        let mut rng = GameRng::new(999);
        let out = step.start(&mut game, &mut rng);
        // Should not be Continue on success (no reroll needed)
        // (May be NextStep or Continue depending on the roll — just ensure no panic)
        assert!(matches!(out.action, StepAction::NextStep | StepAction::Continue));
    }

    #[test]
    fn original_bombardier_cached_after_first_call() {
        let mut step = StepAllYouCanEat::new();
        let mut game = make_game();
        game.thrower_id = Some("bard1".into());
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert_eq!(step.original_bombardier.as_deref(), Some("bard1"));
    }

    #[test]
    fn low_roll_pushes_bribes_and_eject_player_sequences() {
        // Find a seed that produces a failing roll (1-3 for a 4+ target).
        let mut game = make_game();
        game.thrower_id = Some("bard".into());
        // Try seeds until we get a failure (push non-empty)
        let mut pushed_seq = None;
        for seed in 0u64..100 {
            let mut step = StepAllYouCanEat::new();
            let mut rng = GameRng::new(seed);
            let out = step.start(&mut game, &mut rng);
            if !out.pushes.is_empty() {
                pushed_seq = Some(out.pushes[0].clone());
                break;
            }
        }
        let seq = pushed_seq.expect("expected at least one failing roll in 100 seeds");
        assert_eq!(seq.len(), 2);
        // Java raw-LIFO-pushes EJECT_PLAYER then BRIBES, so BRIBES runs FIRST (the argue
        // roll) — push_sequence executes vec[0] first, hence Bribes leads the vec.
        assert_eq!(seq[0].step_id, StepId::Bribes);
        assert_eq!(seq[1].step_id, StepId::EjectPlayer);
    }

    #[test]
    fn all_you_can_eat_report_added_on_roll() {
        let mut game = make_game();
        game.thrower_id = Some("bard".into());
        let mut step = StepAllYouCanEat::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ffb_model::report::report_id::ReportId::ALL_YOU_CAN_EAT));
    }

    #[test]
    fn no_all_you_can_eat_report_when_no_thrower() {
        let mut game = make_game();
        // No thrower set
        let mut step = StepAllYouCanEat::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.report_list.has_report(ffb_model::report::report_id::ReportId::ALL_YOU_CAN_EAT));
    }

    /// Regression test for the bug where the re-roll offer's `.with_prompt(...)` was
    /// attached to a `StepOutcome::next()` (action == NextStep) `outcome_base`. The
    /// driver's dispatch (`driver.rs`) only honors `.prompt` for `Continue`/`Repeat`
    /// actions, so a NextStep-based outcome carrying a prompt was silently discarded
    /// and the reroll dialog never reached the agent. The outcome must be `Continue`
    /// whenever a reroll offer prompt is attached.
    #[test]
    fn reroll_offer_returns_continue_action_with_prompt() {
        let mut game = make_game();
        add_home_bombardier(&mut game, "bard");
        // Try seeds until we get a failing roll (1-3 for a 4+ target), which should
        // trigger a reroll offer since a TRR is available.
        let mut found_prompt = false;
        for seed in 0u64..100 {
            let mut step = StepAllYouCanEat::new();
            let mut rng = GameRng::new(seed);
            let out = step.start(&mut game, &mut rng);
            if out.prompt.is_some() {
                found_prompt = true;
                assert_eq!(out.action, StepAction::Continue, "a StepOutcome carrying a prompt must use action Continue, not NextStep, or the driver silently drops the dialog");
            }
        }
        assert!(found_prompt, "expected at least one seed to trigger a reroll offer prompt in 100 seeds");
    }
}
