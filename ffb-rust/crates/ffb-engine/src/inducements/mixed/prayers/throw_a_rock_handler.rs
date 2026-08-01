/// 1:1 translation of `com.fumbbl.ffb.server.inducements.mixed.prayers.ThrowARockHandler`.
/// Abstract — only provides animationType(); actual initEffect logic lives in edition-specific handlers.
use ffb_model::model::animation_type::AnimationType;
use ffb_model::model::game::Game;
use crate::prayer_state::PrayerState;

pub fn animation_type() -> AnimationType {
    AnimationType::PRAYER_THROW_A_ROCK
}

pub fn init_effect(_prayer_state: &mut PrayerState, _game: &Game, _team_id: &str) -> bool {
    // no-op: base module — actual logic is in edition-specific handlers (bb2020/bb2025)
    true
}

pub fn remove_effect_internal(_prayer_state: &mut PrayerState, _game: &Game, _team_id: &str) {
    // No persistent state to remove for ThrowARock
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
    fn animation_type_is_correct() {
        assert_eq!(animation_type(), AnimationType::PRAYER_THROW_A_ROCK);
    }

    // NOTE (test equalization): the 6 no-op init/remove helper tests
    // (init_effect_returns_true, remove_effect_is_noop, init_effect_returns_true_for_any_team,
    // remove_effect_on_missing_team_is_safe, init_effect_is_idempotent_for_different_teams,
    // remove_effect_after_init_is_safe) were pruned - Rust-structural plumbing: the Java mixed
    // ThrowARockHandler is abstract with ONLY animationType(); init/remove live per-edition.
}
