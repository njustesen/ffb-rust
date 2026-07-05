/// 1:1 translation of `com.fumbbl.ffb.server.inducements.mixed.prayers.PrayerHandler`.
/// Abstract base for all prayer handlers. Each concrete handler is responsible for one prayer type.
///
/// In Java, GameState bundles Game + PrayerState. In Rust we pass them separately since
/// the GameState wrapper is not yet implemented.
use ffb_model::model::animation_type::AnimationType;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::prayer_state::PrayerState;

pub trait PrayerHandler: Send + Sync {
    /// Java: handledPrayer() — the prayer this handler is responsible for (by name).
    fn handled_prayer_name(&self) -> &'static str;

    /// Java: animationType() — the animation to play when this prayer is activated.
    fn animation_type(&self) -> AnimationType;

    /// Java: getName() — class simple name.
    fn get_name(&self) -> &'static str;

    /// Java: handles(Prayer) — true if this handler processes the given prayer.
    fn handles_prayer(&self, prayer_name: &str) -> bool {
        prayer_name == self.handled_prayer_name()
    }

    /// Java: initEffect(GameState, Team) — apply the prayer effect.
    /// Returns true if the effect was fully applied (no dialog needed).
    fn init_effect(&self, prayer_state: &mut PrayerState, game: &mut Game, rng: &mut GameRng, team_id: &str) -> bool;

    /// Java: removeEffectInternal(GameState, Team) — reverse the prayer effect.
    fn remove_effect_internal(&self, prayer_state: &mut PrayerState, game: &mut Game, team_id: &str);

    /// Java: removeEffect(GameState, Team) — called from driver; delegates to removeEffectInternal.
    fn remove_effect(&self, prayer_state: &mut PrayerState, game: &mut Game, team_id: &str) {
        self.remove_effect_internal(prayer_state, game, team_id);
    }

    /// Java: applySelection(Game, PrayerDialogSelection) — called after coach dialog.
    /// Default: no-op (only dialog-based handlers override this).
    fn apply_selection(&self, _prayer_state: &mut PrayerState, _game: &mut Game, _team_id: &str) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_game() -> Game {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    struct TestPrayerHandler;
    impl PrayerHandler for TestPrayerHandler {
        fn handled_prayer_name(&self) -> &'static str { "FOULING_FRENZY" }
        fn animation_type(&self) -> AnimationType { AnimationType::PRAYER_FOULING_FRENZY }
        fn get_name(&self) -> &'static str { "TestPrayerHandler" }
        fn init_effect(&self, _: &mut PrayerState, _: &mut Game, _: &mut GameRng, _: &str) -> bool { true }
        fn remove_effect_internal(&self, _: &mut PrayerState, _: &mut Game, _: &str) {}
    }

    #[test]
    fn handles_prayer_matches_by_name() {
        let h = TestPrayerHandler;
        assert!(h.handles_prayer("FOULING_FRENZY"));
        assert!(!h.handles_prayer("FRIENDS_WITH_THE_REF"));
    }

    #[test]
    fn default_apply_selection_is_noop() {
        let h = TestPrayerHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        h.apply_selection(&mut state, &mut game, "team1");
    }

    #[test]
    fn get_name_returns_handler_name() {
        let h = TestPrayerHandler;
        assert_eq!(h.get_name(), "TestPrayerHandler");
    }
}
