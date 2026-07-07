/// 1:1 translation of `com.fumbbl.ffb.server.inducements.bb2025.prayers.DazzlingCatchingHandler`.
/// BB2025-only prayer handler — no mixed base. Grants additional catches SPP.
/// Java: removeEffectInternal is empty (no persistent state to clear).
use ffb_model::model::animation_type::AnimationType;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::inducements::mixed::prayers::prayer_handler::PrayerHandler;
use crate::prayer_state::PrayerState;

pub struct DazzlingCatchingHandler;

impl DazzlingCatchingHandler {
    pub fn new() -> Self { Self }
}

impl Default for DazzlingCatchingHandler {
    fn default() -> Self { Self::new() }
}

impl PrayerHandler for DazzlingCatchingHandler {
    fn handled_prayer_name(&self) -> &'static str { "DAZZLING_CATCHING" }
    fn animation_type(&self) -> AnimationType { AnimationType::PRAYER_DAZZLING_CATCHING }
    fn get_name(&self) -> &'static str { "DazzlingCatchingHandler" }
    fn init_effect(&self, prayer_state: &mut PrayerState, _game: &mut Game, _rng: &mut GameRng, team_id: &str) -> bool {
        prayer_state.add_get_additional_catches_spp(team_id);
        true
    }
    fn remove_effect_internal(&self, _prayer_state: &mut PrayerState, _game: &mut Game, _team_id: &str) {
        // Java: removeEffectInternal is empty — no persistent state to reverse.
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
    fn handles_prayer_dazzling_catching() {
        let h = DazzlingCatchingHandler;
        assert!(h.handles_prayer("DAZZLING_CATCHING"));
        assert!(!h.handles_prayer("OTHER"));
    }

    #[test]
    fn init_effect_adds_catches_spp_teams() {
        let h = DazzlingCatchingHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        assert!(h.init_effect(&mut state, &mut game, &mut GameRng::new(0), "team1"));
        assert!(state.get_additional_catches_spp_teams().contains("team1"));
    }

    #[test]
    fn remove_effect_is_noop() {
        let h = DazzlingCatchingHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        state.add_get_additional_catches_spp("team1");
        h.remove_effect_internal(&mut state, &mut game, "team1");
        // removeEffectInternal is empty in Java — entry must still be present.
        assert!(state.get_additional_catches_spp_teams().contains("team1"));
    }

    #[test]
    fn get_name_returns_handler_name() {
        let h = DazzlingCatchingHandler;
        assert_eq!(h.get_name(), "DazzlingCatchingHandler");
    }

    #[test]
    fn handles_prayer_is_case_sensitive() {
        let h = DazzlingCatchingHandler;
        let prayer = h.handled_prayer_name();
        assert!(!h.handles_prayer(&prayer.to_lowercase()));
    }
}
