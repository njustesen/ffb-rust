use ffb_model::enums::PlayerType;
use ffb_model::enums::{PS_RIP, PS_MISSING};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::StepId;

/// Determines MVP(s) at end of game: 1 per team (2 if EXTRA_MVP option), adjusted for
/// illegal concession. Uses random selection (nomination dialog is a future enhancement).
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.end.StepMvp`.
pub struct StepMvp {
    /// Java: fNrOfHomeMvps
    nr_of_home_mvps: i32,
    /// Java: fNrOfHomeChoices
    nr_of_home_choices: i32,
    /// Java: fHomePlayersNominated (String[]) — nomination dialog round-trip
    home_players_nominated: Option<Vec<String>>,
    /// Java: fHomePlayersMvp (List<String>)
    home_players_mvp: Vec<String>,
    /// Java: fNrOfAwayMvps
    nr_of_away_mvps: i32,
    /// Java: fNrOfAwayChoices
    nr_of_away_choices: i32,
    /// Java: fAwayPlayersNominated (String[]) — nomination dialog round-trip
    away_players_nominated: Option<Vec<String>>,
    /// Java: fAwayPlayersMvp (List<String>)
    away_players_mvp: Vec<String>,
}

impl StepMvp {
    pub fn new() -> Self {
        Self {
            nr_of_home_mvps: 0,
            nr_of_home_choices: 0,
            home_players_nominated: None,
            home_players_mvp: Vec::new(),
            nr_of_away_mvps: 0,
            nr_of_away_choices: 0,
            away_players_nominated: None,
            away_players_mvp: Vec::new(),
        }
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
}

impl StepMvp {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if self.nr_of_home_mvps == 0 && self.nr_of_away_mvps == 0 {
            self.nr_of_home_mvps = 1;
            self.nr_of_away_mvps = 1;
            if game.options.is_enabled("extraMvp") {
                self.nr_of_home_mvps += 1;
                self.nr_of_away_mvps += 1;
            }
            if game.game_result.home.conceded && !game.conceded_legally {
                self.nr_of_home_mvps = 0;
                self.nr_of_away_mvps += 1;
            }
            if game.game_result.away.conceded && !game.conceded_legally {
                self.nr_of_away_mvps = 0;
                self.nr_of_home_mvps += 1;
            }
        }

        // Random selection (nomination dialog omitted — parity uses random agent).
        let home_eligible = find_eligible_player_ids(game, true);
        for _ in self.nr_of_home_choices..self.nr_of_home_mvps {
            if !home_eligible.is_empty() {
                let idx = (rng.die(home_eligible.len() as u32) - 1) as usize;
                self.home_players_mvp.push(home_eligible[idx].clone());
            }
            self.nr_of_home_choices += 1;
        }

        let away_eligible = find_eligible_player_ids(game, false);
        for _ in self.nr_of_away_choices..self.nr_of_away_mvps {
            if !away_eligible.is_empty() {
                let idx = (rng.die(away_eligible.len() as u32) - 1) as usize;
                self.away_players_mvp.push(away_eligible[idx].clone());
            }
            self.nr_of_away_choices += 1;
        }

        let mvp_spp = crate::mechanic::spp_calc::SppCalc::mvp_spp(game.rules);
        for pid in &self.home_players_mvp {
            let pr = game.game_result.home.player_results.entry(pid.clone()).or_default();
            pr.mvp = true;
            pr.player_awards += 1;
            pr.spp_gained += mvp_spp;
        }
        for pid in &self.away_players_mvp {
            let pr = game.game_result.away.player_results.entry(pid.clone()).or_default();
            pr.mvp = true;
            pr.player_awards += 1;
            pr.spp_gained += mvp_spp;
        }

        StepOutcome::next()
    }
}

fn find_eligible_player_ids(game: &Game, home: bool) -> Vec<String> {
    let team = if home { &game.team_home } else { &game.team_away };
    team.players.iter()
        .filter(|p| {
            !matches!(p.player_type, PlayerType::Star | PlayerType::Mercenary | PlayerType::InfamousStaff)
        })
        .filter(|p| {
            // Exclude killed (PS_RIP) and missing players. Java: isKilled() || base == MISSING.
            let state = game.field_model.player_state(&p.id);
            !matches!(state.map(|s| s.base()), Some(b) if b == PS_RIP || b == PS_MISSING)
        })
        .map(|p| p.id.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{PlayerType, PlayerGender, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::game_result::PlayerResult;

    fn make_player(id: &str, player_type: PlayerType) -> Player {
        Player {
            id: id.into(),
            name: id.into(),
            nr: 1,
            position_id: "lineman".into(),
            player_type,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![],
            extra_skills: vec![],
            temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0,
            stat_injuries: vec![],
            current_spps: 0,
            career_spps: 0,
            race: None,
            ..Default::default()
        }
    }

    fn make_game_with_players() -> Game {
        let mut home = test_team("home", 0);
        let mut away = test_team("away", 0);
        home.players.push(make_player("h1", PlayerType::Regular));
        home.players.push(make_player("h2", PlayerType::Regular));
        away.players.push(make_player("a1", PlayerType::Regular));
        away.players.push(make_player("a2", PlayerType::Regular));
        let mut game = Game::new(home, away, Rules::Bb2025);
        // Pre-populate player_results so MVP marking has somewhere to write.
        game.game_result.home.player_results.insert("h1".into(), PlayerResult::default());
        game.game_result.home.player_results.insert("h2".into(), PlayerResult::default());
        game.game_result.away.player_results.insert("a1".into(), PlayerResult::default());
        game.game_result.away.player_results.insert("a2".into(), PlayerResult::default());
        game
    }

    /// Normal game: one MVP is selected per side and marked in player_results.
    #[test]
    fn normal_game_assigns_one_mvp_per_team() {
        let mut game = make_game_with_players();
        let mut step = StepMvp::new();
        let out = step.start(&mut game, &mut GameRng::new(42));
        assert_eq!(out.action, StepAction::NextStep);
        let home_mvps: Vec<_> = game.game_result.home.player_results.values().filter(|pr| pr.mvp).collect();
        let away_mvps: Vec<_> = game.game_result.away.player_results.values().filter(|pr| pr.mvp).collect();
        assert_eq!(home_mvps.len(), 1, "exactly one home MVP");
        assert_eq!(away_mvps.len(), 1, "exactly one away MVP");
    }

    /// Illegal home concession: home gets 0 MVPs, away gets 2.
    #[test]
    fn illegal_home_concession_gives_away_extra_mvp() {
        let mut game = make_game_with_players();
        game.game_result.home.conceded = true;
        game.conceded_legally = false;
        let mut step = StepMvp::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(step.nr_of_home_mvps, 0);
        assert_eq!(step.nr_of_away_mvps, 2);
        let home_mvps = game.game_result.home.player_results.values().filter(|pr| pr.mvp).count();
        assert_eq!(home_mvps, 0, "conceding home team has no MVPs");
    }

    /// Illegal away concession: away gets 0 MVPs, home gets 2.
    #[test]
    fn illegal_away_concession_gives_home_extra_mvp() {
        let mut game = make_game_with_players();
        game.game_result.away.conceded = true;
        game.conceded_legally = false;
        let mut step = StepMvp::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(step.nr_of_away_mvps, 0);
        assert_eq!(step.nr_of_home_mvps, 2);
        let away_mvps = game.game_result.away.player_results.values().filter(|pr| pr.mvp).count();
        assert_eq!(away_mvps, 0, "conceding away team has no MVPs");
    }

    /// Star and Mercenary players are excluded from MVP eligibility.
    #[test]
    fn star_and_mercenary_excluded_from_eligibility() {
        let mut home = test_team("home", 0);
        let away = test_team("away", 0);
        // Only star and mercenary players — home should get 0 eligible.
        home.players.push(make_player("s1", PlayerType::Star));
        home.players.push(make_player("m1", PlayerType::Mercenary));
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.game_result.home.player_results.insert("s1".into(), PlayerResult::default());
        game.game_result.home.player_results.insert("m1".into(), PlayerResult::default());
        let mut step = StepMvp::new();
        step.start(&mut game, &mut GameRng::new(0));
        let home_mvps = game.game_result.home.player_results.values().filter(|pr| pr.mvp).count();
        assert_eq!(home_mvps, 0, "star/mercenary not eligible for MVP");
    }

    /// nr_of_mvps initialised to 1 each on first call (zero guard).
    #[test]
    fn mvp_counts_initialised_from_zero_on_first_call() {
        let mut game = make_game_with_players();
        let mut step = StepMvp::new();
        assert_eq!(step.nr_of_home_mvps, 0);
        assert_eq!(step.nr_of_away_mvps, 0);
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(step.nr_of_home_mvps, 1);
        assert_eq!(step.nr_of_away_mvps, 1);
    }
}
