/// 1:1 translation of `com.fumbbl.ffb.server.inducements.bb2025.prayers.BadHabitsHandler`.
/// Extends mixed BadHabitsHandler with BB2025 OpponentPlayerSelector.
/// Selects D3 random opponent players, marks prayer, and grants Loner (2+) via apply_prayer_player_effect.
use ffb_model::model::animation_type::AnimationType;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::inducements::bb2025::prayers::opponent_player_selector::OpponentPlayerSelector;
use crate::inducements::mixed::prayers::bad_habits_handler::{self, PRAYER_NAME};
use crate::inducements::mixed::prayers::prayer_handler::PrayerHandler;
use crate::prayer_state::PrayerState;

pub struct BadHabitsHandler;

impl BadHabitsHandler {
    pub fn new() -> Self { Self }
}

impl Default for BadHabitsHandler {
    fn default() -> Self { Self::new() }
}

impl PrayerHandler for BadHabitsHandler {
    fn handled_prayer_name(&self) -> &'static str { PRAYER_NAME }
    fn animation_type(&self) -> AnimationType { AnimationType::PRAYER_BAD_HABITS }
    fn get_name(&self) -> &'static str { "BadHabitsHandler" }

    /// Java: initEffect — rolls D3, selects that many opponent RESERVE players, marks prayer, grants Loner (2+).
    fn init_effect(&self, prayer_state: &mut PrayerState, game: &mut Game, rng: &mut GameRng, team_id: &str) -> bool {
        let d3_roll = rng.d3();
        bad_habits_handler::init_effect(prayer_state, game, rng, team_id, d3_roll, &OpponentPlayerSelector::new())
    }

    fn remove_effect_internal(&self, _prayer_state: &mut PrayerState, game: &mut Game, team_id: &str) {
        bad_habits_handler::remove_effect_internal(game, team_id);
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
    fn handles_prayer_bad_habits() {
        assert!(BadHabitsHandler.handles_prayer("BAD_HABITS"));
        assert!(!BadHabitsHandler.handles_prayer("OTHER"));
    }

    #[test]
    fn init_effect_returns_true() {
        let h = BadHabitsHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        assert!(h.init_effect(&mut state, &mut game, &mut GameRng::new(0), "home"));
    }

    #[test]
    fn animation_type_is_correct() {
        assert_eq!(BadHabitsHandler.animation_type(), AnimationType::PRAYER_BAD_HABITS);
    }

    #[test]
    fn init_effect_selects_from_opponent_team() {
        use ffb_model::enums::{PS_RESERVE, PlayerState};
        use ffb_model::model::player::Player;
        use ffb_model::enums::PlayerType;
        let mut game = make_game();
        // Add an away player in RESERVE
        game.team_away.players.push(Player {
            id: "a1".into(), name: "a1".into(), nr: 1,
            position_id: "pos".into(),
            player_type: PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: std::collections::HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0,
            race: None,
            ..Default::default()
});
        game.field_model.set_player_state("a1", PlayerState::new(PS_RESERVE));
        let h = BadHabitsHandler;
        let mut state = PrayerState::new();
        // Home team gets bad habits — should affect away player
        h.init_effect(&mut state, &mut game, &mut GameRng::new(0), "home");
        assert!(game.field_model.has_prayer_enhancement("a1", "BAD_HABITS"));
    }
}
