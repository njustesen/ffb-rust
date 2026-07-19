use ffb_model::enums::PlayerType;
use ffb_model::enums::{PS_RIP, PS_MISSING};
use ffb_model::model::game::Game;
use ffb_model::report::report_most_valuable_players::ReportMostValuablePlayers;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::StepId;

const MVP_NOMINATIONS_OPTION: &str = "mvpNominations";

/// Determines MVP(s) at end of game: 1 per team (2 if EXTRA_MVP option), adjusted for
/// illegal concession. When MVP_NOMINATIONS > 0 and not in admin mode, defers to a
/// player-choice dialog per team (headless: waits for `Action::SelectPlayer`); otherwise
/// auto-rolls a random eligible player per MVP slot.
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

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_PLAYER_CHOICE (PlayerChoiceMode.MVP) → sets home/away nominated players.
        // client-only: proper home/away routing requires dialog side-tracking; headless never receives this action
        if let Action::SelectPlayer { player_id } = action {
            self.home_players_nominated = None;
            self.away_players_nominated = None;
            self.home_players_nominated = Some(vec![player_id.clone()]);
        }
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

        // Java: int mvpNominations = UtilGameOption.getIntOption(game, GameOptionId.MVP_NOMINATIONS);
        let mvp_nominations: i32 = game.options.get_int(MVP_NOMINATIONS_OPTION).unwrap_or(0);

        if mvp_nominations > 0 && !game.admin_mode {
            // Java: if (fHomePlayersNominated != null) { random pick; fNrOfHomeChoices++; clear }
            if let Some(nominated) = self.home_players_nominated.take() {
                if !nominated.is_empty() {
                    let idx = (rng.die(nominated.len() as u32) - 1) as usize;
                    self.home_players_mvp.push(nominated[idx].clone());
                }
                self.nr_of_home_choices += 1;
            }

            // Java: if (fAwayPlayersNominated != null) { random pick; fNrOfAwayChoices++; clear }
            if let Some(nominated) = self.away_players_nominated.take() {
                if !nominated.is_empty() {
                    let idx = (rng.die(nominated.len() as u32) - 1) as usize;
                    self.away_players_mvp.push(nominated[idx].clone());
                }
                self.nr_of_away_choices += 1;
            }

            // Java: if (fNrOfHomeChoices < fNrOfHomeMvps) { ... ; return; }
            if self.nr_of_home_choices < self.nr_of_home_mvps {
                let players_for_mvp = find_eligible_player_ids(game, true);
                if !players_for_mvp.is_empty() {
                    if players_for_mvp.len() == 1 {
                        self.home_players_mvp.push(players_for_mvp[0].clone());
                        self.nr_of_home_choices += 1;
                    } else {
                        // client-only: DialogPlayerChoiceParameter(home, MVP) — headless waits for SelectPlayer
                        return StepOutcome::cont();
                    }
                } else {
                    self.nr_of_home_mvps = 0;
                }
                return StepOutcome::cont();
            }

            // Java: if (fNrOfAwayChoices < fNrOfAwayMvps) { ... ; return; }
            if self.nr_of_away_choices < self.nr_of_away_mvps {
                let players_for_mvp = find_eligible_player_ids(game, false);
                if !players_for_mvp.is_empty() {
                    if players_for_mvp.len() == 1 {
                        self.away_players_mvp.push(players_for_mvp[0].clone());
                        self.nr_of_away_choices += 1;
                    } else {
                        // client-only: DialogPlayerChoiceParameter(away, MVP) — headless waits for SelectPlayer
                        return StepOutcome::cont();
                    }
                } else {
                    self.nr_of_away_mvps = 0;
                }
                return StepOutcome::cont();
            }
        } else {
            // Java: else branch — auto-roll one random eligible player per MVP slot per team.
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

        // Java: getResult().addReport(mvpReport) — emits ReportMostValuablePlayers
        game.report_list.add(ReportMostValuablePlayers::new(
            self.home_players_mvp.clone(),
            self.away_players_mvp.clone(),
        ));

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
            is_big_guy: false,
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

    /// With MVP_NOMINATIONS > 0 and non-admin mode, the step must defer to a player-choice
    /// dialog (StepOutcome::cont()) rather than immediately auto-rolling from the whole team,
    /// matching Java's `if ((mvpNominations > 0) && !game.isAdminMode())` dialog branch.
    /// Before the fix, the Rust code ignored mvpNominations/adminMode entirely and always
    /// took the Java "else" auto-roll path.
    #[test]
    fn mvp_nominations_option_defers_to_dialog_when_multiple_eligible() {
        let mut game = make_game_with_players();
        game.options.set("mvpNominations", "2");
        let mut step = StepMvp::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue, "should wait for a player-choice dialog");
        // No MVPs should have been auto-assigned yet.
        assert!(step.home_players_mvp.is_empty());
        let home_mvps = game.game_result.home.player_results.values().filter(|pr| pr.mvp).count();
        assert_eq!(home_mvps, 0);
    }

    /// After a SelectPlayer command resolves the home nomination, home MVP is picked from
    /// the nominated players and the step proceeds to resolve away nominations.
    #[test]
    fn mvp_nominations_resolves_after_select_player_command() {
        let mut game = make_game_with_players();
        game.options.set("mvpNominations", "2");
        let mut step = StepMvp::new();
        step.start(&mut game, &mut GameRng::new(0));
        // Simulate coach nominating "h1" for home MVP.
        let out = step.handle_command(&Action::SelectPlayer { player_id: "h1".into() }, &mut game, &mut GameRng::new(0));
        assert_eq!(step.home_players_mvp, vec!["h1".to_string()]);
        // Away side still has 2 eligible players, so it now waits on the away dialog.
        assert_eq!(out.action, StepAction::Continue);
    }

    /// mvpNominations > 0 but admin mode is active: Java's `!game.isAdminMode()` guard means
    /// the dialog branch is skipped and the else (auto-roll) branch runs instead.
    #[test]
    fn mvp_nominations_ignored_in_admin_mode() {
        let mut game = make_game_with_players();
        game.options.set("mvpNominations", "2");
        game.admin_mode = true;
        let mut step = StepMvp::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(step.home_players_mvp.len(), 1);
        assert_eq!(step.away_players_mvp.len(), 1);
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

    /// Report MOST_VALUABLE_PLAYERS is added to report_list after normal game.
    #[test]
    fn report_most_valuable_players_added_to_report_list() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game_with_players();
        let mut step = StepMvp::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(
            game.report_list.has_report(ReportId::MOST_VALUABLE_PLAYERS),
            "expected MOST_VALUABLE_PLAYERS in report_list"
        );
    }

    /// Report contains the selected home and away MVP player IDs.
    #[test]
    fn report_contains_home_and_away_mvp_ids() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game_with_players();
        let mut step = StepMvp::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::MOST_VALUABLE_PLAYERS));
        // One MVP selected per side.
        assert_eq!(step.home_players_mvp.len(), 1);
        assert_eq!(step.away_players_mvp.len(), 1);
    }
}
