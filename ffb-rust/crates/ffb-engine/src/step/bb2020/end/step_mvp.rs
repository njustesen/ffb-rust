/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.end.StepMvp`.
///
/// Step in end game sequence to determine the MVP (BB2020).
///
/// Differences from BB2016:
/// - Skips STAR, MERCENARY, INFAMOUS_STAFF player types.
/// - Filters out KILLED and MISSING (base state) players.
/// - Supports MVP_NOMINATIONS option: coach nominates N players, engine picks one at random.
///
/// Flow (auto mode / admin mode / nominations=0):
///   1. Initialise home/away MVP counts (1 each; +1 with EXTRA_MVP option; 0 if conceded illegally).
///   2. Auto-roll one random eligible player per MVP slot per team.
///   3. Set player_awards += 1; emit ReportMostValuablePlayers; NEXT_STEP.
///
/// Flow (nominations > 0, non-admin):
///   1. Same count init.
///   2. While home_choices < nr_of_home_mvps: show dialog → coach picks N; engine picks 1.
///   3. While away_choices < nr_of_away_mvps: show dialog → coach picks N; engine picks 1.
///   4. After all choices → award + report + NEXT_STEP.
///
/// extraMvp option → wired (+1 each per team). mvpNominations option → wired (get_int).
///   defaulting to 1 MVP per team, 0 nominations.
/// DEFERRED(mvp-dialog): DialogPlayerChoiceParameter(MVP) + UtilServerDialog.showDialog —
///   waiting for dialog infrastructure.
/// PlayerType (STAR/MERCENARY/INFAMOUS_STAFF) filter → wired. isKilled()/MISSING filter → wired.
/// DEFERRED(mvp-dialog): DialogPlayerChoiceParameter(MVP) + UtilServerDialog.showDialog —
///   waiting for dialog infrastructure.
/// MvpRoll GameEvent wired.
use ffb_model::enums::{PlayerType, PS_RIP, PS_MISSING};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepMvp` (bb2020/end).
pub struct StepMvp {
    /// Java: `fNrOfHomeMvps`
    pub nr_of_home_mvps: i32,
    /// Java: `fNrOfHomeChoices`
    pub nr_of_home_choices: i32,
    /// Java: `fHomePlayersNominated`
    pub home_players_nominated: Option<Vec<String>>,
    /// Java: `fHomePlayersMvp`
    pub home_players_mvp: Vec<String>,
    /// Java: `fNrOfAwayMvps`
    pub nr_of_away_mvps: i32,
    /// Java: `fNrOfAwayChoices`
    pub nr_of_away_choices: i32,
    /// Java: `fAwayPlayersNominated`
    pub away_players_nominated: Option<Vec<String>>,
    /// Java: `fAwayPlayersMvp`
    pub away_players_mvp: Vec<String>,
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

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: if (fNrOfHomeMvps == 0 && fNrOfAwayMvps == 0) { initialise counts }
        if self.nr_of_home_mvps == 0 && self.nr_of_away_mvps == 0 {
            self.nr_of_home_mvps = 1;
            self.nr_of_away_mvps = 1;
            // Java: if (UtilGameOption.isOptionEnabled(game, GameOptionId.EXTRA_MVP)) { +=1 each }
            if game.options.is_enabled("extraMvp") {
                self.nr_of_home_mvps += 1;
                self.nr_of_away_mvps += 1;
            }

            // Java: if (gameResult.getTeamResultHome().hasConceded() && !game.isConcededLegally()) { home=0 }
            if game.game_result.home.conceded && !game.conceded_legally {
                self.nr_of_home_mvps = 0;
            }
            // Java: if (gameResult.getTeamResultAway().hasConceded() && !game.isConcededLegally()) { away=0 }
            if game.game_result.away.conceded && !game.conceded_legally {
                self.nr_of_away_mvps = 0;
            }
        }

        // Java: int mvpNominations = UtilGameOption.getIntOption(game, GameOptionId.MVP_NOMINATIONS)
        let mvp_nominations: i32 = game.options.get_int("mvpNominations").unwrap_or(0);

