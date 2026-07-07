/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeCrowd (abstract).
///
/// Provides shared crowd-push handleInjury logic for CrowdPush/TrapDoorFall variants.
use ffb_model::enums::{ApothecaryMode, PlayerState, PS_RESERVE};
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use crate::injury::{InjuryContext, do_injury_roll_for_player};

pub(crate) fn crowd_handle_injury(
    ctx: &mut InjuryContext, _game: &Game, rng: &mut GameRng,
    attacker_id: Option<&str>, defender_id: &str,
    coord: FieldCoordinate, apo_mode: ApothecaryMode,
) {
    ctx.defender_id = Some(defender_id.to_owned());
    ctx.attacker_id = attacker_id.map(str::to_owned);
    ctx.defender_coordinate = Some(coord);
    ctx.apothecary_mode = apo_mode;
    ctx.armor_broken = true;
    do_injury_roll_for_player(rng, ctx, _game, defender_id);
    if !ctx.is_casualty() && !ctx.is_knocked_out() {
        ctx.injury = Some(PlayerState::new(PS_RESERVE));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{ApothecaryMode, Rules};
    use ffb_model::model::game::Game;
    use crate::step::framework::test_team;
    use ffb_model::types::FieldCoordinate;

    #[test]
    fn sets_defender_and_attacker_ids() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let mut rng = GameRng::new(1);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        crowd_handle_injury(&mut ctx, &game, &mut rng, Some("atk1"), "def1",
            FieldCoordinate::new(0, 0), ApothecaryMode::Defender);
        assert_eq!(ctx.defender_id.as_deref(), Some("def1"));
        assert_eq!(ctx.attacker_id.as_deref(), Some("atk1"));
    }

    #[test]
    fn sets_armor_broken_true() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let mut rng = GameRng::new(1);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        crowd_handle_injury(&mut ctx, &game, &mut rng, None, "def1",
            FieldCoordinate::new(0, 0), ApothecaryMode::Defender);
        assert!(ctx.armor_broken);
    }

    #[test]
    fn injury_is_set_after_call() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let mut rng = GameRng::new(1);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        crowd_handle_injury(&mut ctx, &game, &mut rng, None, "def1",
            FieldCoordinate::new(0, 0), ApothecaryMode::Defender);
        assert!(ctx.injury.is_some());
    }
    #[test]
    fn sets_defender_coordinate() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let mut rng = GameRng::new(1);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        let coord = FieldCoordinate::new(3, 7);
        crowd_handle_injury(&mut ctx, &game, &mut rng, None, "def1", coord, ApothecaryMode::Defender);
        assert_eq!(ctx.defender_coordinate, Some(coord));
    }
    #[test]
    fn sets_apothecary_mode() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let mut rng = GameRng::new(1);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        crowd_handle_injury(&mut ctx, &game, &mut rng, None, "def1",
            FieldCoordinate::new(0, 0), ApothecaryMode::Attacker);
        assert_eq!(ctx.apothecary_mode, ApothecaryMode::Attacker);
    }

    #[test]
    fn attacker_id_is_none_when_not_provided() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let mut rng = GameRng::new(1);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        crowd_handle_injury(&mut ctx, &game, &mut rng, None, "def1",
            FieldCoordinate::new(0, 0), ApothecaryMode::Defender);
        assert_eq!(ctx.attacker_id, None);
    }

    #[test]
    fn coordinate_values_are_preserved() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let mut rng = GameRng::new(1);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        let coord = FieldCoordinate::new(5, 12);
        crowd_handle_injury(&mut ctx, &game, &mut rng, None, "def1", coord, ApothecaryMode::Defender);
        assert_eq!(ctx.defender_coordinate, Some(coord));
    }

    #[test]
    fn injury_is_some_after_call() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016);
        let mut rng = GameRng::new(3);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        crowd_handle_injury(&mut ctx, &game, &mut rng, None, "def1",
            FieldCoordinate::new(1, 2), ApothecaryMode::Defender);
        assert!(ctx.injury.is_some());
    }
}
