/// 1:1 translation of `com.fumbbl.ffb.server.step.phase.inducement.StepRiotousRookies`.
///
/// Java: for each team, if their InducementSet contains an inducement with `Usage::ADD_LINEMEN`,
/// roll 2d6+1 per inducement value to determine how many riotous rookies to add. Each rookie is
/// a dynamically created `RosterPlayer` at the `riotousRookiesPosition()` from the roster, given
/// the Loner skill, `PlayerStatus::JOURNEYMAN`, and a randomly generated name via an HTTP call to
/// the FUMBBL name-generator service.
///
/// DEFERRED dependencies (all gated on infra not yet ported):
/// - `TurnData::inducement_set` / `InducementSet` stub — finding ADD_LINEMEN inducements.
/// - `GameMechanic::riotousRookiesPosition()` — finding the correct roster position.
/// - `RosterPlayer` creation / `Team::addPlayer()` — dynamic player creation.
/// - HTTP name-generator (`UtilServerHttpClient`) — `rookieName()`.
/// - `UtilBox::putPlayerIntoBox()` — box placement.
/// - Server communication (`sendAddPlayer`) — network I/O.
///
/// When all DEFERRED items are resolved, `hire_riotous_rookies()` and `riotous_player()` can
/// be fully implemented. Until then the step calls `start()` and immediately returns `NextStep`.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepId, StepOutcome};

/// Java: `StepRiotousRookies` (no instance fields in Java — all work in `start()`).
pub struct StepRiotousRookies;

impl StepRiotousRookies {
    pub fn new() -> Self { Self }
}

impl Default for StepRiotousRookies {
    fn default() -> Self { Self::new() }
}

impl Step for StepRiotousRookies {
    fn id(&self) -> StepId { StepId::RiotousRookies }

    /// Java: `start()` — calls `hireRiotousRookies` for both teams, then `NEXT_STEP`.
    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.hire_riotous_rookies_for_team(game, rng, true);
        self.hire_riotous_rookies_for_team(game, rng, false);
        StepOutcome::next()
    }

    fn handle_command(&mut self, _action: &Action, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        StepOutcome::cont()
    }
}

impl StepRiotousRookies {
    /// Java: `hireRiotousRookies(TurnData, Team)`.
    ///
    /// Finds the ADD_LINEMEN inducement (if any) in the team's InducementSet and hires rookies.
    ///
    /// DEFERRED: requires `TurnData::inducement_set`, `InducementSet::inducement_mapping`,
    /// `InducementType::has_usage(Usage::ADD_LINEMEN)`, and `GameMechanic::riotousRookiesPosition()`.
    fn hire_riotous_rookies_for_team(&self, _game: &mut Game, _rng: &mut GameRng, _home: bool) {
        // DEFERRED(inducement_set): iterate TurnData.inducement_set.inducement_mapping
        //   to find an InducementType with Usage::ADD_LINEMEN.
        // DEFERRED(mechanic): call GameMechanic::riotousRookiesPosition(team.roster).
        // DEFERRED(player_creation): call self.riotous_player() for each rookie.
        // DEFERRED(report): push ReportRiotousRookies event.
    }

    /// Java: `riotousPlayer(Game, Team, int index, RosterPosition)`.
    ///
    /// Creates a new RosterPlayer at the given position with Loner skill,
    /// a generated name, and JOURNEYMAN status, then places them in the box.
    ///
    /// DEFERRED: requires RosterPlayer creation, SkillFactory, UtilBox, server communication.
    fn riotous_player(&self, _game: &mut Game, _rng: &mut GameRng, _home: bool, _index: i32) {
        // DEFERRED(player_creation): new RosterPlayer with Loner, JOURNEYMAN status.
        // DEFERRED(name_generator): fetch name via UtilServerHttpClient (HTTP).
        // DEFERRED(box_placement): UtilBox::putPlayerIntoBox().
        // DEFERRED(communication): server.sendAddPlayer().
    }

    /// Java: `rookieName(String generator, PlayerGender gender, String fallback)`.
    ///
    /// Fetches a player name from the FUMBBL name-generator service.
    /// Falls back to the provided string if the HTTP call fails.
    ///
    /// DEFERRED: requires UtilServerHttpClient (HTTP) and ServerUrlProperty access.
    fn rookie_name(&self, _generator: &str, _fallback: &str) -> String {
        // DEFERRED(http): call ServerUrlProperty::FUMBBL_NAMEGENERATOR_BASE + generator/gender.
        _fallback.to_string()
    }

    /// Java: `rollRiotousRookies()` → two d3 dice + 1 = number of rookies to hire.
    ///
    /// Java: `DiceRoller.rollRiotousRookies()` → `{ rollDice(3), rollDice(3) }`.
    /// `rookies = rookiesRoll[0] + rookiesRoll[1] + 1` → range 3–7.
    fn roll_rookies_count(rng: &mut GameRng) -> i32 {
        let roll = rng.roll_riotous_rookies();
        roll[0] + roll[1] + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn id_is_riotous_rookies() {
        let step = StepRiotousRookies::new();
        assert_eq!(step.id(), StepId::RiotousRookies);
    }

    #[test]
    fn start_returns_next_step() {
        let mut step = StepRiotousRookies::new();
        let mut game = make_game();
        let mut rng = GameRng::new(42);
        let outcome = step.start(&mut game, &mut rng);
        assert_eq!(outcome.action, StepAction::NextStep);
    }

    #[test]
    fn handle_command_returns_continue() {
        let mut step = StepRiotousRookies::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.handle_command(&Action::EndTurn, &mut game, &mut rng);
        assert_eq!(outcome.action, StepAction::Continue);
    }

    #[test]
    fn default_creates_same_as_new() {
        let s = StepRiotousRookies::default();
        assert_eq!(s.id(), StepId::RiotousRookies);
    }

    /// Java: roll = rollRiotousRookies()[0] + rollRiotousRookies()[1] + 1.
    /// Two d3 dice → minimum 1+1+1 = 3, maximum 3+3+1 = 7.
    #[test]
    fn roll_rookies_count_is_at_least_three() {
        // Run 100 seeds and verify count is always >= 3 (d3+d3+1 minimum = 3).
        for seed in 0..100u64 {
            let mut rng = GameRng::new(seed);
            let count = StepRiotousRookies::roll_rookies_count(&mut rng);
            assert!(count >= 3, "seed {seed}: count {count} < 3");
        }
    }

    #[test]
    fn roll_rookies_count_is_at_most_seven() {
        // d3+d3+1 maximum = 3+3+1 = 7.
        for seed in 0..100u64 {
            let mut rng = GameRng::new(seed);
            let count = StepRiotousRookies::roll_rookies_count(&mut rng);
            assert!(count <= 7, "seed {seed}: count {count} > 7");
        }
    }

    #[test]
    fn rookie_name_returns_fallback_when_deferred() {
        let step = StepRiotousRookies::new();
        let name = step.rookie_name("human", "RiotousRookie #0");
        assert_eq!(name, "RiotousRookie #0");
    }
}
