/// 1:1 translation of `com.fumbbl.ffb.server.inducements.mixed.prayers.IntensiveTrainingHandler`.
/// Extends DialogPrayerHandler — shuffles eligible players, picks one, shows a dialog
/// for the coach to select a primary skill from that player's position categories.
///
/// Headless mode: selects the player and marks the prayer, but skips the skill-selection
/// dialog — position skill categories are not available in the server-side model.
/// The player's enhancement is tracked so remove_effect_internal can clean it up.
use ffb_model::model::animation_type::AnimationType;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::inducements::mixed::prayers::player_selector::PlayerSelector;
use crate::inducements::mixed::prayers::random_selection_prayer_handler::{
    init_effect_random_selection, remove_effect_internal_random_selection,
};
use crate::prayer_state::PrayerState;

pub const PRAYER_NAME: &str = "INTENSIVE_TRAINING";

pub fn animation_type() -> AnimationType {
    AnimationType::PRAYER_INTENSIVE_TRAINING
}

/// Java: createDialog — shuffles eligible players, picks one, shows DialogSelectSkill.
/// Headless: picks one player randomly, marks prayer, skips skill dialog (no position data available).
pub fn init_effect(prayer_state: &mut PrayerState, game: &mut Game, rng: &mut GameRng, team_id: &str, selector: &dyn PlayerSelector) -> bool {
    // Java: shuffle + pick 1 player → show dialog. Headless: just mark the enhancement, no skill granted.
    // client-only: DialogSelectSkillParameter — position skill categories not in server-side model.
    init_effect_random_selection(prayer_state, game, rng, team_id, PRAYER_NAME, 1, selector, &[])
}

/// Java: applySelection — coach-chosen skill is granted via game.getFieldModel().addIntensiveTrainingSkill.
/// Headless: no-op — skill selection requires position category data not available server-side.
pub fn apply_selection(_prayer_state: &mut PrayerState, _game: &mut Game, _player_id: &str) {
    // client-only: addIntensiveTrainingSkill — skill selection dialog skipped in headless mode.
}

pub fn remove_effect_internal(game: &mut Game, team_id: &str) {
    remove_effect_internal_random_selection(game, team_id, PRAYER_NAME);
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
    use crate::inducements::mixed::prayers::player_selector::StubPlayerSelector;
    use crate::inducements::bb2020::prayers::player_selector::PlayerSelector as BB2020Selector;

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
    fn animation_type_is_correct() {
        assert_eq!(animation_type(), AnimationType::PRAYER_INTENSIVE_TRAINING);
    }

    #[test]
    fn init_effect_returns_true() {
        let mut state = PrayerState::new();
        let mut game = make_game();
        let stub = StubPlayerSelector;
        assert!(init_effect(&mut state, &mut game, &mut GameRng::new(0), "home", &stub));
    }

    #[test]
    fn init_effect_marks_prayer_on_selected_player() {
        let mut state = PrayerState::new();
        let mut game = make_game();
        add_reserve_player(&mut game, "h1");
        let selector = BB2020Selector::new();
        init_effect(&mut state, &mut game, &mut GameRng::new(0), "home", &selector);
        assert!(game.field_model.has_prayer_enhancement("h1", PRAYER_NAME));
    }

    #[test]
    fn apply_selection_is_noop() {
        let mut state = PrayerState::new();
        let mut game = make_game();
        apply_selection(&mut state, &mut game, "player1");
    }

    #[test]
    fn remove_effect_clears_enhancement() {
        let mut game = make_game();
        add_reserve_player(&mut game, "h1");
        game.field_model.add_prayer_enhancement("h1", PRAYER_NAME);
        assert!(game.field_model.has_prayer_enhancement("h1", PRAYER_NAME));
        remove_effect_internal(&mut game, "home");
        assert!(!game.field_model.has_prayer_enhancement("h1", PRAYER_NAME));
    }
}
