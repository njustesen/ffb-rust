/// 1:1 translation of `com.fumbbl.ffb.server.inducements.mixed.prayers.SelectPlayerPrayerHandler`.
/// Abstract handler that shows a dialog for the coach to pick a player.
/// client-only: dialog system not yet ported.
use ffb_model::model::game::Game;
use crate::prayer_state::PrayerState;

/// Java: SelectPlayerPrayerHandler.initEffect — opens dialog, returns false (waiting).
pub fn init_effect_select_player(
    _prayer_state: &mut PrayerState,
    _game: &Game,
    _team_id: &str,
    _prayer_name: &str,
) -> bool {
    // client-only: DialogPrayerParameter — abstract handler; concrete handlers use random selection
    false
}

/// Java: SelectPlayerPrayerHandler.applySelection — applies enhancement after dialog.
pub fn apply_selection_select_player(
    _game: &mut Game,
    _player_id: &str,
    _prayer_name: &str,
) {
    // client-only: apply_selection called by concrete handlers after dialog choice; abstract stub is no-op
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
        assert!(!init_effect_select_player(&mut state, &game, "team1", "KNUCKLE_DUSTERS"));
    }

    #[test]
    fn apply_selection_is_callable() {
        let mut game = make_game();
        apply_selection_select_player(&mut game, "player1", "KNUCKLE_DUSTERS");
    }

    #[test]
    fn init_effect_returns_false_for_iron_man() {
        let mut state = PrayerState::new();
        let game = make_game();
        assert!(!init_effect_select_player(&mut state, &game, "team1", "IRON_MAN"));
    }

    #[test]
    fn apply_selection_with_empty_strings_is_safe() {
        let mut game = make_game();
        apply_selection_select_player(&mut game, "", "");
    }

    #[test]
    fn init_effect_does_not_mutate_state() {
        let mut state = PrayerState::new();
        let game = make_game();
        init_effect_select_player(&mut state, &game, "team1", "GREASY_CLEATS");
        assert!(state.get_additional_completion_spp_teams().is_empty());
    }

    #[test]
    fn init_effect_returns_false_for_stiletto() {
        let mut state = PrayerState::new();
        let game = make_game();
        assert!(!init_effect_select_player(&mut state, &game, "away", "STILETTO"));
    }

    #[test]
    fn apply_selection_with_real_prayer_name_is_safe() {
        let mut game = make_game();
        apply_selection_select_player(&mut game, "player_abc", "IRON_MAN");
    }
}
