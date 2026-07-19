use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::report::mixed::report_defecting_players::ReportDefectingPlayers;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::StepId;

/// Rolls for player defection when a team concedes illegally (BB2020).
/// A player with ≥3 extra skills (beyond position starting skills) risks defecting on roll ≤ 2.
///
/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.end.StepPlayerLoss.
/// Logic is identical to BB2025 StepPlayerLoss.
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

        let player_ids: Vec<String> = {
            let team = if conceding_home { &game.team_home } else { &game.team_away };
            let player_results = if conceding_home {
                &game.game_result.home.player_results
            } else {
                &game.game_result.away.player_results
            };
            // Java: playerResult.getSeriousInjury() == null || !playerResult.getSeriousInjury().isDead()
            team.players.iter()
                .filter(|p| p.extra_skills.len() as i32 >= 3)
                .filter(|p| {
                    player_results.get(&p.id)
                        .and_then(|pr| pr.serious_injury)
                        .map(|si| !si.is_dead())
                        .unwrap_or(true)
                })
                .map(|p| p.id.clone())
                .collect()
        };

        let mut rolls: Vec<i32> = Vec::new();
        let mut defectings: Vec<bool> = Vec::new();
        for pid in &player_ids {
            let roll = rng.d6();
            let defecting = roll <= 2;
            rolls.push(roll);
            defectings.push(defecting);
            let results = if conceding_home {
                &mut game.game_result.home.player_results
            } else {
                &mut game.game_result.away.player_results
            };
            if let Some(pr) = results.get_mut(pid) {
                pr.defecting = defecting;
            }
        }

        // Java: .addReport(new ReportDefectingPlayers(defectingPlayerIds.toArray(new String[0]), rolls, defectings))
        if !player_ids.is_empty() {
            game.report_list.add(ReportDefectingPlayers::new(player_ids, rolls, defectings));
        }

        StepOutcome::next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use ffb_model::report::report_id::ReportId;
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
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills, temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
        }
    }

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    #[test]
    fn no_concession_returns_next() {
        let mut game = make_game();
        let mut step = StepPlayerLoss;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

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

    #[test]
    fn player_with_2_extra_skills_never_defects() {
        let mut game = make_game();
        game.game_result.home.conceded = true;
        game.conceded_legally = false;
        game.team_home.players.push(make_player_with_extra_skills("h1", 2));
        game.game_result.home.player_results.insert("h1".into(), PlayerResult::default());
        let mut step = StepPlayerLoss;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.game_result.home.player_results["h1"].defecting);
    }

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
        let _ = game.game_result.home.player_results["h1"].defecting;
    }

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
        assert!(!game.game_result.home.player_results["h1"].defecting);
    }

    #[test]
    fn defecting_players_report_added_when_player_has_enough_extra_skills() {
        let mut game = make_game();
        game.game_result.home.conceded = true;
        game.conceded_legally = false;
        game.team_home.players.push(make_player_with_extra_skills("h1", 3));
        game.game_result.home.player_results.insert("h1".into(), PlayerResult::default());
        let mut step = StepPlayerLoss;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::DEFECTING_PLAYERS), "should add ReportDefectingPlayers");
    }

    #[test]
    fn dead_player_excluded_from_defection_roll() {
        // Java: playerResult.getSeriousInjury() == null || !playerResult.getSeriousInjury().isDead()
        // A player who died from a serious injury this game must not be rolled for defection,
        // even if they otherwise have >= 3 extra skills.
        use ffb_model::enums::SeriousInjuryKind;
        let mut game = make_game();
        game.game_result.home.conceded = true;
        game.conceded_legally = false;
        game.team_home.players.push(make_player_with_extra_skills("h1", 4));
        let mut pr = PlayerResult::default();
        pr.serious_injury = Some(SeriousInjuryKind::Dead);
        game.game_result.home.player_results.insert("h1".into(), pr);
        let mut step = StepPlayerLoss;
        step.start(&mut game, &mut GameRng::new(0));
        // Dead player should not be marked as defecting (never rolled) and no report emitted.
        assert!(!game.game_result.home.player_results["h1"].defecting);
        assert!(!game.report_list.has_report(ReportId::DEFECTING_PLAYERS),
            "dead player should be excluded from defection roll and report");
    }

    #[test]
    fn no_defecting_report_when_no_eligible_players() {
        let mut game = make_game();
        game.game_result.home.conceded = true;
        game.conceded_legally = false;
        // player with only 2 extra skills — below threshold
        game.team_home.players.push(make_player_with_extra_skills("h1", 2));
        game.game_result.home.player_results.insert("h1".into(), PlayerResult::default());
        let mut step = StepPlayerLoss;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.report_list.has_report(ReportId::DEFECTING_PLAYERS), "should NOT add report when no eligible players");
    }
}
