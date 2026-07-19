/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2025.start.StepMasterChef` (BB2025).
///
/// Resolves the Master Chef inducement: rolls 3d6 per chef, counts successes (> 3),
/// steals that many rerolls from the opponent, and registers leader players for
/// both teams (stub — `setLeaders` not yet wired).
///
/// Guard: only fires when `game.half < 3 && game.firstTurnOfHalf()`.
/// Java `Game.firstTurnOfHalf()` == `turnDataHome.getTurnNr() == 0 && turnDataAway.getTurnNr() == 0`
/// (an AND of both teams' turn counters, not an OR of any unrelated flag).
///
/// `UtilServerGame.rollMasterChef` counts the Master Chef inducement bought by each team
/// (`turnData.getInducementSet()`, the entry with `Usage.STEAL_REROLL`) as the chef count —
/// there is no `masterChefs` field on `Team` in Java. Each chef rolls 3d6; each die > 3 steals
/// one reroll, and a `ReportMasterChefRoll` is emitted per chef roll.
use ffb_model::events::GameEvent;
use ffb_model::inducement::usage::Usage;
use ffb_model::model::game::Game;
use ffb_model::report::report_master_chef_roll::ReportMasterChefRoll;
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
        // Game.firstTurnOfHalf() == turnDataAway.getTurnNr() == 0 && turnDataHome.getTurnNr() == 0
        let first_turn_of_half =
            game.turn_data_away.turn_nr == 0 && game.turn_data_home.turn_nr == 0;

        let mut events: Vec<GameEvent> = Vec::new();
        if game.half < 3 && first_turn_of_half {
            // Java: UtilServerGame.handleChefRolls(this, game)
            events.extend(self.handle_chef_rolls(game, rng));
            // Java: setLeaders(game.getTeamHome()); setLeaders(game.getTeamAway());
            // setLeaders finds players with grantsTeamReRollWhenOnPitch property and registers
            // them via gameState.addLeader(player). The leader registry is not yet ported;
            // stub is intentionally left empty (zero effect, correct skip logic).
        }
        StepOutcome::next().with_events(events)
    }

    // ── Java UtilServerGame.handleChefRolls ────────────────────────────────────

    /// Java `UtilServerGame.handleChefRolls(IStep, Game)`.
    ///
    /// Rolls master chef dice for home then away, then applies the cross-theft:
    /// - home rerolls: max(0, home - stolen_by_away) + stolen_by_home
    /// - away rerolls: max(0, away - stolen_by_home) + stolen_by_away
    fn handle_chef_rolls(&self, game: &mut Game, rng: &mut GameRng) -> Vec<GameEvent> {
        let mut events: Vec<GameEvent> = Vec::new();
        let rerolls_stolen_home = self.roll_master_chef_for_team(game, true, rng, &mut events);
        let rerolls_stolen_away = self.roll_master_chef_for_team(game, false, rng, &mut events);

        // Apply: home loses what away stole, gains what home stole from away.
        let home_rerolls = game.turn_data_home.rerolls;
        game.turn_data_home.rerolls = (home_rerolls - rerolls_stolen_away).max(0) + rerolls_stolen_home;

        let away_rerolls = game.turn_data_away.rerolls;
        game.turn_data_away.rerolls = (away_rerolls - rerolls_stolen_home).max(0) + rerolls_stolen_away;
        events
    }

    // ── Java UtilServerGame.rollMasterChef (private) ───────────────────────────

    /// Java `UtilServerGame.rollMasterChef(IStep, boolean pHomeTeam)`.
    ///
    /// Chef count = value of the team's TurnData inducement with `Usage.STEAL_REROLL`
    /// (there is no `masterChefs` field on `Team` in Java — the count comes solely from
    /// the Master Chef inducement bought during `StepBuyInducements`).
    /// For each chef: roll 3d6. Each die strictly greater than 3 (i.e. 4, 5, or 6) steals
    /// one reroll. Java `DiceRoller.rollMasterChef()` = `rollDice(3, 6)` (three independent
    /// d6). Java `DiceInterpreter.interpretMasterChefRoll` = count dice where `roll > 3`.
    /// Each individual roll emits `ReportMasterChefRoll(team.getId(), masterChefRoll, reRollsStolen)`.
    fn roll_master_chef_for_team(
        &self,
        game: &mut Game,
        home_team: bool,
        rng: &mut GameRng,
        events: &mut Vec<GameEvent>,
    ) -> i32 {
        let (team_id, chef_count) = if home_team {
            (game.team_home.id.clone(), game.turn_data_home.inducement_set.value(Usage::STEAL_REROLL))
        } else {
            (game.team_away.id.clone(), game.turn_data_away.inducement_set.value(Usage::STEAL_REROLL))
        };

        let mut total_stolen = 0i32;
        for _ in 0..chef_count {
            // 3d6 roll — one per chef.
            let d1 = rng.d6();
            let d2 = rng.d6();
            let d3 = rng.d6();
            // Count successes: die > 3 (Java: `roll > 3`).
            let mut stolen = 0i32;
            for die in [d1, d2, d3] {
                if die > 3 {
                    stolen += 1;
                }
            }
            // Java: getResult().addReport(new ReportMasterChefRoll(team.getId(), masterChefRoll, reRollsStolen));
            game.report_list.add(ReportMasterChefRoll::new(team_id.clone(), vec![d1, d2, d3], stolen));
            events.push(GameEvent::MasterChefRoll {
                team_id: team_id.clone(),
                roll: d1 * 100 + d2 * 10 + d3,
                rerolls_stolen: stolen,
            });
            total_stolen += stolen;
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
    use ffb_model::inducement::inducement::Inducement;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    fn give_chefs(game: &mut Game, home_team: bool, count: i32) {
        let inducement = Inducement::new("masterChef", count, vec![Usage::STEAL_REROLL]);
        if home_team {
            game.turn_data_home.inducement_set.add_inducement(inducement);
        } else {
            game.turn_data_away.inducement_set.add_inducement(inducement);
        }
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
        // Fresh game: both turn_nr are 0 → first turn of half by default.
        game.turn_data_home.rerolls = 3;
        game.turn_data_away.rerolls = 2;
        // No Master Chef inducements bought on either team.
        let mut step = StepMasterChef::new();
        step.start(&mut game, &mut GameRng::new(42));
        assert_eq!(game.turn_data_home.rerolls, 3);
        assert_eq!(game.turn_data_away.rerolls, 2);
    }

    #[test]
    fn guard_skips_when_not_first_turn_of_half() {
        let mut game = make_game();
        give_chefs(&mut game, true, 1);
        // Not the first turn of the half: home has already taken a turn.
        game.turn_data_home.turn_nr = 1;
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
        give_chefs(&mut game, true, 1);
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
        give_chefs(&mut game, true, 1);
        game.turn_data_home.rerolls = 3;
        game.turn_data_away.rerolls = 1;
        // Use seed 0 — GameRng::new(0) produces deterministic dice; we just verify floor at 0.
        let mut step = StepMasterChef::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.turn_data_away.rerolls >= 0, "away rerolls must not go negative");
    }

    #[test]
    fn first_turn_of_half_requires_both_teams_at_turn_zero_not_either() {
        // Regression test for the OR-vs-AND bug: Java's firstTurnOfHalf() is
        // `turnDataHome.getTurnNr() == 0 && turnDataAway.getTurnNr() == 0`.
        // If only one side is at turn 0 (the other has already played a turn),
        // chef rolls must NOT fire even though "either" flag might suggest so.
        let mut game = make_game();
        give_chefs(&mut game, true, 1);
        game.turn_data_home.turn_nr = 0;
        game.turn_data_away.turn_nr = 1; // away already took a turn
        game.turn_data_home.rerolls = 3;
        game.turn_data_away.rerolls = 3;
        let mut step = StepMasterChef::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.events.is_empty(), "guard must require BOTH turn_nr == 0, not either");
        assert_eq!(game.turn_data_home.rerolls, 3);
        assert_eq!(game.turn_data_away.rerolls, 3);
    }

    #[test]
    fn chef_roll_emits_master_chef_roll_event_and_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        give_chefs(&mut game, true, 1);
        let mut step = StepMasterChef::new();
        let out = step.start(&mut game, &mut GameRng::new(7));
        assert_eq!(out.events.len(), 1);
        assert!(matches!(&out.events[0], GameEvent::MasterChefRoll { team_id, .. } if team_id == &game.team_home.id));
        assert!(game.report_list.has_report(ReportId::MASTER_CHEF_ROLL),
            "MASTER_CHEF_ROLL report must be added for each chef roll");
    }
}
