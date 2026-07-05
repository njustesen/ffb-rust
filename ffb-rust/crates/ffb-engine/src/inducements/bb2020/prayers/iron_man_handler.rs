/// 1:1 translation of `com.fumbbl.ffb.server.inducements.bb2020.prayers.IronManHandler`.
/// Extends mixed IronManHandler (SelectPlayerPrayerHandler). handledPrayer() = Prayer.IRON_MAN.
use ffb_model::model::animation_type::AnimationType;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::inducements::mixed::prayers::{iron_man_handler as base, prayer_handler::PrayerHandler};
use crate::prayer_state::PrayerState;

pub struct IronManHandler;

impl IronManHandler {
    pub fn new() -> Self { Self }
}

impl Default for IronManHandler {
    fn default() -> Self { Self::new() }
}

impl PrayerHandler for IronManHandler {
    fn handled_prayer_name(&self) -> &'static str { "IRON_MAN" }
    fn animation_type(&self) -> AnimationType { base::animation_type() }
    fn get_name(&self) -> &'static str { "IronManHandler" }
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
    fn handles_prayer_iron_man() {
        let h = IronManHandler;
        assert!(h.handles_prayer("IRON_MAN"));
        assert!(!h.handles_prayer("FOULING_FRENZY"));
    }

    #[test]
    fn init_effect_returns_false_pending_dialog() {
        let h = IronManHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        assert!(!h.init_effect(&mut state, &mut game, &mut GameRng::new(0), "team1"));
    }

    #[test]
    fn animation_type_is_correct() {
        let h = IronManHandler;
        assert_eq!(h.animation_type(), AnimationType::PRAYER_IRON_MAN);
    }
}
