/// 1:1 translation of `com.fumbbl.ffb.server.inducements.bb2020.prayers.FanInteractionHandler`.
use ffb_model::model::animation_type::AnimationType;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::inducements::mixed::prayers::{fan_interaction_handler as base, prayer_handler::PrayerHandler};
use crate::prayer_state::PrayerState;

pub struct FanInteractionHandler;

impl FanInteractionHandler {
    pub fn new() -> Self { Self }
}

impl Default for FanInteractionHandler {
    fn default() -> Self { Self::new() }
}

impl PrayerHandler for FanInteractionHandler {
    fn handled_prayer_name(&self) -> &'static str { "FAN_INTERACTION" }
    fn animation_type(&self) -> AnimationType { base::animation_type() }
    fn get_name(&self) -> &'static str { "FanInteractionHandler" }
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
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    #[test]
    fn handled_prayer_is_fan_interaction() {
        let h = FanInteractionHandler;
        assert!(h.handles_prayer("FAN_INTERACTION"));
    }

    #[test]
    fn init_effect_sets_prayer_state() {
        let h = FanInteractionHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        assert!(h.init_effect(&mut state, &mut game, &mut GameRng::new(0), "teamA"));
        assert!(state.has_fan_interaction("teamA"));
    }

    #[test]
    fn remove_effect_clears_prayer_state() {
        let h = FanInteractionHandler;
        let mut state = PrayerState::new();
        state.add_fan_interaction("teamA");
        let mut game = make_game();
        h.remove_effect_internal(&mut state, &mut game, "teamA");
        assert!(!state.has_fan_interaction("teamA"));
    }
}
