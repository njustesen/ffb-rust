/// 1:1 translation of `com.fumbbl.ffb.server.inducements.bb2020.prayers.NecessaryViolenceHandler`.
use ffb_model::model::animation_type::AnimationType;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::inducements::mixed::prayers::prayer_handler::PrayerHandler;
use crate::prayer_state::PrayerState;

pub struct NecessaryViolenceHandler;

impl NecessaryViolenceHandler {
    pub fn new() -> Self { Self }
}

impl Default for NecessaryViolenceHandler {
    fn default() -> Self { Self::new() }
}

impl PrayerHandler for NecessaryViolenceHandler {
    fn handled_prayer_name(&self) -> &'static str { "NECESSARY_VIOLENCE" }
    fn animation_type(&self) -> AnimationType { AnimationType::PRAYER_NECESSARY_VIOLENCE }
    fn get_name(&self) -> &'static str { "NecessaryViolenceHandler" }
    fn init_effect(&self, prayer_state: &mut PrayerState, _game: &mut Game, _rng: &mut GameRng, team_id: &str) -> bool {
        prayer_state.add_get_additional_cas_spp(team_id);
        true
    }
    fn remove_effect_internal(&self, prayer_state: &mut PrayerState, _game: &mut Game, team_id: &str) {
        prayer_state.remove_get_additional_cas_spp(team_id);
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
    fn handled_prayer_is_necessary_violence() {
        let h = NecessaryViolenceHandler;
        assert_eq!(h.handled_prayer_name(), "NECESSARY_VIOLENCE");
        assert!(h.handles_prayer("NECESSARY_VIOLENCE"));
        assert!(!h.handles_prayer("PERFECT_PASSING"));
    }

    #[test]
    fn init_effect_adds_cas_spp_team() {
        let h = NecessaryViolenceHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        assert!(h.init_effect(&mut state, &mut game, &mut GameRng::new(0), "team1"));
        assert!(state.get_additional_cas_spp_teams().contains("team1"));
    }

    #[test]
    fn remove_effect_clears_cas_spp_team() {
        let h = NecessaryViolenceHandler;
        let mut state = PrayerState::new();
        state.add_get_additional_cas_spp("team1");
        let mut game = make_game();
        h.remove_effect_internal(&mut state, &mut game, "team1");
        assert!(!state.get_additional_cas_spp_teams().contains("team1"));
    }

    #[test]
    fn get_name_returns_handler_name() {
        let h = NecessaryViolenceHandler;
        assert_eq!(h.get_name(), "NecessaryViolenceHandler");
    }

    #[test]
    fn handles_prayer_is_case_sensitive() {
        let h = NecessaryViolenceHandler;
        let prayer = h.handled_prayer_name();
        assert!(!h.handles_prayer(&prayer.to_lowercase()));
    }
}
