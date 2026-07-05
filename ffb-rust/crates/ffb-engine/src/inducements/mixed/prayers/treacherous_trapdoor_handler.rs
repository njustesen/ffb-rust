/// 1:1 translation of `com.fumbbl.ffb.server.inducements.mixed.prayers.TreacherousTrapdoorHandler`.
/// Adds trap-door markers at fixed field coordinates; removes them on removeEffect.
use ffb_model::model::animation_type::AnimationType;
use ffb_model::model::game::Game;
use ffb_model::types::FieldCoordinate;
use crate::prayer_state::PrayerState;

/// Fixed coordinates where trapdoors appear. Java: (6,1) and (19,13).
const TRAPDOOR_COORDINATES: [(i32, i32); 2] = [(6, 1), (19, 13)];

pub fn animation_type() -> AnimationType {
    AnimationType::PRAYER_TREACHEROUS_TRAPDOOR
}

/// Java: initEffect(GameState, Team) — places TrapDoor objects at fixed coordinates.
pub fn init_effect(_prayer_state: &mut PrayerState, game: &mut Game, _team_id: &str) -> bool {
    for (x, y) in TRAPDOOR_COORDINATES {
        game.field_model.add_trap_door(FieldCoordinate::new(x, y));
    }
    true
}

/// Java: removeEffectInternal(GameState, Team) — clears all trapdoors from the field.
pub fn remove_effect_internal(_prayer_state: &mut PrayerState, game: &mut Game, _team_id: &str) {
    game.field_model.clear_trap_doors();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_game() -> Game {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    #[test]
    fn animation_type_is_correct() {
        assert_eq!(animation_type(), AnimationType::PRAYER_TREACHEROUS_TRAPDOOR);
    }

    #[test]
    fn trapdoor_coordinates_are_correct() {
        assert!(TRAPDOOR_COORDINATES.contains(&(6, 1)));
        assert!(TRAPDOOR_COORDINATES.contains(&(19, 13)));
    }

    #[test]
    fn init_effect_adds_two_trapdoors() {
        let mut state = PrayerState::new();
        let mut game = make_game();
        assert!(init_effect(&mut state, &mut game, "team1"));
        assert!(game.field_model.has_trap_door(FieldCoordinate::new(6, 1)));
        assert!(game.field_model.has_trap_door(FieldCoordinate::new(19, 13)));
    }

    #[test]
    fn init_effect_returns_true() {
        let mut state = PrayerState::new();
        let mut game = make_game();
        assert!(init_effect(&mut state, &mut game, "team1"));
    }

    #[test]
    fn remove_effect_clears_trapdoors() {
        let mut state = PrayerState::new();
        let mut game = make_game();
        init_effect(&mut state, &mut game, "team1");
        remove_effect_internal(&mut state, &mut game, "team1");
        assert!(!game.field_model.has_trap_door(FieldCoordinate::new(6, 1)));
        assert!(!game.field_model.has_trap_door(FieldCoordinate::new(19, 13)));
        assert!(game.field_model.trap_doors.is_empty());
    }

    #[test]
    fn remove_effect_no_panic_when_no_trapdoors() {
        let mut state = PrayerState::new();
        let mut game = make_game();
        remove_effect_internal(&mut state, &mut game, "team1");
    }
}
