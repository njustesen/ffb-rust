/// 1:1 translation of `com.fumbbl.ffb.server.inducements.bb2020.prayers.BlessedStatueOfNuffleHandler`.
/// Extends mixed BlessedStatueOfNuffleHandler (SelectPlayerPrayerHandler).
/// handledPrayer() = Prayer.BLESSED_STATUE_OF_NUFFLE.
use ffb_model::model::animation_type::AnimationType;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::inducements::mixed::prayers::{blessed_statue_of_nuffle_handler as base, prayer_handler::PrayerHandler};
use crate::prayer_state::PrayerState;

pub struct BlessedStatueOfNuffleHandler;

impl BlessedStatueOfNuffleHandler {
    pub fn new() -> Self { Self }
}

impl Default for BlessedStatueOfNuffleHandler {
    fn default() -> Self { Self::new() }
}

impl PrayerHandler for BlessedStatueOfNuffleHandler {
    fn handled_prayer_name(&self) -> &'static str { "BLESSED_STATUE_OF_NUFFLE" }
    fn animation_type(&self) -> AnimationType { base::animation_type() }
    fn get_name(&self) -> &'static str { "BlessedStatueOfNuffleHandler" }
    fn init_effect(&self, prayer_state: &mut PrayerState, game: &mut Game, _rng: &mut GameRng, team_id: &str) -> bool {
        // DEFERRED(prayer-dialog): needs SelectPlayerPrayerHandler + dialog + fieldModel.addPrayerEnhancements
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
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    #[test]
    fn handles_prayer_blessed_statue_of_nuffle() {
        let h = BlessedStatueOfNuffleHandler;
        assert!(h.handles_prayer("BLESSED_STATUE_OF_NUFFLE"));
        assert!(!h.handles_prayer("IRON_MAN"));
    }

    #[test]
    fn init_effect_returns_false_pending_dialog() {
        let h = BlessedStatueOfNuffleHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        assert!(!h.init_effect(&mut state, &mut game, &mut GameRng::new(0), "team1"));
    }

    #[test]
    fn animation_type_is_correct() {
        let h = BlessedStatueOfNuffleHandler;
        assert_eq!(h.animation_type(), AnimationType::PRAYER_BLESSED_STATUE_OF_NUFFLE);
    }
}
