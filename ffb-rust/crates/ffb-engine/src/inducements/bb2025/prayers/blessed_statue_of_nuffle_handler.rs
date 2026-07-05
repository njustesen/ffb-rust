/// 1:1 translation of `com.fumbbl.ffb.server.inducements.bb2025.prayers.BlessedStatueOfNuffleHandler`.
/// DEFERRED(prayer-enhancement): dialog-based handler — coach selects the player to be blessed.
/// Returns false from initEffect (waiting for dialog response).
use ffb_model::model::animation_type::AnimationType;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::inducements::mixed::prayers::enhancement_remover::EnhancementRemover;
use crate::inducements::mixed::prayers::prayer_handler::PrayerHandler;
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
    fn animation_type(&self) -> AnimationType { AnimationType::PRAYER_BLESSED_STATUE_OF_NUFFLE }
    fn get_name(&self) -> &'static str { "BlessedStatueOfNuffleHandler" }
    fn init_effect(&self, _prayer_state: &mut PrayerState, _game: &mut Game, _rng: &mut GameRng, _team_id: &str) -> bool {
        // DEFERRED(prayer-enhancement): open coach dialog to select player; waiting for dialog response.
        false
    }
    fn remove_effect_internal(&self, _prayer_state: &mut PrayerState, game: &mut Game, team_id: &str) {
        EnhancementRemover::new().remove_enhancement(game, team_id, "BLESSED_STATUE_OF_NUFFLE");
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
    fn handles_prayer_blessed_statue_of_nuffle() {
        let h = BlessedStatueOfNuffleHandler;
        assert!(h.handles_prayer("BLESSED_STATUE_OF_NUFFLE"));
        assert!(!h.handles_prayer("OTHER"));
    }

    #[test]
    fn init_effect_returns_false_waiting_for_dialog() {
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
