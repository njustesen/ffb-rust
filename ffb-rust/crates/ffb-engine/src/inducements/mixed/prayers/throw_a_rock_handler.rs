/// 1:1 translation of `com.fumbbl.ffb.server.inducements.mixed.prayers.ThrowARockHandler`.
/// Abstract — only provides animationType(); actual initEffect logic lives in edition-specific handlers.
use ffb_model::model::animation_type::AnimationType;
use ffb_model::model::game::Game;
use crate::prayer_state::PrayerState;

pub fn animation_type() -> AnimationType {
    AnimationType::PRAYER_THROW_A_ROCK
}

pub fn init_effect(_prayer_state: &mut PrayerState, _game: &Game, _team_id: &str) -> bool {
    // headless: edition-specific throw-a-rock logic not yet ported
    true
}

pub fn remove_effect_internal(_prayer_state: &mut PrayerState, _game: &Game, _team_id: &str) {
    // No persistent state to remove for ThrowARock
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
        assert_eq!(animation_type(), AnimationType::PRAYER_THROW_A_ROCK);
    }

    #[test]
    fn init_effect_returns_true() {
        let mut state = PrayerState::new();
        let game = make_game();
        assert!(init_effect(&mut state, &game, "team1"));
    }

    #[test]
    fn remove_effect_is_noop() {
        let mut state = PrayerState::new();
        let game = make_game();
        remove_effect_internal(&mut state, &game, "team1");
    }
}
