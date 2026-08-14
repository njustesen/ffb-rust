pub mod casualty_calc;
pub mod injury_calc;
pub mod roll_mechanic;
pub mod setup_mechanic;
pub mod state_mechanic;
pub mod spp_calc;

pub mod bb2016;
pub mod bb2020;
pub mod bb2025;
pub mod mixed;

use ffb_model::enums::Rules;
use crate::mechanic::roll_mechanic::RollMechanic as RollMechanicTrait;
use crate::mechanic::setup_mechanic::SetupMechanic as SetupMechanicTrait;
use crate::mechanic::state_mechanic::StateMechanic as StateMechanicTrait;
use ffb_mechanics::game_mechanic::GameMechanic as GameMechanicTrait;
use ffb_mechanics::jump_mechanic::JumpMechanic as JumpMechanicTrait;
use ffb_mechanics::ttm_mechanic::TtmMechanic as TtmMechanicTrait;
use ffb_mechanics::pass_mechanic::PassMechanic as PassMechanicTrait;
use ffb_mechanics::on_the_ball_mechanic::OnTheBallMechanic as OnTheBallMechanicTrait;

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

/// Returns the edition-appropriate `JumpMechanic` for the given rules.
/// Mirrors Java's `game.getFactory(MECHANIC).forName(Mechanic.Type.JUMP.name())`.
pub fn jump_mechanic_for(rules: Rules) -> Box<dyn JumpMechanicTrait> {
    match rules {
        Rules::Bb2025 | Rules::Common => Box::new(ffb_mechanics::bb2025::jump_mechanic::JumpMechanic::new()),
        Rules::Bb2020 => Box::new(ffb_mechanics::bb2020::jump_mechanic::JumpMechanic::new()),
        Rules::Bb2016 => Box::new(ffb_mechanics::bb2016::jump_mechanic::JumpMechanic::new()),
    }
}

/// Returns the edition-appropriate `TtmMechanic` for the given rules.
/// Mirrors Java's `game.getFactory(MECHANIC).forName(Mechanic.Type.TTM.name())`.
pub fn ttm_mechanic_for(rules: Rules) -> Box<dyn TtmMechanicTrait> {
    match rules {
        Rules::Bb2025 | Rules::Common => Box::new(ffb_mechanics::bb2025::ttm_mechanic::TtmMechanic::new()),
        Rules::Bb2020 => Box::new(ffb_mechanics::bb2020::ttm_mechanic::TtmMechanic::new()),
        Rules::Bb2016 => Box::new(ffb_mechanics::bb2016::ttm_mechanic::TtmMechanic::new()),
    }
}

/// Returns the edition-appropriate `PassMechanic` for the given rules.
/// Mirrors Java's `game.getFactory(MECHANIC).forName(Mechanic.Type.PASS.name())`.
pub fn pass_mechanic_for(rules: Rules) -> Box<dyn PassMechanicTrait> {
    match rules {
        Rules::Bb2025 | Rules::Common => Box::new(ffb_mechanics::bb2025::pass_mechanic::PassMechanic::new()),
        Rules::Bb2020 => Box::new(ffb_mechanics::bb2020::pass_mechanic::PassMechanic::new()),
        Rules::Bb2016 => Box::new(ffb_mechanics::bb2016::pass_mechanic::PassMechanic::new()),
    }
}

/// Returns the edition-appropriate `ThrowInMechanic` for the given rules.
/// Mirrors Java's `game.getMechanic(Mechanic.Type.THROW_IN)`.
///
/// The editions genuinely differ: BB2020's `distance` is `d1 + d2 + 1` where BB2016's and BB2025's
/// are `d1 + d2`, and only BB2025 has corner throw-ins (`isCornerThrowIn`, a d3 direction instead
/// of a d6). The shared BB2025 step hard-coded the BB2025 mechanic, so every BB2020 throw-in landed
/// one square short — and a short landing that happens to hold a player adds a catch d6 Java never
/// rolls (bb2020 human seed 43 pos 76).
pub fn throw_in_mechanic_for(rules: Rules) -> Box<dyn ffb_mechanics::throw_in_mechanic::ThrowInMechanic> {
    match rules {
        Rules::Bb2025 | Rules::Common => Box::new(ffb_mechanics::bb2025::throw_in_mechanic::ThrowInMechanic::new()),
        Rules::Bb2020 => Box::new(ffb_mechanics::bb2020::throw_in_mechanic::ThrowInMechanic::new()),
        Rules::Bb2016 => Box::new(ffb_mechanics::bb2016::throw_in_mechanic::ThrowInMechanic::new()),
    }
}

/// Returns the edition-appropriate `OnTheBallMechanic` for the given rules.
/// Mirrors Java's `game.getFactory(MECHANIC).forName(Mechanic.Type.ON_THE_BALL.name())`.
pub fn on_the_ball_mechanic_for(rules: Rules) -> Box<dyn OnTheBallMechanicTrait> {
    match rules {
        Rules::Bb2025 | Rules::Common | Rules::Bb2020 => {
            Box::new(ffb_mechanics::mixed::on_the_ball_mechanic::OnTheBallMechanic::new())
        }
        Rules::Bb2016 => Box::new(ffb_mechanics::bb2016::on_the_ball_mechanic::OnTheBallMechanic::new()),
    }
}

#[cfg(test)]
mod tests {
    use ffb_mechanics::throw_in_mechanic::ThrowInMechanic as _;

    /// Java resolves the throw-in mechanic per edition and the three genuinely differ. BB2020 adds
    /// +1 to the distance; only BB2025 has corner throw-ins (which roll a d3 direction, not a d6).
    /// The shared BB2025 step hard-coded the BB2025 mechanic, so every BB2020 throw-in landed one
    /// square short — and when the short square held a player, Rust rolled a catch d6 that Java
    /// never rolls, putting it a die ahead for the rest of the game (human seed 43 pos 76).
    #[test]
    fn throw_in_mechanic_distance_is_edition_specific() {
        assert_eq!(throw_in_mechanic_for(Rules::Bb2020).distance(&[3, 4]), 8, "bb2020 is d1+d2+1");
        assert_eq!(throw_in_mechanic_for(Rules::Bb2025).distance(&[3, 4]), 7, "bb2025 is d1+d2");
        assert_eq!(throw_in_mechanic_for(Rules::Bb2016).distance(&[3, 4]), 7, "bb2016 is d1+d2");
    }

    /// Corner throw-ins are BB2025-only; a BB2020/BB2016 corner must keep the plain d6 direction.
    #[test]
    fn corner_throw_ins_are_bb2025_only() {
        use ffb_model::types::FieldCoordinate;
        let corner = FieldCoordinate::new(0, 0);
        assert!(throw_in_mechanic_for(Rules::Bb2025).is_corner_throw_in(corner));
        assert!(!throw_in_mechanic_for(Rules::Bb2020).is_corner_throw_in(corner));
        assert!(!throw_in_mechanic_for(Rules::Bb2016).is_corner_throw_in(corner));
    }

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
