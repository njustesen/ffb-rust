/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.end.StepMvp`.
///
/// Step in end game sequence to determine the MVP (BB2016).
/// - Sets nr_of_home_mvps = nr_of_away_mvps = 1 (or 2 with EXTRA_MVP option).
/// - If home/away conceded illegally: winning side gets +1, losers get 0.
/// - If MVP_NOMINATIONS > 0 and not admin mode: shows player-choice dialog per team.
///   - Each coach nominates `mvp_nominations` players; engine picks one at random.
/// - Otherwise: auto-rolls random MVP for each team.
/// - Records player awards and emits ReportMostValuablePlayers.
///
/// TODO(Mvp-options): GameOptionId.EXTRA_MVP, MVP_NOMINATIONS deferred.
/// TODO(Mvp-dialog): DialogPlayerChoiceParameter deferred.
/// TODO(Mvp-randomPlayerId): DiceRoller.randomPlayerId deferred.
/// TODO(Mvp-findPlayers): player state filtering (killed, recovering injury, NurglesRot) deferred.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepMvp` (bb2016/end).
pub struct StepMvp {
    /// Java: `fNrOfHomeMvps`
    nr_of_home_mvps: i32,
    /// Java: `fNrOfHomeChoices`
    nr_of_home_choices: i32,
    /// Java: `fHomePlayersNominated`
    home_players_nominated: Vec<String>,
    /// Java: `fHomePlayersMvp`
    home_players_mvp: Vec<String>,
    /// Java: `fNrOfAwayMvps`
    nr_of_away_mvps: i32,
    /// Java: `fNrOfAwayChoices`
    nr_of_away_choices: i32,
    /// Java: `fAwayPlayersNominated`
    away_players_nominated: Vec<String>,
    /// Java: `fAwayPlayersMvp`
    away_players_mvp: Vec<String>,
}

impl StepMvp {
    pub fn new() -> Self {
        Self {
            nr_of_home_mvps: 0,
            nr_of_home_choices: 0,
            home_players_nominated: Vec::new(),
            home_players_mvp: Vec::new(),
            nr_of_away_mvps: 0,
            nr_of_away_choices: 0,
            away_players_nominated: Vec::new(),
            away_players_mvp: Vec::new(),
        }
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Initialise MVP counts on first call.
        if self.nr_of_home_mvps == 0 && self.nr_of_away_mvps == 0 {
            self.nr_of_home_mvps = 1;
            self.nr_of_away_mvps = 1;
            // TODO(Mvp-options): EXTRA_MVP → +1 each.
            // Illegal concession bonus.
            if game.game_result.home.conceded && !game.conceded_legally {
                self.nr_of_home_mvps = 0;
                self.nr_of_away_mvps += 1;
            }
            if game.game_result.away.conceded && !game.conceded_legally {
                self.nr_of_home_mvps += 1;
                self.nr_of_away_mvps = 0;
            }
        }
        // TODO(Mvp-nominations): MVP_NOMINATIONS dialog loop.
        // Auto-roll: pick one random player per side using rng.
        if self.nr_of_home_choices < self.nr_of_home_mvps {
            let players: Vec<String> = game.team_home.players.iter().map(|p| p.id.clone()).collect();
            if !players.is_empty() {
                let idx = rng.range(players.len());
                let mvp_id = players[idx].clone();
                self.home_players_mvp.push(mvp_id);
            }
            self.nr_of_home_choices += 1;
        }
        if self.nr_of_away_choices < self.nr_of_away_mvps {
            let players: Vec<String> = game.team_away.players.iter().map(|p| p.id.clone()).collect();
            if !players.is_empty() {
                let idx = rng.range(players.len());
                let mvp_id = players[idx].clone();
                self.away_players_mvp.push(mvp_id);
            }
            self.nr_of_away_choices += 1;
        }
        StepOutcome::next()
    }
}

impl Default for StepMvp {
    fn default() -> Self { Self::new() }
}

impl Step for StepMvp {
    fn id(&self) -> StepId { StepId::Mvp }

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
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    #[test]
    fn id_is_mvp() {
        assert_eq!(StepMvp::new().id(), StepId::Mvp);
    }

    #[test]
    fn start_returns_next_step() {
        let mut game = make_game();
        let mut step = StepMvp::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
    }

    #[test]
    fn sets_mvp_counts_on_first_call() {
        let mut game = make_game();
        let mut step = StepMvp::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(step.nr_of_home_mvps >= 1);
        assert!(step.nr_of_away_mvps >= 1);
    }

    #[test]
    fn home_concede_gives_away_extra_mvp() {
        let mut game = make_game();
        game.game_result.home.conceded = true;
        game.conceded_legally = false;
        let mut step = StepMvp::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(step.nr_of_home_mvps, 0);
        assert_eq!(step.nr_of_away_mvps, 2);
    }

    #[test]
    fn away_concede_gives_home_extra_mvp() {
        let mut game = make_game();
        game.game_result.away.conceded = true;
        game.conceded_legally = false;
        let mut step = StepMvp::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(step.nr_of_home_mvps, 2);
        assert_eq!(step.nr_of_away_mvps, 0);
    }
}
