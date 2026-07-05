/// 1:1 translation of `com.fumbbl.ffb.server.inducements.mixed.prayers.GreasyCleatsHandler`.
/// Extends RandomSelectionPrayerHandler — selects 1 random opponent player, reduces MA by 1.
///
/// Effect: -1 MA to the selected player via `apply_prayer_player_effect`.
/// Prayer enhancement tracking (which player has the prayer) is fully implemented.
use ffb_model::model::animation_type::AnimationType;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::enums::SkillId;
use crate::inducements::mixed::prayers::player_selector::PlayerSelector;
use crate::inducements::mixed::prayers::random_selection_prayer_handler::{
    init_effect_random_selection, remove_effect_internal_random_selection,
};
use crate::prayer_state::PrayerState;

pub const AFFECTED_PLAYERS: i32 = 1;
pub const PRAYER_NAME: &str = "GREASY_CLEATS";

pub fn animation_type() -> AnimationType {
    AnimationType::PRAYER_GREASY_CLEATS
}

/// Java: RandomSelectionPrayerHandler.initEffect — selects player via selector, marks prayer.
pub fn init_effect(prayer_state: &mut PrayerState, game: &mut Game, rng: &mut GameRng, team_id: &str, selector: &dyn PlayerSelector) -> bool {
    init_effect_random_selection(prayer_state, game, rng, team_id, PRAYER_NAME, AFFECTED_PLAYERS, selector, &[])
}

/// Java: RandomSelectionPrayerHandler.removeEffectInternal — clears prayer tracking.
pub fn remove_effect_internal(game: &mut Game, team_id: &str) {
    remove_effect_internal_random_selection(game, team_id, PRAYER_NAME);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::util::rng::GameRng;
    use crate::step::framework::test_team;
    use crate::inducements::mixed::prayers::player_selector::StubPlayerSelector;

    fn make_game() -> Game {
        use ffb_model::enums::Rules;
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    #[test]
    fn affected_players_is_one() {
        assert_eq!(AFFECTED_PLAYERS, 1);
    }

    #[test]
    fn animation_type_is_correct() {
        assert_eq!(animation_type(), AnimationType::PRAYER_GREASY_CLEATS);
    }

    #[test]
    fn init_effect_returns_true() {
        let mut state = PrayerState::new();
        let mut game = make_game();
        let stub = StubPlayerSelector;
        assert!(init_effect(&mut state, &mut game, &mut GameRng::new(0), "home", &stub));
    }

    #[test]
    fn remove_effect_is_callable() {
        let mut game = make_game();
        remove_effect_internal(&mut game, "home");
    }
}