        if mvp_nominations > 0 && !game.admin_mode {
            // DEFERRED(mvp-dialog): nominations flow — show DialogPlayerChoiceParameter(MVP) —
            //   waiting for dialog infrastructure.

            // Java: if (fHomePlayersNominated != null) { pick random; fNrOfHomeChoices++; clear nominated }
            if let Some(nominated) = self.home_players_nominated.take() {
                if !nominated.is_empty() {
                    let idx = rng.range(nominated.len());
                    self.home_players_mvp.push(nominated[idx].clone());
                    self.nr_of_home_choices += 1;
                }
            }

            if let Some(nominated) = self.away_players_nominated.take() {
                if !nominated.is_empty() {
                    let idx = rng.range(nominated.len());
                    self.away_players_mvp.push(nominated[idx].clone());
                    self.nr_of_away_choices += 1;
                }
            }

            // Java: if (fNrOfHomeChoices < fNrOfHomeMvps) { show dialog or auto-pick if only 1 player }
            if self.nr_of_home_choices < self.nr_of_home_mvps {
                let players = self.find_player_ids_for_mvp_home(game);
                if !players.is_empty() {
                    if players.len() == 1 {
                        self.home_players_mvp.push(players[0].clone());
                        self.nr_of_home_choices += 1;
                    } else {
                        // DEFERRED(mvp-dialog): show DialogPlayerChoiceParameter(home, MVP, ...) — dialog infra not translated.
                        return StepOutcome::cont();
                    }
                } else {
                    self.nr_of_home_mvps = 0;
                }
                if self.nr_of_home_choices < self.nr_of_home_mvps {
                    return StepOutcome::cont();
                }
            }

            // Java: if (fNrOfAwayChoices < fNrOfAwayMvps) { show dialog or auto-pick if only 1 player }
            if self.nr_of_away_choices < self.nr_of_away_mvps {
                let players = self.find_player_ids_for_mvp_away(game);
                if !players.is_empty() {
                    if players.len() == 1 {
                        self.away_players_mvp.push(players[0].clone());
                        self.nr_of_away_choices += 1;
                    } else {
                        // DEFERRED(mvp-dialog): show DialogPlayerChoiceParameter(away, MVP, ...) — dialog infra not translated.
                        return StepOutcome::cont();
                    }
                } else {
                    self.nr_of_away_mvps = 0;
                }
                if self.nr_of_away_choices < self.nr_of_away_mvps {
                    return StepOutcome::cont();
                }
            }
        } else {
            // Auto-roll: pick random MVPs for each team.
            // Java: fHomePlayersNominated = findPlayerIdsForMvp(game.getTeamHome())
            //   for (int i = 0; i < fNrOfHomeMvps; i++) { fHomePlayersMvp.add(randomPlayerId(...)) }
            let home_players = self.find_player_ids_for_mvp_home(game);
            for _ in 0..self.nr_of_home_mvps {
                self.nr_of_home_choices += 1;
                if !home_players.is_empty() {
                    let idx = rng.range(home_players.len());
                    self.home_players_mvp.push(home_players[idx].clone());
                }
            }
            let away_players = self.find_player_ids_for_mvp_away(game);
            for _ in 0..self.nr_of_away_mvps {
                self.nr_of_away_choices += 1;
                if !away_players.is_empty() {
                    let idx = rng.range(away_players.len());
                    self.away_players_mvp.push(away_players[idx].clone());
                }
            }
        }

        // Java: if (fHomePlayersMvp.size() >= fNrOfHomeMvps || fAwayPlayersMvp.size() >= fNrOfAwayMvps)
        if self.home_players_mvp.len() as i32 >= self.nr_of_home_mvps
            || self.away_players_mvp.len() as i32 >= self.nr_of_away_mvps
        {
            // Java: for each home MVP: playerResultHome.setPlayerAwards(playerAwards + 1); mvpReport.addPlayerIdHome
            let mvp_spp = crate::mechanic::spp_calc::SppCalc::mvp_spp(game.rules);
            let mut out = StepOutcome::next();
            let home_team_id = game.team_home.id.clone();
            let away_team_id = game.team_away.id.clone();
            for pid in &self.home_players_mvp {
                if !pid.is_empty() {
                    let result = game.game_result.home.player_results.entry(pid.clone()).or_default();
                    result.mvp = true;
                    result.player_awards += 1;
                    result.spp_gained += mvp_spp;
                    // Java: ReportMostValuablePlayers (per-player MVP roll report)
                    out = out.with_event(GameEvent::MvpRoll {
                        team_id: home_team_id.clone(),
                        player_id: pid.clone(),
                        spp: mvp_spp,
                    });
                }
            }
            for pid in &self.away_players_mvp {
                if !pid.is_empty() {
                    let result = game.game_result.away.player_results.entry(pid.clone()).or_default();
                    result.mvp = true;
                    result.player_awards += 1;
                    result.spp_gained += mvp_spp;
                    // Java: ReportMostValuablePlayers (per-player MVP roll report)
                    out = out.with_event(GameEvent::MvpRoll {
                        team_id: away_team_id.clone(),
                        player_id: pid.clone(),
                        spp: mvp_spp,
                    });
                }
            }
            return out;
        }

