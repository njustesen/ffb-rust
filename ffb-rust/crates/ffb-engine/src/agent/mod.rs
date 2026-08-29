//! The step-engine agent boundary — a SEPARATE module from the driver, with a single clear
//! interface: `Agent::act(&GameState) -> Action`.
//!
//! Dependency direction is one-way: the agent reads the engine (`GameState`), never the
//! reverse. One `act` call per prompt — the agent inspects `gs.current_prompt()` (and `gs.game`
//! for legal-action queries) and returns the `Action` the driver should `apply`. State-in /
//! action-out: no separate response type.
//!
//! Two independent agent implementations live here:
//! - [`RandomAgent`] — the Java-parity driver. Its RNG-consumption order is load-bearing (see
//!   `AGENT_CONTRACT.md`); do not change its behavior.
//! - [`UniformAgent`] — a coverage-oriented driver with no RNG-stream discipline. It samples
//!   uniformly among all truly legal actions at every decision point, for "how much of the
//!   game's mechanic surface does random play exercise" runs.

pub mod det_math;
mod heuristic_agent;
mod random_agent;
mod uniform_agent;

pub use heuristic_agent::{
    ClassMask, HeuristicAgent, Mode, PromptClass, ScoredOption, prompt_class_of,
};
pub use random_agent::RandomAgent;
pub use uniform_agent::UniformAgent;

use crate::action::Action;
use crate::step::GameState;

/// The step engine's decision-maker. Reads the game state (including the pending prompt) and
/// returns the action to apply. `&mut self` carries the agent's own RNG/turn state; `&GameState`
/// is read-only — the agent never mutates the engine.
pub trait Agent {
    fn act(&mut self, gs: &GameState) -> Action;
}

/// 1:1 mirror of `ParityRunner.filterStaleActions`.
///
/// Both agents snapshot their eligible list at TURN START, so an action that was legal then may
/// have been consumed by an earlier activation in the same turn — the team's one Blitz, its one
/// Pass, its one Foul. Java drops those before the action pick, and the pick is POSITIONAL, so a
/// list that is one entry longer on one side chooses a different action from identical weights.
///
/// Lives here rather than in either agent because Java's own comment makes the coupling explicit:
/// "Rust's RandomAgent applies the identical filter so idx % N stays aligned." The heuristic needs
/// the same rule for the same reason, and two copies would drift.
///
/// `Move`/`StandUp` always survive, so the result is never empty.
pub(crate) fn filter_stale_actions(
    game: &ffb_model::model::game::Game,
    actions: &[ffb_model::enums::PlayerAction],
) -> Vec<ffb_model::enums::PlayerAction> {
    use ffb_model::enums::{PlayerAction, Rules, TurnMode};
    let td = if game.home_playing { &game.turn_data_home } else { &game.turn_data_away };
    let live: Vec<PlayerAction> = actions
        .iter()
        .filter(|a| match a {
            PlayerAction::Block | PlayerAction::Blitz | PlayerAction::StandUpBlitz => !td.blitz_used,
            PlayerAction::Pass | PlayerAction::HailMaryPass => !td.pass_used,
            PlayerAction::HandOver => !td.hand_over_used,
            PlayerAction::Foul => !td.foul_used,
            // BB2016 and BB2020 spend the team's PASS on a Throw Team-Mate; only BB2025 tracks it
            // on its own flag.
            PlayerAction::ThrowTeamMate => {
                if game.rules == Rules::Bb2025 { !td.ttm_used } else { !td.ttm_used && !td.pass_used }
            }
            // BB2016 spends the team's BLITZ on a Kick Team-Mate; later editions use ktm_used.
            PlayerAction::KickTeamMate => {
                if game.rules == Rules::Bb2016 { !td.blitz_used } else { !td.ktm_used }
            }
            _ => true,
        })
        .cloned()
        .collect();
    // Non-REGULAR window modes shrink the list to MOVE + the UseSkill specials. A window
    // Block/Blitz/Foul was always a declare-then-deselect no-op, and a window Blitz against a
    // suspended thrower re-fires CONFIRM_END_ACTION forever.
    if game.turn_mode == TurnMode::PassBlock {
        return live
            .into_iter()
            .filter(|a| {
                matches!(a, PlayerAction::Move | PlayerAction::Treacherous | PlayerAction::BlackInk)
            })
            .collect();
    }
    live
}

