/// 1:1 translation of `com.fumbbl.ffb.server.inducements.bb2020.prayers.StilettoHandler`.
/// Extends mixed StilettoHandler (RandomSelectionPrayerHandler) — selects 1 random player and grants Stab.
use ffb_model::model::animation_type::AnimationType;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::inducements::bb2020::prayers::player_selector::PlayerSelector as BB2020PlayerSelector;
use crate::inducements::mixed::prayers::{stiletto_handler as base, prayer_handler::PrayerHandler};
use crate::prayer_state::PrayerState;

pub struct StilettoHandler;

impl StilettoHandler {
    pub fn new() -> Self { Self }
}

impl Default for StilettoHandler {
    fn default() -> Self { Self::new() }
}

impl PrayerHandler for StilettoHandler {
    fn handled_prayer_name(&self) -> &'static str { "STILETTO" }
    fn animation_type(&self) -> AnimationType { base::animation_type() }
    fn get_name(&self) -> &'static str { "StilettoHandler" }
    fn init_effect(&self, prayer_state: &mut PrayerState, game: &mut Game, rng: &mut GameRng, team_id: &str) -> bool {
        base::init_effect(prayer_state, game, rng, team_id, &BB2020PlayerSelector::new())
    }
    fn remove_effect_internal(&self, _prayer_state: &mut PrayerState, game: &mut Game, team_id: &str) {
        base::remove_effect_internal(game, team_id);
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
    fn handles_prayer_stiletto() {
        let h = StilettoHandler;
        assert!(h.handles_prayer("STILETTO"));
        assert!(!h.handles_prayer("GREASY_CLEATS"));
    }

    #[test]
    fn animation_type_is_correct() {
        let h = StilettoHandler;
        assert_eq!(h.animation_type(), AnimationType::PRAYER_STILETTO);
    }

    #[test]
    fn init_effect_returns_true() {
        let h = StilettoHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        assert!(h.init_effect(&mut state, &mut game, &mut GameRng::new(0), "home"));
    }

    #[test]
    fn remove_effect_is_callable() {
        let h = StilettoHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        h.remove_effect_internal(&mut state, &mut game, "home");
    }
    #[test]
    fn does_not_handle_other_prayers() {
        let h = StilettoHandler;
        assert!(!h.handles_prayer("PERFECT_PASSING"));
        assert!(!h.handles_prayer(""));
    }
}
