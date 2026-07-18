/// 1:1 translation of `com.fumbbl.ffb.server.inducements.bb2020.prayers.BadHabitsHandler`.
/// Extends mixed BadHabitsHandler (RandomSelectionPrayerHandler) — selects D3 random opponents.
use ffb_model::model::animation_type::AnimationType;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::inducements::bb2020::prayers::opponent_player_selector::OpponentPlayerSelector;
use crate::inducements::mixed::prayers::{bad_habits_handler as base, prayer_handler::PrayerHandler};
use crate::prayer_state::PrayerState;

pub struct BadHabitsHandler;

impl BadHabitsHandler {
    pub fn new() -> Self { Self }
}

impl Default for BadHabitsHandler {
    fn default() -> Self { Self::new() }
}

impl PrayerHandler for BadHabitsHandler {
    fn handled_prayer_name(&self) -> &'static str { "BAD_HABITS" }
    fn animation_type(&self) -> AnimationType { base::animation_type() }
    fn get_name(&self) -> &'static str { "BadHabitsHandler" }
    fn init_effect(&self, prayer_state: &mut PrayerState, game: &mut Game, rng: &mut GameRng, team_id: &str) -> bool {
        let d3_roll = rng.d3();
        base::init_effect(prayer_state, game, rng, team_id, d3_roll, &OpponentPlayerSelector::new())
    }
    fn remove_effect_internal(&self, _prayer_state: &mut PrayerState, game: &mut Game, team_id: &str) {
        base::remove_effect_internal(game, team_id, &OpponentPlayerSelector::new());
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
    fn handled_prayer_is_bad_habits() {
        let h = BadHabitsHandler;
        assert_eq!(h.handled_prayer_name(), "BAD_HABITS");
        assert!(h.handles_prayer("BAD_HABITS"));
        assert!(!h.handles_prayer("PERFECT_PASSING"));
    }

    #[test]
    fn init_effect_returns_true() {
        let h = BadHabitsHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        assert!(h.init_effect(&mut state, &mut game, &mut GameRng::new(0), "home"));
    }

    #[test]
    fn remove_effect_is_callable() {
        let h = BadHabitsHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        h.remove_effect_internal(&mut state, &mut game, "home");
    }

    #[test]
    fn animation_type_is_prayer_bad_habits() {
        let h = BadHabitsHandler;
        assert_eq!(h.animation_type(), AnimationType::PRAYER_BAD_HABITS);
    }
    #[test]
    fn does_not_handle_other_prayers() {
        let h = BadHabitsHandler;
        assert!(!h.handles_prayer("PERFECT_PASSING"));
        assert!(!h.handles_prayer(""));
    }
}