#[cfg(test)]
mod filter_stale_actions_tests {
    use super::filter_stale_actions;
    use crate::step::framework::test_team;
    use ffb_model::enums::{PlayerAction, Rules, TurnMode};
    use ffb_model::model::game::Game;

    fn game(rules: Rules) -> Game {
        let mut g = Game::new(test_team("home", 0), test_team("away", 0), rules);
        g.home_playing = true;
        g
    }

    /// ITER22 regression. The heuristic scored the engine's live action list RAW while Java ran
    /// `filterStaleActions` over its own recomputed list first (`ParityRunner.eligibleFor`). The
    /// action pick is positional, so one extra entry on the Rust side picks a different action
    /// from identical weights.
    #[test]
    fn spent_team_actions_are_dropped() {
        let mut g = game(Rules::Bb2025);
        let all = [
            PlayerAction::Move,
            PlayerAction::Block,
            PlayerAction::Blitz,
            PlayerAction::Pass,
            PlayerAction::HandOver,
            PlayerAction::Foul,
        ];
        assert_eq!(filter_stale_actions(&g, &all).len(), all.len(), "nothing spent yet");

        g.turn_data_home.blitz_used = true;
        g.turn_data_home.pass_used = true;
        g.turn_data_home.hand_over_used = true;
        g.turn_data_home.foul_used = true;
        assert_eq!(
            filter_stale_actions(&g, &all),
            vec![PlayerAction::Move],
            "Move always survives; every once-per-turn action is spent"
        );
    }

    /// The half that is NOT a no-op on a live list: a window shrinks to MOVE + the UseSkill
    /// specials, whatever the engine reports.
    #[test]
    fn pass_block_window_shrinks_to_move_and_specials() {
        let mut g = game(Rules::Bb2025);
        g.turn_mode = TurnMode::PassBlock;
        let offered = [
            PlayerAction::Move,
            PlayerAction::Block,
            PlayerAction::Blitz,
            PlayerAction::Foul,
            PlayerAction::Treacherous,
        ];
        assert_eq!(
            filter_stale_actions(&g, &offered),
            vec![PlayerAction::Move, PlayerAction::Treacherous]
        );
    }

    /// The other non-no-op half: BB2016/BB2020 spend the team's PASS on a Throw Team-Mate and its
    /// BLITZ on a Kick Team-Mate; only BB2025 tracks either on its own flag.
    #[test]
    fn team_mate_throws_follow_the_edition_rules() {
        let ttm = [PlayerAction::ThrowTeamMate];
        let ktm = [PlayerAction::KickTeamMate];

        for rules in [Rules::Bb2016, Rules::Bb2020] {
            let mut g = game(rules);
            g.turn_data_home.pass_used = true;
            assert!(filter_stale_actions(&g, &ttm).is_empty(), "{rules:?} TTM spends the pass");
        }
        let mut g25 = game(Rules::Bb2025);
        g25.turn_data_home.pass_used = true;
        assert_eq!(filter_stale_actions(&g25, &ttm), ttm, "bb2025 TTM has its own flag");

        let mut g16 = game(Rules::Bb2016);
        g16.turn_data_home.blitz_used = true;
        assert!(filter_stale_actions(&g16, &ktm).is_empty(), "bb2016 KTM spends the blitz");
        let mut g20 = game(Rules::Bb2020);
        g20.turn_data_home.blitz_used = true;
        assert_eq!(filter_stale_actions(&g20, &ktm), ktm, "bb2020 KTM has its own flag");
    }
}
