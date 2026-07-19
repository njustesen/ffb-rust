/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.end.StepPlayerLoss`.
///
/// For illegal concession: any player with ≥51 current SPPs (`fCurrentSpps`/`current_spps`,
/// the player's SPP total, not SPP gained this game) risks defecting.
/// Java: `DiceInterpreter.isPlayerDefecting(roll)` = roll > 0 && roll < 4 (i.e., 1–3 on D6).
use ffb_model::model::game::Game;
use ffb_model::report::report_defecting_players::ReportDefectingPlayers;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

const SPP_DEFECTION_THRESHOLD: i32 = 51;

/// Java: `StepPlayerLoss` (bb2016/end).
pub struct StepPlayerLoss;

impl StepPlayerLoss {
    pub fn new() -> Self { Self }

    /// Java: `DiceInterpreter.isPlayerDefecting(roll)` = `roll > 0 && roll < 4`.
    fn is_player_defecting(roll: i32) -> bool {
        roll > 0 && roll < 4
    }

    fn execute_step(game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let conceding_home = game.game_result.home.conceded && !game.conceded_legally;
        let conceding_away = game.game_result.away.conceded && !game.conceded_legally;

        if !conceding_home && !conceding_away {
            return StepOutcome::next();
        }

        let player_ids: Vec<String> = {
            let team = if conceding_home { &game.team_home } else { &game.team_away };
            let results = if conceding_home {
                &game.game_result.home.player_results
            } else {
                &game.game_result.away.player_results
            };
            team.players.iter()
                .filter(|p| {
                    results.get(&p.id)
                        .map(|r| r.current_spps >= SPP_DEFECTION_THRESHOLD)
                        .unwrap_or(false)
                })
                .map(|p| p.id.clone())
                .collect()
        };

        // Java: collect defectingPlayerIds, defectingRolls, defectingFlags in parallel
        let mut defecting_rolls: Vec<i32> = Vec::new();
        let mut defecting_flags: Vec<bool> = Vec::new();

        for pid in &player_ids {
            let roll = rng.d6();
            let defecting = Self::is_player_defecting(roll);
            defecting_rolls.push(roll);
            defecting_flags.push(defecting);
            let results = if conceding_home {
                &mut game.game_result.home.player_results
            } else {
                &mut game.game_result.away.player_results
            };
            if let Some(pr) = results.get_mut(pid) {
                pr.defecting = defecting;
            }
        }

        // Java: if (defectingPlayerIds.size() > 0) { getResult().addReport(new ReportDefectingPlayers(...)) }
        if !player_ids.is_empty() {
            game.report_list.add(ReportDefectingPlayers::new(player_ids, defecting_rolls, defecting_flags));
        }

        StepOutcome::next()
    }
}

impl Default for StepPlayerLoss {
    fn default() -> Self { Self::new() }
}

impl Step for StepPlayerLoss {
    fn id(&self) -> StepId { StepId::PlayerLoss }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        Self::execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        Self::execute_step(game, rng)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use ffb_model::enums::{Rules, PlayerType, PlayerGender};
    use ffb_model::model::player::Player;
    use ffb_model::model::game_result::PlayerResult;

    fn make_player(id: &str) -> Player {
        Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        }
    }

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    #[test]
    fn id_is_player_loss() {
        assert_eq!(StepPlayerLoss::new().id(), StepId::PlayerLoss);
    }

    #[test]
    fn is_player_defecting_rolls_1_2_3() {
        assert!(StepPlayerLoss::is_player_defecting(1));
        assert!(StepPlayerLoss::is_player_defecting(2));
        assert!(StepPlayerLoss::is_player_defecting(3));
        assert!(!StepPlayerLoss::is_player_defecting(4));
        assert!(!StepPlayerLoss::is_player_defecting(5));
        assert!(!StepPlayerLoss::is_player_defecting(6));
    }

    #[test]
    fn no_concession_returns_next() {
        let mut step = StepPlayerLoss::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, crate::step::framework::StepAction::NextStep));
    }

    #[test]
    fn legal_concession_skips_defection() {
        let mut step = StepPlayerLoss::new();
        let mut game = make_game();
        game.game_result.home.conceded = true;
        game.conceded_legally = true;
        game.team_home.players.push(make_player("p1"));
        let mut pr = PlayerResult::default();
        pr.current_spps = 60;
        game.game_result.home.player_results.insert("p1".into(), pr);
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(!game.game_result.home.player_results["p1"].defecting);
    }

    #[test]
    fn low_spp_player_not_rolled() {
        let mut step = StepPlayerLoss::new();
        let mut game = make_game();
        game.game_result.home.conceded = true;
        game.conceded_legally = false;
        game.team_home.players.push(make_player("p1"));
        let mut pr = PlayerResult::default();
        pr.current_spps = 50; // below threshold of 51
        game.game_result.home.player_results.insert("p1".into(), pr);
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(!game.game_result.home.player_results["p1"].defecting);
    }

    #[test]
    fn high_spp_player_gets_defection_check() {
        let mut step = StepPlayerLoss::new();
        let mut game = make_game();
        game.game_result.away.conceded = true;
        game.conceded_legally = false;
        game.team_away.players.push(make_player("a1"));
        let mut pr = PlayerResult::default();
        pr.current_spps = 51;
        game.game_result.away.player_results.insert("a1".into(), pr);
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, crate::step::framework::StepAction::NextStep));
        // defecting flag was accessed, verifying no panic
        let _ = game.game_result.away.player_results["a1"].defecting;
    }

    #[test]
    fn defecting_players_report_emitted_when_eligible_player_exists() {
        use ffb_model::report::report_id::ReportId;
        let mut step = StepPlayerLoss::new();
        let mut game = make_game();
        game.game_result.home.conceded = true;
        game.conceded_legally = false;
        game.team_home.players.push(make_player("p1"));
        let mut pr = PlayerResult::default();
        pr.current_spps = 55; // above threshold
        game.game_result.home.player_results.insert("p1".into(), pr);
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(game.report_list.has_report(ReportId::DEFECTING_PLAYERS));
    }

    #[test]
    fn uses_current_spps_not_spp_gained_this_game() {
        // Java checks `playerResult.getCurrentSpps()` (the player's SPP total), not SPP gained
        // during this game. A player with a huge spp_gained this game but low current_spps must
        // NOT be flagged for the defection roll; this would have incorrectly triggered a roll
        // before the fix (which mistakenly read `spp_gained` instead of `current_spps`).
        use ffb_model::report::report_id::ReportId;
        let mut step = StepPlayerLoss::new();
        let mut game = make_game();
        game.game_result.home.conceded = true;
        game.conceded_legally = false;
        game.team_home.players.push(make_player("p1"));
        let mut pr = PlayerResult::default();
        pr.spp_gained = 999; // huge SPP gained this game — irrelevant to the check
        pr.current_spps = 10; // well below the 51 threshold
        game.game_result.home.player_results.insert("p1".into(), pr);
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(!game.game_result.home.player_results["p1"].defecting);
        assert!(!game.report_list.has_report(ReportId::DEFECTING_PLAYERS));
    }

    #[test]
    fn no_report_when_no_eligible_players() {
        use ffb_model::report::report_id::ReportId;
        let mut step = StepPlayerLoss::new();
        let mut game = make_game();
        game.game_result.home.conceded = true;
        game.conceded_legally = false;
        // Player below threshold — no defection roll, no report
        game.team_home.players.push(make_player("p1"));
        let mut pr = PlayerResult::default();
        pr.current_spps = 40;
        game.game_result.home.player_results.insert("p1".into(), pr);
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(!game.report_list.has_report(ReportId::DEFECTING_PLAYERS));
    }
}
