/// 1:1 translation of `com.fumbbl.ffb.server.inducements.bb2020.prayers.IntensiveTrainingHandler`.
/// Extends mixed IntensiveTrainingHandler with BB2020 PlayerSelector (own team RESERVE).
///
/// Headless: selects one player and marks the prayer enhancement; skips skill-selection
/// dialog (position skill categories not available server-side).
use ffb_model::model::animation_type::AnimationType;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::inducements::bb2020::prayers::player_selector::PlayerSelector as BB2020PlayerSelector;
use crate::inducements::mixed::prayers::{intensive_training_handler as base, prayer_handler::PrayerHandler};
use crate::prayer_state::PrayerState;

pub struct IntensiveTrainingHandler;

impl IntensiveTrainingHandler {
    pub fn new() -> Self { Self }
}

impl Default for IntensiveTrainingHandler {
    fn default() -> Self { Self::new() }
}

impl PrayerHandler for IntensiveTrainingHandler {
    fn handled_prayer_name(&self) -> &'static str { "INTENSIVE_TRAINING" }
    fn animation_type(&self) -> AnimationType { base::animation_type() }
    fn get_name(&self) -> &'static str { "IntensiveTrainingHandler" }
    fn init_effect(&self, prayer_state: &mut PrayerState, game: &mut Game, rng: &mut GameRng, team_id: &str) -> bool {
        base::init_effect(prayer_state, game, rng, team_id, &BB2020PlayerSelector::new())
    }
    fn remove_effect_internal(&self, _prayer_state: &mut PrayerState, game: &mut Game, team_id: &str) {
        base::remove_effect_internal(game, team_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, PS_RESERVE, PlayerState};
    use ffb_model::model::player::Player;
    use ffb_model::model::player_status::PlayerStatus;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::util::rng::GameRng;
    use crate::step::framework::test_team;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    fn add_reserve_player(game: &mut Game, id: &str) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            player_status: PlayerStatus::ACTIVE,
            ..Default::default()
        });
        game.field_model.set_player_state(id, PlayerState::new(PS_RESERVE));
    }

    #[test]
    fn handles_prayer_intensive_training() {
        let h = IntensiveTrainingHandler;
        assert!(h.handles_prayer("INTENSIVE_TRAINING"));
        assert!(!h.handles_prayer("IRON_MAN"));
    }

    #[test]
    fn init_effect_returns_true() {
        let h = IntensiveTrainingHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        assert!(h.init_effect(&mut state, &mut game, &mut GameRng::new(0), "home"));
    }

    #[test]
    fn init_effect_marks_prayer_on_reserve_player() {
        let h = IntensiveTrainingHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        add_reserve_player(&mut game, "h1");
        h.init_effect(&mut state, &mut game, &mut GameRng::new(0), "home");
        assert!(game.field_model.has_prayer_enhancement("h1", "INTENSIVE_TRAINING"));
    }

    #[test]
    fn animation_type_is_correct() {
        let h = IntensiveTrainingHandler;
        assert_eq!(h.animation_type(), AnimationType::PRAYER_INTENSIVE_TRAINING);
    }
}
