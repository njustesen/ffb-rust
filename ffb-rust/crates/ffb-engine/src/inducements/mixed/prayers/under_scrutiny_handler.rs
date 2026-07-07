/// 1:1 translation of `com.fumbbl.ffb.server.inducements.mixed.prayers.UnderScrutinyHandler`.
/// Java: adds under-scrutiny to the OTHER team (the opponent of the praying team).
use ffb_model::model::animation_type::AnimationType;
use ffb_model::model::game::Game;
use crate::prayer_state::PrayerState;

/// Returns the ID of the team that is NOT `team_id`.
fn other_team_id<'a>(game: &'a Game, team_id: &str) -> &'a str {
    if game.team_home.id == team_id { &game.team_away.id } else { &game.team_home.id }
}

pub fn init_effect(prayer_state: &mut PrayerState, game: &Game, team_id: &str) -> bool {
    prayer_state.add_under_scrutiny(other_team_id(game, team_id));
    true
}

pub fn remove_effect_internal(prayer_state: &mut PrayerState, game: &Game, team_id: &str) {
    prayer_state.remove_under_scrutiny(other_team_id(game, team_id));
}

pub fn animation_type() -> AnimationType {
    AnimationType::PRAYER_UNDER_SCRUTINY
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::game::Game;
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        use crate::step::framework::test_team;
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    #[test]
    fn init_effect_adds_scrutiny_to_opponent() {
        let mut state = PrayerState::new();
        let game = make_game();
        assert!(init_effect(&mut state, &game, "home"));
        assert!(state.is_under_scrutiny("away"));
        assert!(!state.is_under_scrutiny("home"));
    }

    #[test]
    fn remove_effect_removes_scrutiny_from_opponent() {
        let mut state = PrayerState::new();
        let game = make_game();
        state.add_under_scrutiny("away");
        remove_effect_internal(&mut state, &game, "home");
        assert!(!state.is_under_scrutiny("away"));
    }

    #[test]
    fn animation_type_is_correct() {
        assert_eq!(animation_type(), AnimationType::PRAYER_UNDER_SCRUTINY);
    }

    #[test]
    fn init_effect_does_not_add_scrutiny_to_praying_team() {
        let mut state = PrayerState::new();
        let game = make_game();
        init_effect(&mut state, &game, "home");
        assert!(!state.is_under_scrutiny("home"));
    }

    #[test]
    fn remove_effect_on_missing_team_is_safe() {
        let mut state = PrayerState::new();
        let game = make_game();
        remove_effect_internal(&mut state, &game, "home");
        assert!(!state.is_under_scrutiny("away"));
    }

    #[test]
    fn animation_type_is_under_scrutiny() {
        assert_eq!(animation_type(), AnimationType::PRAYER_UNDER_SCRUTINY);
    }

    #[test]
    fn init_effect_away_team_adds_scrutiny_to_home() {
        let mut state = PrayerState::new();
        let game = make_game();
        assert!(init_effect(&mut state, &game, "away"));
        assert!(state.is_under_scrutiny("home"));
        assert!(!state.is_under_scrutiny("away"));
    }
}
