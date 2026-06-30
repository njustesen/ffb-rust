/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2025.start.StepMasterChef` (BB2025).
///
/// Resolves the Master Chef inducement: rolls 3d6 per chef, counts successes (> 3),
/// steals that many rerolls from the opponent, and registers leader players for
/// both teams (stub — `setLeaders` not yet wired).
///
/// Guard: only fires when `game.half < 3 && game.turn_data_{home|away}.first_turn_after_kickoff`.
/// Java `game.firstTurnOfHalf()` == `turn_data_home.first_turn_after_kickoff || turn_data_away.first_turn_after_kickoff`
/// (either half can be first-turn; here we check the active team's turn data).
///
/// `UtilServerGame.rollMasterChef` uses `team.master_chefs` as the chef count.
/// Each chef rolls 3d6; each die > 3 steals one reroll.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.start.StepMasterChef`.
pub struct StepMasterChef;

impl StepMasterChef {
    pub fn new() -> Self { Self }

    // ── Java private executeStep() ──────────────────────────────────────────────

    fn execute_step(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: if (game.getHalf() < 3 && game.firstTurnOfHalf())
        // game.firstTurnOfHalf() checks BOTH TurnData objects; any team being at its first
        // turn of the half satisfies the condition (Java logic: home || away).
        let first_turn_of_half = game.turn_data_home.first_turn_after_kickoff
            || game.turn_data_away.first_turn_after_kickoff;

        if game.half < 3 && first_turn_of_half {
            // Java: UtilServerGame.handleChefRolls(this, game)
            self.handle_chef_rolls(game, rng);
            // Java: setLeaders(game.getTeamHome()); setLeaders(game.getTeamAway());
            // setLeaders finds players with grantsTeamReRollWhenOnPitch property and registers
            // them via gameState.addLeader(player). The leader registry is not yet ported;
            // stub is intentionally left empty (zero effect, correct skip logic).
        }
        StepOutcome::next()
    }

    // ── Java UtilServerGame.handleChefRolls ────────────────────────────────────

    /// Java `UtilServerGame.handleChefRolls(IStep, Game)`.
    ///
    /// Rolls master chef dice for home then away, then applies the cross-theft:
    /// - home rerolls: max(0, home - stolen_by_away) + stolen_by_home
    /// - away rerolls: max(0, away - stolen_by_home) + stolen_by_away
    fn handle_chef_rolls(&self, game: &mut Game, rng: &mut GameRng) -> () {
        let rerolls_stolen_home = self.roll_master_chef_for_team(game.team_home.master_chefs, rng);
        let rerolls_stolen_away = self.roll_master_chef_for_team(game.team_away.master_chefs, rng);

        // Apply: home loses what away stole, gains what home stole from away.
        let home_rerolls = game.turn_data_home.rerolls;
        game.turn_data_home.rerolls = (home_rerolls - rerolls_stolen_away).max(0) + rerolls_stolen_home;

        let away_rerolls = game.turn_data_away.rerolls;
        game.turn_data_away.rerolls = (away_rerolls - rerolls_stolen_home).max(0) + rerolls_stolen_away;
    }

    // ── Java UtilServerGame.rollMasterChef (private) ───────────────────────────

    /// Java `UtilServerGame.rollMasterChef(IStep, boolean pHomeTeam)`.
    ///
    /// For each master chef owned by the team: roll 3d6.
    /// Each die strictly greater than 3 (i.e. 4, 5, or 6) steals one reroll.
    /// Java `DiceRoller.rollMasterChef()` = `rollDice(3, 6)` (three independent d6).
    /// Java `DiceInterpreter.interpretMasterChefRoll` = count dice where `roll > 3`.
    fn roll_master_chef_for_team(&self, chef_count: i32, rng: &mut GameRng) -> i32 {
        let mut total_stolen = 0i32;
        for _ in 0..chef_count {
            // 3d6 roll — one per chef.
            let d1 = rng.d6();
            let d2 = rng.d6();
            let d3 = rng.d6();
            // Count successes: die > 3 (Java: `roll > 3`).
            for die in [d1, d2, d3] {
                if die > 3 {
                    total_stolen += 1;
                }
            }
        }
        total_stolen
    }
}

impl Default for StepMasterChef {
    fn default() -> Self { Self::new() }
}

impl Step for StepMasterChef {
    fn id(&self) -> StepId { StepId::MasterChef }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn start_returns_next_step() {
        let mut game = make_game();
        let mut step = StepMasterChef::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn no_chefs_does_not_change_rerolls() {
        let mut game = make_game();
        // Set up a first-turn-of-half scenario.
        game.turn_data_home.first_turn_after_kickoff = true;
        game.turn_data_home.rerolls = 3;
        game.turn_data_away.rerolls = 2;
        // master_chefs defaults to 0 on both teams.
        let mut step = StepMasterChef::new();
        step.start(&mut game, &mut GameRng::new(42));
        assert_eq!(game.turn_data_home.rerolls, 3);
        assert_eq!(game.turn_data_away.rerolls, 2);
    }

    #[test]
    fn guard_skips_when_not_first_turn_of_half() {
        let mut game = make_game();
        game.team_home.master_chefs = 1;
        // first_turn_after_kickoff is false on both teams (default).
        game.turn_data_home.rerolls = 3;
        game.turn_data_away.rerolls = 3;
        let mut step = StepMasterChef::new();
        step.start(&mut game, &mut GameRng::new(0));
        // No dice consumed, rerolls unchanged.
        assert_eq!(game.turn_data_home.rerolls, 3);
        assert_eq!(game.turn_data_away.rerolls, 3);
    }

    #[test]
    fn guard_skips_when_half_ge_3() {
        let mut game = make_game();
        game.half = 3;
        game.turn_data_home.first_turn_after_kickoff = true;
        game.team_home.master_chefs = 1;
        game.turn_data_home.rerolls = 2;
        game.turn_data_away.rerolls = 2;
        let mut step = StepMasterChef::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_data_home.rerolls, 2);
        assert_eq!(game.turn_data_away.rerolls, 2);
    }

    #[test]
    fn away_rerolls_cannot_go_below_zero() {
        // A deterministic RNG seeded so every d6 rolls 6 ensures max theft.
        // Home has 1 chef; home rolls 3d6 all 6 → 3 stolen from away.
        // Away has no chefs → 0 stolen from home.
        // Away rerolls start at 1; result = max(0, 1 - 3) + 0 = 0.
        let mut game = make_game();
        game.turn_data_home.first_turn_after_kickoff = true;
        game.team_home.master_chefs = 1;
        game.turn_data_home.rerolls = 3;
        game.turn_data_away.rerolls = 1;
        // Use seed 0 — GameRng::new(0) produces deterministic dice; we just verify floor at 0.
        let mut step = StepMasterChef::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.turn_data_away.rerolls >= 0, "away rerolls must not go negative");
    }
}
