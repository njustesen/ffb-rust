/// 1:1 translation of `com.fumbbl.ffb.server.inducements.bb2020.prayers.KnuckleDustersHandler`.
/// Extends mixed KnuckleDustersHandler (SelectPlayerPrayerHandler) — coach selects a player who gains Mighty Blow (+1).
use ffb_model::model::animation_type::AnimationType;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::inducements::mixed::prayers::{knuckle_dusters_handler as base, prayer_handler::PrayerHandler};
use crate::prayer_state::PrayerState;

pub struct KnuckleDustersHandler;

impl KnuckleDustersHandler {
    pub fn new() -> Self { Self }
}

impl Default for KnuckleDustersHandler {
    fn default() -> Self { Self::new() }
}

impl PrayerHandler for KnuckleDustersHandler {
    fn handled_prayer_name(&self) -> &'static str { "KNUCKLE_DUSTERS" }
    fn animation_type(&self) -> AnimationType { base::animation_type() }
    fn get_name(&self) -> &'static str { "KnuckleDustersHandler" }
    fn init_effect(&self, prayer_state: &mut PrayerState, _game: &mut Game, _rng: &mut GameRng, team_id: &str) -> bool {
        // DEFERRED(prayer-dialog): needs SelectPlayerPrayerHandler + dialog
        base::init_effect(prayer_state, _game, team_id)
    }
    fn remove_effect_internal(&self, prayer_state: &mut PrayerState, game: &mut Game, team_id: &str) {
        base::remove_effect_internal(prayer_state, game, team_id);
    }
    fn apply_selection(&self, prayer_state: &mut PrayerState, game: &mut Game, team_id: &str) {
        // DEFERRED(prayer-dialog): needs fieldModel.addPrayerEnhancements for KNUCKLE_DUSTERS
        base::apply_selection(prayer_state, game, team_id);
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
    fn handles_prayer_knuckle_dusters() {
        let h = KnuckleDustersHandler;
        assert!(h.handles_prayer("KNUCKLE_DUSTERS"));
        assert!(!h.handles_prayer("STILETTO"));
    }

    #[test]
    fn animation_type_is_correct() {
        let h = KnuckleDustersHandler;
        assert_eq!(h.animation_type(), AnimationType::PRAYER_KNUCKLE_DUSTERS);
    }

    #[test]
    fn init_effect_returns_false_pending_dialog() {
        let h = KnuckleDustersHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        assert!(!h.init_effect(&mut state, &mut game, &mut GameRng::new(0), "team1"));
    }
}
