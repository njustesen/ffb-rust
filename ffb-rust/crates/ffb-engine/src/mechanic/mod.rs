pub mod casualty_calc;
pub mod injury_calc;
pub mod roll_mechanic;
pub mod setup_mechanic;
pub mod state_mechanic;
pub mod spp_calc;
pub mod armor_modifier_values;
pub mod injury_modifier_values;
pub mod weather_modifier_values;

pub mod bb2016;
pub mod bb2020;
pub mod bb2025;
pub mod mixed;

use ffb_model::enums::Rules;
use crate::mechanic::roll_mechanic::RollMechanic as RollMechanicTrait;
use crate::mechanic::setup_mechanic::SetupMechanic as SetupMechanicTrait;
use crate::mechanic::state_mechanic::StateMechanic as StateMechanicTrait;
use ffb_mechanics::game_mechanic::GameMechanic as GameMechanicTrait;

/// Returns the edition-appropriate `RollMechanic` for the given rules.
/// Mirrors Java's `game.getFactory(MECHANIC).forName(Mechanic.Type.ROLL.name())`.
pub fn roll_mechanic_for(rules: Rules) -> Box<dyn RollMechanicTrait> {
    match rules {
        Rules::Bb2016 => Box::new(bb2016::roll_mechanic::RollMechanic::new()),
        Rules::Bb2020 => Box::new(bb2020::roll_mechanic::RollMechanic::new()),
        Rules::Bb2025 | Rules::Common => Box::new(bb2025::roll_mechanic::RollMechanic::new()),
    }
}

/// Returns the edition-appropriate `StateMechanic` for the given rules.
/// Mirrors Java's `game.getFactory(MECHANIC).forName(Mechanic.Type.STATE.name())`.
pub fn state_mechanic_for(rules: Rules) -> Box<dyn StateMechanicTrait> {
    match rules {
        Rules::Bb2025 | Rules::Common => Box::new(bb2025::state_mechanic::StateMechanic::new()),
        Rules::Bb2016 | Rules::Bb2020 => Box::new(mixed::state_mechanic::StateMechanic::new()),
    }
}

/// Returns the edition-appropriate `SetupMechanic` for the given rules.
/// Mirrors Java's `game.getFactory(MECHANIC).forName(Mechanic.Type.SETUP.name())`.
pub fn setup_mechanic_for(rules: Rules) -> Box<dyn SetupMechanicTrait> {
    match rules {
        Rules::Bb2025 | Rules::Common => Box::new(bb2025::setup_mechanic::SetupMechanic::new()),
        Rules::Bb2016 | Rules::Bb2020 => Box::new(mixed::setup_mechanic::SetupMechanic::new()),
    }
}

/// Returns the edition-appropriate `GameMechanic` for the given rules.
/// Mirrors Java's `game.getFactory(MECHANIC).forName(Mechanic.Type.GAME.name())`.
pub fn game_mechanic_for(rules: Rules) -> Box<dyn GameMechanicTrait> {
    match rules {
        Rules::Bb2025 | Rules::Common => Box::new(ffb_mechanics::bb2025::game_mechanic::GameMechanic::new()),
        Rules::Bb2020 => Box::new(ffb_mechanics::bb2020::game_mechanic::GameMechanic::new()),
        Rules::Bb2016 => Box::new(ffb_mechanics::bb2016::game_mechanic::GameMechanic::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{LeaderState, TurnMode};

    #[test]
    fn bb2016_roll_mechanic_blocks_kickoff() {
        let m = roll_mechanic_for(Rules::Bb2016);
        assert!(!m.allows_team_re_roll(TurnMode::Kickoff));
        assert!(m.allows_team_re_roll(TurnMode::Regular));
    }

    #[test]
    fn bb2020_roll_mechanic_blocks_blitz() {
        let m = roll_mechanic_for(Rules::Bb2020);
        assert!(!m.allows_team_re_roll(TurnMode::Blitz));
        assert!(m.allows_team_re_roll(TurnMode::Regular));
    }

    #[test]
    fn bb2025_roll_mechanic_blocks_between_turns() {
        let m = roll_mechanic_for(Rules::Bb2025);
        assert!(!m.allows_team_re_roll(TurnMode::BetweenTurns));
        assert!(m.allows_team_re_roll(TurnMode::Regular));
    }

    #[test]
    fn common_rules_uses_bb2025() {
        let m = roll_mechanic_for(Rules::Common);
        // BB2025 pro roll minimum is 3
        assert_eq!(m.minimum_pro_roll(), 3);
    }

    fn make_game(rules: Rules) -> ffb_model::model::game::Game {
        ffb_model::model::game::Game::new(
            crate::step::framework::test_team("home", 0),
            crate::step::framework::test_team("away", 0),
            rules,
        )
    }

    #[test]
    fn state_mechanic_bb2025_start_half() {
        let m = state_mechanic_for(Rules::Bb2025);
        let mut g = make_game(Rules::Bb2025);
        m.start_half(&mut g, 1);
        assert_eq!(g.half, 1);
    }

    #[test]
    fn state_mechanic_bb2016_start_half() {
        let m = state_mechanic_for(Rules::Bb2016);
        let mut g = make_game(Rules::Bb2016);
        m.start_half(&mut g, 1);
        assert_eq!(g.half, 1);
    }

    #[test]
    fn state_mechanic_bb2020_start_half() {
        let m = state_mechanic_for(Rules::Bb2020);
        let mut g = make_game(Rules::Bb2020);
        m.start_half(&mut g, 2);
        assert_eq!(g.half, 2);
    }

    #[test]
    fn state_mechanic_common_uses_bb2025() {
        let m = state_mechanic_for(Rules::Common);
        let mut g = make_game(Rules::Bb2025);
        g.turn_data_home.leader_state = LeaderState::Available;
        m.start_half(&mut g, 1);
        // BB2025 resets leader state at half <= 2
        assert_eq!(g.turn_data_home.leader_state, LeaderState::None);
    }

    #[test]
    fn setup_mechanic_for_bb2016_check_setup_empty_is_valid() {
        let m = setup_mechanic_for(Rules::Bb2016);
        let mut g = make_game(Rules::Bb2016);
        assert!(m.check_setup(&mut g, true));
    }

    #[test]
    fn setup_mechanic_for_bb2020_check_setup_empty_is_valid() {
        let m = setup_mechanic_for(Rules::Bb2020);
        let mut g = make_game(Rules::Bb2020);
        assert!(m.check_setup(&mut g, true));
    }

    #[test]
    fn setup_mechanic_for_bb2025_check_setup_empty_is_valid() {
        let m = setup_mechanic_for(Rules::Bb2025);
        let mut g = make_game(Rules::Bb2025);
        assert!(m.check_setup(&mut g, true));
    }

    #[test]
    fn setup_mechanic_for_common_uses_bb2025() {
        let m = setup_mechanic_for(Rules::Common);
        let mut g = make_game(Rules::Bb2025);
        // Both editions share the same pin logic; just confirm no panic.
        m.pin_players_in_tacklezones(&mut g, "home");
    }
}
