/// 1:1 translation of `com.fumbbl.ffb.server.inducements.bb2025.prayers.UnderScrutinyHandler`.
/// BB2025 edition handler — delegates to the shared mixed implementation.
/// Scrutinises the opponent of the praying team, not the praying team itself.
use ffb_model::model::animation_type::AnimationType;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::inducements::mixed::prayers::{under_scrutiny_handler as base, prayer_handler::PrayerHandler};
use crate::prayer_state::PrayerState;

pub struct UnderScrutinyHandler;

impl UnderScrutinyHandler {
    pub fn new() -> Self { Self }
}

impl Default for UnderScrutinyHandler {
    fn default() -> Self { Self::new() }
}

impl PrayerHandler for UnderScrutinyHandler {
    fn handled_prayer_name(&self) -> &'static str { "UNDER_SCRUTINY" }
    fn animation_type(&self) -> AnimationType { base::animation_type() }
    fn get_name(&self) -> &'static str { "UnderScrutinyHandler" }
    fn init_effect(&self, prayer_state: &mut PrayerState, game: &mut Game, _rng: &mut GameRng, team_id: &str) -> bool {
        base::init_effect(prayer_state, game, team_id)
    }
    fn remove_effect_internal(&self, prayer_state: &mut PrayerState, game: &mut Game, team_id: &str) {
        base::remove_effect_internal(prayer_state, game, team_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn handles_prayer_under_scrutiny() {
        let h = UnderScrutinyHandler;
        assert!(h.handles_prayer("UNDER_SCRUTINY"));
        assert!(!h.handles_prayer("OTHER"));
    }

    #[test]
    fn init_effect_scrutinises_opponent() {
        let h = UnderScrutinyHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        assert!(h.init_effect(&mut state, &mut game, &mut GameRng::new(0), "home"));
        assert!(state.is_under_scrutiny("away"));
        assert!(!state.is_under_scrutiny("home"));
    }

    #[test]
    fn remove_effect_clears_opponent_scrutiny() {
        let h = UnderScrutinyHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        state.add_under_scrutiny("away");
        h.remove_effect(&mut state, &mut game, "home");
        assert!(!state.is_under_scrutiny("away"));
    }

    #[test]
    fn get_name_returns_handler_name() {
        let h = UnderScrutinyHandler;
        assert_eq!(h.get_name(), "UnderScrutinyHandler");
    }

    #[test]
    fn handles_prayer_is_case_sensitive() {
        let h = UnderScrutinyHandler;
        let prayer = h.handled_prayer_name();
        assert!(!h.handles_prayer(&prayer.to_lowercase()));
    }
}
