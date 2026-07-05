/// 1:1 translation of `com.fumbbl.ffb.server.inducements.mixed.prayers.DialogPrayerHandler`.
/// Abstract handler for prayers that require a coach dialog before taking effect.
/// DEFERRED — dialog system not yet ported.
use ffb_model::model::game::Game;
use crate::prayer_state::PrayerState;

/// Java: DialogPrayerHandler.initEffect — starts dialog flow. Returns false (waiting).
pub fn init_effect_dialog(
    _prayer_state: &mut PrayerState,
    _game: &Game,
    _team_id: &str,
    _prayer_name: &str,
) -> bool {
    // DEFERRED(prayer-dialog): needs dialog creation + candidate player listing
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_game() -> Game {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    #[test]
    fn init_effect_returns_false_waiting_for_dialog() {
        let mut state = PrayerState::new();
        let game = make_game();
        assert!(!init_effect_dialog(&mut state, &game, "team1", "INTENSIVE_TRAINING"));
    }

    #[test]
    fn dialog_prayer_handler_is_independent_of_state() {
        let mut state = PrayerState::new();
        let game = make_game();
        init_effect_dialog(&mut state, &game, "team1", "BLESSED_STATUE_OF_NUFFLE");
        assert!(state.get_additional_cas_spp_teams().is_empty());
    }

    #[test]
    fn dialog_handler_returns_false_for_any_prayer() {
        let mut state = PrayerState::new();
        let game = make_game();
        assert!(!init_effect_dialog(&mut state, &game, "team1", "IRON_MAN"));
    }
}
