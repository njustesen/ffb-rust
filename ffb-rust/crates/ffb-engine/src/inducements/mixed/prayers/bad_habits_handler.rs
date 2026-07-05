/// 1:1 translation of `com.fumbbl.ffb.server.inducements.mixed.prayers.BadHabitsHandler`.
/// Extends RandomSelectionPrayerHandler — selects D3 random opponent players and grants Loner (2+).
///
/// Effect: grants Loner (2+) skill to D3 selected opponent players via `apply_prayer_player_effect`.
/// Prayer enhancement tracking is fully implemented.
use ffb_model::model::animation_type::AnimationType;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::enums::SkillId;
use crate::inducements::mixed::prayers::player_selector::PlayerSelector;
use crate::inducements::mixed::prayers::random_selection_prayer_handler::{
    init_effect_random_selection, remove_effect_internal_random_selection,
};
use crate::prayer_state::PrayerState;

pub const PRAYER_NAME: &str = "BAD_HABITS";

pub fn animation_type() -> AnimationType {
    AnimationType::PRAYER_BAD_HABITS
}

/// Java: affectedPlayers(GameState) rolls D3.
pub fn affected_players(d3_roll: i32) -> i32 {
    d3_roll
}

pub fn init_effect(
    prayer_state: &mut PrayerState,
    game: &mut Game,
    rng: &mut GameRng,
    team_id: &str,
    d3_roll: i32,
    selector: &dyn PlayerSelector,
) -> bool {
    init_effect_random_selection(prayer_state, game, rng, team_id, PRAYER_NAME, affected_players(d3_roll), selector, &[SkillId::Loner])
}

pub fn remove_effect_internal(game: &mut Game, team_id: &str) {
    remove_effect_internal_random_selection(game, team_id, PRAYER_NAME);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::util::rng::GameRng;
    use crate::step::framework::test_team;
    use crate::inducements::mixed::prayers::player_selector::StubPlayerSelector;

    fn make_game() -> Game {
        use ffb_model::enums::Rules;
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    #[test]
    fn animation_type_is_correct() {
        assert_eq!(animation_type(), AnimationType::PRAYER_BAD_HABITS);
    }

    #[test]
    fn affected_players_returns_d3_roll() {
        assert_eq!(affected_players(2), 2);
        assert_eq!(affected_players(3), 3);
    }

    #[test]
    fn init_effect_returns_true() {
        let mut state = PrayerState::new();
        let mut game = make_game();
        let stub = StubPlayerSelector;
        assert!(init_effect(&mut state, &mut game, &mut GameRng::new(0), "home", 2, &stub));
    }

    #[test]
    fn remove_effect_is_callable() {
        let mut game = make_game();
        remove_effect_internal(&mut game, "home");
    }
}
