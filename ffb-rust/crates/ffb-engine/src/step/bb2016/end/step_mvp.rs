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
/// extraMvp option → wired (+1 each per team). mvpNominations option → wired (get_int).
/// client-only: MVP dialog (DialogPlayerChoiceParameter) — headless auto-selects MVP without dialog.
use ffb_model::enums::SendToBoxReason;
use ffb_model::model::game::Game;
use ffb_model::report::report_most_valuable_players::ReportMostValuablePlayers;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::mechanic::spp_calc::SppCalc;
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
            // Java: if (UtilGameOption.isOptionEnabled(game, GameOptionId.EXTRA_MVP)) { +=1 each }
            if game.options.is_enabled("extraMvp") {
                self.nr_of_home_mvps += 1;
                self.nr_of_away_mvps += 1;
            }
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
        // Java: int mvpNominations = UtilGameOption.getIntOption(game, GameOptionId.MVP_NOMINATIONS)
        let mvp_nominations: i32 = game.options.get_int("mvpNominations").unwrap_or(0);
        let _ = mvp_nominations; // client-only: nominations dialog — headless always auto-picks randomly
        // Auto-roll: pick one random player per side using rng.
        // Java: DiceRoller.randomPlayerId(findPlayerIdsForMvp(team)) — filter killed players.
        if self.nr_of_home_choices < self.nr_of_home_mvps {
            let players: Vec<String> = game.team_home.players.iter()
                .filter(|p| !game.field_model.player_state(&p.id).map(|s| s.is_killed()).unwrap_or(false))
                .filter(|p| p.recovering_injury.is_none())
                .filter(|p| game.game_result.home.player_result(&p.id)
                    .map(|pr| pr.send_to_box_reason != Some(SendToBoxReason::NurglesRot))
                    .unwrap_or(true))
                .map(|p| p.id.clone())
                .collect();
            if !players.is_empty() {
                let idx = rng.range(players.len());
                let mvp_id = players[idx].clone();
                let mvp_spp = SppCalc::mvp_spp(game.rules);
                let pr = game.game_result.home.player_results.entry(mvp_id.clone()).or_default();
                pr.mvp = true;
                pr.player_awards += 1;
                pr.spp_gained += mvp_spp;
                self.home_players_mvp.push(mvp_id);
            }
            self.nr_of_home_choices += 1;
        }
        if self.nr_of_away_choices < self.nr_of_away_mvps {
            let players: Vec<String> = game.team_away.players.iter()
                .filter(|p| !game.field_model.player_state(&p.id).map(|s| s.is_killed()).unwrap_or(false))
                .filter(|p| p.recovering_injury.is_none())
                .filter(|p| game.game_result.away.player_result(&p.id)
                    .map(|pr| pr.send_to_box_reason != Some(SendToBoxReason::NurglesRot))
                    .unwrap_or(true))
                .map(|p| p.id.clone())
                .collect();
            if !players.is_empty() {
                let idx = rng.range(players.len());
                let mvp_id = players[idx].clone();
                let mvp_spp = SppCalc::mvp_spp(game.rules);
                let pr = game.game_result.away.player_results.entry(mvp_id.clone()).or_default();
                pr.mvp = true;
                pr.player_awards += 1;
                pr.spp_gained += mvp_spp;
                self.away_players_mvp.push(mvp_id);
            }
            self.nr_of_away_choices += 1;
        }
        // Java: if (fHomePlayersMvp.size() >= fNrOfHomeMvps || fAwayPlayersMvp.size() >= fNrOfAwayMvps)
        //         addReport(new ReportMostValuablePlayers(...))
        // Note: when no eligible players exist, fNrOfHomeMvps/Away is set to 0 by Java; treat
        // nr_of_choices >= nr_of_mvps (including 0-mvp case) as "done".
        let home_done = self.nr_of_home_choices >= self.nr_of_home_mvps
            || self.home_players_mvp.len() >= self.nr_of_home_mvps as usize;
        let away_done = self.nr_of_away_choices >= self.nr_of_away_mvps
            || self.away_players_mvp.len() >= self.nr_of_away_mvps as usize;
        if home_done || away_done {
            game.report_list.add(ReportMostValuablePlayers::new(
                self.home_players_mvp.clone(),
                self.away_players_mvp.clone(),
            ));
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
    fn extra_mvp_option_adds_one_to_each_team() {
        let mut game = make_game();
        game.options.set("extraMvp", "true");
        let mut step = StepMvp::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(step.nr_of_home_mvps, 2);
        assert_eq!(step.nr_of_away_mvps, 2);
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

    #[test]
    fn mvp_player_result_updated_with_spp_and_award() {
        use ffb_model::enums::{PlayerType, PlayerGender};
        use ffb_model::model::player::Player;
        use std::collections::HashSet;
        let mut game = make_game();
        // Add a player to the home team so there's someone to pick.
        game.team_home.players.push(Player {
            id: "p1".into(), name: "Player1".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
                    ..Default::default()
});
        let mut step = StepMvp::new();
        step.start(&mut game, &mut GameRng::new(0));
        // The home MVP should have spp_gained = 5 (BB2016 mvp_spp) and mvp = true.
        let mvp_id = step.home_players_mvp.first().cloned().unwrap();
        let pr = game.game_result.home.player_results.get(&mvp_id).unwrap();
        assert!(pr.mvp);
        assert_eq!(pr.player_awards, 1);
        assert_eq!(pr.spp_gained, 5); // SppCalc::mvp_spp(Rules::Bb2016) = 5
    }

    #[test]
    fn report_most_valuable_players_added_to_report_list() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        let mut step = StepMvp::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::MOST_VALUABLE_PLAYERS),
            "expected MOST_VALUABLE_PLAYERS in report list");
    }

    #[test]
    fn report_contains_home_and_away_mvp_ids() {
        use ffb_model::enums::{PlayerType, PlayerGender};
        use ffb_model::model::player::Player;
        use std::collections::HashSet;
        let mut game = make_game();
        game.team_home.players.push(Player {
            id: "h1".into(), name: "Home1".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.team_away.players.push(Player {
            id: "a1".into(), name: "Away1".into(), nr: 2, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        let mut step = StepMvp::new();
        step.start(&mut game, &mut GameRng::new(0));
        // The report list should have exactly one report.
        assert_eq!(game.report_list.size(), 1);
        // The step should have picked one home MVP and one away MVP.
        assert_eq!(step.home_players_mvp.len(), 1);
        assert_eq!(step.away_players_mvp.len(), 1);
    }

    #[test]
    fn killed_player_is_not_eligible_for_mvp() {
        use ffb_model::enums::{PlayerType, PlayerGender, PlayerState, PS_RIP};
        use ffb_model::model::player::Player;
        use std::collections::HashSet;
        let mut game = make_game();
        // Add one killed player to home team.
        game.team_home.players.push(Player {
            id: "dead1".into(), name: "Dead".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
                    ..Default::default()
});
        game.field_model.set_player_state("dead1", PlayerState::new(PS_RIP));
        let mut step = StepMvp::new();
        step.start(&mut game, &mut GameRng::new(0));
        // dead1 must not appear in home_players_mvp.
        assert!(!step.home_players_mvp.contains(&"dead1".to_string()));
    }
}
