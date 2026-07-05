/// 1:1 translation of `com.fumbbl.ffb.server.inducements.bb2020.prayers.IntensiveTrainingHandler`.
/// Extends mixed IntensiveTrainingHandler (SelectPlayerPrayerHandler).
/// handledPrayer() = Prayer.INTENSIVE_TRAINING.
use ffb_model::model::animation_type::AnimationType;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::inducements::mixed::prayers::{intensive_training_handler as base, prayer_handler::PrayerHandler};
use crate::prayer_state::PrayerState;

pub struct IntensiveTrainingHandler;

impl IntensiveTrainingHandler {
    pub fn new() -> Self { Self }
}

impl Default for IntensiveTrainingHandler {
    fn default() -> Self { Self::new() }
}

impl PrayerHandler for IntensiveTrainingHandler {
    fn handled_prayer_name(&self) -> &'static str { "INTENSIVE_TRAINING" }
    fn animation_type(&self) -> AnimationType { base::animation_type() }
    fn get_name(&self) -> &'static str { "IntensiveTrainingHandler" }
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
    fn handles_prayer_intensive_training() {
        let h = IntensiveTrainingHandler;
        assert!(h.handles_prayer("INTENSIVE_TRAINING"));
        assert!(!h.handles_prayer("IRON_MAN"));
    }

    #[test]
    fn init_effect_returns_false_pending_dialog() {
        let h = IntensiveTrainingHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        assert!(!h.init_effect(&mut state, &mut game, &mut GameRng::new(0), "team1"));
    }

    #[test]
    fn animation_type_is_correct() {
        let h = IntensiveTrainingHandler;
        assert_eq!(h.animation_type(), AnimationType::PRAYER_INTENSIVE_TRAINING);
    }
}
