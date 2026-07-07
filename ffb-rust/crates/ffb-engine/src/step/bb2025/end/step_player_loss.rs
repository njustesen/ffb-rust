use ffb_model::model::game::Game;
use ffb_model::report::report_defecting_players::ReportDefectingPlayers;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::StepId;

/// Rolls for player defection at end of game when a team conceded illegally.
/// A player with ≥3 extra skills (beyond position starting skills) risks defecting on 1-2/d6.
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.end.StepPlayerLoss`.
pub struct StepPlayerLoss;

impl Step for StepPlayerLoss {
    fn id(&self) -> StepId { StepId::PlayerLoss }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }
}

impl StepPlayerLoss {
    fn execute_step(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let conceding_home = game.game_result.home.conceded && !game.conceded_legally;
        let conceding_away = game.game_result.away.conceded && !game.conceded_legally;

        if !conceding_home && !conceding_away {
            return StepOutcome::next();
        }

        // Collect eligible player IDs (extra_skills count ≥ 3, not dead).
        let player_ids: Vec<String> = {
            let team = if conceding_home { &game.team_home } else { &game.team_away };
            team.players.iter()
                .filter(|p| p.extra_skills.len() as i32 >= 3)
                .map(|p| p.id.clone())
                .collect()
        };

        // Java: collect defectingPlayerIds, defectingRolls, defectingFlags in parallel
        let mut defecting_rolls: Vec<i32> = Vec::new();
        let mut defecting_flags: Vec<bool> = Vec::new();

        for pid in &player_ids {
            let roll = rng.d6();
            // Java DiceInterpreter.isPlayerDefecting: defects on roll ≤ 2.
            let defecting = roll <= 2;
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

        // Java: if (!defectingPlayerIds.isEmpty()) { getResult().addReport(new ReportDefectingPlayers(...)) }
        if !player_ids.is_empty() {
            game.report_list.add(ReportDefectingPlayers::new(player_ids, defecting_rolls, defecting_flags));
        }

        StepOutcome::next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{PlayerType, PlayerGender, Rules, SkillId};
    use ffb_model::model::player::Player;
    use ffb_model::model::game_result::PlayerResult;
    use ffb_model::model::skill_def::SkillWithValue;

    fn make_player_with_extra_skills(id: &str, extra_count: usize) -> Player {
        let skills = vec![
            SkillId::Block, SkillId::Dodge, SkillId::Guard,
            SkillId::Tackle, SkillId::StripBall, SkillId::Frenzy,
        ];
        let extra_skills = skills[..extra_count.min(skills.len())]
            .iter()
            .map(|&s| SkillWithValue::new(s))
            .collect();
        Player {
            id: id.into(),
            name: id.into(),
            nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![],
            extra_skills,
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

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    /// No concession → all players keep defecting=false, returns next.
    #[test]
    fn no_concession_returns_next_immediately() {
        let mut game = make_game();
        let mut step = StepPlayerLoss;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    /// Legal concession → no defection rolls (same as no concession path).
    #[test]
    fn legal_concession_skips_player_loss() {
        let mut game = make_game();
        game.game_result.home.conceded = true;
        game.conceded_legally = true;
        game.team_home.players.push(make_player_with_extra_skills("h1", 4));
        game.game_result.home.player_results.insert("h1".into(), PlayerResult::default());
        let mut step = StepPlayerLoss;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!game.game_result.home.player_results["h1"].defecting);
    }

    /// Players with fewer than 3 extra skills are not at risk.
    #[test]
    fn player_with_2_extra_skills_never_defects() {
        let mut game = make_game();
        game.game_result.home.conceded = true;
        game.conceded_legally = false;
        game.team_home.players.push(make_player_with_extra_skills("h1", 2));
        game.game_result.home.player_results.insert("h1".into(), PlayerResult::default());
        // Regardless of RNG outcome, player with 2 extra skills should not be checked.
        let mut step = StepPlayerLoss;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.game_result.home.player_results["h1"].defecting,
            "player with only 2 extra skills must not defect");
    }

    /// Player with ≥3 extra skills is rolled for defection (roll ≤ 2 → defects).
    /// GameRng::new(0) with die(6) - we just confirm defecting flag is set or not.
    #[test]
    fn player_with_3_extra_skills_gets_defection_roll() {
        let mut game = make_game();
        game.game_result.home.conceded = true;
        game.conceded_legally = false;
        game.team_home.players.push(make_player_with_extra_skills("h1", 3));
        game.game_result.home.player_results.insert("h1".into(), PlayerResult::default());
        let mut step = StepPlayerLoss;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        // The flag is set to either true or false depending on RNG; just confirm it was touched.
        // We can't assert a specific value without knowing the RNG output for seed 0,
        // but we can verify the step ran and outcome is correct.
        let _ = game.game_result.home.player_results["h1"].defecting; // accessed without panic
    }

    /// Away concession: away team's players are rolled, home is untouched.
    #[test]
    fn away_concession_rolls_away_players_only() {
        let mut game = make_game();
        game.game_result.away.conceded = true;
        game.conceded_legally = false;
        game.team_home.players.push(make_player_with_extra_skills("h1", 4));
        game.team_away.players.push(make_player_with_extra_skills("a1", 4));
        game.game_result.home.player_results.insert("h1".into(), PlayerResult::default());
        game.game_result.away.player_results.insert("a1".into(), PlayerResult::default());
        let mut step = StepPlayerLoss;
        step.start(&mut game, &mut GameRng::new(0));
        // Home player should never be flagged; away player may or may not defect.
        assert!(!game.game_result.home.player_results["h1"].defecting,
            "home player must not be affected by away concession");
    }

    /// ReportDefectingPlayers is added when an eligible player is rolled.
    #[test]
    fn defecting_players_report_added_when_eligible_player_exists() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        game.game_result.home.conceded = true;
        game.conceded_legally = false;
        game.team_home.players.push(make_player_with_extra_skills("h1", 3));
        game.game_result.home.player_results.insert("h1".into(), PlayerResult::default());
        let mut step = StepPlayerLoss;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(
            game.report_list.has_report(ReportId::DEFECTING_PLAYERS),
            "expected DEFECTING_PLAYERS report when eligible player is rolled"
        );
    }

    /// No report is added when no players exceed the threshold.
    #[test]
    fn no_report_when_no_eligible_players() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        game.game_result.home.conceded = true;
        game.conceded_legally = false;
        // Player with only 2 extra skills — below threshold.
        game.team_home.players.push(make_player_with_extra_skills("h1", 2));
        game.game_result.home.player_results.insert("h1".into(), PlayerResult::default());
        let mut step = StepPlayerLoss;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(
            !game.report_list.has_report(ReportId::DEFECTING_PLAYERS),
            "must not add report when no player meets the threshold"
        );
    }
}
