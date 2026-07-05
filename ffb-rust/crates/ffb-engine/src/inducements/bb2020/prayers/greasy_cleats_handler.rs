/// 1:1 translation of `com.fumbbl.ffb.server.inducements.bb2020.prayers.GreasyCleatsHandler`.
/// Extends mixed GreasyCleatsHandler (RandomSelectionPrayerHandler) — selects 1 random opponent, reduces MA by 1.
use ffb_model::model::animation_type::AnimationType;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::inducements::bb2020::prayers::opponent_player_selector::OpponentPlayerSelector;
use crate::inducements::mixed::prayers::{greasy_cleats_handler as base, prayer_handler::PrayerHandler};
use crate::prayer_state::PrayerState;

pub struct GreasyCleatsHandler;

impl GreasyCleatsHandler {
    pub fn new() -> Self { Self }
}

impl Default for GreasyCleatsHandler {
    fn default() -> Self { Self::new() }
}

impl PrayerHandler for GreasyCleatsHandler {
    fn handled_prayer_name(&self) -> &'static str { "GREASY_CLEATS" }
    fn animation_type(&self) -> AnimationType { base::animation_type() }
    fn get_name(&self) -> &'static str { "GreasyCleatsHandler" }
    fn init_effect(&self, prayer_state: &mut PrayerState, game: &mut Game, rng: &mut GameRng, team_id: &str) -> bool {
        base::init_effect(prayer_state, game, rng, team_id, &OpponentPlayerSelector::new())
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
    fn handles_prayer_greasy_cleats() {
        let h = GreasyCleatsHandler;
        assert!(h.handles_prayer("GREASY_CLEATS"));
        assert!(!h.handles_prayer("STILETTO"));
    }

    #[test]
    fn animation_type_is_correct() {
        let h = GreasyCleatsHandler;
        assert_eq!(h.animation_type(), AnimationType::PRAYER_GREASY_CLEATS);
    }

    #[test]
    fn init_effect_returns_true() {
        let h = GreasyCleatsHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        assert!(h.init_effect(&mut state, &mut game, &mut GameRng::new(0), "home"));
    }

    #[test]
    fn remove_effect_is_callable() {
        let h = GreasyCleatsHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        h.remove_effect_internal(&mut state, &mut game, "home");
    }
}
