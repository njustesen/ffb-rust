/// 1:1 translation of `com.fumbbl.ffb.server.inducements.bb2020.prayers.UnderScrutinyHandler`.
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
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        use crate::step::framework::test_team;
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    #[test]
    fn handled_prayer_is_under_scrutiny() {
        let h = UnderScrutinyHandler;
        assert!(h.handles_prayer("UNDER_SCRUTINY"));
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
        h.remove_effect_internal(&mut state, &mut game, "home");
        assert!(!state.is_under_scrutiny("away"));
    }
}
