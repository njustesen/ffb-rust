/// 1:1 translation of `com.fumbbl.ffb.server.inducements.bb2020.prayers.PerfectPassingHandler`.
use ffb_model::model::animation_type::AnimationType;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::inducements::mixed::prayers::{perfect_passing_handler as base, prayer_handler::PrayerHandler};
use crate::prayer_state::PrayerState;

pub struct PerfectPassingHandler;

impl PerfectPassingHandler {
    pub fn new() -> Self { Self }
}

impl Default for PerfectPassingHandler {
    fn default() -> Self { Self::new() }
}

impl PrayerHandler for PerfectPassingHandler {
    fn handled_prayer_name(&self) -> &'static str { "PERFECT_PASSING" }
    fn animation_type(&self) -> AnimationType { base::animation_type() }
    fn get_name(&self) -> &'static str { "PerfectPassingHandler" }
    fn init_effect(&self, prayer_state: &mut PrayerState, _game: &mut Game, _rng: &mut GameRng, team_id: &str) -> bool {
        base::init_effect(prayer_state, team_id)
    }
    fn remove_effect_internal(&self, prayer_state: &mut PrayerState, _game: &mut Game, team_id: &str) {
        base::remove_effect_internal(prayer_state, team_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;
    use crate::step::framework::test_team;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    #[test]
    fn handled_prayer_is_perfect_passing() {
        let h = PerfectPassingHandler;
        assert_eq!(h.handled_prayer_name(), "PERFECT_PASSING");
        assert!(h.handles_prayer("PERFECT_PASSING"));
        assert!(!h.handles_prayer("FOULING_FRENZY"));
    }

    #[test]
    fn init_effect_adds_completion_spp_team() {
        let h = PerfectPassingHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        assert!(h.init_effect(&mut state, &mut game, &mut GameRng::new(0), "team1"));
        assert!(state.get_additional_completion_spp_teams().contains("team1"));
    }

    #[test]
    fn remove_effect_clears_completion_spp_team() {
        let h = PerfectPassingHandler;
        let mut state = PrayerState::new();
        state.add_get_additional_completion_spp("team1");
        let mut game = make_game();
        h.remove_effect_internal(&mut state, &mut game, "team1");
        assert!(!state.get_additional_completion_spp_teams().contains("team1"));
    }

    #[test]
    fn get_name_returns_handler_name() {
        let h = PerfectPassingHandler;
        assert_eq!(h.get_name(), "PerfectPassingHandler");
    }

    #[test]
    fn handles_prayer_is_case_sensitive() {
        let h = PerfectPassingHandler;
        let prayer = h.handled_prayer_name();
        assert!(!h.handles_prayer(&prayer.to_lowercase()));
    }
}