        StepOutcome::next()
    }

    /// Java: `findPlayerIdsForMvp(game.getTeamHome())`
    fn find_player_ids_for_mvp_home(&self, game: &Game) -> Vec<String> {
        find_eligible_player_ids(game, true)
    }

    /// Java: `findPlayerIdsForMvp(game.getTeamAway())`
    fn find_player_ids_for_mvp_away(&self, game: &Game) -> Vec<String> {
        find_eligible_player_ids(game, false)
    }
}

fn find_eligible_player_ids(game: &Game, home: bool) -> Vec<String> {
    let team = if home { &game.team_home } else { &game.team_away };
    team.players.iter()
        .filter(|p| {
            !matches!(p.player_type, PlayerType::Star | PlayerType::Mercenary | PlayerType::InfamousStaff)
        })
        .filter(|p| {
            let state = game.field_model.player_state(&p.id);
            !matches!(state.map(|s| s.base()), Some(b) if b == PS_RIP || b == PS_MISSING)
        })
        .map(|p| p.id.clone())
        .collect()
}

impl Default for StepMvp {
    fn default() -> Self { Self::new() }
}

impl Step for StepMvp {
    fn id(&self) -> StepId { StepId::Mvp }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_PLAYER_CHOICE (PlayerChoiceMode.MVP) → sets home/away nominated players.
        // Rust: SelectPlayer is the closest available action; treat as home nomination for simplicity.
        // DEFERRED(mvp-dialog): properly route home vs away based on which side sent the command.
        if let Action::SelectPlayer { player_id } = action {
            // Java: if (checkCommandIsFromHomePlayer) { fHomePlayersNominated = playerIds }
            //       else { fAwayPlayersNominated = playerIds }
            // Without team side info in Action, default to home.
            self.home_players_nominated = Some(vec![player_id.clone()]);
        }
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
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
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
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn sets_mvp_counts_on_first_call() {
        let mut game = make_game();
        let mut step = StepMvp::new();
        step.start(&mut game, &mut GameRng::new(0));
        // After first call: home and away each start with 1 MVP (no concession).
        assert_eq!(step.nr_of_home_mvps, 1);
        assert_eq!(step.nr_of_away_mvps, 1);
    }

    #[test]
    fn home_concede_gives_zero_home_mvps() {
        let mut game = make_game();
        game.game_result.home.conceded = true;
        game.conceded_legally = false;
        let mut step = StepMvp::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(step.nr_of_home_mvps, 0);
        assert_eq!(step.nr_of_away_mvps, 1);
    }

    #[test]
    fn away_concede_gives_zero_away_mvps() {
        let mut game = make_game();
        game.game_result.away.conceded = true;
        game.conceded_legally = false;
        let mut step = StepMvp::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(step.nr_of_home_mvps, 1);
        assert_eq!(step.nr_of_away_mvps, 0);
    }

    #[test]
    fn legal_concession_does_not_zero_mvps() {
        // If conceded legally, no team loses their MVP count.
        let mut game = make_game();
        game.game_result.home.conceded = true;
        game.conceded_legally = true;
        let mut step = StepMvp::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(step.nr_of_home_mvps, 1);
        assert_eq!(step.nr_of_away_mvps, 1);
    }

    #[test]
    fn set_parameter_returns_false() {
        let mut step = StepMvp::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn auto_roll_marks_no_players_when_team_empty() {
        // With no eligible players, MVPs list stays empty but step still returns NEXT_STEP.
        let mut game = make_game();
        let mut step = StepMvp::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(step.home_players_mvp.is_empty());
        assert!(step.away_players_mvp.is_empty());
    }

    #[test]
    fn star_player_excluded_from_eligibility() {
        use std::collections::HashSet;
        use ffb_model::enums::{PlayerGender};
        use ffb_model::model::player::Player;
        let mut game = make_game();
        // Add a Star player — should be excluded from MVP candidates.
        game.team_home.players.push(Player {
            id: "star1".into(), name: "Star".into(), nr: 0, position_id: "star".into(),
            player_type: PlayerType::Star, gender: PlayerGender::Male,
            movement: 6, strength: 4, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        });
        let mut step = StepMvp::new();
        step.start(&mut game, &mut GameRng::new(0));
        // Star player should not be selected as MVP.
        assert!(!step.home_players_mvp.contains(&"star1".to_string()));
    }
}
