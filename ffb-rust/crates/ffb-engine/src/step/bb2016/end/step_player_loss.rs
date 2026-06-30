/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.end.StepPlayerLoss`.
///
/// For illegal concession: any player with ≥51 current SPPs risks defecting.
/// Java: `DiceInterpreter.isPlayerDefecting(roll)` = roll > 0 && roll < 4 (i.e., 1–3 on D6).
use ffb_model::model::game::Game;
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
                        .map(|r| r.spp_gained >= SPP_DEFECTION_THRESHOLD)
                        .unwrap_or(false)
                })
                .map(|p| p.id.clone())
                .collect()
        };

        for pid in &player_ids {
            let roll = rng.d6();
            let defecting = Self::is_player_defecting(roll);
            let results = if conceding_home {
                &mut game.game_result.home.player_results
            } else {
                &mut game.game_result.away.player_results
            };
            if let Some(pr) = results.get_mut(pid) {
                pr.defecting = defecting;
            }
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
        pr.spp_gained = 60;
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
        pr.spp_gained = 50; // below threshold of 51
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
        pr.spp_gained = 51;
        game.game_result.away.player_results.insert("a1".into(), pr);
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, crate::step::framework::StepAction::NextStep));
        // defecting flag was accessed, verifying no panic
        let _ = game.game_result.away.player_results["a1"].defecting;
    }
}
