/// 1:1 translation of com.fumbbl.ffb.server.step.action.block.StepBlockStatistics (COMMON).
///
/// Tracks that the acting player has blocked, marks the turn as started, and increments
/// the player's block count by `increment` (default 1, can be overridden via PLAYER_ID_TO_REMOVE
/// which decrements it — used when a block target disappears before the block resolves).
///
/// Only runs the first time a player blocks each turn (has_blocked guard).
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

pub struct StepBlockStatistics {
    /// Java: increment — how much to add to the block count (default 1).
    /// Decremented when PLAYER_ID_TO_REMOVE is received (e.g. target disappears).
    pub increment: i32,
}

impl StepBlockStatistics {
    pub fn new() -> Self { Self { increment: 1 } }
}

impl Default for StepBlockStatistics {
    fn default() -> Self { Self::new() }
}

impl Step for StepBlockStatistics {
    fn id(&self) -> StepId { StepId::BlockStatistics }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            // Java: if PLAYER_ID_TO_REMOVE → increment-- (target disappeared)
            StepParameter::PlayerIdToRemove(_) => { self.increment -= 1; true }
            // Java: init: if INCREMENT → increment = value
            StepParameter::Increment(v) => { self.increment = *v; true }
            _ => false,
        }
    }
}

impl StepBlockStatistics {
    fn execute_step(&mut self, game: &mut Game) -> StepOutcome {
        let player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        // Java: if (!actingPlayer.hasBlocked()) { ... }
        if !game.acting_player.has_blocked {
            game.acting_player.has_blocked = true;

            // Java: game.getTurnData().setTurnStarted(true)
            game.turn_data_mut().turn_started = true;

            // Java: game.setConcessionPossible(false)
            game.concession_possible = false;

            // Java: playerResult.setBlocks(playerResult.getBlocks() + increment)
            let is_home = game.team_home.player(&player_id).is_some();
            let pr = if is_home {
                game.game_result.home.player_results.entry(player_id).or_default()
            } else {
                game.game_result.away.player_results.entry(player_id).or_default()
            };
            pr.blocks += self.increment;
        }

        StepOutcome::next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{Rules, PS_STANDING};
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::enums::{PlayerState, SkillId};
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> (Game, String) {
        let pid = "att".to_string();
        let mut home = test_team("home", 0);
        home.players.push(ffb_model::model::player::Player {
            id: pid.clone(), name: pid.clone(), nr: 1, position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![SkillWithValue { skill_id: SkillId::Block, value: None }],
            extra_skills: vec![], temporary_skills: vec![], used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
                    ..Default::default()
});
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.home_playing = true;
        game.acting_player.player_id = Some(pid.clone());
        game.field_model.set_player_coordinate(&pid, FieldCoordinate::new(5, 5));
        game.field_model.set_player_state(&pid, PlayerState::new(PS_STANDING).change_active(true));
        (game, pid)
    }

    #[test]
    fn no_acting_player_returns_next() {
        let (mut game, _) = make_game();
        game.acting_player.player_id = None;
        let outcome = StepBlockStatistics::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
    }

    #[test]
    fn first_block_sets_has_blocked_and_turn_started() {
        let (mut game, _) = make_game();
        StepBlockStatistics::new().start(&mut game, &mut GameRng::new(0));
        assert!(game.acting_player.has_blocked);
        assert!(game.turn_data().turn_started);
    }

    #[test]
    fn first_block_clears_concession_possible() {
        let (mut game, _) = make_game();
        game.concession_possible = true;
        StepBlockStatistics::new().start(&mut game, &mut GameRng::new(0));
        assert!(!game.concession_possible);
    }

    #[test]
    fn first_block_increments_player_block_count() {
        let (mut game, pid) = make_game();
        StepBlockStatistics::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.game_result.home.player_results[&pid].blocks, 1);
    }

    #[test]
    fn second_call_skips_stats() {
        let (mut game, pid) = make_game();
        game.acting_player.has_blocked = true;
        game.game_result.home.player_results.entry(pid.clone()).or_default().blocks = 5;
        StepBlockStatistics::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.game_result.home.player_results[&pid].blocks, 5); // not incremented
    }

    #[test]
    fn increment_can_be_set_via_parameter() {
        let (mut game, pid) = make_game();
        let mut step = StepBlockStatistics::new();
        step.set_parameter(&StepParameter::Increment(3));
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.game_result.home.player_results[&pid].blocks, 3);
    }

    #[test]
    fn player_id_to_remove_decrements_increment() {
        let mut step = StepBlockStatistics::new();
        step.set_parameter(&StepParameter::PlayerIdToRemove("x".into()));
        assert_eq!(step.increment, 0);
    }
}
