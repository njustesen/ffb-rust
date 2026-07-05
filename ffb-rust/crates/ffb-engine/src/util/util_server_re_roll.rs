/// 1:1 translation of `com.fumbbl.ffb.server.util.UtilServerReRoll`.
///
/// Java delegates to `RollMechanic.useReRoll` / `askForReRollIfAvailable` (deferred — require IStep).
/// The availability checks delegate to the edition's `RollMechanic`.
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;

pub struct UtilServerReRoll;

impl UtilServerReRoll {
    /// Java: `UtilServerReRoll.isProReRollAvailable(player, game, passState)`.
    /// Checks whether the player has an unused Pro skill that can be used as a re-roll.
    pub fn is_pro_re_roll_available(game: &Game, player: &Player) -> bool {
        let mechanic = crate::mechanic::roll_mechanic_for(game.rules);
        mechanic.is_pro_re_roll_available(game, player)
    }

    /// Java: `UtilServerReRoll.isSingleUseReRollAvailable(gameState, player)`.
    /// Checks if there are single-use re-rolls available for the player's team.
    pub fn is_single_use_re_roll_available(game: &Game, player: &Player) -> bool {
        let mechanic = crate::mechanic::roll_mechanic_for(game.rules);
        mechanic.is_single_use_re_roll_available(game, player)
    }

    /// Java: `UtilServerReRoll.isTeamReRollAvailable(gameState, player)`.
    /// Checks if the team re-roll token can be used in the current turn mode.
    pub fn is_team_re_roll_available(game: &Game, player: &Player) -> bool {
        let mechanic = crate::mechanic::roll_mechanic_for(game.rules);
        mechanic.is_team_re_roll_available(game, player)
    }

    // Java: `useReRoll` and `askForReRollIfAvailable` — DEFERRED (require IStep/dialog infra).
    // The step-layer equivalents live in `crate::step::util_server_re_roll`.
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use ffb_model::enums::{Rules, TurnMode};
    use ffb_model::model::game::Game;
    use ffb_model::model::player::Player;

    fn make_game(rules: Rules) -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), rules)
    }

    #[test]
    fn pro_reroll_unavailable_without_skill() {
        let game = make_game(Rules::Bb2025);
        let player = Player::default();
        assert!(!UtilServerReRoll::is_pro_re_roll_available(&game, &player));
    }

    #[test]
    fn team_reroll_unavailable_no_trr() {
        let game = make_game(Rules::Bb2025);
        let player = Player::default();
        assert!(!UtilServerReRoll::is_team_re_roll_available(&game, &player));
    }

    #[test]
    fn team_reroll_unavailable_on_kickoff_mode() {
        let mut game = make_game(Rules::Bb2025);
        game.turn_mode = TurnMode::Kickoff;
        game.turn_data_home.rerolls = 2;
        let player = Player::default();
        // BB2025 blocks re-rolls during kickoff
        assert!(!UtilServerReRoll::is_team_re_roll_available(&game, &player));
    }

    #[test]
    fn single_use_reroll_unavailable_when_empty() {
        let game = make_game(Rules::Bb2016);
        let player = Player::default();
        assert!(!UtilServerReRoll::is_single_use_re_roll_available(&game, &player));
    }

    #[test]
    fn bb2020_blocks_blitz_team_reroll() {
        let mut game = make_game(Rules::Bb2020);
        game.turn_mode = TurnMode::Blitz;
        game.home_playing = true;
        game.turn_data_home.rerolls = 2;
        let player = Player::default();
        // BB2020 blocks re-rolls during Blitz mode
        assert!(!UtilServerReRoll::is_team_re_roll_available(&game, &player));
    }
}
