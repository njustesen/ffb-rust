/// 1:1 translation of `com.fumbbl.ffb.server.inducements.mixed.prayers.StilettoHandler`.
/// Extends RandomSelectionPrayerHandler — selects 1 random player and grants Stab.
///
/// Effect: grants Stab skill to the selected player via `apply_prayer_player_effect`.
/// Prayer enhancement tracking is fully implemented.
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
pub const PRAYER_NAME: &str = "STILETTO";

pub fn animation_type() -> AnimationType {
    AnimationType::PRAYER_STILETTO
}

pub fn init_effect(prayer_state: &mut PrayerState, game: &mut Game, rng: &mut GameRng, team_id: &str, selector: &dyn PlayerSelector) -> bool {
    init_effect_random_selection(prayer_state, game, rng, team_id, PRAYER_NAME, AFFECTED_PLAYERS, selector, &[SkillId::Stab])
}

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
        assert_eq!(animation_type(), AnimationType::PRAYER_STILETTO);
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

    #[test]
    fn prayer_name_constant_is_stiletto() {
        assert_eq!(PRAYER_NAME, "STILETTO");
    }

    #[test]
    fn remove_effect_after_init_is_safe() {
        let mut state = PrayerState::new();
        let mut game = make_game();
        let stub = StubPlayerSelector;
        init_effect(&mut state, &mut game, &mut GameRng::new(0), "home", &stub);
        remove_effect_internal(&mut game, "home");
    }
}
