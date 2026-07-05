/// 1:1 translation of `com.fumbbl.ffb.server.inducements.bb2020.prayers.TreacherousTrapdoorHandler`.
/// Extends mixed TreacherousTrapdoorHandler.
/// handledPrayer() = Prayer.TREACHEROUS_TRAPDOOR.
use ffb_model::model::animation_type::AnimationType;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::inducements::mixed::prayers::{treacherous_trapdoor_handler as base, prayer_handler::PrayerHandler};
use crate::prayer_state::PrayerState;

pub struct TreacherousTrapdoorHandler;

impl TreacherousTrapdoorHandler {
    pub fn new() -> Self { Self }
}

impl Default for TreacherousTrapdoorHandler {
    fn default() -> Self { Self::new() }
}

impl PrayerHandler for TreacherousTrapdoorHandler {
    fn handled_prayer_name(&self) -> &'static str { "TREACHEROUS_TRAPDOOR" }
    fn animation_type(&self) -> AnimationType { base::animation_type() }
    fn get_name(&self) -> &'static str { "TreacherousTrapdoorHandler" }
    fn init_effect(&self, prayer_state: &mut PrayerState, game: &mut Game, _rng: &mut GameRng, team_id: &str) -> bool {
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
    fn handles_prayer_treacherous_trapdoor() {
        let h = TreacherousTrapdoorHandler;
        assert!(h.handles_prayer("TREACHEROUS_TRAPDOOR"));
        assert!(!h.handles_prayer("THROW_A_ROCK"));
    }

    #[test]
    fn init_effect_returns_true() {
        let h = TreacherousTrapdoorHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        assert!(h.init_effect(&mut state, &mut game, &mut GameRng::new(0), "team1"));
    }

    #[test]
    fn init_effect_adds_two_trapdoors() {
        use ffb_model::types::FieldCoordinate;
        let h = TreacherousTrapdoorHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        h.init_effect(&mut state, &mut game, &mut GameRng::new(0), "team1");
        assert!(game.field_model.has_trap_door(FieldCoordinate::new(6, 1)));
        assert!(game.field_model.has_trap_door(FieldCoordinate::new(19, 13)));
    }

    #[test]
    fn remove_effect_clears_trapdoors() {
        use ffb_model::types::FieldCoordinate;
        let h = TreacherousTrapdoorHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        h.init_effect(&mut state, &mut game, &mut GameRng::new(0), "team1");
        h.remove_effect_internal(&mut state, &mut game, "team1");
        assert!(!game.field_model.has_trap_door(FieldCoordinate::new(6, 1)));
    }

    #[test]
    fn animation_type_is_correct() {
        let h = TreacherousTrapdoorHandler;
        assert_eq!(h.animation_type(), AnimationType::PRAYER_TREACHEROUS_TRAPDOOR);
    }
}
