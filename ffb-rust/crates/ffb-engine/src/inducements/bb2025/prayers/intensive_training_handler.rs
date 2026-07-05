/// 1:1 translation of `com.fumbbl.ffb.server.inducements.bb2025.prayers.IntensiveTrainingHandler`.
/// DEFERRED(prayer-enhancement): dialog-based handler — coach selects the player to receive intensive training.
/// Returns false from initEffect (waiting for dialog response).
use ffb_model::model::animation_type::AnimationType;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::inducements::mixed::prayers::enhancement_remover::EnhancementRemover;
use crate::inducements::mixed::prayers::prayer_handler::PrayerHandler;
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
    fn animation_type(&self) -> AnimationType { AnimationType::PRAYER_INTENSIVE_TRAINING }
    fn get_name(&self) -> &'static str { "IntensiveTrainingHandler" }
    fn init_effect(&self, _prayer_state: &mut PrayerState, _game: &mut Game, _rng: &mut GameRng, _team_id: &str) -> bool {
        // DEFERRED(prayer-enhancement): open coach dialog to select player; waiting for dialog response.
        false
    }
    fn remove_effect_internal(&self, _prayer_state: &mut PrayerState, game: &mut Game, team_id: &str) {
        EnhancementRemover::new().remove_enhancement(game, team_id, "INTENSIVE_TRAINING");
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
    fn handles_prayer_intensive_training() {
        let h = IntensiveTrainingHandler;
        assert!(h.handles_prayer("INTENSIVE_TRAINING"));
        assert!(!h.handles_prayer("OTHER"));
    }

    #[test]
    fn init_effect_returns_false_waiting_for_dialog() {
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
