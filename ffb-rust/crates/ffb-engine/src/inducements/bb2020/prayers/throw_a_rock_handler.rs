/// 1:1 translation of `com.fumbbl.ffb.server.inducements.bb2020.prayers.ThrowARockHandler`.
/// Java: initEffect adds shouldNotStall to the OTHER team; removeEffectInternal reverses it.
use ffb_model::model::animation_type::AnimationType;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::inducements::mixed::prayers::{throw_a_rock_handler as base, prayer_handler::PrayerHandler};
use crate::prayer_state::PrayerState;

pub struct ThrowARockHandler;

impl ThrowARockHandler {
    pub fn new() -> Self { Self }
}

impl Default for ThrowARockHandler {
    fn default() -> Self { Self::new() }
}

impl PrayerHandler for ThrowARockHandler {
    fn handled_prayer_name(&self) -> &'static str { "THROW_A_ROCK" }
    fn animation_type(&self) -> AnimationType { base::animation_type() }
    fn get_name(&self) -> &'static str { "ThrowARockHandler" }

    /// Java: addShouldNotStall on the OTHER team (so opponent must not stall).
    fn init_effect(&self, prayer_state: &mut PrayerState, game: &mut Game, _rng: &mut GameRng, team_id: &str) -> bool {
        let other = if game.is_home_team(team_id) {
            game.team_away.id.clone()
        } else {
            game.team_home.id.clone()
        };
        prayer_state.add_should_not_stall(&other);
        true
    }

    /// Java: removeShouldNotStall on the OTHER team.
    fn remove_effect_internal(&self, prayer_state: &mut PrayerState, game: &mut Game, team_id: &str) {
        let other = if game.is_home_team(team_id) {
            game.team_away.id.clone()
        } else {
            game.team_home.id.clone()
        };
        prayer_state.remove_should_not_stall(&other);
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
    fn handles_prayer_throw_a_rock() {
        let h = ThrowARockHandler;
        assert!(h.handles_prayer("THROW_A_ROCK"));
        assert!(!h.handles_prayer("IRON_MAN"));
    }

    #[test]
    fn init_effect_adds_should_not_stall_to_other_team() {
        let h = ThrowARockHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        // praying team = "home", so "away" should get shouldNotStall
        assert!(h.init_effect(&mut state, &mut game, &mut GameRng::new(0), "home"));
        assert!(state.should_not_stall("away"));
        assert!(!state.should_not_stall("home"));
    }

    #[test]
    fn init_effect_away_team_praying_marks_home() {
        let h = ThrowARockHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        assert!(h.init_effect(&mut state, &mut game, &mut GameRng::new(0), "away"));
        assert!(state.should_not_stall("home"));
        assert!(!state.should_not_stall("away"));
    }

    #[test]
    fn remove_effect_clears_should_not_stall() {
        let h = ThrowARockHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        h.init_effect(&mut state, &mut game, &mut GameRng::new(0), "home");
        h.remove_effect_internal(&mut state, &mut game, "home");
        assert!(!state.should_not_stall("away"));
    }

    #[test]
    fn animation_type_is_correct() {
        let h = ThrowARockHandler;
        assert_eq!(h.animation_type(), AnimationType::PRAYER_THROW_A_ROCK);
    }
}
